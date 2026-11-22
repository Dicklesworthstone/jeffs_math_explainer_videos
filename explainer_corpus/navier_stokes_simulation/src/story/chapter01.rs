//! Chapter 01, "A tunnel inside a computer" (navier-full-script03.md and the opening storyboard
//! navier-opening-script03.md, cues 01_1 to 01_6).
//!
//! The real flow comes first. 01_2 draws the saved velocity of every frame of the retained start-up
//! run (`--tunnel DIR`, bound in story.rs): the arrows of opening.rs at one arrow scale for the
//! whole sequence, each frame shown with its own saved clock and switched at once, not blended.
//! That run is navier-solver-transfer04/transfer09, the adaptive cylinder start-up with a steady
//! inflow, saved from 0.05 s to 0.25 s. Its last frame is the one chapter 02 draws. 01_4 names
//! that last frame's measured cell count and cell sizes.
//!
//! What the inputs do not contain is not drawn. The saved frames carry no tracer, so the five dye
//! nozzles stay still and say there is no dye in this run. No wake, no swirl and no swing
//! frequency: the test's readouts in 01_5 stay "?". A later wake run is drawn only when story.rs
//! passes a `Wake`, which loads only from an `opening-wake01` file that
//! tools/opening_inputs01.py writes after a separate qualification check. Without one, nothing
//! is drawn behind the cylinder.
//!
//! The teaching grid is chapter 09's `tunnel` (the same objects later chapters return to), an
//! illustration and labelled so. Halving it draws the midlines of its cells. The chapter ends on
//! chapter 02's first picture: the full static tunnel and nothing else.
//!
//! Voice: the kit narrator. Each cue is `say`d before its pictures, which wait on its sentence
//! starts (`until`).

use std::path::Path;

use explainer_kit::{Facts, Kit, Palette, Timed};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::chapter09;
use crate::error;
use crate::field::{Frame, TunnelGeometry};

/// Chapter 01's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the test and the check recount
/// them.
pub const CUES: [(&str, usize); 6] = [
    ("01_1", 1),
    ("01_2", 2),
    ("01_3", 3),
    ("01_4", 2),
    ("01_5", 3),
    ("01_6", 4),
];

/// Halving a cell's width and its height (01_5): two pieces each way.
pub const HALVE_PER_SIDE: u32 = 2;
/// "which takes four times as many cells" (01_5).
pub const HALVE_CELLS: u32 = HALVE_PER_SIDE * HALVE_PER_SIDE;

const TITLE: &str = "A wind tunnel inside a computer";
const INLET: &str = "Inlet";
const OUTLET: &str = "Outlet";
const DYE_NOTE: &str = "dye release points: no dye in this run";
const QUESTION: [&str; 2] = [
    "Which of those swirls belong to the air,",
    "and which did our computer invent, or erase?",
];
const GRID_LABEL: &str = "Teaching grid (illustration)";
const GAUGE_LABEL: &str = "measure here";
const READOUT_HEADER: &str = "how often it swings (not measured yet)";
const READOUT_ROWS: [&str; 2] = ["these cells", "every cell half as wide and half as tall"];
/// An entry nobody has measured.
const PENDING: &str = "?";
const IF_LINE: &str = "If the answer changes a lot, the cells are still shaping it";
/// The open question card, drawn as chapter 09 draws it.
const QUESTION_CARD: &str = "Does halving the cells change the answer?";
const CARDS: [&str; 3] = ["store", "law", "pressure"];
const WAKE_NOTE: &str = "a later run, at its own arrow scale";

// ------------------------------------------------------------------------------------- layout

/// The full tunnel, as chapter 02 (`FULL`), chapter 09 (`TUNNEL_WIDTH`, `TUNNEL_CENTER_Y`) and
/// opening.rs (`WIDTH`, `CENTER_Y`) place it.
pub const FULL_WIDTH: f64 = 11.5;
pub const FULL_CENTER: [f64; 2] = [0.0, 1.15];
/// The quadtree box: field.rs loads a tunnel frame only with this box width, in metres. A level-L
/// cell is BOX_WIDTH / 2^L across.
const BOX_WIDTH: f64 = 2.56;
/// opening.rs `velocity_arrows`: 23 columns by 5 rows of samples, the fastest arrow 0.43 units
/// long, shorter than 0.045 units skipped, stroke 1.1.
const ARROW_COLUMNS: usize = 23;
const ARROW_ROWS: usize = 5;
const ARROW_REACH: f64 = 0.43;
const ARROW_MIN: f64 = 0.045;
const ARROW_THICKNESS: f64 = 1.1;
/// How long each saved frame stays on screen before the next replaces it.
const FRAME_SECONDS: f64 = 0.4;
const FIRST_FADE: f64 = 0.6;
const FADE: f64 = 0.4;
const PULSE: f64 = 0.9;
/// The dye nozzles: five discs just outside the inlet wall, at the arrows' five rows.
const NOZZLES: usize = 5;
const NOZZLE_X: f64 = -5.92;
const NOZZLE_RADIUS: f64 = 0.06;
/// The region behind the cylinder, marked below the tunnel as chapter 02 (02_11) marks it.
pub const BRACKET_X: [f64; 2] = [-4.25, 0.45];
pub const BRACKET_GAP: f64 = 0.20;
/// The gauge (01_5), in metres: downstream of the cylinder, on the centre line, inside the bracket.
const GAUGE_M: [f64; 2] = [0.75, 0.2];
const GAUGE_RADIUS: f64 = 0.12;

const TITLE_AT: [f64; 2] = [0.0, 3.57];
const INLET_AT: [f64; 2] = [-5.2, 2.59];
const OUTLET_AT: [f64; 2] = [5.2, 2.59];
const STAMP_AT: [f64; 2] = [0.0, 2.60];
const DYE_AT: [f64; 2] = [-5.75, 2.95];
const NOTE_AT: [f64; 2] = [0.0, -0.50];
const QUESTION_Y: [f64; 2] = [-0.85, -1.40];
const CELLS_AT: [f64; 2] = [0.0, -0.75];
const GAUGE_LABEL_Y: f64 = -0.50;
const HEADER_AT: [f64; 2] = [3.6, -1.05];
const ROW_X: f64 = -5.2;
const VALUE_X: f64 = 3.6;
const ROW_Y: [f64; 2] = [-1.65, -2.30];
const PAIR_Y: f64 = -3.10;
const PAIR_X: [f64; 2] = [-2.0, 2.0];
const PAIR_SIDE: f64 = 0.8;
const PAIR_LABEL_Y: f64 = -3.80;
const IF_AT: [f64; 2] = [0.0, -3.10];
/// Chapter 09's question card: centre, size, word size.
const CARD_AT: [f64; 2] = [0.0, -1.25];
const CARD_SIZE: [f64; 2] = [8.2, 0.8];
const CARD_FONT: f64 = 30.0;
const STEP_CARD_Y: f64 = -2.55;
const STEP_CARD_X: [f64; 3] = [-2.7, 0.0, 2.7];
const STEP_CARD_SIZE: [f64; 2] = [2.2, 0.7];

// ------------------------------------------------------------------------------------- interface

pub struct Chapter01 {
    pub kit: Kit,
    /// Every saved frame of the retained start-up run (`--tunnel DIR`), on an ascending clock. The
    /// last is the frame chapter 02 draws.
    pub startup: Option<&'static [Frame]>,
    /// A later wake run, separately qualified; None draws nothing behind the cylinder.
    pub wake: Option<&'static Wake>,
}

/// A later wake run's last saved frame and the record of root's qualification, shown with it.
/// It loads only from an `opening-wake01` file (`parse_wake`), which tools/opening_inputs01.py
/// writes after its separate check.
pub struct Wake {
    frame: Frame,
    record: String,
}

/// The longest qualification record the note under the tunnel has room for.
pub const WAKE_RECORD_CHARS: usize = 40;

impl Wake {
    pub fn load(file: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let (frame, record) = parse_wake(&text)?;
        let frame = Frame::load(Path::new(&frame)).map_err(|e| format!("{frame}: {e}"))?;
        if frame.tunnel_geometry().is_none() {
            return Err("opening-wake01: the frame is not a tunnel frame".to_owned());
        }
        Ok(Self { frame, record })
    }
}

/// `opening-wake01`: exactly three tab-separated lines, `schema` `opening-wake01`, `frame` PATH
/// and `record` WORDS (at most WAKE_RECORD_CHARS characters).
pub fn parse_wake(text: &str) -> Result<(String, String), String> {
    fn field(line: Option<&str>, key: &str) -> Result<String, String> {
        let line = line.ok_or_else(|| format!("opening-wake01: missing {key}"))?;
        match line.split_once('\t') {
            Some((k, v)) if k == key && !v.trim().is_empty() && !v.contains('\t') => Ok(v.to_owned()),
            _ => Err(format!("opening-wake01: expected a {key} line")),
        }
    }
    let mut lines = text.lines();
    if field(lines.next(), "schema")? != "opening-wake01" {
        return Err("opening-wake01: unknown schema".to_owned());
    }
    let frame = field(lines.next(), "frame")?;
    let record = field(lines.next(), "record")?;
    if record.chars().count() > WAKE_RECORD_CHARS {
        return Err(format!(
            "opening-wake01: the record is longer than {WAKE_RECORD_CHARS} characters"
        ));
    }
    if lines.next().is_some() {
        return Err("opening-wake01: unexpected extra line".to_owned());
    }
    Ok((frame, record))
}

pub fn register(facts: &mut Facts) -> Result<(), String> {
    facts.pin(
        "c01.halve.cells",
        HALVE_CELLS,
        "4",
        "script 01_5: half as wide and half as tall takes four times as many cells",
    )?;
    Ok(())
}

// ------------------------------------------------------------------------------------- the data

fn same_tunnel(a: TunnelGeometry, b: TunnelGeometry) -> bool {
    a.length == b.length && a.height == b.height && a.cylinder == b.cylinder
}

/// The frames' shared tunnel, or why they cannot be drawn: at least `least` frames, every one a
/// tunnel frame of one geometry, on a strictly ascending clock.
fn check_sequence(frames: &[Frame], what: &str, least: usize) -> Result<TunnelGeometry, String> {
    if frames.len() < least {
        return Err(format!(
            "chapter 01: the {what} run needs at least {least} saved frames"
        ));
    }
    let g = frames
        .first()
        .and_then(Frame::tunnel_geometry)
        .ok_or_else(|| format!("chapter 01: the {what} run is not a tunnel run"))?;
    for frame in frames {
        if !frame.tunnel_geometry().is_some_and(|other| same_tunnel(g, other)) {
            return Err(format!("chapter 01: the {what} frames do not share one tunnel"));
        }
    }
    if frames.windows(2).any(|pair| pair[0].time >= pair[1].time) {
        return Err(format!("chapter 01: the {what} frames are not on an ascending clock"));
    }
    Ok(g)
}

/// The scene point of (x, y) in metres in the full tunnel.
fn tunnel_point(g: TunnelGeometry, x: f64, y: f64) -> [f64; 2] {
    let scale = FULL_WIDTH / g.length;
    [
        FULL_CENTER[0] + scale * (x - g.length / 2.0),
        FULL_CENTER[1] + scale * (y - g.height / 2.0),
    ]
}

/// opening.rs `velocity_arrows`'s samples: cell centres of an even ARROW_COLUMNS × ARROW_ROWS grid
/// over the tunnel, each with the velocity stored in the leaf that owns it
/// (`Frame::cell_velocity_at`); none inside the cylinder or in a dry leaf.
fn samples(frame: &Frame, g: TunnelGeometry) -> Vec<([f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for i in 0..ARROW_COLUMNS {
        let x = g.length * (i as f64 + 0.5) / ARROW_COLUMNS as f64;
        for j in 0..ARROW_ROWS {
            let y = g.height * (j as f64 + 0.5) / ARROW_ROWS as f64;
            if let Some(velocity) = frame.cell_velocity_at(x, y) {
                out.push((tunnel_point(g, x, y), velocity));
            }
        }
    }
    out
}

/// The fastest sampled speed over all `frames`, which sets one arrow scale for them all.
fn peak_speed(frames: &[Frame], g: TunnelGeometry) -> Result<f64, String> {
    let peak = frames
        .iter()
        .flat_map(|frame| samples(frame, g))
        .map(|(_, [u, v])| u.hypot(v))
        .fold(0.0, f64::max);
    if peak > 0.0 && peak.is_finite() {
        Ok(peak)
    } else {
        Err("chapter 01 needs a saved, moving tunnel flow".to_owned())
    }
}

/// "Saved velocity at 0.250 s", as opening.rs and chapter 02 stamp a saved frame.
pub fn stamp(time: f64) -> String {
    format!("Saved velocity at {time:.3} s")
}

/// The note under the start-up arrows: how many frames, how far apart (when evenly), and how
/// they are shown.
fn run_note(frames: &[Frame]) -> String {
    let gaps: Vec<f64> = frames.windows(2).map(|pair| pair[1].time - pair[0].time).collect();
    let even = gaps
        .first()
        .is_some_and(|first| gaps.iter().all(|gap| (gap - first).abs() < 1e-9));
    match (even, gaps.first()) {
        (true, Some(gap)) => format!(
            "{} saved frames from our solver, {gap:.3} s apart, not blended; arrow length shows speed",
            frames.len()
        ),
        _ => format!(
            "{} saved frames from our solver, not blended; arrow length shows speed",
            frames.len()
        ),
    }
}

/// 11378 as "11,378".
pub fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (k, c) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A level-`level` cell's width in millimetres: 10 for level 8, 5 for 9, 2.5 for 10, 1.25 for 11.
pub fn millimetres(level: u32) -> String {
    let mm = BOX_WIDTH * 1000.0 / f64::from(1u32 << level);
    format!("{}", (mm * 1000.0).round() / 1000.0)
}

/// The cell widths between two levels: "5 mm", "10 mm and 5 mm" or "10 mm down to 1.25 mm".
pub fn sizes(level_min: u32, level_max: u32) -> String {
    let (coarse, fine) = (millimetres(level_min), millimetres(level_max));
    match level_max.saturating_sub(level_min) {
        0 => format!("{fine} mm"),
        1 => format!("{coarse} mm and {fine} mm"),
        _ => format!("{coarse} mm down to {fine} mm"),
    }
}

/// 01_4's measured line, from the start-up run's last frame.
fn cells_line(frame: &Frame) -> String {
    format!(
        "Our solver at {:.3} s: {} fluid cells, {} across",
        frame.time,
        grouped(frame.wet_leaves),
        sizes(frame.level_min, frame.level_max)
    )
}

/// The midlines of chapter 09's teaching cells (side H/3, columns about the middle): new rows at
/// H/6, H/2 and 5H/6, and new columns half a cell from each old one, strictly inside the tunnel.
pub fn half_positions(g: TunnelGeometry) -> (Vec<f64>, Vec<f64>) {
    let cell = g.height / 3.0;
    let rows = (0..3).map(|k| (f64::from(k) + 0.5) * cell).collect();
    let middle = g.length / 2.0;
    let reach = (middle / cell).floor() as i32 + 1;
    let columns = (-reach..reach)
        .map(|k| middle + (f64::from(k) + 0.5) * cell)
        .filter(|&x| x > 0.0 && x < g.length)
        .collect();
    (rows, columns)
}

/// Where chapter 02 draws its bracket under the tunnel (02_11): BRACKET_GAP below the bottom wall.
pub fn bracket_y(g: TunnelGeometry) -> f64 {
    FULL_CENTER[1] - FULL_WIDTH * g.height / g.length / 2.0 - BRACKET_GAP
}

// ----------------------------------------------------------------------------------- builders

fn p2(at: [f64; 2]) -> Vec3 {
    [at[0], at[1], 0.0]
}

fn line(stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(p2(a), p2(b))
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

fn square(stage: &mut Stage<'_>, at: [f64; 2], side: f64, color: Srgb, opacity: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(side)
            .height(side)
            .style(Style::default().fill(color, opacity).stroke(color, 1.7, 1.0))
            .build()?
            .moved_to(p2(at)),
    ))
}

fn card(stage: &mut Stage<'_>, at: [f64; 2], size: [f64; 2], corner: f64, edge: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(size[0])
            .height(size[1])
            .corner_radius(corner)
            .style(Style::default().fill(Palette::ink(), 0.0).stroke(edge, 2.0, 1.0))
            .build()?
            .moved_to(p2(at)),
    ))
}

/// One frame's arrows at `scale` scene units per metre per second, as opening.rs draws them.
fn arrows(stage: &mut Stage<'_>, frame: &Frame, g: TunnelGeometry, scale: f64) -> fmn::Result<Vec<Mob>> {
    let color = Palette::blue();
    let mut out = Vec::new();
    for (origin, [u, v]) in samples(frame, g) {
        if scale * u.hypot(v) < ARROW_MIN {
            continue;
        }
        let end = [origin[0] + scale * u, origin[1] + scale * v];
        out.push(
            stage.arena_mut().add(
                Arrow::new(p2(origin), p2(end))
                    .buff(0.0)
                    .thickness(ARROW_THICKNESS)
                    .style(Style::default().fill(color, 1.0).stroke(color, ARROW_THICKNESS, 1.0))
                    .build()?,
            ),
        );
    }
    Ok(out)
}

// ------------------------------------------------------------------------------------- staging

/// The chapter's state: what is on screen, and a pending batch of animations.
struct Ch01<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
}

impl Ch01<'_> {
    // Text through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, p2(at))
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, p2(at))
    }

    /// Put on screen at once.
    fn put(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                stage.add_to_scene(m)?;
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Remove at once (no fade).
    fn cut(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) {
        let gone: Vec<Mob> = mobs.iter().copied().filter(|m| self.on.contains(m)).collect();
        stage.scene_mut().remove(&gone);
        self.on.retain(|m| !gone.contains(m));
    }

    fn appear(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Fade out what is on screen; anything already gone is skipped (no second fade).
    fn vanish(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.retain(|&x| x != m);
            }
        }
        Ok(())
    }

    fn draw(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                self.pending.push(stage.prepare(show_creation(m).rt(secs))?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    fn pulse(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = indicate(stage.arena_mut(), m, 1.15, None)?.rt(PULSE);
                self.pending.push(stage.prepare(a)?);
            }
        }
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    /// Finish what is queued, then start line `id`.
    fn say(&mut self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(error(format!("chapter 01 has no cue {id}")));
        }
        self.play(stage)?;
        println!("Chapter 01 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(error(format!("chapter 01 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// Finish what is queued, let the line end (plus the kit's breath), before a wipe.
    fn hold(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.play(stage)?;
        self.k.hold(stage)
    }

    /// Each saved frame's arrows and clock stamp, the first faded in, the rest switched in at once
    /// FRAME_SECONDS apart (not blended); returns what the last frame left on screen.
    fn play_frames(
        &mut self,
        stage: &mut Stage<'_>,
        frames: &[Frame],
        g: TunnelGeometry,
        scale: f64,
    ) -> fmn::Result<Vec<Mob>> {
        let mut shown: Vec<Mob> = Vec::new();
        for (k, frame) in frames.iter().enumerate() {
            let mut next = arrows(stage, frame, g, scale)?;
            println!(
                "Chapter 01 start-up frame {} of {}: {:.6} s, {} arrows, {} fluid cells",
                k + 1,
                frames.len(),
                frame.time,
                next.len(),
                frame.wet_leaves
            );
            next.push(self.text(stage, &stamp(frame.time), 27.0, Palette::blue(), STAMP_AT)?);
            if k == 0 {
                self.appear(stage, &next, FIRST_FADE)?;
                self.play(stage)?;
            } else {
                stage.wait(FRAME_SECONDS)?;
                self.cut(stage, &shown);
                self.put(stage, &next)?;
            }
            shown = next;
        }
        Ok(shown)
    }
}

impl SceneConstruct for Chapter01 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // Every refusal comes before the first frame.
        let frames = self
            .startup
            .ok_or_else(|| error("Chapter 01 requires --tunnel and the retained start-up frames".to_owned()))?;
        let g = check_sequence(frames, "start-up", 2).map_err(error)?;
        let scale = ARROW_REACH / peak_speed(frames, g).map_err(error)?;
        let last = frames
            .last()
            .ok_or_else(|| error("chapter 01: no start-up frame".to_owned()))?;
        let wake = match self.wake {
            Some(w) => {
                let wg = check_sequence(std::slice::from_ref(&w.frame), "wake", 1).map_err(error)?;
                if !same_tunnel(g, wg) {
                    return Err(error("chapter 01: the wake run is in another tunnel".to_owned()));
                }
                let peak = peak_speed(std::slice::from_ref(&w.frame), g).map_err(error)?;
                Some((w, ARROW_REACH / peak))
            }
            None => None,
        };
        println!(
            "Chapter 01: {} start-up frames, {:.6} s to {:.6} s, one arrow scale {scale:.6} units per m/s; wake {}",
            frames.len(),
            frames[0].time,
            last.time,
            wake.map_or_else(
                || "none (nothing drawn behind the cylinder)".to_owned(),
                |(w, _)| w.record.clone()
            )
        );
        let mut facts = Facts::new();
        register(&mut facts).map_err(error)?;
        for fact in facts.print().lines() {
            println!("Chapter 01 fact {fact}");
        }
        let (ink, dim, hero) = (Palette::ink(), Palette::dim(), Palette::hero());
        let mut s = Ch01 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
        };

        // 01_1: the full tunnel, walls first, then the cylinder.
        s.say(stage, "01_1")?;
        let drawing = chapter09::tunnel(stage, g, FULL_WIDTH, FULL_CENTER)?;
        let (tank, cylinder) = (drawing[0], drawing[1]);
        let grid: Vec<Mob> = drawing[2..].to_vec();
        s.appear(stage, &[tank], 0.42)?;
        s.play(stage)?;
        let title = s.text(stage, TITLE, 45.0, ink, TITLE_AT)?;
        s.appear(stage, &[cylinder], 0.32)?;
        s.appear(stage, &[title], 0.5)?;

        // 01_2: in at the left, out at the right; the saved start-up velocity, frame by frame.
        s.say(stage, "01_2")?;
        let ends = [
            s.text(stage, INLET, 24.0, dim, INLET_AT)?,
            s.text(stage, OUTLET, 24.0, dim, OUTLET_AT)?,
        ];
        let note = s.text(stage, &run_note(frames), 24.0, dim, NOTE_AT)?;
        s.appear(stage, &ends, 0.35)?;
        s.appear(stage, &[note], 0.35)?;
        let shown = s.play_frames(stage, frames, g, scale)?;
        // The five dye release points stay still: the saved run has no dye.
        s.until(stage, "01_2", 1)?;
        let mut nozzles = Vec::new();
        for k in 0..NOZZLES {
            let y = g.height * (k as f64 + 0.5) / NOZZLES as f64;
            let at = [NOZZLE_X, tunnel_point(g, 0.0, y)[1]];
            nozzles.push(
                stage.arena_mut().add(
                    Dot::new()
                        .point(p2(at))
                        .radius(NOZZLE_RADIUS)
                        .style(
                            Style::default()
                                .fill(Palette::purple(), 1.0)
                                .stroke(Palette::purple(), 0.0, 1.0),
                        )
                        .build(),
                ),
            );
        }
        let dye = s.text_left(stage, DYE_NOTE, 24.0, Palette::purple(), DYE_AT)?;
        s.appear(stage, &nozzles, 0.4)?;
        s.appear(stage, &[dye], 0.4)?;

        // 01_3: the saved arrows leave and the region behind the cylinder is marked. Nothing is
        // drawn in it, unless a separately qualified later wake was given.
        s.say(stage, "01_3")?;
        s.vanish(stage, &shown, 0.35)?;
        s.vanish(stage, &[note], 0.35)?;
        let edge = bracket_y(g);
        let bracket = line(stage, [BRACKET_X[0], edge], [BRACKET_X[1], edge], hero, 2.0)?;
        s.appear(stage, &[bracket], 0.35)?;
        let mut wake_shown = Vec::new();
        if let Some((w, wake_scale)) = wake {
            wake_shown = arrows(stage, &w.frame, g, wake_scale)?;
            println!(
                "Chapter 01 wake: {:.6} s, {} arrows at {wake_scale:.6} units per m/s, {}",
                w.frame.time,
                wake_shown.len(),
                w.record
            );
            wake_shown.push(s.text(stage, &stamp(w.frame.time), 27.0, Palette::blue(), STAMP_AT)?);
            wake_shown.push(s.text(stage, &format!("{WAKE_NOTE}: {}", w.record), 24.0, dim, NOTE_AT)?);
            s.appear(stage, &wake_shown, 0.5)?;
        }
        s.until(stage, "01_3", 2)?;
        s.vanish(stage, &wake_shown, 0.35)?;
        let question = [
            s.text(stage, QUESTION[0], 30.0, hero, [0.0, QUESTION_Y[0]])?,
            s.text(stage, QUESTION[1], 30.0, hero, [0.0, QUESTION_Y[1]])?,
        ];
        s.appear(stage, &question, 0.5)?;

        // 01_4: a computer cuts the tunnel into cells; the teaching grid, and the saved run's cells.
        s.say(stage, "01_4")?;
        s.vanish(stage, &question, FADE)?;
        s.until(stage, "01_4", 1)?;
        s.draw(stage, &grid, 0.8)?;
        let grid_label = s.text(stage, GRID_LABEL, 27.0, dim, STAMP_AT)?;
        s.appear(stage, &[grid_label], 0.5)?;
        s.play(stage)?;
        let side = FULL_WIDTH * g.height / g.length / 3.0;
        let one = square(
            stage,
            [FULL_CENTER[0] + side / 2.0, FULL_CENTER[1]],
            side,
            Palette::gold(),
            0.16,
        )?;
        let cells = s.text(stage, &cells_line(last), 26.0, ink, CELLS_AT)?;
        println!("Chapter 01 cells: {}", cells_line(last));
        s.appear(stage, &[one], 0.3)?;
        s.appear(stage, &[cells], 0.5)?;

        // 01_5: the test. Measure one thing downstream; halve every cell; measure again. Nobody has
        // measured either yet.
        s.say(stage, "01_5")?;
        s.vanish(stage, &[cells, one], FADE)?;
        s.pulse(stage, &[bracket])?;
        s.until(stage, "01_5", 1)?;
        let gauge_at = tunnel_point(g, GAUGE_M[0], GAUGE_M[1]);
        let gauge = stage.arena_mut().add(
            Circle::new()
                .radius(GAUGE_RADIUS)
                .style(Style::default().fill(hero, 0.0).stroke(hero, 2.4, 1.0))
                .build()
                .moved_to(p2(gauge_at)),
        );
        let gauge_label = s.text(stage, GAUGE_LABEL, 24.0, hero, [gauge_at[0], GAUGE_LABEL_Y])?;
        let header = s.text(stage, READOUT_HEADER, 24.0, dim, HEADER_AT)?;
        let first = [
            s.text_left(stage, READOUT_ROWS[0], 28.0, ink, [ROW_X, ROW_Y[0]])?,
            s.text(stage, PENDING, 34.0, hero, [VALUE_X, ROW_Y[0]])?,
        ];
        s.appear(stage, &[gauge, gauge_label], 0.4)?;
        s.appear(stage, &[header], 0.4)?;
        s.appear(stage, &first, 0.4)?;
        s.until(stage, "01_5", 2)?;
        let (rows, columns) = half_positions(g);
        let mut halves = Vec::new();
        for y in rows {
            halves.push(line(
                stage,
                tunnel_point(g, 0.0, y),
                tunnel_point(g, g.length, y),
                Palette::faint(),
                1.1,
            )?);
        }
        for x in columns {
            halves.push(line(
                stage,
                tunnel_point(g, x, 0.0),
                tunnel_point(g, x, g.height),
                Palette::faint(),
                1.1,
            )?);
        }
        s.draw(stage, &halves, 0.8)?;
        let gold = Palette::gold();
        let mut pair = vec![
            square(stage, [PAIR_X[0], PAIR_Y], PAIR_SIDE, gold, 0.18)?,
            square(stage, [PAIR_X[1], PAIR_Y], PAIR_SIDE, gold, 0.18)?,
        ];
        let half = PAIR_SIDE / 2.0;
        pair.push(line(
            stage,
            [PAIR_X[1] - half, PAIR_Y],
            [PAIR_X[1] + half, PAIR_Y],
            gold,
            2.0,
        )?);
        pair.push(line(
            stage,
            [PAIR_X[1], PAIR_Y - half],
            [PAIR_X[1], PAIR_Y + half],
            gold,
            2.0,
        )?);
        pair.push(s.text(
            stage,
            &format!("{HALVE_CELLS} × as many cells"),
            26.0,
            ink,
            [0.0, PAIR_Y],
        )?);
        pair.push(s.text(stage, "1 cell", 26.0, ink, [PAIR_X[0], PAIR_LABEL_Y])?);
        pair.push(s.text(
            stage,
            &format!("{HALVE_CELLS} cells"),
            26.0,
            ink,
            [PAIR_X[1], PAIR_LABEL_Y],
        )?);
        let second = [
            s.text_left(stage, READOUT_ROWS[1], 28.0, ink, [ROW_X, ROW_Y[1]])?,
            s.text(stage, PENDING, 34.0, hero, [VALUE_X, ROW_Y[1]])?,
        ];
        s.appear(stage, &pair, 0.5)?;
        s.appear(stage, &second, 0.5)?;

        // 01_6: a large change means the cells are still shaping the answer. Then the plan, and the
        // open question that stays.
        s.say(stage, "01_6")?;
        s.vanish(stage, &pair, FADE)?;
        let if_line = s.text(stage, IF_LINE, 28.0, ink, IF_AT)?;
        s.appear(stage, &[if_line], 0.5)?;
        s.until(stage, "01_6", 1)?;
        let mut clear: Vec<Mob> = grid.clone();
        clear.extend(&halves);
        clear.extend([grid_label, gauge, gauge_label, header, if_line, bracket]);
        clear.extend(first);
        clear.extend(second);
        s.vanish(stage, &clear, 0.6)?;
        let open = [
            card(stage, CARD_AT, CARD_SIZE, 0.12, dim)?,
            s.text(stage, QUESTION_CARD, CARD_FONT, ink, CARD_AT)?,
        ];
        s.appear(stage, &open, 0.6)?;
        s.until(stage, "01_6", 2)?;
        let mut steps = Vec::new();
        for (k, name) in CARDS.iter().enumerate() {
            let edge = if k == 2 { hero } else { dim };
            steps.push([
                card(stage, [STEP_CARD_X[k], STEP_CARD_Y], STEP_CARD_SIZE, 0.1, edge)?,
                s.text(stage, name, 28.0, ink, [STEP_CARD_X[k], STEP_CARD_Y])?,
            ]);
        }
        s.appear(stage, &steps[0], 0.4)?;
        s.play(stage)?;
        s.appear(stage, &steps[1], 0.4)?;
        s.until(stage, "01_6", 3)?;
        s.appear(stage, &steps[2], 0.4)?;
        s.play(stage)?;
        s.pulse(stage, &steps[2])?;
        s.hold(stage)?;

        // End on chapter 02's first picture: the full static tunnel, nothing else.
        let rest: Vec<Mob> = s.on.iter().copied().filter(|&m| m != tank && m != cylinder).collect();
        s.vanish(stage, &rest, 0.6)?;
        s.play(stage)?;
        println!(
            "Chapter 01 ends on chapter 02's start: the full tunnel at width {FULL_WIDTH}, centre {:?}, {} objects",
            FULL_CENTER,
            s.on.len()
        );
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = include_str!("../../script.tsv");
    const BENCH: TunnelGeometry = TunnelGeometry {
        length: 2.2,
        height: 0.41,
        cylinder: [0.2, 0.2, 0.05],
    };

    fn sentences(text: &str) -> usize {
        let b = text.as_bytes();
        let mut count = 0;
        let mut from = 0;
        for (i, &c) in b.iter().enumerate() {
            if matches!(c, b'.' | b'?' | b'!') && (i + 1 == b.len() || b[i + 1] == b' ') {
                if !text[from..=i].trim().is_empty() {
                    count += 1;
                }
                from = i + 1;
            }
        }
        count + usize::from(!text[from..].trim().is_empty())
    }

    #[test]
    fn cues_are_the_scripts_lines_in_order_with_their_sentences() {
        let lines: Vec<(&str, &str)> = SCRIPT
            .lines()
            .filter(|line| line.starts_with("01_"))
            .map(|line| {
                let mut parts = line.split('\t');
                (parts.next().unwrap(), parts.next().unwrap())
            })
            .collect();
        assert_eq!(lines.len(), CUES.len());
        for ((id, text), (cue, count)) in lines.iter().zip(CUES) {
            assert_eq!(*id, cue);
            assert_eq!(sentences(text), count, "{id}");
        }
    }

    #[test]
    fn halving_takes_four_times_as_many_cells() {
        assert_eq!(HALVE_CELLS, 4);
        let mut facts = Facts::new();
        register(&mut facts).unwrap();
        assert_eq!(facts.get("c01.halve.cells"), Some("4"));
    }

    #[test]
    fn measured_words_are_formatted_as_shown() {
        assert_eq!(grouped(11_378), "11,378");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_000), "1,000");
        assert_eq!(grouped(1_234_567), "1,234,567");
        assert_eq!(millimetres(8), "10");
        assert_eq!(millimetres(9), "5");
        assert_eq!(millimetres(10), "2.5");
        assert_eq!(millimetres(11), "1.25");
        assert_eq!(sizes(9, 9), "5 mm");
        assert_eq!(sizes(8, 9), "10 mm and 5 mm");
        assert_eq!(sizes(8, 11), "10 mm down to 1.25 mm");
        assert_eq!(stamp(0.05), "Saved velocity at 0.050 s");
        assert_eq!(stamp(0.24999999999999997), "Saved velocity at 0.250 s");
    }

    #[test]
    fn the_halved_grid_draws_the_midlines_of_chapter_09s_cells() {
        let (rows, columns) = half_positions(BENCH);
        let cell = BENCH.height / 3.0;
        assert_eq!(rows.len(), 3);
        assert!((rows[1] - BENCH.height / 2.0).abs() < 1e-12);
        assert_eq!(columns.len(), 16);
        let middle = BENCH.length / 2.0;
        for x in columns {
            let offset = (x - middle) / cell;
            assert!((offset - offset.floor() - 0.5).abs() < 1e-9, "{x}");
            assert!(x > 0.0 && x < BENCH.length);
        }
    }

    #[test]
    fn the_bracket_and_the_tunnel_match_chapter_02() {
        // chapter 02: FULL = (0, 1.15) and 11.5 wide; its bracket 0.20 below the bottom wall.
        assert_eq!(FULL_WIDTH, 11.5);
        assert_eq!(FULL_CENTER, [0.0, 1.15]);
        let edge = bracket_y(BENCH);
        assert!((edge - (1.15 - 11.5 * 0.41 / 2.2 / 2.0 - 0.20)).abs() < 1e-12);
        let behind = tunnel_point(BENCH, BENCH.cylinder[0] + BENCH.cylinder[2], 0.0)[0];
        assert!(BRACKET_X[0] > behind, "the bracket starts behind the cylinder");
        let gauge = tunnel_point(BENCH, GAUGE_M[0], GAUGE_M[1]);
        assert!(gauge[0] > BRACKET_X[0] && gauge[0] < BRACKET_X[1]);
    }

    #[test]
    fn a_wake_needs_a_complete_opening_wake01_file() {
        let good = "schema\topening-wake01\nframe\t/runs/w/frame-0000001.flow\nrecord\tW1 accepted, Mail 1\n";
        assert_eq!(
            parse_wake(good),
            Ok((
                "/runs/w/frame-0000001.flow".to_owned(),
                "W1 accepted, Mail 1".to_owned()
            ))
        );
        for bad in [
            "",
            "schema\topening-wake02\nframe\tf\nrecord\tr\n",
            "schema\topening-wake01\nframe\tf\n",
            "schema\topening-wake01\nframe\tf\nrecord\t \n",
            "schema\topening-wake01\nrecord\tr\nframe\tf\n",
            "schema\topening-wake01\nframe\tf\nrecord\tr\nextra\tx\n",
            "schema\topening-wake01\nframe\tf\nrecord\tan accepted wake run with a record far too long for the stamp\n",
        ] {
            assert!(parse_wake(bad).is_err(), "{bad:?}");
        }
    }

    /// A beat: (cue number, sentence). A piece is on screen from its first beat until (not
    /// including) the beat it leaves at; END is after the last line.
    type Beat = (u8, u8);
    const END: Beat = (7, 0);

    /// CMR10 advance widths in em (the table of logarithm_mysteries' unreachable.rs test, which
    /// came within 2% of lint-measured widths); capitals take a typical capital's.
    fn em(c: char) -> f64 {
        match c {
            'i' | 'l' | ',' | '.' | ':' | ';' | '!' | '[' | ']' => 0.278,
            'j' | 'f' => 0.306,
            ' ' | '-' => 0.333,
            'I' => 0.361,
            'r' | 's' | 't' | '(' | ')' => 0.39,
            'a' | 'c' | 'e' | 'z' => 0.444,
            'g' | 'o' | '^' | '/' | '0'..='9' => 0.5,
            'k' | 'q' | 'v' | 'x' | 'y' => 0.528,
            'b' | 'd' | 'h' | 'n' | 'p' | 'u' => 0.556,
            'w' => 0.722,
            '=' | '+' | '−' | '×' | '±' | '<' | '>' => 0.778,
            'm' => 0.833,
            _ => 0.75,
        }
    }

    /// A text's half width and half height in scene units: ems × size × 0.01, and one em tall.
    fn half(words: &str, size: f64) -> [f64; 2] {
        let width = words.chars().map(em).sum::<f64>() * size * 0.01;
        [width / 2.0, size * 0.01 / 2.0]
    }

    /// Every text and card the chapter places: (what, box [left, right, bottom, top], from, to),
    /// as `construct` places and removes them. Data-made strings at the start-up run's values.
    fn placed() -> Vec<(String, [f64; 4], Beat, Beat)> {
        let mut out = Vec::new();
        let mut text = |words: &str, at: [f64; 2], size: f64, left: bool, from: Beat, to: Beat| {
            let [w, h] = half(words, size);
            let x = if left { at[0] + w } else { at[0] };
            out.push((words.to_owned(), [x - w, x + w, at[1] - h, at[1] + h], from, to));
        };
        let note = "9 saved frames from our solver, 0.025 s apart, not blended; arrow length shows speed";
        let cells = "Our solver at 0.250 s: 11,378 fluid cells, 10 mm and 5 mm across";
        let gauge_x = tunnel_point(BENCH, GAUGE_M[0], GAUGE_M[1])[0];
        text(TITLE, TITLE_AT, 45.0, false, (1, 0), END);
        text(INLET, INLET_AT, 24.0, false, (2, 0), END);
        text(OUTLET, OUTLET_AT, 24.0, false, (2, 0), END);
        text(stamp(0.25).as_str(), STAMP_AT, 27.0, false, (2, 0), (3, 0));
        text(note, NOTE_AT, 24.0, false, (2, 0), (3, 0));
        text(DYE_NOTE, DYE_AT, 24.0, true, (2, 1), END);
        text(QUESTION[0], [0.0, QUESTION_Y[0]], 30.0, false, (3, 2), (4, 0));
        text(QUESTION[1], [0.0, QUESTION_Y[1]], 30.0, false, (3, 2), (4, 0));
        text(GRID_LABEL, STAMP_AT, 27.0, false, (4, 1), (6, 1));
        text(cells, CELLS_AT, 26.0, false, (4, 1), (5, 0));
        text(GAUGE_LABEL, [gauge_x, GAUGE_LABEL_Y], 24.0, false, (5, 1), (6, 1));
        text(READOUT_HEADER, HEADER_AT, 24.0, false, (5, 1), (6, 1));
        text(READOUT_ROWS[0], [ROW_X, ROW_Y[0]], 28.0, true, (5, 1), (6, 1));
        text(PENDING, [VALUE_X, ROW_Y[0]], 34.0, false, (5, 1), (6, 1));
        text(READOUT_ROWS[1], [ROW_X, ROW_Y[1]], 28.0, true, (5, 2), (6, 1));
        text(PENDING, [VALUE_X, ROW_Y[1]], 34.0, false, (5, 2), (6, 1));
        text("4 × as many cells", [0.0, PAIR_Y], 26.0, false, (5, 2), (6, 0));
        text("1 cell", [PAIR_X[0], PAIR_LABEL_Y], 26.0, false, (5, 2), (6, 0));
        text("4 cells", [PAIR_X[1], PAIR_LABEL_Y], 26.0, false, (5, 2), (6, 0));
        text(IF_LINE, IF_AT, 28.0, false, (6, 0), (6, 1));
        text(QUESTION_CARD, CARD_AT, CARD_FONT, false, (6, 1), END);
        for (k, name) in CARDS.iter().enumerate() {
            let from = if k < 2 { (6, 2) } else { (6, 3) };
            text(name, [STEP_CARD_X[k], STEP_CARD_Y], 28.0, false, from, END);
        }
        let mut shape = |what: &str, at: [f64; 2], size: [f64; 2], from: Beat, to: Beat| {
            let [w, h] = [size[0] / 2.0, size[1] / 2.0];
            out.push((what.to_owned(), [at[0] - w, at[0] + w, at[1] - h, at[1] + h], from, to));
        };
        shape("one cell", [PAIR_X[0], PAIR_Y], [PAIR_SIDE; 2], (5, 2), (6, 0));
        shape("four cells", [PAIR_X[1], PAIR_Y], [PAIR_SIDE; 2], (5, 2), (6, 0));
        shape("question card", CARD_AT, CARD_SIZE, (6, 1), END);
        for (k, name) in CARDS.iter().enumerate() {
            let from = if k < 2 { (6, 2) } else { (6, 3) };
            shape(name, [STEP_CARD_X[k], STEP_CARD_Y], STEP_CARD_SIZE, from, END);
        }
        out
    }

    #[test]
    fn layout_keeps_what_is_on_screen_together_apart_and_inside_the_frame() {
        let pieces = placed();
        let top = FULL_CENTER[1] + FULL_WIDTH * BENCH.height / BENCH.length / 2.0;
        for (what, b, _, _) in &pieces {
            assert!(
                b[0] >= -7.1 && b[1] <= 7.1 && b[2] >= -4.0 && b[3] <= 4.0,
                "{what} {b:?}"
            );
            // Above the tunnel's top wall, or below the bracket: never over the tunnel.
            assert!(b[2] >= top + 0.1 || b[3] <= bracket_y(BENCH) - 0.05, "{what} {b:?}");
        }
        let overlap_in_time = |a: (Beat, Beat), b: (Beat, Beat)| a.0 < b.1 && b.0 < a.1;
        for (i, (wa, a, fa, ta)) in pieces.iter().enumerate() {
            for (wb, b, fb, tb) in &pieces[i + 1..] {
                let inside = |p: &[f64; 4], q: &[f64; 4]| p[0] >= q[0] && p[1] <= q[1] && p[2] >= q[2] && p[3] <= q[3];
                if !overlap_in_time((*fa, *ta), (*fb, *tb)) || inside(a, b) || inside(b, a) {
                    continue; // a card's own words sit inside it
                }
                let apart = a[1] <= b[0] || b[1] <= a[0] || a[3] <= b[2] || b[3] <= a[2];
                assert!(apart, "{wa} {a:?} and {wb} {b:?} overlap");
            }
        }
    }
}
