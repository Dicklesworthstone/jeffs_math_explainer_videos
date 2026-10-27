//! Narration and the cue log (plan §3.3, §7.4, §7.7).
//!
//! - [`Script`]: `script.tsv` (`<line id>\t<text>`), the single source of
//!   every spoken line, compiled into each video with `include_str!`.
//! - [`Narrator`]: the picture follows the voice. `say` waits for the line
//!   before it (plus a breath) and places `<id>.wav`; `hold` lets a line
//!   finish before a wipe; `mark`, `music_breath` and `listen` author what the
//!   score answers.
//! - [`LineGaps`]: the breath is one constant ([`BREATH`]) until a video opts
//!   in to [`LineGaps::Thought`], where the silence between two lines follows
//!   the thought ([`classify_gap`]) and lands on the length human readers
//!   leave there ([`GapClass::audible_s`]).
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

// ----------------------------------------------------------------- line gaps

/// How the silence between two consecutive narration lines is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineGaps {
    /// [`BREATH`] after every line: the original rule, and what a video keeps
    /// until its owner has heard the other.
    #[default]
    Constant,
    /// The silence follows the thought: [`classify_gap`] reads the two lines'
    /// text and [`GapClass::audible_s`] gives the silence the ear should
    /// hear, less the two takes' own quiet edges ([`quiet_edges`]). A contiguous
    /// split inside a coherent take keeps its recorded phrasing instead of
    /// receiving another generic gap. A
    /// chapter's first line is placed by the class of the pair that crosses
    /// the chapter join. Authored windows (`music_breath`, `listen`,
    /// `sonify`) keep their length and the breath after them, and `hold`
    /// never waits longer than [`BREATH`], so whatever an author puts after a
    /// `hold` keeps its place.
    Thought,
}

/// What lies between two consecutive lines, read from their text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapClass {
    /// The next line carries the thought on: it opens with a connective, or
    /// the line before ends unfinished.
    Continuation,
    /// A thought ended and the next starts.
    Beat,
    /// The next line opens a new example or a quiz: the end of a paragraph.
    Topic,
}

/// Human readers of teaching prose leave this much audible silence between
/// two sentences of one paragraph, seconds: the median of 18 sentence ends in
/// three volunteer recordings of one calculus text. Audible silence is the
/// stretch under the read's own top level less 24 dB, 40 ms or more, so it
/// includes the quiet edges of the two takes.
pub const WITHIN_PARAGRAPH_S: f64 = 0.58;
/// The same at any sentence end (60 ends: the three recordings and one
/// scripted explainer's narration).
pub const SENTENCE_END_S: f64 = 0.81;
/// The same at a paragraph end (11 ends in the three recordings).
pub const PARAGRAPH_END_S: f64 = 0.99;
// a continuation is shorter than a beat, and a beat shorter than a topic
const _: () = assert!(WITHIN_PARAGRAPH_S < SENTENCE_END_S && SENTENCE_END_S < PARAGRAPH_END_S);
/// The shortest generic silence the kit inserts between separate reads, seconds.
pub const MIN_GAP_S: f64 = 0.10;
/// The coefficient of variation the plain line gaps of a video must reach
/// for [`check_gap_spread`] to pass. The constant breath, with the takes' own
/// quiet edges added, varied by about 0.04 across Video F's plain line gaps;
/// the three classes above, on that video's script, vary by 0.16.
pub const MIN_GAP_CV: f64 = 0.10;
/// The longest gap between two lines that is a plain one, seconds: the
/// silence between the end of a line and the start of the next, with no
/// authored window between them, under this. A longer one is what the scene
/// waited for (a picture wait, a prediction pause): counted, it hides a
/// constant breath behind a few long waits (Video F under the constant rule:
/// the 52 pairs with no window between vary by 0.72, its 45 under 1.0 s by
/// 0.04).
pub const PLAIN_GAP_MAX_S: f64 = 1.0;

/// Openers of the lines that start a new example or a quiz (the next line
/// begins a paragraph); matched at the start of a line, on a word boundary.
const TOPIC_OPENERS: &[&str] = &[
    "Your turn",
    "Take ",
    "Here's",
    "Last one",
    "One more",
    "Most ",
    "The next",
];
/// Words that continue the thought of the line before when they open a line.
const CONNECTIVES: &[&str] = &[
    "And", "But", "So", "Then", "Now", "Because", "Which", "Plus", "Or", "That's",
];

impl GapClass {
    /// The audible silence a human reader leaves at this kind of boundary.
    pub const fn audible_s(self) -> f64 {
        match self {
            Self::Continuation => WITHIN_PARAGRAPH_S,
            Self::Beat => SENTENCE_END_S,
            Self::Topic => PARAGRAPH_END_S,
        }
    }
}

fn opens_with(text: &str, phrase: &str) -> bool {
    text.strip_prefix(phrase)
        .is_some_and(|rest| phrase.ends_with(' ') || !rest.starts_with(|c: char| c.is_alphanumeric() || c == '\''))
}

/// The class of the boundary between two consecutive lines, from their text:
/// [`GapClass::Topic`] when `next` opens with one of the new-example or quiz
/// openers; otherwise [`GapClass::Continuation`] when it opens with a
/// connective (And, But, So, Then, Now, Because, Which, Plus, Or, That's) or
/// `prev` does not end a sentence (a closing quote or bracket after the end
/// mark does not hide it); otherwise [`GapClass::Beat`].
pub fn classify_gap(prev: &str, next: &str) -> GapClass {
    let next = next.trim_start().replace('\u{2019}', "'");
    if TOPIC_OPENERS.iter().any(|o| opens_with(&next, o)) {
        return GapClass::Topic;
    }
    let first = next.split_whitespace().next().unwrap_or("");
    let word = first.trim_end_matches(|c: char| !c.is_alphanumeric());
    if CONNECTIVES.contains(&word) {
        return GapClass::Continuation;
    }
    let end = prev
        .trim_end()
        .trim_end_matches(['"', '\u{201d}', '\'', '\u{2019}', ')', ']']);
    if end.ends_with(['.', '?', '!']) {
        GapClass::Beat
    } else {
        GapClass::Continuation
    }
}

const EDGE_FRAME_S: f64 = 0.020;
const EDGE_HOP_S: f64 = 0.010;
const EDGE_BELOW_TOP_DB: f64 = 24.0;
const EDGE_MIN_FRAMES: usize = 4;

/// The quiet stretch at the start and at the end of a take, seconds
/// (lead, tail): 20 ms frames every 10 ms, a frame is quiet when its level is
/// more than 24 dB under the 95th percentile of the take's frame levels, and
/// a quiet run counts from 40 ms. This is the detector the human silences
/// above were measured with, so a gap built as `class - tail - lead` is heard
/// as about the class: to within a frame on synthetic takes with a constant
/// level, and to about 0.1 s on Video F's real takes (a median of 0.03 to
/// 0.05 s longer than the class), because the detector here takes each take's
/// own 95th percentile as its reference and the human numbers were measured
/// against a whole recording's. A take with no speech has none.
pub fn quiet_edges(mono: &[f64], rate: u32) -> (f64, f64) {
    let frame = (f64::from(rate) * EDGE_FRAME_S).round() as usize;
    let hop = (f64::from(rate) * EDGE_HOP_S).round() as usize;
    if frame == 0 || hop == 0 || mono.len() < frame {
        return (0.0, 0.0);
    }
    let levels: Vec<f64> = mono
        .windows(frame)
        .step_by(hop)
        .map(|s| 10.0 * (s.iter().map(|x| x * x).sum::<f64>() / frame as f64 + 1e-12).log10())
        .collect();
    let n = levels.len();
    let mut sorted = levels.clone();
    sorted.sort_by(f64::total_cmp);
    let at = 0.95 * (n - 1) as f64;
    let (lo, hi) = (at.floor() as usize, at.ceil() as usize);
    let value = |k: usize| sorted.get(k).copied().unwrap_or(f64::NEG_INFINITY);
    let top = value(lo) + (value(hi) - value(lo)) * (at - lo as f64);
    let quiet = |l: f64| l < top - EDGE_BELOW_TOP_DB;
    let lead = levels.iter().take_while(|&&l| quiet(l)).count();
    let tail = levels.iter().rev().take_while(|&&l| quiet(l)).count();
    if lead == n {
        return (0.0, 0.0);
    }
    // a run of `count` quiet frames reaches from its first frame's start to
    // its last frame's end
    let seconds = |count: usize| {
        if count >= EDGE_MIN_FRAMES {
            ((count - 1) * hop + frame) as f64 / f64::from(rate)
        } else {
            0.0
        }
    };
    (seconds(lead), seconds(tail))
}

/// Conservative endpoint level evidence, without the 40 ms run requirement.
/// This is a seam guard, not a speech or perceptual classifier. Unknown short
/// clips count as sounding. Inspect the actual last frame, including any
/// remainder outside the hop grid used for the reference percentile.
fn quiet_boundary_frames(mono: &[f64], rate: u32) -> (bool, bool) {
    let frame = (f64::from(rate) * EDGE_FRAME_S).round() as usize;
    let hop = (f64::from(rate) * EDGE_HOP_S).round() as usize;
    if frame == 0 || hop == 0 || mono.len() < frame {
        return (false, false);
    }
    let level = |s: &[f64]| 10.0 * (s.iter().map(|x| x * x).sum::<f64>() / s.len() as f64 + 1e-12).log10();
    let mut levels: Vec<f64> = mono.windows(frame).step_by(hop).map(level).collect();
    levels.sort_by(f64::total_cmp);
    let at = 0.95 * (levels.len() - 1) as f64;
    let lo = at.floor() as usize;
    let hi = at.ceil() as usize;
    let top = levels[lo] + (levels[hi] - levels[lo]) * (at - lo as f64);
    let quiet = |s: &[f64]| s.iter().all(|x| *x == 0.0) || level(s) < top - EDGE_BELOW_TOP_DB;
    (quiet(&mono[..frame]), quiet(&mono[mono.len() - frame..]))
}

/// The coefficient of variation (population standard deviation over the
/// mean) of a set of gaps; `None` for fewer than two or a non-positive mean.
pub fn gap_cv(gaps: &[f64]) -> Option<f64> {
    if gaps.len() < 2 {
        return None;
    }
    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let var = gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len() as f64;
    Some(var.sqrt() / mean)
}

/// The plain gaps of a chapter, seconds: for each pair of consecutive lines
/// (`lines`: each one's start and length, seconds, in order) the silence from
/// the end of the first to the start of the second, kept when no authored
/// window starts between them (`windows`: each window's start, seconds) and it
/// is under [`PLAIN_GAP_MAX_S`]. The input of [`check_gap_spread`].
pub fn plain_line_gaps(lines: &[(f64, f64)], windows: &[f64]) -> Vec<f64> {
    lines
        .windows(2)
        .filter_map(|pair| {
            let end = pair[0].0 + pair[0].1;
            let gap = pair[1].0 - end;
            let windowed = windows.iter().any(|&w| w >= end && w <= pair[1].0);
            (!windowed && gap < PLAIN_GAP_MAX_S).then_some(gap)
        })
        .collect()
}

/// The spread check on a video's plain line gaps: the gaps between a line and
/// the next with no authored window between them (see [`plain_line_gaps`]),
/// of which only those under [`PLAIN_GAP_MAX_S`] count (a longer one is what
/// the scene waited for, not the narration's breath). At least three of them
/// must be left, and their [`gap_cv`] must reach [`MIN_GAP_CV`]: one value
/// everywhere fails it.
///
/// # Errors
/// Names the count and the spread found.
pub fn check_gap_spread(gaps: &[f64]) -> Result<(), String> {
    let plain: Vec<f64> = gaps.iter().copied().filter(|&g| g < PLAIN_GAP_MAX_S).collect();
    if plain.len() < 3 {
        return Err(format!(
            "{} plain line gaps (of {} given, under {PLAIN_GAP_MAX_S} s): too few to judge their spread",
            plain.len(),
            gaps.len()
        ));
    }
    let cv = gap_cv(&plain).unwrap_or(0.0);
    if cv < MIN_GAP_CV {
        return Err(format!(
            "{} plain line gaps vary by {cv:.3} (coefficient of variation), under {MIN_GAP_CV}: one value serves every thought",
            plain.len()
        ));
    }
    Ok(())
}

/// Extract the plain line gaps (seconds) from a rendered [`ChapterTimeline`] and
/// the line lengths (seconds): consecutive lines in the chapter with no
/// authored listen or breath window starting between them and a gap under
/// [`PLAIN_GAP_MAX_S`].
pub fn timeline_plain_line_gaps(tl: &ChapterTimeline, lengths: &HashMap<String, f64>) -> Vec<f64> {
    let fps = f64::from(tl.fps.max(1));
    let lines: Vec<(f64, f64)> = tl
        .lines
        .iter()
        .map(|l| (l.start_frame as f64 / fps, lengths.get(&l.id).copied().unwrap_or(0.0)))
        .collect();
    let windows: Vec<f64> = tl
        .listens
        .iter()
        .chain(&tl.breaths)
        .map(|w| w.frame as f64 / fps)
        .collect();
    plain_line_gaps(&lines, &windows)
}

/// Run [`plain_line_gaps`] and [`check_gap_spread`] across the rendered
/// [`ChapterTimeline`]s of a video when `line_gaps == LineGaps::Thought`.
/// Videos using [`LineGaps::Constant`] are skipped by design (`Ok(())`).
///
/// # Errors
/// Returns an error when `line_gaps == LineGaps::Thought` and the pooled plain
/// line gaps across `timelines` fail [`check_gap_spread`].
pub fn check_timelines_gap_spread<'a>(
    line_gaps: LineGaps,
    timelines: impl IntoIterator<Item = &'a ChapterTimeline>,
    lengths: &HashMap<String, f64>,
) -> Result<(), String> {
    if line_gaps == LineGaps::Constant {
        return Ok(());
    }
    let plain: Vec<f64> = timelines
        .into_iter()
        .flat_map(|tl| timeline_plain_line_gaps(tl, lengths))
        .collect();
    check_gap_spread(&plain)
}

// ----------------------------------------------------------------- script

/// The spoken lines of a video, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Script {
    lines: Vec<(String, String)>,
    /// Every line of the video in order. [`Script::select`] keeps it, so a
    /// partial render still knows the line before its first.
    all: Vec<(String, String)>,
    line_gaps: LineGaps,
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
        Ok(Self {
            all: lines.clone(),
            lines,
            line_gaps: LineGaps::Constant,
        })
    }

    /// The script read under another line-gap rule (the video's choice; see
    /// [`LineGaps`]).
    #[must_use]
    pub fn with_line_gaps(mut self, line_gaps: LineGaps) -> Self {
        self.line_gaps = line_gaps;
        self
    }

    pub fn line_gaps(&self) -> LineGaps {
        self.line_gaps
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
            all: self.all.clone(),
            line_gaps: self.line_gaps,
        }
    }
}

// ----------------------------------------------------------------- cues

/// Check a coherent-unit plan against the exact script it will synthesize.
/// Metadata may repeat cue names; only `line_ids` assigns spoken coverage.
pub fn check_coherent_units(script_source: &str, json: &str) -> Result<(), String> {
    let plan: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let hash = explainer_score::digest::sha256_hex(script_source.as_bytes());
    if plan["schema_version"].as_u64() != Some(1) || plan["script_sha256"].as_str() != Some(&hash) {
        return Err("unit plan does not bind the current script".into());
    }
    let script = Script::parse(script_source)?;
    let units = plan["units"].as_array().ok_or("units must be an array")?;
    let mut covered = Vec::new();
    let mut unit_ids = HashSet::new();
    for unit in units {
        let id = unit["id"].as_str().ok_or("unit id must be text")?;
        if id.is_empty() || !unit_ids.insert(id) {
            return Err(format!("empty or repeated unit id {id:?}"));
        }
        let cues = unit["line_ids"].as_array().ok_or("line_ids must be an array")?;
        if cues.is_empty() {
            return Err(format!("unit {id} is empty"));
        }
        let mut chapter = None;
        let mut parts = Vec::new();
        for cue in cues {
            let cue = cue.as_str().ok_or("cue id must be text")?;
            let prefix = cue.split_once('_').ok_or("cue id lacks chapter")?.0;
            if chapter.is_some_and(|c| c != prefix) {
                return Err(format!("unit {id} crosses chapters"));
            }
            chapter = Some(prefix);
            parts.push(script.text(cue).ok_or_else(|| format!("unknown cue {cue}"))?);
            covered.push(cue);
        }
        let text = parts.join(" ");
        if unit["text"].as_str() != Some(&text) || !(1..=100).contains(&text.split_whitespace().count()) {
            return Err(format!(
                "unit {id} changes text or exceeds the coherent-unit word bound"
            ));
        }
    }
    let expected: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).collect();
    if covered != expected {
        return Err("units omit, repeat or reorder script cues".into());
    }
    Ok(())
}

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
    inspect_decoded(path).map(|(file, _)| file)
}

/// [`inspect_cue`] with the decoded samples, for a caller that also reads them.
fn inspect_decoded(path: &Path) -> io::Result<(CueFile, wav::Decoded)> {
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
    let file = CueFile {
        path: path.to_path_buf(),
        sha256: digest::sha256_hex(&bytes),
        bytes: bytes.len() as u64,
        rate: d.rate,
        channels: d.channels,
        frames: d.frames(),
    };
    Ok((file, d))
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
    line_gaps: LineGaps,
    /// Every line of the full script in order: the text a gap class is read from.
    order: Vec<(String, String)>,
    /// The (lead, tail) quiet seconds of each loaded take, measured only under
    /// [`LineGaps::Thought`]; empty for the scratch voice.
    edges: HashMap<String, (f64, f64)>,
    /// Endpoint levels for refusing silence inserted inside coherent speech.
    boundary_quiet: HashMap<String, (bool, bool)>,
    /// Adjacent cues cut from contiguous spans of one coherent take. Their
    /// recorded phrasing, including authored PCM silence, already spans the seam.
    contiguous_cues: HashSet<(String, String)>,
    after: RefCell<After>,
}

/// Where the narration stands, for the gap before the next line.
#[derive(Clone, Debug, PartialEq, Eq)]
enum After {
    /// A chapter's start: nothing said yet.
    Start,
    /// This line was the last thing said.
    Line(String),
    /// An authored window was the last thing: its length and the breath after
    /// it are the author's.
    Window,
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

/// Only the coherent assembler records enough provenance to identify a cue
/// boundary inside one take. Missing or malformed provenance is ineligible;
/// it must never turn two unrelated reads into a continuous phrase.
fn report_contiguous_cues(json: &str, script: &Script) -> io::Result<HashSet<(String, String)>> {
    struct Span<'a> {
        unit: &'a str,
        source: &'a str,
        hash: &'a str,
        start: u64,
        end: u64,
    }
    fn span(row: &serde_json::Value) -> Option<Span<'_>> {
        let text = |key: &str| row.get(key)?.as_str().filter(|s| !s.trim().is_empty());
        let unit = text("unit_id")?;
        let source = text("source")?;
        let hash = text("source_sha256")?;
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let start = row.get("source_in_sample")?.as_u64()?;
        let end = row.get("source_out_sample")?.as_u64()?;
        let samples = row.get("out_samples")?.as_u64()?;
        row.get("gap_after_samples")?.as_u64()?;
        if samples == 0 || end.checked_sub(start)? != samples {
            return None;
        }
        Some(Span {
            unit,
            source,
            hash,
            start,
            end,
        })
    }

    let report: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("report.json: {e}")))?;
    let mut pairs = HashSet::new();
    if report.pointer("/meta/adapter").and_then(serde_json::Value::as_str) != Some("coherent-narration-v1") {
        return Ok(pairs);
    }
    let Some(lines) = report.get("lines").and_then(serde_json::Value::as_object) else {
        return Ok(pairs);
    };
    // Full-script order prevents a partial selection from making formerly
    // nonadjacent cues neighbours. JSON key order is not spoken order.
    for pair in script.all.windows(2) {
        let (prev, next) = (&pair[0].0, &pair[1].0);
        if script.text(prev).is_none() || script.text(next).is_none() {
            continue;
        }
        let (Some((chapter, _)), Some((next_chapter, _))) = (prev.split_once('_'), next.split_once('_')) else {
            continue;
        };
        if chapter.is_empty() || chapter != next_chapter {
            continue;
        }
        let a = lines
            .get(prev)
            .and_then(serde_json::Value::as_array)
            .and_then(|rows| rows.last());
        let b = lines
            .get(next)
            .and_then(serde_json::Value::as_array)
            .and_then(|rows| rows.first());
        let (Some(a), Some(b)) = (a.and_then(span), b.and_then(span)) else {
            continue;
        };
        if a.unit == b.unit && a.source == b.source && a.hash == b.hash && a.end == b.start {
            pairs.insert((prev.clone(), next.clone()));
        }
    }
    Ok(pairs)
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
        let expected = sentences(text);
        let recorded = lines
            .get(id)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| bad(format!("report.json: no sentence list for scripted line {id}")))?;
        if expected.len() != recorded.len() {
            return Err(bad(format!(
                "report.json: {id} has {} scripted sentences but {} recorded sentences; reassemble this line from the current script before rendering",
                expected.len(),
                recorded.len()
            )));
        }
        // Offsets belong to individual assembled reads. Joined words can
        // agree while punctuation moves a word across a sentence boundary.
        for (k, (expected, recorded)) in expected.iter().zip(recorded).enumerate() {
            let spoken = recorded
                .get("sentence")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| bad(format!("report.json: {id} sentence {k}: no display text")))?;
            if normalize(expected) != normalize(spoken) {
                return Err(bad(format!(
                    "report.json: narration text for {id} sentence {k} differs from script.tsv; reassemble this line from the current script before rendering"
                )));
            }
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
        let mut edges = HashMap::new();
        let mut boundary_quiet = HashMap::new();
        for (id, _) in script.lines() {
            let (f, decoded) = inspect_decoded(&dir.join(format!("{id}.wav")))?;
            lengths.insert(id.clone(), f.seconds());
            files.insert(id.clone(), f);
            let mono = decoded.mono();
            boundary_quiet.insert(id.clone(), quiet_boundary_frames(&mono, decoded.rate));
            if script.line_gaps() == LineGaps::Thought {
                edges.insert(id.clone(), quiet_edges(&mono, decoded.rate));
            }
        }
        let report_path = dir.join("report.json");
        let report = std::fs::read_to_string(&report_path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", report_path.display())))?;
        check_report_script(&report, script)?;
        let contiguous_cues = report_contiguous_cues(&report, script)?;
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
            line_gaps: script.line_gaps(),
            order: script.all.clone(),
            edges,
            boundary_quiet,
            contiguous_cues,
            after: RefCell::new(After::Start),
        }))
    }

    /// A scratch voice for drafts: lengths from word counts, no audio.
    pub fn estimate(script: &Script) -> Rc<Self> {
        Rc::new(Self::scratch(script))
    }

    fn scratch(script: &Script) -> Self {
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
        Self {
            dir: None,
            files: HashMap::new(),
            lengths,
            starts,
            said_at: RefCell::new(HashMap::new()),
            gain_db: GAIN_DB,
            busy_until: Cell::new(0.0),
            log: RefCell::new(Log::default()),
            line_gaps: script.line_gaps(),
            order: script.all.clone(),
            edges: HashMap::new(),
            boundary_quiet: HashMap::new(),
            contiguous_cues: HashSet::new(),
            after: RefCell::new(After::Start),
        }
    }

    /// The scratch voice with the real narration's measured timing for some
    /// lines, each `(id, sentence onsets in seconds, length in seconds)` as
    /// `report.json` records them: a chapter's test can then place a
    /// picture against the sentences the finished voice says, which the
    /// estimate puts where the characters fall. An id that is not in the
    /// script, or onsets that are not one per sentence, is a mistake in the
    /// test and an error.
    pub fn estimate_timed(script: &Script, timed: &[(&str, &[f64], f64)]) -> Result<Rc<Self>, String> {
        let mut n = Self::scratch(script);
        for &(id, onsets, len) in timed {
            let text = script
                .text(id)
                .ok_or_else(|| format!("estimate_timed: {id} is not in the script"))?;
            let count = sentences(text).len();
            if onsets.len() != count {
                return Err(format!(
                    "estimate_timed: {id} has {count} sentences, not {}",
                    onsets.len()
                ));
            }
            n.starts.insert(id.to_owned(), onsets.to_vec());
            n.lengths.insert(id.to_owned(), len);
        }
        Ok(Rc::new(n))
    }

    /// Every chapter is its own scene whose clock starts at zero.
    pub fn reset(&self) {
        self.busy_until.set(0.0);
        *self.after.borrow_mut() = After::Start;
        *self.log.borrow_mut() = Log::default();
        self.said_at.borrow_mut().clear();
    }

    /// The lines said in this chapter so far and the scene time each began,
    /// in order: what a scratch-voice draft's cue sheets are made from (a
    /// narrated render has the exact frames in its timeline instead).
    pub fn said(&self) -> Vec<(String, f64)> {
        let mut said: Vec<(String, f64)> = self.said_at.borrow().iter().map(|(id, &at)| (id.clone(), at)).collect();
        said.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        said
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

    /// The text of line `id` in the full script.
    fn text_of(&self, id: &str) -> Option<&str> {
        self.order.iter().find(|l| l.0 == id).map(|l| l.1.as_str())
    }

    /// The id of the line before `id` in the full script.
    fn line_before(&self, id: &str) -> Option<&str> {
        let k = self.order.iter().position(|l| l.0 == id)?;
        k.checked_sub(1).map(|k| self.order[k].0.as_str())
    }

    /// The id of the line after `id` in the full script.
    fn line_after(&self, id: &str) -> Option<&str> {
        let k = self.order.iter().position(|l| l.0 == id)?;
        self.order.get(k + 1).map(|l| l.0.as_str())
    }

    /// The silence kept after the end of line `prev` before the start of line
    /// `next` when `next` is said right after `prev`, seconds: [`BREATH`]
    /// under [`LineGaps::Constant`]; under [`LineGaps::Thought`] the class of
    /// the pair ([`classify_gap`]) less the two takes' quiet edges, never under
    /// [`MIN_GAP_S`], except that contiguous coherent spans need no generic
    /// gap. A line the script does not have gets the breath.
    pub fn line_gap(&self, prev: &str, next: &str) -> f64 {
        if self.line_gaps == LineGaps::Constant {
            return BREATH;
        }
        self.class_gap(prev, next, true)
    }

    fn class_gap(&self, prev: &str, next: &str, with_tail: bool) -> f64 {
        // Only a preceding line in this scene can carry recorded phrasing
        // across the seam. Starts and authored windows retain their own rule.
        if with_tail
            && self.line_gaps == LineGaps::Thought
            && self.contiguous_cues.contains(&(prev.to_owned(), next.to_owned()))
        {
            return 0.0;
        }
        let (Some(a), Some(b)) = (self.text_of(prev), self.text_of(next)) else {
            return BREATH;
        };
        let tail = if with_tail {
            self.edges.get(prev).map_or(0.0, |e| e.1)
        } else {
            0.0
        };
        let lead = self.edges.get(next).map_or(0.0, |e| e.0);
        (classify_gap(a, b).audible_s() - tail - lead).max(MIN_GAP_S)
    }

    /// The silence before `next`, given what was said last. A chapter's first
    /// line has no take before it in this render (a partial render must place
    /// it as the full render does), so only its own lead is taken off; after
    /// an authored window the window's own breath stands.
    fn gap_before(&self, next: &str) -> f64 {
        if self.line_gaps == LineGaps::Constant {
            return BREATH;
        }
        match &*self.after.borrow() {
            After::Window => BREATH,
            After::Line(prev) => self.class_gap(prev, next, true),
            After::Start => self
                .line_before(next)
                .map_or(BREATH, |prev| self.class_gap(prev, next, false)),
        }
    }

    fn wait_gap(&self, stage: &mut Stage<'_>, gap: f64) -> fmn::Result<()> {
        let (frame, fps, _) = clock(stage);
        let target = frames_covering(self.busy_until.get() + gap, fps);
        wait_frames(stage, target.saturating_sub(frame))
    }

    /// Wait until the current line, plus the breath, has finished: what every
    /// authored window does first.
    fn hold_breath(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.wait_gap(stage, BREATH)
    }

    /// Wait until the current line, plus a breath, has finished. Under
    /// [`LineGaps::Thought`] the breath is the gap the script's next line
    /// will get when it is shorter than [`BREATH`] (a continuation), and never
    /// longer, so an authored window or tail after a `hold` keeps its place.
    pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let gap = match &*self.after.borrow() {
            After::Line(prev) if self.line_gaps == LineGaps::Thought => self
                .line_after(prev)
                .map_or(BREATH, |next| self.class_gap(prev, next, true).min(BREATH)),
            _ => BREATH,
        };
        self.wait_gap(stage, gap)
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
        self.wait_gap(stage, self.gap_before(id))?;
        let (frame, fps, now) = clock(stage);
        if let Some(prev) = self.line_before(id)
            && self.contiguous_cues.contains(&(prev.to_owned(), id.to_owned()))
            && !self.boundary_quiet.get(prev).is_some_and(|edge| edge.1)
            && !self.boundary_quiet.get(id).is_some_and(|edge| edge.0)
        {
            let log = self.log.borrow();
            let previous = log.lines.last().filter(|line| line.id == prev).ok_or_else(|| {
                fail(format!(
                    "sounding coherent seam {prev}|{id}: predecessor was skipped or reordered"
                ))
            })?;
            let file = self
                .files
                .get(prev)
                .ok_or_else(|| fail(format!("missing source for coherent seam {prev}|{id}")))?;
            let samples = (file.frames as u128 * u128::from(SAMPLE_RATE)).div_ceil(u128::from(file.rate));
            let end = u128::from(frame_to_sample(previous.start_frame, fps)) + samples;
            let start = u128::from(frame_to_sample(frame, fps));
            if start != end {
                return Err(fail(format!(
                    "sounding coherent seam {prev}|{id}: next starts at sample {start}, previous ends at {end}; require zero inserted samples; re-slice on the frame grid, fix animation overrun, or merge the cues"
                )));
            }
        }
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
        *self.after.borrow_mut() = After::Line(id.to_owned());
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
        self.hold_breath(stage)?;
        let (frame, _, now) = clock(stage);
        self.log.borrow_mut().listens.push(Window {
            frame,
            seconds,
            label: label.to_owned(),
            sound: Some(wav.to_owned()),
        });
        self.busy_until.set(now + seconds - BREATH);
        *self.after.borrow_mut() = After::Window;
        Ok(())
    }

    fn window(&self, stage: &mut Stage<'_>, seconds: f64, label: Option<&str>) -> fmn::Result<()> {
        self.hold_breath(stage)?;
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
        *self.after.borrow_mut() = After::Window;
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

    /// Run [`check_timelines_gap_spread`] across the rendered chapters when
    /// this narrator's rule is [`LineGaps::Thought`]; skipped for
    /// [`LineGaps::Constant`].
    ///
    /// # Errors
    /// Returns the error from [`check_gap_spread`] when the pooled plain line
    /// gaps across `timelines` do not vary enough.
    pub fn check_timelines<'a>(&self, timelines: impl IntoIterator<Item = &'a ChapterTimeline>) -> Result<(), String> {
        check_timelines_gap_spread(self.line_gaps, timelines, &self.lengths)
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
    #[test]
    fn coherent_unit_coverage_uses_assignments_not_metadata_mentions() {
        let script = "01_1\tLook at the square.\n01_2\tNow count its sides.\n";
        let plan = serde_json::json!({
            "schema_version": 1,
            "script_sha256": explainer_score::digest::sha256_hex(script.as_bytes()),
            "units": [{"id": "01u1", "cues": ["01_1", "01_2"],
                "line_ids": ["01_1", "01_2"],
                "text": "Look at the square. Now count its sides."}]
        });
        super::check_coherent_units(script, &plan.to_string()).unwrap();
        for ids in [
            serde_json::json!(["01_1"]),
            serde_json::json!(["01_2", "01_1"]),
            serde_json::json!(["01_1", "01_1", "01_2"]),
        ] {
            let mut wrong = plan.clone();
            wrong["units"][0]["line_ids"] = ids.clone();
            wrong["units"][0]["text"] = serde_json::Value::String(
                ids.as_array()
                    .unwrap()
                    .iter()
                    .map(|id| {
                        if id == "01_1" {
                            "Look at the square."
                        } else {
                            "Now count its sides."
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            assert!(super::check_coherent_units(script, &wrong.to_string()).is_err());
        }
        let mut wrong = plan.clone();
        wrong["units"][0]["text"] = "Changed narration.".into();
        assert!(super::check_coherent_units(script, &wrong.to_string()).is_err());
        assert!(super::check_coherent_units(&format!("{script}# changed bytes\n"), &plan.to_string()).is_err());
    }

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
    fn report_sentence_boundaries_match_the_script_even_when_joined_words_agree() {
        let script = Script::parse("01_1\tOne two. Three four.\n").unwrap();
        let complete = r#"{"lines":{"01_1":[
            {"sentence":"1 two!"},{"sentence":"Three 4?"}]}}"#;
        check_report_script(complete, &script).unwrap();
        for report in [
            r#"{"lines":{"01_1":[{"sentence":"One."},{"sentence":"Two three four."}]}}"#,
            r#"{"lines":{"01_1":[{"sentence":"One two three."},{"sentence":"Four."}]}}"#,
            r#"{"lines":{"01_1":[{"sentence":"One two. Three four."}]}}"#,
        ] {
            assert!(
                check_report_script(report, &script)
                    .unwrap_err()
                    .to_string()
                    .contains("01_1"),
                "shifted or merged sentence boundaries must refuse: {report}"
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
    fn a_timed_scratch_voice_keeps_the_real_onsets_for_its_lines_and_the_estimate_for_the_rest() {
        let script = Script::parse("01_1\tNo. It settles. Done now.\n01_2\tOne two. Three four five.\n").unwrap();
        let n = Narrator::estimate_timed(&script, &[("01_1", &[0.0, 0.82, 2.0], 3.5)]).unwrap();
        assert_eq!(n.sentence_starts("01_1"), [0.0, 0.82, 2.0]);
        assert_eq!(n.length("01_1"), Some(3.5));
        let est = Narrator::estimate(&script);
        assert_eq!(n.sentence_starts("01_2"), est.sentence_starts("01_2"));
        assert_eq!(n.length("01_2"), est.length("01_2"));
        // two onsets for three sentences, and a line the script lacks, are the test's mistakes
        let wrong_count = Narrator::estimate_timed(&script, &[("01_1", &[0.0, 0.82], 3.5)]);
        assert!(wrong_count.is_err_and(|e| e.contains("3 sentences, not 2")));
        let no_line = Narrator::estimate_timed(&script, &[("09_9", &[0.0], 1.0)]);
        assert!(no_line.is_err_and(|e| e.contains("09_9")));
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

    // ------------------------------------------------------------- line gaps

    #[test]
    fn a_boundary_is_classified_from_the_text_of_its_two_lines() {
        use GapClass::{Beat, Continuation, Topic};
        for next in [
            "And the small square is the twenty-five.",
            "But why the next number up?",
            "So leave the zeros out.",
            "Then write twenty-five.",
            "Now draw it.",
            "Because it is.",
            "Which is rare.",
            "Plus a bit.",
            "Or not.",
            "That\u{2019}s the trick.",
        ] {
            assert_eq!(classify_gap("Done.", next), Continuation, "{next}");
        }
        // a connective is the whole first word
        for next in [
            "Anders had it.",
            "Soon after, it ends.",
            "Thenceforth it holds.",
            "Nowhere else.",
        ] {
            assert_eq!(classify_gap("Done.", next), Beat, "{next}");
        }
        // a line before that ends unfinished carries on; a closing quote or
        // bracket after the end mark does not hide the end
        assert_eq!(classify_gap("Add these up", "Twelve hundred."), Continuation);
        assert_eq!(classify_gap("Why?", "Twelve hundred."), Beat);
        assert_eq!(classify_gap("He said \"stop.\"", "Twelve hundred."), Beat);
        assert_eq!(classify_gap("It is rare (see above.)", "Twelve hundred."), Beat);
        // a new example or a quiz ends the paragraph, whatever else it opens with
        for next in [
            "Your turn. Sixteen times forty-five?",
            "Take a price just under a round number.",
            "Here's that trick on a smaller number.",
            "Here\u{2019}s another.",
            "Last one. Eighty-five squared.",
            "One more.",
            "Most problems aren't that neat.",
            "The next number comes in.",
        ] {
            assert_eq!(classify_gap("Done.", next), Topic, "{next}");
        }
        // an opener is a whole phrase at the start of the line, never a prefix of a word
        for next in [
            "Takes a while.",
            "Mostly it works.",
            "Herein lies it.",
            "Your turns come later.",
            "It is your turn.",
        ] {
            assert_ne!(classify_gap("Done.", next), Topic, "{next}");
        }
    }

    /// A take as a test file: `lead` s of silence, `voiced` s of a tone, `tail` s of silence, at 48 kHz.
    fn take(lead: f64, voiced: f64, tail: f64) -> Vec<f64> {
        let n = |s: f64| (s * 48_000.0).round() as usize;
        let mut x = vec![0.0; n(lead)];
        x.extend((0..n(voiced)).map(|i| 0.2 * (i as f64 * 0.3).sin()));
        x.extend(vec![0.0; n(tail)]);
        x
    }

    #[test]
    fn quiet_edges_are_a_takes_own_silence_to_within_a_frame() {
        for rate in [24_000, 48_000] {
            let at = |s: f64| (s * f64::from(rate)).round() as usize;
            let mut x = vec![0.0; at(0.30)];
            x.extend((0..at(1.0)).map(|i| 0.2 * (i as f64 * 0.3).sin()));
            x.extend(vec![0.0; at(0.25)]);
            let (lead, tail) = quiet_edges(&x, rate);
            assert!((lead - 0.30).abs() <= 0.02, "{rate}: lead {lead}");
            assert!((tail - 0.25).abs() <= 0.02, "{rate}: tail {tail}");
        }
        // planted negatives: no silence, a lead under the 40 ms a quiet run needs,
        // nothing but silence, and a take shorter than one frame have no edges
        assert_eq!(quiet_edges(&take(0.0, 1.0, 0.0), 48_000), (0.0, 0.0));
        let (lead, tail) = quiet_edges(&take(0.03, 1.0, 0.0), 48_000);
        assert_eq!((lead, tail), (0.0, 0.0));
        assert_eq!(quiet_edges(&vec![0.0; 48_000], 48_000), (0.0, 0.0));
        assert_eq!(quiet_edges(&[0.1; 100], 48_000), (0.0, 0.0));
        // a quiet breath well under the voice counts as the take's silence, a
        // soft word 12 dB under it does not
        let before = |level: f64| {
            let mut x: Vec<f64> = (0..24_000).map(|i| level * (i as f64 * 0.7).sin()).collect();
            x.extend(take(0.0, 1.0, 0.0));
            x
        };
        let (lead, _) = quiet_edges(&before(0.0005), 48_000);
        assert!((lead - 0.5).abs() <= 0.02, "breath lead {lead}");
        assert_eq!(quiet_edges(&before(0.05), 48_000).0, 0.0, "a soft word is not silence");
    }

    /// A narration directory for `script` with the given takes and the report
    /// the narrator needs (one sentence per line).
    fn narration(tag: &str, script: &Script, takes: &[Vec<f64>]) -> PathBuf {
        let dir = tmp(tag);
        std::fs::create_dir_all(&dir).unwrap();
        let mut lines = serde_json::Map::new();
        for ((id, text), x) in script.lines().iter().zip(takes) {
            let stereo = explainer_score::Stereo::from_mono(x);
            wav::write(&dir.join(format!("{id}.wav")), &stereo, wav::Format::F32).unwrap();
            lines.insert(
                id.clone(),
                serde_json::json!([{"sentence": text, "out_samples": x.len(), "gap_after_samples": 0}]),
            );
        }
        let report = serde_json::json!({"meta": {"rate": 48_000}, "lines": lines});
        std::fs::write(dir.join("report.json"), report.to_string()).unwrap();
        dir
    }

    /// Synthetic source slices exercise the real loader, not listening quality.
    fn coherent_narration(tag: &str, script: &Script) -> (PathBuf, serde_json::Value) {
        let takes: Vec<_> = script.lines().iter().map(|_| take(0.0, 0.25, 0.0)).collect();
        let dir = narration(tag, script, &takes);
        let source = dir.join("source.wav");
        let samples: Vec<_> = takes.iter().flatten().copied().collect();
        wav::write(&source, &explainer_score::Stereo::from_mono(&samples), wav::Format::F32).unwrap();
        let hash = digest::sha256_hex(&std::fs::read(&source).unwrap());
        let mut report: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("report.json")).unwrap()).unwrap();
        report["meta"]["adapter"] = "coherent-narration-v1".into();
        for (i, (id, _)) in script.lines().iter().enumerate() {
            let row = &mut report["lines"][id][0];
            row["unit_id"] = "test-unit".into();
            row["source"] = source.to_str().unwrap().into();
            row["source_sha256"] = hash.clone().into();
            row["source_in_sample"] = (i * 12_000).into();
            row["source_out_sample"] = ((i + 1) * 12_000).into();
        }
        std::fs::write(dir.join("report.json"), report.to_string()).unwrap();
        (dir, report)
    }

    #[test]
    fn coherent_seams_require_exact_adjacent_source_provenance() {
        let script = Script::parse("01_1\tOne.\n01_2\tThen two.\n01_3\tThree.\n02_1\tFour.\n")
            .unwrap()
            .with_line_gaps(LineGaps::Thought);
        let (dir, report) = coherent_narration("coherent_provenance", &script);
        let load = |value: &serde_json::Value| {
            std::fs::write(dir.join("report.json"), value.to_string()).unwrap();
            Narrator::load(&script, &dir, 0.0).unwrap()
        };
        let valid = load(&report);
        assert_eq!(valid.line_gap("01_1", "01_2"), 0.0);
        assert_eq!(valid.line_gap("01_2", "01_3"), 0.0);
        assert_eq!(
            valid.line_gap("01_3", "02_1"),
            SENTENCE_END_S,
            "chapters cannot share a seam"
        );
        assert_eq!(
            valid.line_gap("01_1", "01_3"),
            SENTENCE_END_S,
            "script adjacency is required"
        );

        for key in ["unit_id", "source", "source_sha256"] {
            let mut changed = report.clone();
            changed["lines"]["01_2"][0][key] = if key == "source_sha256" {
                "f".repeat(64).into()
            } else {
                "different".into()
            };
            assert_eq!(load(&changed).line_gap("01_1", "01_2"), WITHIN_PARAGRAPH_S, "{key}");
        }
        // Both a hole and an overlap retain the original gap. Each changed
        // row still has a positive span accounting for its own output samples.
        for start in [11_999, 12_001] {
            let mut changed = report.clone();
            changed["lines"]["01_2"][0]["source_in_sample"] = start.into();
            changed["lines"]["01_2"][0]["source_out_sample"] = (start + 12_000).into();
            assert_eq!(load(&changed).line_gap("01_1", "01_2"), WITHIN_PARAGRAPH_S);
        }
        for adapter in [serde_json::Value::Null, "other-adapter".into()] {
            let mut changed = report.clone();
            changed["meta"]["adapter"] = adapter;
            assert_eq!(load(&changed).line_gap("01_1", "01_2"), WITHIN_PARAGRAPH_S);
        }
        // Selecting nonadjacent cues must not turn them into a coherent pair,
        // even if a malformed report claims their source positions touch.
        let partial = script.select(|id| id == "01_1" || id == "01_3");
        let mut changed = report.clone();
        changed["lines"]["01_3"][0]["source_in_sample"] = 12_000.into();
        changed["lines"]["01_3"][0]["source_out_sample"] = 24_000.into();
        assert!(
            report_contiguous_cues(&changed.to_string(), &partial)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn malformed_coherent_boundaries_never_receive_a_continuity_exemption() {
        let script = Script::parse("01_1\tOne.\n01_2\tThen two.\n").unwrap();
        let (_, report) = coherent_narration("coherent_malformed", &script);
        for key in [
            "unit_id",
            "source",
            "source_sha256",
            "source_in_sample",
            "source_out_sample",
            "out_samples",
        ] {
            let mut changed = report.clone();
            changed["lines"]["01_2"][0].as_object_mut().unwrap().remove(key);
            assert!(
                report_contiguous_cues(&changed.to_string(), &script)
                    .unwrap()
                    .is_empty(),
                "missing {key}"
            );
        }
        for (key, value) in [
            ("unit_id", serde_json::json!(" ")),
            ("source", serde_json::json!("")),
            ("source_sha256", serde_json::json!("short")),
            ("source_sha256", serde_json::json!("z".repeat(64))),
            ("source_in_sample", serde_json::json!(-1)),
            ("source_out_sample", serde_json::json!(12_000)),
            ("out_samples", serde_json::json!(0)),
            ("out_samples", serde_json::json!(11_999)),
            ("gap_after_samples", serde_json::json!(-1)),
        ] {
            let mut changed = report.clone();
            changed["lines"]["01_2"][0][key] = value;
            assert!(
                report_contiguous_cues(&changed.to_string(), &script)
                    .unwrap()
                    .is_empty(),
                "malformed {key}"
            );
        }
    }

    #[test]
    fn coherent_seams_leave_windows_starts_constant_and_scratch_gaps_alone() {
        let script = Script::parse("01_1\tOne.\n01_2\tThen two.\n")
            .unwrap()
            .with_line_gaps(LineGaps::Thought);
        let (dir, _) = coherent_narration("coherent_states", &script);
        let thought = Narrator::load(&script, &dir, 0.0).unwrap();
        assert_eq!(thought.gap_before("01_1"), BREATH);
        assert_eq!(
            thought.gap_before("01_2"),
            WITHIN_PARAGRAPH_S,
            "a start has no previous placed cue"
        );
        *thought.after.borrow_mut() = After::Line("01_1".into());
        assert_eq!(thought.gap_before("01_2"), 0.0);
        *thought.after.borrow_mut() = After::Window;
        assert_eq!(thought.gap_before("01_2"), BREATH);
        let constant = Narrator::load(&script.clone().with_line_gaps(LineGaps::Constant), &dir, 0.0).unwrap();
        *constant.after.borrow_mut() = After::Line("01_1".into());
        assert_eq!(constant.gap_before("01_2"), BREATH);
        assert_eq!(constant.line_gap("01_1", "01_2"), BREATH);
        let scratch = Narrator::estimate(&script);
        assert!(scratch.contiguous_cues.is_empty());
        assert_eq!(scratch.line_gap("01_1", "01_2"), WITHIN_PARAGRAPH_S);
    }

    #[test]
    fn a_line_gap_is_the_class_less_the_takes_own_quiet_edges() {
        let script =
            Script::parse("01_1\tOne thing.\n01_2\tThen another.\n01_3\tYour turn.\n01_4\tIt ends.\n").unwrap();
        let takes = [
            take(0.15, 1.0, 0.20),
            take(0.15, 1.0, 0.20),
            take(0.15, 1.0, 0.20),
            take(0.70, 1.0, 0.50),
        ];
        let dir = narration("gap_edges", &script, &takes);
        let thought = Narrator::load(&script.clone().with_line_gaps(LineGaps::Thought), &dir, 0.0).unwrap();
        let constant = Narrator::load(&script, &dir, 0.0).unwrap();
        let near = |got: f64, want: f64| assert!((got - want).abs() < 0.03, "{got} against {want}");
        // a continuation: 0.58 s less the 0.20 s tail and the 0.15 s lead
        near(thought.line_gap("01_1", "01_2"), WITHIN_PARAGRAPH_S - 0.35);
        // a topic: 0.99 s less the same
        near(thought.line_gap("01_2", "01_3"), PARAGRAPH_END_S - 0.35);
        // a beat whose edges are longer than the class leaves the floor, never zero
        assert_eq!(thought.line_gap("01_3", "01_4"), MIN_GAP_S);
        // the constant rule ignores text and edges; a line the script lacks gets the breath
        for (a, b) in [("01_1", "01_2"), ("01_2", "01_3"), ("01_3", "01_4")] {
            assert_eq!(constant.line_gap(a, b), BREATH);
        }
        assert_eq!(thought.line_gap("01_1", "elsewhere"), BREATH);
        // what was said last decides the rule: a window keeps its own breath, a
        // chapter's first line has no take before it in this render (a partial
        // render must place it as the whole one does) so only its lead comes off,
        // and the video's very first line has nothing before it
        *thought.after.borrow_mut() = After::Window;
        assert_eq!(thought.gap_before("01_2"), BREATH);
        *thought.after.borrow_mut() = After::Line("01_1".into());
        near(thought.gap_before("01_2"), WITHIN_PARAGRAPH_S - 0.35);
        *thought.after.borrow_mut() = After::Start;
        near(thought.gap_before("01_2"), WITHIN_PARAGRAPH_S - 0.15);
        assert_eq!(thought.gap_before("01_1"), BREATH);
        assert_eq!(constant.gap_before("01_2"), BREATH);
        // the takes' edges were measured only because the rule needed them
        assert_eq!(thought.edges.len(), 4);
        assert!(constant.edges.is_empty());
    }

    #[test]
    fn a_partial_render_places_a_chapters_first_line_as_the_whole_render_does() {
        let full = Script::parse("01_1\tOne thing.\n02_1\tThen another.\n02_2\tIt ends.\n")
            .unwrap()
            .with_line_gaps(LineGaps::Thought);
        let part = full.select(|id| id.starts_with("02_"));
        assert_eq!(part.line_gaps(), LineGaps::Thought, "select keeps the rule");
        assert_eq!(part.lines().len(), 2);
        let all = [take(0.15, 1.0, 0.20), take(0.15, 1.0, 0.20), take(0.15, 1.0, 0.20)];
        let whole = Narrator::load(&full, &narration("gap_whole", &full, &all), 0.0).unwrap();
        let some = Narrator::load(&part, &narration("gap_part", &part, &all[1..]), 0.0).unwrap();
        // 02_1 opens with "Then": a continuation across the join, though 01_1 is not loaded
        assert_eq!(whole.gap_before("02_1"), some.gap_before("02_1"));
        assert!((some.gap_before("02_1") - (WITHIN_PARAGRAPH_S - 0.15)).abs() < 0.03);
        // a script without the rule reads every pair as the breath
        let off = Narrator::load(
            &part.clone().with_line_gaps(LineGaps::Constant),
            &narration("gap_off", &part, &all[1..]),
            0.0,
        )
        .unwrap();
        assert_eq!(off.gap_before("02_1"), BREATH);
    }

    #[test]
    fn constant_gaps_fail_the_spread_check_and_class_gaps_pass_it() {
        // planted negatives: the breath plus the takes' own edges, as Video F's plain line gaps were
        let constant = [0.69, 0.70, 0.68, 0.71, 0.69, 0.70];
        assert!(
            check_gap_spread(&constant)
                .unwrap_err()
                .contains("coefficient of variation")
        );
        assert!(check_gap_spread(&[BREATH; 8]).is_err());
        assert_eq!(gap_cv(&[0.6; 8]), Some(0.0));
        assert!(check_gap_spread(&[0.5, 0.9]).unwrap_err().contains("too few"));
        assert_eq!((gap_cv(&[1.0]), gap_cv(&[0.0, 0.0])), (None, None));
        // the three classes on a mix of boundaries clear it
        let classes = [
            GapClass::Continuation,
            GapClass::Beat,
            GapClass::Beat,
            GapClass::Topic,
            GapClass::Continuation,
            GapClass::Beat,
        ];
        let gaps: Vec<f64> = classes.iter().map(|c| c.audible_s()).collect();
        check_gap_spread(&gaps).unwrap();
        assert!(gap_cv(&gaps).unwrap() > MIN_GAP_CV);
    }

    #[test]
    fn plain_gaps_leave_out_what_the_scene_waited_for() {
        // lines 2 s long: a plain 0.6 s; a 0.9 s with a window opening in it; a
        // 1.5 s picture wait; a plain 0.99 s; a plain 0.91 s. A window inside
        // the first line, or after the last, is not between any pair.
        let lines = [
            (0.0, 2.0),
            (2.6, 2.0),
            (5.5, 2.0),
            (9.0, 2.0),
            (11.99, 2.0),
            (14.9, 2.0),
        ];
        let plain = plain_line_gaps(&lines, &[1.0, 5.0, 20.0]);
        let want = [0.6, 0.99, 0.91];
        assert_eq!(plain.len(), want.len(), "{plain:?}");
        assert!(plain.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-9), "{plain:?}");
        // without the window the 0.9 s pair is plain too
        assert_eq!(plain_line_gaps(&lines, &[]).len(), 4);
        assert!(plain_line_gaps(&lines[..1], &[]).is_empty());

        // Video F under the constant rule, in miniature: 45 gaps of 0.66 to 0.72 s
        // and seven things the scene waited for. Counted whole they vary by more
        // than 0.5 and would pass; the plain ones vary by next to nothing.
        let mut f_like: Vec<f64> = (0..45u32).map(|k| 0.66 + 0.01 * f64::from(k % 7)).collect();
        f_like.extend([2.6, 2.6, 1.5, 1.54, 1.4, 6.0, 4.0]);
        assert!(gap_cv(&f_like).unwrap() > 0.5, "the waits hide the constant breath");
        let err = check_gap_spread(&f_like).unwrap_err();
        assert!(err.contains("45 plain line gaps vary by"), "{err}");
        // the classes, with the same waits, still pass
        let mut thought: Vec<f64> = [0.58, 0.81, 0.99, 0.58, 0.81, 0.81, 0.58, 0.99]
            .iter()
            .cycle()
            .take(45)
            .copied()
            .collect();
        thought.extend([2.6, 2.6, 1.5, 1.54, 1.4, 6.0, 4.0]);
        check_gap_spread(&thought).unwrap();
        // the limit: 0.99 s counts, 1.0 s does not
        check_gap_spread(&[0.7, 0.7, 0.7, 0.99]).unwrap();
        assert!(check_gap_spread(&[0.7, 0.7, 0.7, 1.0]).is_err());
        let few = check_gap_spread(&[0.4, 0.6, 1.0, 1.2]).unwrap_err();
        assert!(few.contains("2 plain line gaps (of 4 given"), "{few}");
    }

    #[test]
    fn check_timelines_gap_spread_skips_constant_and_verifies_thought_on_video_f_shape() {
        let build_tl = |gaps: &[f64], windows_at: &[usize]| -> (ChapterTimeline, HashMap<String, f64>) {
            let fps = 60u32;
            let mut lengths = HashMap::new();
            let mut lines = Vec::new();
            let mut listens = Vec::new();
            let mut t = 0.6f64;
            for (k, &g) in std::iter::once(&0.0).chain(gaps.iter()).enumerate() {
                if k > 0 {
                    if windows_at.contains(&(k - 1)) {
                        listens.push(Window {
                            frame: (t * f64::from(fps)).round() as u64,
                            seconds: g,
                            label: "quiz".into(),
                            sound: None,
                        });
                    }
                    t += g;
                }
                let id = format!("01_{}", k + 1);
                let len = 2.0;
                lengths.insert(id.clone(), len);
                lines.push(LinePlacement {
                    id,
                    start_frame: (t * f64::from(fps)).round() as u64,
                    gain_db: 0.0,
                    wav: PathBuf::from("line.wav"),
                });
                t += len;
            }
            let tl = ChapterTimeline {
                chapter: "01_test".into(),
                fps,
                frames: ((t + 2.0) * f64::from(fps)).round() as u64,
                lines,
                marks: Vec::new(),
                breaths: Vec::new(),
                listens,
            };
            (tl, lengths)
        };

        // Video F under Constant: 45 plain gaps ~0.66..0.72 s (CV ~0.03) plus 2 windowed gaps and 2 picture waits
        let mut f_constant: Vec<f64> = (0..45u32).map(|k| 0.66 + 0.01 * f64::from(k % 7)).collect();
        f_constant.extend([0.85, 0.90, 2.6, 1.54]);
        let (tl_const, len_const) = build_tl(&f_constant, &[45, 46]);
        let plain_const = timeline_plain_line_gaps(&tl_const, &len_const);
        assert_eq!(plain_const.len(), 45);
        let cv_const = gap_cv(&plain_const).unwrap();
        assert!(cv_const < 0.05, "expected Constant CV ~0.03..0.04, got {cv_const}");
        // Constant-rule video is skipped by design
        assert!(check_timelines_gap_spread(LineGaps::Constant, [&tl_const], &len_const).is_ok());
        // Planted negative: claiming LineGaps::Thought on a Constant-spaced timeline fails check_gap_spread
        let err = check_timelines_gap_spread(LineGaps::Thought, [&tl_const], &len_const).unwrap_err();
        assert!(err.contains("45 plain line gaps vary by"), "{err}");

        // Video F under Thought: 16 CONTINUATION (~0.58 s), 25 BEAT (~0.81 s), 6 TOPIC (~0.97 s), CV ~0.22
        let mut f_thought = Vec::new();
        f_thought.extend((0..16u32).map(|k| 0.55 + 0.01 * f64::from(k % 5)));
        f_thought.extend((0..25u32).map(|k| 0.78 + 0.01 * f64::from(k % 6)));
        f_thought.extend((0..6u32).map(|k| 0.95 + 0.005 * f64::from(k % 4)));
        f_thought.extend([0.85, 0.90, 2.6, 1.54]);
        let (tl_thought, len_thought) = build_tl(&f_thought, &[47, 48]);
        let plain_thought = timeline_plain_line_gaps(&tl_thought, &len_thought);
        assert_eq!(plain_thought.len(), 47);
        let cv_thought = gap_cv(&plain_thought).unwrap();
        assert!(
            (0.16..=0.28).contains(&cv_thought),
            "expected Thought CV ~0.22, got {cv_thought}"
        );
        check_timelines_gap_spread(LineGaps::Thought, [&tl_thought], &len_thought).unwrap();
    }
}
