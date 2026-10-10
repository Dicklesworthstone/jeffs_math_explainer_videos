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

impl Narrator {
    /// Real narration from `dir/<id>.wav` for every line of `script`, placed
    /// at `gain_db` ([`GAIN_DB`] under the new score). Every file is checked
    /// now: a missing or unusable line is an error before anything renders.
    pub fn load(script: &Script, dir: &Path, gain_db: f64) -> io::Result<Rc<Self>> {
        let mut files = HashMap::new();
        let mut lengths = HashMap::new();
        for (id, _) in script.lines() {
            let f = inspect_cue(&dir.join(format!("{id}.wav")))?;
            lengths.insert(id.clone(), f.seconds());
            files.insert(id.clone(), f);
        }
        Ok(Rc::new(Self {
            dir: Some(dir.to_path_buf()),
            files,
            lengths,
            gain_db,
            busy_until: Cell::new(0.0),
            log: RefCell::new(Log::default()),
        }))
    }

    /// A scratch voice for drafts: lengths from word counts, no audio.
    pub fn estimate(script: &Script) -> Rc<Self> {
        let lengths = script
            .lines()
            .iter()
            .map(|(id, text)| (id.clone(), text.split_whitespace().count() as f64 / SCRATCH_WPS + 0.4))
            .collect();
        Rc::new(Self {
            dir: None,
            files: HashMap::new(),
            lengths,
            gain_db: GAIN_DB,
            busy_until: Cell::new(0.0),
            log: RefCell::new(Log::default()),
        })
    }

    /// Every chapter is its own scene whose clock starts at zero.
    pub fn reset(&self) {
        self.busy_until.set(0.0);
        *self.log.borrow_mut() = Log::default();
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
    /// # Panics
    /// If `id` is not in the script (a typo must fail the render, never drop
    /// a line silently), or was already said in this chapter (line ids are
    /// the join keys of the timeline, captions and QA: give a repeat its own
    /// id in script.tsv).
    pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        let len = self
            .length(id)
            .unwrap_or_else(|| panic!("narration line {id:?} is not in script.tsv"));
        assert!(
            !self.log.borrow().lines.iter().any(|l| l.id == id),
            "narration line {id:?} is said twice in one chapter; give the repeat its own id in script.tsv"
        );
        self.hold(stage)?;
        let (frame, _, now) = clock(stage);
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
