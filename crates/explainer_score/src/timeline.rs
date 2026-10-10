//! A chapter's narration timeline: where every line was placed (integer
//! frames, never rounded seconds), authored marks, music breaths and listen
//! windows. The score engine reads it to know exactly when the voice speaks.
//!
//! Text format (one record per line, tab-separated; `#` comments):
//!
//! ```text
//! chapter <name>
//! fps     60
//! frames  5123
//! line    <id>  <start_frame>  <gain_db>  <wav path>
//! mark    <label>  <frame>
//! breath  <frame>  <seconds>
//! listen  <frame>  <seconds>  <label>
//! ```

use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

use crate::resample::resample;
use crate::units::{db_to_gain, frame_to_sample};
use crate::{SAMPLE_RATE, Stereo};

/// One narration line as placed in the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct LinePlacement {
    pub id: String,
    pub start_frame: u64,
    /// The gain the scene placed the line at (dB).
    pub gain_db: f64,
    pub wav: PathBuf,
}

/// An authored event (a reveal) the music may answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub label: String,
    pub frame: u64,
}

/// A span with no narration the score may fill (`breath`), or one reserved for
/// a teaching sonification at full level (`listen`).
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub frame: u64,
    pub seconds: f64,
    pub label: String,
}

/// Everything the score needs to know about one chapter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterTimeline {
    pub chapter: String,
    pub fps: u32,
    pub frames: u64,
    pub lines: Vec<LinePlacement>,
    pub marks: Vec<Mark>,
    pub breaths: Vec<Window>,
    pub listens: Vec<Window>,
}

fn bad(line: usize, msg: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("timeline line {line}: {msg}"))
}

impl ChapterTimeline {
    /// Total samples at 48 kHz: exactly the scene's length.
    pub fn samples(&self) -> usize {
        frame_to_sample(self.frames, self.fps) as usize
    }

    pub fn seconds(&self) -> f64 {
        self.frames as f64 / f64::from(self.fps)
    }

    /// Serialize to the text format.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# explainer timeline v1");
        let _ = writeln!(s, "chapter\t{}", self.chapter);
        let _ = writeln!(s, "fps\t{}", self.fps);
        let _ = writeln!(s, "frames\t{}", self.frames);
        for l in &self.lines {
            let _ = writeln!(
                s,
                "line\t{}\t{}\t{}\t{}",
                l.id,
                l.start_frame,
                l.gain_db,
                l.wav.display()
            );
        }
        for m in &self.marks {
            let _ = writeln!(s, "mark\t{}\t{}", m.label, m.frame);
        }
        for w in &self.breaths {
            let _ = writeln!(s, "breath\t{}\t{}", w.frame, w.seconds);
        }
        for w in &self.listens {
            let _ = writeln!(s, "listen\t{}\t{}\t{}", w.frame, w.seconds, w.label);
        }
        s
    }

    /// Parse the text format; every malformed line is an error with its number.
    pub fn parse(text: &str) -> io::Result<Self> {
        let mut t = Self::default();
        for (i, raw) in text.lines().enumerate() {
            let n = i + 1;
            let line = raw.trim_end_matches('\r');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            let num = |k: usize| -> io::Result<f64> {
                f.get(k)
                    .ok_or_else(|| bad(n, "missing field"))?
                    .parse()
                    .map_err(|_| bad(n, "bad number"))
            };
            let int = |k: usize| -> io::Result<u64> {
                f.get(k)
                    .ok_or_else(|| bad(n, "missing field"))?
                    .parse()
                    .map_err(|_| bad(n, "bad integer"))
            };
            match f[0] {
                "chapter" => t.chapter = f.get(1).ok_or_else(|| bad(n, "missing name"))?.to_string(),
                "fps" => t.fps = int(1)? as u32,
                "frames" => t.frames = int(1)?,
                "line" => {
                    if f.len() != 5 {
                        return Err(bad(n, "line needs id, start_frame, gain_db, path"));
                    }
                    t.lines.push(LinePlacement {
                        id: f[1].to_string(),
                        start_frame: int(2)?,
                        gain_db: num(3)?,
                        wav: PathBuf::from(f[4]),
                    })
                }
                "mark" => t.marks.push(Mark {
                    label: f.get(1).unwrap_or(&"").to_string(),
                    frame: int(2)?,
                }),
                "breath" => t.breaths.push(Window {
                    frame: int(1)?,
                    seconds: num(2)?,
                    label: String::new(),
                }),
                "listen" => t.listens.push(Window {
                    frame: int(1)?,
                    seconds: num(2)?,
                    label: f.get(3).unwrap_or(&"").to_string(),
                }),
                other => return Err(bad(n, format!("unknown record {other:?}"))),
            }
        }
        if t.fps == 0 {
            return Err(bad(0, "missing fps"));
        }
        let mut seen = std::collections::HashSet::new();
        for l in &t.lines {
            if !seen.insert(&l.id) {
                return Err(bad(0, format!("duplicate line id {}", l.id)));
            }
        }
        Ok(t)
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        Self::parse(&std::fs::read_to_string(path)?)
    }

    /// Build the voice bus: every line decoded, resampled to 48 kHz, placed at
    /// its start frame with `gain_db` (or `gain_override_db`), the chapter's
    /// exact length. Lines that run past the end are cut, as the scene would.
    pub fn voice_bus(&self, gain_override_db: Option<f64>) -> io::Result<Stereo> {
        let mut bus = Stereo::silence(self.samples());
        for l in &self.lines {
            let d = crate::wav::read(&l.wav)?;
            let mono = resample(&d.mono(), d.rate, SAMPLE_RATE);
            let g = db_to_gain(gain_override_db.unwrap_or(l.gain_db));
            let at = frame_to_sample(l.start_frame, self.fps) as usize;
            bus.mix_in(&Stereo::from_mono(&mono), at, g);
        }
        Ok(bus)
    }

    /// Sample ranges [start, end) of each line's audio (for planning).
    pub fn line_spans(&self) -> io::Result<Vec<(usize, usize)>> {
        self.lines
            .iter()
            .map(|l| {
                let d = crate::wav::read(&l.wav)?;
                let len = (d.frames() as f64 * f64::from(SAMPLE_RATE) / f64::from(d.rate)).round() as usize;
                let at = frame_to_sample(l.start_frame, self.fps) as usize;
                Ok((at, at + len))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ChapterTimeline {
        ChapterTimeline {
            chapter: "07_shuffle".into(),
            fps: 60,
            frames: 5_123,
            lines: vec![
                LinePlacement {
                    id: "s1".into(),
                    start_frame: 0,
                    gain_db: 0.0,
                    wav: "n/s1.wav".into(),
                },
                LinePlacement {
                    id: "s2".into(),
                    start_frame: 431,
                    gain_db: -9.4,
                    wav: "n/s2.wav".into(),
                },
            ],
            marks: vec![Mark {
                label: "reveal".into(),
                frame: 900,
            }],
            breaths: vec![Window {
                frame: 1_000,
                seconds: 2.5,
                label: String::new(),
            }],
            listens: vec![Window {
                frame: 2_000,
                seconds: 3.0,
                label: "shuffle".into(),
            }],
        }
    }

    #[test]
    fn text_round_trip() {
        let t = sample();
        assert_eq!(ChapterTimeline::parse(&t.to_text()).unwrap(), t);
        assert_eq!(t.samples(), 5_123 * 800);
    }

    #[test]
    fn malformed_timelines_are_refused_with_a_line_number() {
        assert!(
            ChapterTimeline::parse("fps\t60\nline\ts1\t0\t0\n")
                .unwrap_err()
                .to_string()
                .contains("line 2")
        );
        assert!(ChapterTimeline::parse("fps\tsixty\n").is_err());
        assert!(ChapterTimeline::parse("chapter\tx\n").is_err(), "missing fps");
        assert!(ChapterTimeline::parse("fps\t60\nwhat\t1\n").is_err());
        let dup = "fps\t60\nline\ta\t0\t0\tx.wav\nline\ta\t5\t0\ty.wav\n";
        assert!(
            ChapterTimeline::parse(dup)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
    }

    #[test]
    fn voice_bus_places_lines_at_exact_samples() {
        let dir = std::env::temp_dir().join(format!("explainer_score_tl_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // a 24 kHz mono click line, as ftts writes
        let mut click = Stereo::silence(2_400);
        click.l[0] = 0.5;
        click.r[0] = 0.5;
        let p = dir.join("c.wav");
        crate::wav::write(&p, &click, crate::wav::Format::F32).unwrap();
        let t = ChapterTimeline {
            chapter: "x".into(),
            fps: 60,
            frames: 120,
            lines: vec![LinePlacement {
                id: "c".into(),
                start_frame: 30,
                gain_db: -6.0,
                wav: p,
            }],
            ..Default::default()
        };
        let bus = t.voice_bus(None).unwrap();
        assert_eq!(bus.len(), 96_000);
        let at = 30 * 800;
        assert!(bus.l[..at].iter().all(|v| *v == 0.0));
        assert!((bus.l[at] - 0.5 * db_to_gain(-6.0)).abs() < 1e-9);
        assert_eq!(t.line_spans().unwrap(), vec![(at, at + 2_400)]);
    }
}
