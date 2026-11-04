//! A stand-in for the part of explainer_kit that chapter 11 calls. Its narrator
//! has no audio: each cue lasts words / rate seconds, and sentences start by
//! share of characters, the house rule. So its clock is an estimate of the real
//! one, and its text boxes are estimates of real glyph widths.
use fmn::prelude::*;
use fmn::stub;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;
use std::rc::Rc;

pub mod narration {
    use fmn::prelude::*;

    pub fn frames_covering(seconds: f64, fps: u32) -> u64 {
        (seconds * f64::from(fps)).ceil() as u64
    }

    pub fn wait_frames(_: &mut Stage<'_>, frames: u64) -> fmn::Result<()> {
        fmn::stub::advance(frames as f64 / 60.0);
        Ok(())
    }
}

/// The silence the kit leaves between two cues when the picture is already done.
const GAP: f64 = 0.4;

pub trait Timed: Sized {
    fn rt(self, run_time: f64) -> Self;
}

impl Timed for fmn::Anim {
    fn rt(self, run_time: f64) -> Self {
        fmn::Anim { run_time, ..self }
    }
}

#[macro_export]
macro_rules! play {
    ($stage:ident; $($a:expr),+ $(,)?) => {{
        let mut v: Vec<Box<dyn fmn::prelude::Animation>> = Vec::new();
        $( { let anim = $a; v.push($stage.prepare(anim)?); } )+
        $stage.play_prepared(v)?;
    }};
}

pub fn v(x: f64, y: f64) -> Vec3 {
    Vec3 { x, y }
}

pub fn hex(_: &str) -> Srgb {
    Srgb
}

pub fn ink_style(_: Srgb) -> Style {
    Style::default()
}

pub fn stroke_style(_: Srgb, _: f64) -> Style {
    Style::default()
}

pub struct Palette;

impl Palette {
    pub fn ink() -> Srgb {
        Srgb
    }
    pub fn dim() -> Srgb {
        Srgb
    }
    pub fn faint() -> Srgb {
        Srgb
    }
    pub fn hero() -> Srgb {
        Srgb
    }
    pub fn blue() -> Srgb {
        Srgb
    }
    pub fn green() -> Srgb {
        Srgb
    }
    pub fn gold() -> Srgb {
        Srgb
    }
    pub fn red() -> Srgb {
        Srgb
    }
}

#[derive(Default)]
pub struct Facts {
    pub rows: Vec<(String, String)>,
}

impl Facts {
    pub fn add(&mut self, key: &str, value: impl Display) -> Result<(), String> {
        self.rows.push((key.to_owned(), value.to_string()));
        Ok(())
    }
}

/// One cue's timing: the voice, the picture, and its latest beat.
#[derive(Clone, Debug, Default)]
pub struct CueReport {
    pub id: String,
    pub start: f64,
    pub length: f64,
    pub picture_end: f64,
    pub latest: f64,
    pub latest_sentence: usize,
}

#[derive(Default)]
struct Voice {
    rate: f64,
    lines: HashMap<String, String>,
    order: Vec<String>,
    said_at: HashMap<String, f64>,
    busy_until: f64,
    current: String,
    reports: Vec<CueReport>,
    overlaps: BTreeSet<String>,
}

#[derive(Clone)]
pub struct Narrator {
    voice: Rc<RefCell<Voice>>,
}

fn sentence_shares(text: &str) -> Vec<f64> {
    let bytes = text.as_bytes();
    let mut shares = vec![0.0];
    let total = text.chars().count() as f64;
    for (at, &byte) in bytes.iter().enumerate() {
        if matches!(byte, b'.' | b'?' | b'!') && at + 1 < bytes.len() && bytes[at + 1] == b' ' {
            shares.push(text[..at + 2].chars().count() as f64 / total);
        }
    }
    shares
}

impl Narrator {
    pub fn length(&self, id: &str) -> Option<f64> {
        let voice = self.voice.borrow();
        let text = voice.lines.get(id)?;
        Some(text.split_whitespace().count() as f64 / voice.rate)
    }
    pub fn sentence_at(&self, id: &str, j: usize) -> Option<f64> {
        let start = *self.voice.borrow().said_at.get(id)?;
        let length = self.length(id)?;
        let voice = self.voice.borrow();
        let shares = sentence_shares(voice.lines.get(id)?);
        Some(start + shares.get(j)? * length)
    }
    pub fn sentence_starts(&self, id: &str) -> Vec<f64> {
        let voice = self.voice.borrow();
        let start = voice.said_at.get(id).copied().unwrap_or(0.0);
        drop(voice);
        let length = self.length(id).unwrap_or(0.0);
        let voice = self.voice.borrow();
        sentence_shares(&voice.lines[id]).iter().map(|share| start + share * length).collect()
    }
    pub fn phrase_marks(&self, _: &str) -> HashMap<usize, f64> {
        HashMap::new()
    }
}

#[derive(Clone)]
pub struct Kit {
    pub narrator: Option<Narrator>,
}

/// Width in ems, from rough Computer Modern advance widths.
fn ems(character: char) -> f64 {
    match character {
        'i' | 'l' | 'j' | '.' | ',' | ':' | ';' | '\'' | '!' => 0.28,
        't' | 'f' | 'r' | '(' | ')' | '-' => 0.38,
        ' ' => 0.33,
        'm' | 'w' => 0.81,
        'A'..='Z' => 0.72,
        '+' | '=' | '×' => 0.78,
        _ => 0.5,
    }
}

fn text_width(text: &str, size: f64) -> f64 {
    text.chars().map(ems).sum::<f64>() * size * 0.01
}

/// A rough plain-text reading of the TeX, for its width only.
fn tex_width(source: &str, size: f64) -> f64 {
    let mut total = 0.0;
    let mut scale = 1.0;
    let mut letters = source.chars().peekable();
    while let Some(character) = letters.next() {
        match character {
            '\\' => {
                let mut command = String::new();
                while letters.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                    command.push(letters.next().unwrap_or(' '));
                }
                if command.is_empty() {
                    letters.next();
                    total += 0.28 * scale;
                } else {
                    total += match command.as_str() {
                        "text" => 0.0,
                        "quad" => 1.0 * scale,
                        "qquad" => 2.0 * scale,
                        "Longrightarrow" => 1.7 * scale,
                        _ => 0.8 * scale,
                    };
                }
            }
            '^' | '_' => scale = 0.7,
            '{' => {}
            '}' => scale = 1.0,
            other => {
                total += ems(other) * scale;
                if scale < 1.0 && letters.peek() != Some(&'{') {
                    scale = 0.7;
                }
            }
        }
    }
    total * size * 0.01
}

impl Kit {
    pub fn with_script(script: &str, rate: f64) -> Kit {
        let mut voice = Voice {
            rate,
            ..Voice::default()
        };
        for row in script.lines() {
            if let Some((id, text)) = row.split_once('\t') {
                voice.lines.insert(id.to_owned(), text.to_owned());
                if id.starts_with("11_") {
                    voice.order.push(id.to_owned());
                }
            }
        }
        Kit {
            narrator: Some(Narrator {
                voice: Rc::new(RefCell::new(voice)),
            }),
        }
    }

    fn voice(&self) -> Rc<RefCell<Voice>> {
        match &self.narrator {
            Some(narrator) => narrator.voice.clone(),
            None => unreachable!("the stub kit always has a narrator"),
        }
    }

    /// Every pair of on-screen labels whose boxes meet, and any box off the frame.
    fn lint(&self) {
        let voice = self.voice();
        let shown: Vec<stub::Record> = stub::with(|book| {
            book.mobs.iter().filter(|record| record.life == stub::Life::Shown).cloned().collect()
        });
        let place = stub::with(|book| book.place.clone());
        let mut found = Vec::new();
        for (index, a) in shown.iter().enumerate() {
            let Some(name) = &a.text else { continue };
            if a.x.abs() + a.w / 2.0 > 7.0 || a.y.abs() + a.h / 2.0 > 3.9 {
                found.push(format!("{place}: {name:?} leaves the frame (x {:.2}, half width {:.2}, y {:.2})", a.x, a.w / 2.0, a.y));
            }
            for (other, b) in shown.iter().enumerate() {
                if other == index || (b.text.is_some() && other < index) {
                    continue;
                }
                let gap_x = (a.x - b.x).abs() - (a.w + b.w) / 2.0;
                let gap_y = (a.y - b.y).abs() - (a.h + b.h) / 2.0;
                if gap_x >= -0.01 || gap_y >= -0.01 {
                    continue;
                }
                // A number inside its own strip cell or ring is drawn that way.
                let inside = (a.x - b.x).abs() <= b.w / 2.0 && (a.y - b.y).abs() <= b.h / 2.0;
                if b.text.is_none() && inside {
                    continue;
                }
                let them = match &b.text {
                    Some(text) => format!("{text:?}"),
                    None => format!("a {:.2}x{:.2} shape at ({:.2}, {:.2})", b.w, b.h, b.x, b.y),
                };
                found.push(format!("{place}: {name:?} meets {them}"));
            }
        }
        voice.borrow_mut().overlaps.extend(found);
    }

    pub fn text(&self, _: &mut Stage<'_>, s: &str, size: f64, _: Srgb, at: Vec3) -> fmn::Result<Mob> {
        Ok(stub::add(at.x, at.y, text_width(s, size), 0.9 * size * 0.01, Some(s.to_owned())))
    }

    pub fn tex(&self, _: &mut Stage<'_>, src: &str, size: f64, _: Srgb, _: &[(&str, Srgb)], at: Vec3) -> fmn::Result<Mob> {
        Ok(stub::add(at.x, at.y, tex_width(src, size), 1.1 * size * 0.01, Some(src.to_owned())))
    }

    pub fn say(&self, _: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        self.lint();
        let voice = self.voice();
        let now = stub::now();
        let mut state = voice.borrow_mut();
        if let Some(last) = state.reports.last_mut() {
            last.picture_end = now;
        }
        if state.said_at.contains_key(id) {
            return Err(fmn::Error(format!("{id} said twice")));
        }
        let expected = state.order.get(state.reports.len()).cloned();
        if expected.as_deref() != Some(id) {
            return Err(fmn::Error(format!("said {id}, script row is {expected:?}")));
        }
        let Some(text) = state.lines.get(id) else {
            return Err(fmn::Error(format!("{id} is not in script.tsv")));
        };
        let length = text.split_whitespace().count() as f64 / state.rate;
        let start = if state.reports.is_empty() {
            now
        } else {
            now.max(state.busy_until + GAP)
        };
        stub::advance(start - now);
        state.said_at.insert(id.to_owned(), start);
        state.busy_until = start + length;
        state.current = id.to_owned();
        state.reports.push(CueReport {
            id: id.to_owned(),
            start,
            length,
            ..CueReport::default()
        });
        stub::set_place(id);
        Ok(())
    }

    pub fn hold(&self, _: &mut Stage<'_>) -> fmn::Result<()> {
        self.lint();
        let busy = self.voice().borrow().busy_until;
        let now = stub::now();
        if busy > now {
            stub::advance(busy - now);
        }
        Ok(())
    }

    pub fn until_sentence(&self, _: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        self.lint();
        let Some(narrator) = &self.narrator else {
            return Ok(());
        };
        let Some(at) = narrator.sentence_at(id, j) else {
            return Err(fmn::Error(format!("{id} has no sentence {j}, or is not said yet")));
        };
        if self.voice().borrow().current != id {
            stub::problem(format!("waits on {id} while another cue speaks"));
        }
        let now = stub::now();
        if at > now {
            stub::advance(at - now);
        } else if let Some(report) = self.voice().borrow_mut().reports.last_mut()
            && now - at > report.latest
        {
            report.latest = now - at;
            report.latest_sentence = j;
        }
        Ok(())
    }

    pub fn finish(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.hold(stage)?;
        if let Some(last) = self.voice().borrow_mut().reports.last_mut() {
            last.picture_end = stub::now();
        }
        Ok(())
    }

    pub fn reports(&self) -> Vec<CueReport> {
        self.voice().borrow().reports.clone()
    }

    pub fn overlaps(&self) -> Vec<String> {
        self.voice().borrow().overlaps.iter().cloned().collect()
    }
}
