//! A stand-in for the part of explainer_kit that chapters 01 to 04 call. Its narrator
//! has no audio: each cue lasts words / rate seconds, and sentences start by share of
//! characters, the house rule for the scratch voice. So its clock is an estimate, and
//! its text boxes are estimates of real glyph widths. `Facts` is the real source file.
use fmn::prelude::*;
use fmn::stub;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[path = "../../../../../../../crates/explainer_kit/src/facts.rs"]
pub mod facts;
pub use facts::Facts;

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
    [x, y, 0.0]
}

pub fn tween(tracker: Mob, to: f64, run_time: f64) -> fmn::Result<fmn::Anim> {
    Ok(fmn::Anim {
        run_time,
        effects: vec![fmn::Effect::Track(tracker, to)],
    })
}

pub fn thousands(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub struct Palette;

impl Palette {
    pub fn ink() -> Srgb {
        Srgb("ink")
    }
    pub fn dim() -> Srgb {
        Srgb("dim")
    }
    pub fn faint() -> Srgb {
        Srgb("faint")
    }
    pub fn hero() -> Srgb {
        Srgb("hero")
    }
    pub fn blue() -> Srgb {
        Srgb("blue")
    }
    pub fn teal() -> Srgb {
        Srgb("teal")
    }
    pub fn green() -> Srgb {
        Srgb("green")
    }
    pub fn red() -> Srgb {
        Srgb("red")
    }
    pub fn purple() -> Srgb {
        Srgb("purple")
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
}

pub struct Narrator {
    voice: RefCell<Voice>,
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
        let Some(text) = voice.lines.get(id) else {
            return Vec::new();
        };
        let shares = sentence_shares(text);
        drop(voice);
        let length = self.length(id).unwrap_or(0.0);
        shares.iter().map(|share| share * length).collect()
    }
    pub fn phrase_marks(&self, _: &str) -> HashMap<usize, f64> {
        HashMap::new()
    }
}

#[derive(Clone)]
pub struct Kit {
    pub narrator: Option<Rc<Narrator>>,
    moments: Rc<RefCell<Vec<(String, f64)>>>,
}

/// Width in ems: Computer Modern Roman 10 advance widths.
fn ems(character: char) -> f64 {
    match character {
        'a' | 'o' | 'g' | '0'..='9' | '½' => 0.5,
        'b' | 'd' | 'h' | 'n' | 'p' | 'u' => 0.556,
        'c' | 'e' | 'z' => 0.444,
        'f' | 'j' => 0.306,
        'i' | 'l' | '.' | ',' | ':' | ';' | '\'' | '!' => 0.278,
        'k' | 'q' | 'v' | 'x' | 'y' => 0.528,
        'm' | '%' => 0.833,
        'r' => 0.392,
        's' => 0.394,
        't' | '(' | ')' => 0.389,
        'w' => 0.722,
        ' ' => 0.333,
        '?' => 0.472,
        '+' | '=' | '×' | '÷' | '−' | '≈' | '√' => 0.778,
        '→' => 1.0,
        '₂' => 0.4,
        'A' | 'H' | 'N' | 'U' | 'V' | 'X' | 'Y' => 0.75,
        'B' | 'P' | 'E' => 0.69,
        'C' | 'T' => 0.722,
        'D' => 0.764,
        'F' => 0.653,
        'G' | 'O' | 'Q' | 'K' => 0.78,
        'I' => 0.361,
        'J' => 0.514,
        'L' | 'Z' => 0.62,
        'M' => 0.917,
        'R' => 0.736,
        'S' => 0.556,
        'W' => 1.028,
        _ => 0.5,
    }
}

pub fn text_width(text: &str, size: f64) -> f64 {
    text.chars().map(ems).sum::<f64>() * size * 0.01
}

/// The ink height of a line, in scene units: digits and ascenders reach about 0.69 em,
/// lower-case bodies 0.43 em, and descenders drop 0.19 em.
pub fn text_height(text: &str, size: f64) -> f64 {
    let tall = text.chars().any(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || "bdfhklt(/%?√½×=+".contains(c));
    let top = if tall { 0.69 } else { 0.43 };
    let bottom = if text.chars().any(|c| "gjpqy,;()".contains(c)) { 0.19 } else { 0.0 };
    (top + bottom) * size * 0.01
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
                        "text" | "mathrm" | "tfrac" | "frac" => 0.0,
                        "ln" => 0.83 * scale,
                        "log" => 1.28 * scale,
                        "quad" => 1.0 * scale,
                        "qquad" => 2.0 * scale,
                        "Rightarrow" => 1.0 * scale,
                        _ => 0.78 * scale,
                    };
                }
            }
            '^' | '_' => scale = 0.7,
            '{' => {}
            '}' => scale = 1.0,
            other => total += ems(other) * scale,
        }
    }
    total * size * 0.01
}

fn add_text(s: &str, width: f64, height: f64, color: Srgb, at: Vec3) -> Mob {
    stub::add(fmn::Shape {
        style: Style::default().fill(color, 1.0),
        text: Some(s.to_owned()),
        boxed: Some([at[0], at[1], width, height]),
        ..fmn::Shape::default()
    })
}

impl Kit {
    pub fn with_script(script: &str, rate: f64, prefix: &str) -> Kit {
        let mut voice = Voice {
            rate,
            ..Voice::default()
        };
        for row in script.lines() {
            if let Some((id, text)) = row.split_once('\t') {
                voice.lines.insert(id.to_owned(), text.to_owned());
                if id.starts_with(prefix) {
                    voice.order.push(id.to_owned());
                }
            }
        }
        Kit {
            narrator: Some(Rc::new(Narrator {
                voice: RefCell::new(voice),
            })),
            moments: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn voice(&self) -> &RefCell<Voice> {
        match &self.narrator {
            Some(narrator) => &narrator.voice,
            None => unreachable!("the stub kit always has a narrator"),
        }
    }

    pub fn text(&self, _: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        Ok(add_text(s, text_width(s, size), text_height(s, size), color, at))
    }

    pub fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let m = self.text(stage, s, size, color, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }

    pub fn tex(
        &self,
        _: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        _: &[(&str, Srgb)],
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let height = if src.contains("frac") { 2.0 } else { 1.0 } * size * 0.01;
        Ok(add_text(src, tex_width(src, size), height, color, at))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tex_left(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        t2c: &[(&str, Srgb)],
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let m = self.tex(stage, src, size, color, t2c, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn live_value(
        &self,
        stage: &mut Stage<'_>,
        f: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
        places: usize,
        _: bool,
        size: f64,
        color: Srgb,
        at: Vec3,
        edge: Vec3,
    ) -> fmn::Result<Mob> {
        let m = stage.always_redraw(move |s| {
            let shown = format!("{:.*}", places, f(s));
            let m = add_text(&shown, text_width(&shown, size), text_height(&shown, size), color, at);
            s.move_to(m, at, edge);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }

    fn moment(&self, cue: &str, sentence: &str) {
        stub::snapshot(cue, sentence);
    }

    pub fn say(&self, _: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        let previous = self.voice().borrow().current.clone();
        self.moment(&previous, "before-next");
        let now = stub::now();
        let mut state = self.voice().borrow_mut();
        if let Some(last) = state.reports.last_mut() {
            last.picture_end = now;
        }
        if state.said_at.contains_key(id) {
            return Err(fmn::Error::Stub(format!("{id} said twice")));
        }
        let expected = state.order.get(state.reports.len()).cloned();
        if expected.as_deref() != Some(id) {
            return Err(fmn::Error::Stub(format!("said {id}, script row is {expected:?}")));
        }
        let Some(text) = state.lines.get(id) else {
            return Err(fmn::Error::Stub(format!("{id} is not in script.tsv")));
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
        drop(state);
        stub::set_place(id);
        self.moments.borrow_mut().push((id.to_owned(), start));
        self.moment(id, "0");
        Ok(())
    }

    pub fn hold(&self, _: &mut Stage<'_>) -> fmn::Result<()> {
        let busy = self.voice().borrow().busy_until;
        let now = stub::now();
        if busy > now {
            stub::advance(busy - now);
        }
        Ok(())
    }

    /// The prediction hold: music alone for `seconds` after the line.
    pub fn music_breath(&self, _: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
        stub::advance(seconds);
        stub::lint();
        Ok(())
    }

    pub fn until_sentence(&self, _: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let Some(narrator) = &self.narrator else {
            return Ok(());
        };
        let Some(at) = narrator.sentence_at(id, j) else {
            return Err(fmn::Error::Stub(format!("{id} has no sentence {j}, or is not said yet")));
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
        stub::lint();
        self.moment(id, &j.to_string());
        Ok(())
    }

    pub fn finish(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.hold(stage)?;
        stub::lint();
        let current = self.voice().borrow().current.clone();
        self.moment(&current, "before-next");
        if let Some(last) = self.voice().borrow_mut().reports.last_mut() {
            last.picture_end = stub::now();
        }
        Ok(())
    }

    pub fn reports(&self) -> Vec<CueReport> {
        self.voice().borrow().reports.clone()
    }

    pub fn start_of(&self, id: &str) -> Option<f64> {
        self.moments.borrow().iter().find(|m| m.0 == id).map(|m| m.1)
    }
}
