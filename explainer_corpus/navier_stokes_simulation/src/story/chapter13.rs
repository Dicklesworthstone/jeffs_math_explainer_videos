//! Chapter 13, "Which mistake are we measuring?", drawn from the measured runs in `ledger13`.
//!
//! The chapter keeps a ledger: beside each result, what changed, what was compared, what was
//! observed and what the comparison can tell us. Its rows are the measured Taylor–Green
//! studies (the pressure budget with cells and step fixed, the step with cells fixed, the cells
//! with the step fixed), the steady Re = 20 cylinder of Schäfer and Turek's case 2D-1 (its budget
//! met, its lift not settled by the frozen rule, its forces outside the published ranges),
//! chapter 11's dye pulse as an illustration (the step alone shrunk, and its dye budget), and the
//! choices no mesh settles (where the cylinder is, where the outlet is, the flat model,
//! rounding). Every line is gated by `Kit::say` before its picture, and the prediction 13_h1
//! (after 13_5) holds a five-second countdown with 13_6's answer pictures not yet drawn.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Narrator, Palette, hex, v};
use fmn::prelude::*;

use super::ledger13::{
    CHANNEL_STEPS, CHANNEL_TIME, CYLINDER_BUDGET, CYLINDER_CHECKS, CYLINDER_PRESSURE_RESIDUAL, CYLINDER_STEPS,
    CYLINDER_VOLUME_DEFECT, CYLINDER_WINDOW, DYE_RULES, PRESSURE, PRESSURE_FLOW_CHANGE, RangeCheck, Run, Rung, SPACE,
    SPACE_RATIOS, SPECIFIED_OFFSET, STAIRCASE_CELLS, STEP_ONLY_COURANT, STEP_ONLY_LIMIT, Staircase, TG_COURANT_CHECK,
    TG_DIFFUSION_CHECK, TG_END, TIME, TIME_DIFFERENCES, TIME_RATIOS, TIME_REFERENCE_STEPS, channel_euler,
    channel_exact, diffusion_weight, dye_budget, energy_account, ladder, observed_order, step_only_formula,
    step_only_spreading,
};
use crate::transport::{PULSE_WIDTH, Pulse, Rule as Transport, TRAVEL};

pub struct Chapter13 {
    pub kit: Kit,
}

/// Pacing, in seconds of scene time. No number on screen depends on these.
pub mod pacing {
    /// 13_4: one move of the slider knobs.
    pub const SLIDE: f64 = 0.7;
    /// 13_4: a rung held after its move.
    pub const RUNG_HOLD: f64 = 0.35;
    /// 13_14: between the step-only points.
    pub const POINT: f64 = 0.55;
    /// After 13_14: the completed first page of the ledger, held before 13_5.
    pub const LEDGER_BEAT: f64 = 1.6;
    /// The prediction countdown, in whole seconds.
    pub const COUNTDOWN: u32 = 5;
    /// Animations redraw at most this often per second, each redraw a whole number of frames.
    pub const REDRAWS: f64 = 30.0;
}

/// 13_6's last sentence: "If the cylinder is in the wrong place," is 8 of its 24 words; the
/// outlet outline waits for "or the outlet". A share of the sentence, not a heard boundary.
const OUTLET_SHARE: f64 = 8.0 / 24.0;

/// The steady cylinder's test: Schäfer and Turek 1996's "Test case 2D-1 (steady)", "with Um = 0.3
/// m/s, yielding the Reynolds number Re = 20" (the retained text
/// `navier-closing-review01/inputs/sources/schaefer-turek-1996-pdftotext01.txt`, ea2f08f7, lines
/// 167 and 171). `tools/fluid_reference/dfg_tunnel.c` sets that inflow for the steady flow, and the
/// frozen plan's published ranges are their table 3, "Results for steady test case 2D-1". The
/// film's tunnel is their 2D-2 at Re = 100, a different flow.
const STEADY_CASE: &str = "2D-1";
const STEADY_REYNOLDS: u32 = 20;
/// The frozen assessment's settling test for lift over the window: its peak-to-peak drift and the
/// drift allowed (`navier-solver-steady04/steady-assessment.json`, e67de824, `drift_within_limit`
/// false; drag and pressure drop pass theirs).
const LIFT_DRIFT: [f64; 2] = [0.004531346023466468, 0.003];
/// The cylinder card's last line.
const CYLINDER_VERDICT: &str =
    "Budget met; lift not settled; all three miss their ranges: not a validation; cause not identified";

/// 13_10's Precisely, sentences 1 and 2 of the full script's, with its formulas typeset.
const PRECISELY_13_10: [&str; 3] = [
    "For a constant-density incompressible Newtonian fluid, stationary no-slip walls or periodic boundaries",
    "and no body force give dE/dt = −2ρν∫D:D dV ≤ 0, where E = ∫½ρ|u|² dV over the domain is total kinetic energy,",
    "and D is the symmetric velocity gradient. Zero strain permits equality.",
];
/// Its sentence 5, beside the measured energies.
const PRECISELY_13_10_GAP: [&str; 2] = [
    "Compare a discrete measured budget against the declared continuum balance;",
    "any gap may contain several numerical errors, not automatically only dissipation.",
];
/// Its sentence 4, beside the open tunnel.
const PRECISELY_13_10_OPEN: [&str; 1] = [
    "Moving-wall work and open-boundary pressure/viscous work and advective energy flux must be retained when present.",
];

const PRECISELY_Y: f64 = -3.42;
/// The question card, held from 13_2 to the end.
const QUESTION_Y: f64 = 2.62;

// The ledger table.
const COLUMNS: [f64; 4] = [-6.9, -3.3, 0.15, 3.6];
const HEADERS: [&str; 4] = ["change", "quantity compared", "observed difference", "limits"];
const HEAD_Y: f64 = 1.86;
const FIRST_ROW_Y: f64 = 1.2;
const ROW_GAP: f64 = 0.74;
const CELL_SIZE: f64 = 18.0;
/// Longest text line a ledger cell may hold, in characters (checked by the tests).
#[cfg(test)]
const CELL_CHARS: usize = 30;

// The tunnel thumbnail, in scene units: the cylinder's diameter is TUNNEL_D.
const TUNNEL_CENTRE: [f64; 2] = [-1.2, 0.2];
const TUNNEL_SIZE: [f64; 2] = [8.6, 3.0];
const TUNNEL_D: f64 = 0.7;
/// The intended cylinder's centre, 2.6 diameters from the inlet on the centre line.
const CYLINDER_AT: [f64; 2] = [-5.5 + 2.6 * TUNNEL_D, 0.2];

fn dye() -> Srgb {
    hex("#F49BBE")
}

fn card() -> Srgb {
    hex("#182433")
}

fn fail(text: &'static str) -> fmn::Error {
    fmn::Error::Scene(SceneError::InvalidConfig(text))
}

// ------------------------------------------------------------------------------------- timing

fn fps(stage: &Stage<'_>) -> f64 {
    f64::from(stage.scene().time().fps())
}

/// Advance exactly `frames` frames: fmn rounds a wait up to whole frames on the exact f64, and
/// half a frame less always lands on the count.
fn wait_frames(stage: &mut Stage<'_>, frames: u64) -> fmn::Result<()> {
    if frames > 0 {
        let rate = fps(stage);
        stage.wait((frames as f64 - 0.5) / rate)?;
    }
    Ok(())
}

/// Hold the picture for `seconds`, rounded up to whole frames.
fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let frames = (seconds * fps(stage) - 1e-6).ceil().max(1.0) as u64;
    wait_frames(stage, frames)
}

/// Redraws for an animation of `seconds`: how many, and the whole frames each lasts.
fn ticks(stage: &Stage<'_>, seconds: f64) -> (usize, u64) {
    let rate = fps(stage);
    let per = (rate / pacing::REDRAWS).round().max(1.0);
    let count = (seconds * rate / per).round().max(1.0);
    (count as usize, per as u64)
}

fn ease(s: f64) -> f64 {
    s * s * (3.0 - 2.0 * s)
}

fn mix(a: f64, b: f64, s: f64) -> f64 {
    a + (b - a) * s
}

/// Wait until a share of sentence `j` of line `id` has passed, by the narrator's sentence starts
/// and the line's length; nothing if that moment has passed or the line has not been said.
fn until_share(stage: &mut Stage<'_>, narrator: &Narrator, id: &str, j: usize, share: f64) -> fmn::Result<()> {
    let starts = narrator.sentence_starts(id);
    let (Some(at), Some(&begin), Some(length)) = (narrator.sentence_at(id, j), starts.get(j), narrator.length(id))
    else {
        return Ok(());
    };
    let end = starts.get(j + 1).copied().unwrap_or(length);
    let target = at + share * (end - begin);
    let now = stage.scene().time().to_f64();
    if target > now {
        let frames = ((target - now) * fps(stage) - 1e-6).ceil().max(1.0) as u64;
        wait_frames(stage, frames)?;
    }
    Ok(())
}

// ------------------------------------------------------------------------------------ numbers

/// Two decimals with a true minus sign.
fn two(x: f64) -> String {
    digits(x, 2)
}

fn digits(x: f64, decimals: usize) -> String {
    format!("{x:.decimals$}").replace('-', "−")
}

fn superscript(n: i32) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut out = String::new();
    if n < 0 {
        out.push('⁻');
    }
    for c in n.unsigned_abs().to_string().chars() {
        out.push(DIGITS[c.to_digit(10).unwrap_or(0) as usize]);
    }
    out
}

/// A power of ten, "10⁻⁸".
fn power(n: i32) -> String {
    format!("10{}", superscript(n))
}

/// The power of ten a budget is.
fn budget_power(budget: f64) -> String {
    power(budget.log10().round() as i32)
}

/// Scientific notation with `decimals` decimals in the mantissa, "2.1 × 10⁻⁷".
fn sci(x: f64, decimals: usize) -> String {
    let mut exponent = x.abs().log10().floor() as i32;
    let mut mantissa = x / 10f64.powi(exponent);
    if digits(mantissa.abs(), decimals).starts_with("10") {
        exponent += 1;
        mantissa /= 10.0;
    }
    format!("{} × {}", digits(mantissa, decimals), power(exponent))
}

/// Short fractions by name: ½, ¼ and ⅛.
fn fraction(x: f64) -> String {
    if (x - 0.5).abs() < 1e-12 {
        "½".to_owned()
    } else if (x - 0.25).abs() < 1e-12 {
        "¼".to_owned()
    } else if (x - 0.125).abs() < 1e-12 {
        "⅛".to_owned()
    } else {
        two(x)
    }
}

fn whole(x: f64) -> String {
    format!("{}", x.round() as i64)
}

/// A step in seconds: two decimals down to 0.01 s, five below.
fn step_text(dt: f64) -> String {
    digits(dt, if dt < 0.0099 { 5 } else { 2 })
}

// ----------------------------------------------------------------------------------- drawing

// Kit text is registered but detached until the scene admits it.
fn shown(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let text = match super::exponent_tex(words) {
        Some(tex) => kit.tex(stage, &tex, size, color, &[], at)?,
        None => kit.text(stage, words, size, color, at)?,
    };
    stage.scene_mut().add(&[text])?;
    Ok(text)
}

fn shown_left(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let text = match super::exponent_tex(words) {
        Some(tex) => kit.tex_left(stage, &tex, size, color, &[], at)?,
        None => kit.text_left(stage, words, size, color, at)?,
    };
    stage.scene_mut().add(&[text])?;
    Ok(text)
}

fn shown_tex(kit: &Kit, stage: &mut Stage<'_>, source: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let tex = kit.tex(stage, source, size, color, &[], at)?;
    stage.scene_mut().add(&[tex])?;
    Ok(tex)
}

fn clear(stage: &mut Stage<'_>, mobs: &mut Vec<Mob>) {
    stage.scene_mut().remove(mobs.as_slice());
    mobs.clear();
}

fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, weight: f64) -> fmn::Result<Mob> {
    stage.add(
        Line::new(a, b)
            .style(Style::default().stroke(color, weight, 1.0))
            .build()?,
    )
}

/// A dotted segment of `dashes` dashes, each 63% ink (chapter 11's exact outline).
fn dotted(stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], dashes: usize, color: Srgb) -> fmn::Result<Vec<Mob>> {
    (0..dashes)
        .map(|n| {
            let (s0, s1) = (n as f64 / dashes as f64, (n as f64 + 0.63) / dashes as f64);
            let start = v(mix(a[0], b[0], s0), mix(a[1], b[1], s0));
            let end = v(mix(a[0], b[0], s1), mix(a[1], b[1], s1));
            line(stage, start, end, color, 1.45)
        })
        .collect()
}

/// A dotted circle, the exact-reference outline's style.
fn dotted_circle(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let dashes = 28;
    (0..dashes)
        .map(|n| {
            let (a, b) = (TAU * n as f64 / dashes as f64, TAU * (n as f64 + 0.6) / dashes as f64);
            line(
                stage,
                v(at[0] + radius * a.cos(), at[1] + radius * a.sin()),
                v(at[0] + radius * b.cos(), at[1] + radius * b.sin()),
                color,
                1.6,
            )
        })
        .collect()
}

/// A rectangle centred at `at`: fill colour and opacity, edge colour and width (0 for none).
fn panel(
    stage: &mut Stage<'_>,
    at: [f64; 2],
    size: [f64; 2],
    fill: (Srgb, f64),
    edge: (Srgb, f64),
) -> fmn::Result<Mob> {
    stage.add(
        Rectangle::new()
            .width(size[0])
            .height(size[1])
            .style(Style::default().fill(fill.0, fill.1).stroke(edge.0, edge.1, 1.0))
            .build()?
            .moved_to(v(at[0], at[1])),
    )
}

/// An outline only.
fn frame(stage: &mut Stage<'_>, at: [f64; 2], size: [f64; 2], color: Srgb, weight: f64) -> fmn::Result<Mob> {
    panel(stage, at, size, (color, 0.0), (color, weight))
}

fn ring(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb, weight: f64) -> fmn::Result<Mob> {
    stage.add(
        Circle::new()
            .radius(radius)
            .style(Style::default().fill(color, 0.0).stroke(color, weight, 1.0))
            .build()
            .moved_to(v(at[0], at[1])),
    )
}

fn disc(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb) -> fmn::Result<Mob> {
    stage.add(
        Dot::new()
            .point(v(at[0], at[1]))
            .radius(radius)
            .style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0))
            .build(),
    )
}

fn arrow(stage: &mut Stage<'_>, from: [f64; 2], to: [f64; 2], color: Srgb, thickness: f64) -> fmn::Result<Mob> {
    stage.add(
        Arrow::new(v(from[0], from[1]), v(to[0], to[1]))
            .buff(0.0)
            .thickness(thickness)
            .style(Style::default().fill(color, 1.0).stroke(color, 1.0, 1.0))
            .build()?,
    )
}

/// A horizontal bracket over `x` at height `y`, its end ticks pointing up (+1) or down (−1).
fn bracket(stage: &mut Stage<'_>, x: [f64; 2], y: f64, toward: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let tick = 0.14 * toward;
    Ok(vec![
        line(stage, v(x[0], y), v(x[1], y), color, 2.0)?,
        line(stage, v(x[0], y), v(x[0], y + tick), color, 2.0)?,
        line(stage, v(x[1], y), v(x[1], y + tick), color, 2.0)?,
    ])
}

/// A tick (true) or a cross (false), drawn as lines so no glyph is needed.
fn verdict(stage: &mut Stage<'_>, at: [f64; 2], good: bool) -> fmn::Result<Vec<Mob>> {
    let [x, y] = at;
    if good {
        let color = Palette::green();
        Ok(vec![
            line(stage, v(x - 0.13, y), v(x - 0.04, y - 0.1), color, 2.6)?,
            line(stage, v(x - 0.04, y - 0.1), v(x + 0.14, y + 0.13), color, 2.6)?,
        ])
    } else {
        let color = Palette::red();
        Ok(vec![
            line(stage, v(x - 0.12, y - 0.12), v(x + 0.12, y + 0.12), color, 2.6)?,
            line(stage, v(x - 0.12, y + 0.12), v(x + 0.12, y - 0.12), color, 2.6)?,
        ])
    }
}

/// A lower third with the exact form of a simplification, one text line per entry.
fn precisely<S: AsRef<str>>(kit: &Kit, stage: &mut Stage<'_>, lines: &[S]) -> fmn::Result<Vec<Mob>> {
    lines
        .iter()
        .enumerate()
        .map(|(i, words)| {
            let at = v(0.0, PRECISELY_Y - 0.4 * i as f64);
            shown(kit, stage, words.as_ref(), 19.0, Palette::dim(), at)
        })
        .collect()
}

/// A rough text width for sizing chips: about half an em per character.
fn text_width(words: &str, size: f64) -> f64 {
    words.chars().count() as f64 * size * 0.0052
}

/// Whether a control was held fixed or changed: a chip makes it visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Fixed,
    Changed,
}

/// "fixed   cells 32 × 32" in a dim outline, or "changed   …" in gold.
fn chip(stage: &mut Stage<'_>, kit: &Kit, at: [f64; 2], words: &str, control: Control) -> fmn::Result<Vec<Mob>> {
    let (tag, edge, ink) = match control {
        Control::Fixed => ("fixed", Palette::dim(), Palette::ink()),
        Control::Changed => ("changed", Palette::gold(), Palette::gold()),
    };
    let label = format!("{tag}   {words}");
    let width = text_width(&label, 20.0) + 0.45;
    Ok(vec![
        frame(stage, at, [width, 0.46], edge, 1.6)?,
        shown(kit, stage, &label, 20.0, ink, v(at[0], at[1]))?,
    ])
}

/// The prediction card, held through the countdown (chapter 11's `predict`). Whatever would give
/// the answer away is not drawn until the caller's reveal line.
fn predict(stage: &mut Stage<'_>, kit: &Kit, question: [&str; 2], options: &str) -> fmn::Result<()> {
    let asked = [
        panel(stage, [-0.85, 0.51], [10.2, 2.6], (card(), 1.0), (card(), 1.4))?,
        shown(kit, stage, question[0], 34.0, Palette::ink(), v(-0.85, 1.35))?,
        shown(kit, stage, question[1], 34.0, Palette::ink(), v(-0.85, 0.82))?,
        shown(kit, stage, options, 25.0, Palette::dim(), v(-0.85, 0.15))?,
    ];
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    for second in (1..=pacing::COUNTDOWN).rev() {
        let clock = shown(kit, stage, &second.to_string(), 34.0, Palette::hero(), v(-0.85, -0.38))?;
        pause(stage, 1.0)?;
        stage.scene_mut().remove(&[clock]);
    }
    stage.scene_mut().remove(&asked);
    Ok(())
}

/// The opening's question card (chapter 12's), held as a strip above the ledger.
fn question_card(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    Ok(vec![
        panel(
            stage,
            [0.0, QUESTION_Y],
            [7.6, 0.58],
            (card(), 1.0),
            (Palette::dim(), 1.4),
        )?,
        shown(
            kit,
            stage,
            "Does halving the cells change the answer?",
            25.0,
            Palette::ink(),
            v(0.0, QUESTION_Y),
        )?,
    ])
}

/// A dial with tick marks and a needle (chapter 12's), with its label below.
fn dial(stage: &mut Stage<'_>, kit: &Kit, at: [f64; 2], label: &str) -> fmn::Result<Vec<Mob>> {
    let ink = Palette::ink();
    let [x, y] = at;
    let mut out = vec![ring(stage, at, 0.62, ink, 2.0)?];
    for n in 0..5 {
        let angle = PI * (0.9 - 0.2 * f64::from(n));
        let (c, s) = (angle.cos(), angle.sin());
        out.push(line(
            stage,
            v(x + 0.5 * c, y + 0.5 * s),
            v(x + 0.62 * c, y + 0.62 * s),
            ink,
            1.6,
        )?);
    }
    let needle = 0.62 * PI;
    out.push(line(
        stage,
        v(x, y),
        v(x + 0.46 * needle.cos(), y + 0.46 * needle.sin()),
        Palette::gold(),
        2.4,
    )?);
    out.push(shown(kit, stage, label, 25.0, ink, v(x, y - 0.95))?);
    Ok(out)
}

/// Chapter 12's "declared tolerance" band on a dial's rim by its needle.
fn tolerance_band(stage: &mut Stage<'_>, kit: &Kit, at: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let [x, y] = at;
    let (a, b) = (0.57 * PI, 0.67 * PI);
    Ok(vec![
        line(
            stage,
            v(x + 0.66 * a.cos(), y + 0.66 * a.sin()),
            v(x + 0.66 * b.cos(), y + 0.66 * b.sin()),
            Palette::teal(),
            6.0,
        )?,
        shown(kit, stage, "declared tolerance", 16.0, Palette::dim(), v(x, y + 0.95))?,
    ])
}

// ---------------------------------------------------------------------------------- the ledger

/// What kind of entry a ledger cell holds; the colour says it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    Plain,
    /// A measured result that passed what it was checked against.
    Measured,
    /// A measured result that missed what it was checked against.
    Missed,
    /// Not measured.
    Open,
    /// A schematic or a plan, not a run.
    Illustration,
    /// The declared tolerance (chapter 12's band colour).
    Declared,
}

fn tone_color(tone: Tone) -> Srgb {
    match tone {
        Tone::Plain => Palette::ink(),
        Tone::Measured => Palette::green(),
        Tone::Missed => Palette::red(),
        Tone::Open => Palette::dim(),
        Tone::Illustration => Palette::gold(),
        Tone::Declared => Palette::teal(),
    }
}

#[derive(Clone, Debug)]
struct Cell {
    lines: Vec<String>,
    tone: Tone,
}

fn cell(lines: &[&str], tone: Tone) -> Cell {
    Cell {
        lines: lines.iter().map(|s| (*s).to_owned()).collect(),
        tone,
    }
}

fn plain(lines: &[&str]) -> Cell {
    cell(lines, Tone::Plain)
}

fn open() -> Cell {
    cell(&["not measured"], Tone::Open)
}

/// One ledger row: change, quantity compared, observed difference, limits.
#[derive(Clone, Debug)]
struct Entry {
    cells: [Cell; 4],
}

impl Entry {
    fn new(cells: [Cell; 4]) -> Self {
        Self { cells }
    }

    /// The row as planned: its change named, the rest not yet measured.
    fn planned(&self) -> Self {
        Self::new([self.cells[0].clone(), open(), open(), open()])
    }
}

fn row_y(row: usize) -> f64 {
    FIRST_ROW_Y - ROW_GAP * row as f64
}

/// The ledger's headers, a rule, and the page's name.
fn ledger_head(stage: &mut Stage<'_>, kit: &Kit, page: &str) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for (x, header) in COLUMNS.iter().zip(HEADERS) {
        out.push(shown_left(kit, stage, header, 20.0, Palette::dim(), v(*x, HEAD_Y))?);
    }
    out.push(line(
        stage,
        v(-7.0, HEAD_Y - 0.26),
        v(7.0, HEAD_Y - 0.26),
        Palette::faint(),
        1.4,
    )?);
    out.push(shown_left(
        kit,
        stage,
        &format!("Ledger: {page}"),
        19.0,
        Palette::gold(),
        v(-6.9, -2.62),
    )?);
    Ok(out)
}

fn ledger_row(stage: &mut Stage<'_>, kit: &Kit, row: usize, entry: &Entry) -> fmn::Result<Vec<Mob>> {
    let y = row_y(row);
    let mut out = Vec::new();
    for (x, cell) in COLUMNS.iter().zip(&entry.cells) {
        let lines = cell.lines.len() as f64;
        for (k, words) in cell.lines.iter().enumerate() {
            let ly = y + 0.17 * (lines - 1.0) - 0.34 * k as f64;
            out.push(shown_left(
                kit,
                stage,
                words,
                CELL_SIZE,
                tone_color(cell.tone),
                v(*x, ly),
            )?);
        }
    }
    Ok(out)
}

fn ledger(stage: &mut Stage<'_>, kit: &Kit, page: &str, entries: &[Entry]) -> fmn::Result<Vec<Mob>> {
    let mut out = ledger_head(stage, kit, page)?;
    for (row, entry) in entries.iter().enumerate() {
        out.extend(ledger_row(stage, kit, row, entry)?);
    }
    Ok(out)
}

/// A gold outline around a row, or around one column of a row.
fn highlight(stage: &mut Stage<'_>, row: usize, column: Option<usize>) -> fmn::Result<Mob> {
    let y = row_y(row);
    let (left, right) = match column {
        Some(c) => (COLUMNS[c] - 0.1, COLUMNS.get(c + 1).map_or(7.0, |next| next - 0.15)),
        None => (-7.0, 7.0),
    };
    frame(
        stage,
        [(left + right) / 2.0, y],
        [right - left, 0.7],
        Palette::gold(),
        2.0,
    )
}

const PAGE_NUMERICS: &str = "numerical choices";
const PAGE_PROBLEM: &str = "the problem and the model";
const PAGE_UNCERTAIN: &str = "what remains uncertain";

/// Page one, numerical choices, every row as measured or planned.
fn numerics_entries() -> [Entry; 5] {
    let [loose, tight] = PRESSURE;
    let budgets = format!("{} → {}", budget_power(loose.budget), budget_power(tight.budget));
    let imbalance = format!(
        "imbalance ÷ {}",
        whole(loose.max_volume_defect / tight.max_volume_defect)
    );
    let error = format!("error {} both", sci(tight.error, 3));
    let pressure_fixed = format!("{0} × {0}, Δt {1} s, fixed", tight.cells, tight.dt);
    let pressure_limits = format!(
        "C {}, r {}",
        two(tight.max_courant),
        digits(tight.diffusion_weight(), 3)
    );
    let steady = steady_change();
    let cylinder = format!(
        "{CYLINDER_STEPS} steps, last {} s",
        CYLINDER_WINDOW[1] - CYLINDER_WINDOW[0]
    );
    let steps = format!("Δt {} → {} → {} s", TIME[0].dt, TIME[1].dt, TIME[2].dt);
    let cells = format!("cells {0} × {0} fixed", TIME[0].cells);
    let reference = format!("{TIME_REFERENCE_STEPS}-step run, same cells");
    let time_ratios = format!("÷ {}, then ÷ {}", digits(TIME_RATIOS[0], 1), digits(TIME_RATIOS[1], 1));
    let time_span = format!("{} → {}", sci(TIME_DIFFERENCES[0], 1), sci(TIME_DIFFERENCES[2], 1));
    let largest_c = TIME.iter().map(|r| r.max_courant).fold(0.0, f64::max);
    let largest_r = TIME.iter().map(Run::diffusion_weight).fold(0.0, f64::max);
    let time_limits = format!("C ≤ {}, r ≤ {}", two(largest_c), digits(largest_r, 3));
    let rungs = ladder();
    let lock = rungs.iter().find(|r| r.locked).map_or_else(
        || "r never binds".to_owned(),
        |r| format!("r < ¼ binds at {} cells", r.cells),
    );
    let fallen: Vec<String> = rungs.iter().filter(|r| r.locked).map(|r| two(r.courant)).collect();
    let fallen = format!("C falls to {}", fallen.join(", "));
    let spread: Vec<String> = STEP_ONLY_COURANT
        .iter()
        .map(|&c| whole(step_only_spreading(c)))
        .collect();
    let pulse = format!("{} → {}", spread.join(", "), whole(STEP_ONLY_LIMIT));
    let pulse_steps: Vec<String> = STEP_ONLY_COURANT.iter().map(|&c| fraction(c)).collect();
    let pulse_steps = format!("C = {}", pulse_steps.join(", "));
    [
        Entry::new([
            plain(&["pressure budget", &budgets]),
            plain(&["leftover imbalance;", "error against exact flow"]),
            cell(&[&imbalance, &error], Tone::Measured),
            plain(&[&pressure_fixed, &pressure_limits]),
        ]),
        Entry::new([
            plain(&[&steady, &cylinder]),
            plain(&["budget; drag, lift,", "pressure drop vs published"]),
            cell(
                &["budget met; lift not settled;", "all three miss their ranges"],
                Tone::Missed,
            ),
            plain(&["cause not identified;", "not a validation"]),
        ]),
        Entry::new([
            plain(&[&steps, &cells]),
            plain(&["difference from the", &reference]),
            cell(&[&time_ratios, &time_span], Tone::Measured),
            plain(&[&time_limits, "the step's own error"]),
        ]),
        Entry::new([
            plain(&["cells and step together", "C held while r allows"]),
            open(),
            open(),
            cell(&[&lock, &fallen], Tone::Illustration),
        ]),
        Entry::new([
            plain(&["dye pulse: step only", "cells fixed"]),
            plain(&["smearing against the", "exact pulse"]),
            cell(&[&pulse, "exact pulse: 0"], Tone::Illustration),
            plain(&["illustration: uniform wind", &pulse_steps]),
        ]),
    ]
}

/// Page two, the problem and the model. The rounding row is written once `rounding` is true.
fn problem_entries(rounding: bool) -> [Entry; 4] {
    let cells: Vec<String> = STAIRCASE_CELLS.iter().map(ToString::to_string).collect();
    let offset = format!("offset stays {} D", two(Staircase::new(STAIRCASE_CELLS[0]).offset()));
    let at = format!("at {} cells per D", cells.join(", "));
    let digits_row = Entry::new([
        plain(&["more digits"]),
        plain(&["rounding error"]),
        open(),
        plain(&["Taylor–Green ran in 64-bit;", "no higher-precision run"]),
    ]);
    [
        Entry::new([
            plain(&["more cells"]),
            plain(&["cylinder position"]),
            cell(&[&offset, &at], Tone::Illustration),
            plain(&["fix the input instead;", "illustration"]),
        ]),
        Entry::new([
            plain(&["outlet farther away"]),
            plain(&["wake near the outlet"]),
            open(),
            plain(&["a boundary choice"]),
        ]),
        Entry::new([
            plain(&["a 3D model"]),
            plain(&["2D against 3D results"]),
            open(),
            plain(&["a model question,", "not just the mesh"]),
        ]),
        if rounding { digits_row } else { digits_row.planned() },
    ]
}

/// Page three, what remains uncertain: the declared tolerance first, then the open slots.
fn uncertain_entries() -> [Entry; 5] {
    [
        Entry::new([
            cell(&["declared tolerance", "set before comparing"], Tone::Declared),
            plain(&["the tunnel's answer"]),
            cell(&["not yet declared"], Tone::Open),
            plain(&["no universal threshold"]),
        ]),
        Entry::new([
            plain(&[&steady_change()]),
            plain(&["drag, lift, pressure drop"]),
            cell(&["lift not settled;", "all three miss their ranges"], Tone::Missed),
            plain(&["cause not identified"]),
        ]),
        Entry::new([
            plain(&["outlet rule, 2D model"]),
            plain(&["the wake; 3D results"]),
            open(),
            plain(&["boundary and model choices"]),
        ]),
        Entry::new([
            plain(&["wake swing period"]),
            plain(&["a defined quantity"]),
            open(),
            plain(&["same problem first"]),
        ]),
        Entry::new([
            plain(&["tunnel energy budget"]),
            plain(&["in − out − viscous loss"]),
            open(),
            plain(&["an open boundary"]),
        ]),
    ]
}

// ------------------------------------------------------------------------- 13_1's illustrations

/// Chapter 11's blend after its whole trip, small, over the exact pulse's dotted outline.
fn mini_pulse(stage: &mut Stage<'_>, centre: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let mut pulse = Pulse::new(0);
    for _ in 0..pulse.step_count() {
        pulse.advance(Transport::Blend, false);
    }
    let window = [-8.0, 12.0];
    let (width, height) = (3.4, 0.9);
    let x = |c: f64| centre[0] - width / 2.0 + width * (c - window[0]) / (window[1] - window[0]);
    let base = centre[1] - height / 2.0;
    let mut out = vec![line(
        stage,
        v(x(window[0]), base),
        v(x(window[1]), base),
        Palette::dim(),
        1.4,
    )?];
    for (index, &q) in pulse.values.iter().enumerate() {
        let [a, b] = pulse.riding_bounds(index);
        if q.abs() < 1e-5 || b <= window[0] || a >= window[1] {
            continue;
        }
        let at = [(x(a) + x(b)) / 2.0, base + height * q / 2.0];
        out.push(panel(
            stage,
            at,
            [(x(b) - x(a)) * 0.9, height * q],
            (dye(), 0.73),
            (dye(), 0.0),
        )?);
    }
    let (top, edge) = (base + height, PULSE_WIDTH as f64);
    out.extend(dotted(stage, [x(0.0), top], [x(edge), top], 6, Palette::ink())?);
    out.extend(dotted(stage, [x(0.0), base], [x(0.0), top], 5, Palette::ink())?);
    out.extend(dotted(stage, [x(edge), base], [x(edge), top], 5, Palette::ink())?);
    Ok(out)
}

/// Chapter 12's parcel: the exact position after ½ s, and forward Euler's after four steps.
fn parcel_miss(stage: &mut Stage<'_>, kit: &Kit, centre: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let exact = channel_exact(CHANNEL_TIME);
    let euler = channel_euler(CHANNEL_STEPS, CHANNEL_TIME / CHANNEL_STEPS as f64);
    let x = |m: f64| centre[0] - 1.6 + 3.2 * (m - 1.0) / 0.4;
    let y = centre[1];
    let miss = format!("{} cm short", whole(100.0 * (exact - euler)));
    let mut out = vec![
        line(stage, v(x(1.0), y), v(x(1.4), y), Palette::dim(), 1.4)?,
        disc(stage, [x(exact), y], 0.09, dye())?,
        ring(stage, [x(euler), y], 0.09, Palette::gold(), 2.0)?,
    ];
    out.extend(bracket(stage, [x(euler), x(exact)], y + 0.28, -1.0, Palette::gold())?);
    out.push(shown(
        kit,
        stage,
        &miss,
        20.0,
        Palette::gold(),
        v((x(euler) + x(exact)) / 2.0, y + 0.55),
    )?);
    out.push(shown(
        kit,
        stage,
        "a step too far: chapter 12's parcel",
        18.0,
        Palette::dim(),
        v(centre[0], y - 0.45),
    )?);
    Ok(out)
}

// ---------------------------------------------------------------------------- 13_3's evidence

/// A vertical log axis from 10⁻¹¹ to 10⁻³.
const LOG_AXIS: [i32; 2] = [-11, -3];
const LOG_X: f64 = -3.9;
const LOG_Y: [f64; 2] = [-2.25, 1.0];

fn log_y(value: f64) -> f64 {
    let s = (value.log10() - f64::from(LOG_AXIS[0])) / f64::from(LOG_AXIS[1] - LOG_AXIS[0]);
    mix(LOG_Y[0], LOG_Y[1], s)
}

fn log_axis(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let mut out = vec![line(stage, v(LOG_X, LOG_Y[0]), v(LOG_X, LOG_Y[1]), dim, 1.6)?];
    for exponent in (LOG_AXIS[0]..=LOG_AXIS[1]).step_by(2) {
        let y = log_y(10f64.powi(exponent));
        out.push(line(stage, v(LOG_X - 0.08, y), v(LOG_X, y), dim, 1.4)?);
        out.push(shown(kit, stage, &power(exponent), 18.0, dim, v(LOG_X - 0.5, y))?);
    }
    Ok(out)
}

/// The residual column's x, and the observable column's.
const RESIDUAL_X: f64 = -1.9;
const OBSERVABLE_X: f64 = 2.6;

/// 13_3, sentence 1: the controls, then the leftover imbalance at both budgets.
fn residual_column(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let [loose, tight] = PRESSURE;
    let (red, x) = (Palette::red(), RESIDUAL_X);
    let mut out = chip(
        stage,
        kit,
        [-4.55, 1.72],
        &format!("cells {0} × {0}", tight.cells),
        Control::Fixed,
    )?;
    out.extend(chip(
        stage,
        kit,
        [-1.25, 1.72],
        &format!("step {} s", tight.dt),
        Control::Fixed,
    )?);
    let budgets = format!(
        "pressure budget {} → {}",
        budget_power(loose.budget),
        budget_power(tight.budget)
    );
    out.extend(chip(stage, kit, [3.35, 1.72], &budgets, Control::Changed)?);
    out.extend(log_axis(stage, kit)?);
    out.push(shown(kit, stage, "leftover imbalance", 21.0, red, v(x, -2.72))?);
    for run in [loose, tight] {
        let y = log_y(run.max_volume_defect);
        out.push(disc(stage, [x, y], 0.09, red)?);
        let text = format!(
            "budget {}:  {}",
            budget_power(run.budget),
            sci(run.max_volume_defect, 1)
        );
        out.push(shown_left(kit, stage, &text, 18.0, red, v(x + 0.2, y))?);
    }
    let (top, bottom) = (
        log_y(loose.max_volume_defect) - 0.15,
        log_y(tight.max_volume_defect) + 0.15,
    );
    out.push(arrow(stage, [x - 0.35, top], [x - 0.35, bottom], red, 1.4)?);
    let ratio = format!("÷ {}", whole(loose.max_volume_defect / tight.max_volume_defect));
    out.push(shown(kit, stage, &ratio, 19.0, red, v(x - 0.95, (top + bottom) / 2.0))?);
    Ok(out)
}

/// 13_3, sentence 2: the flow and the quantity we care about, in velocity blue.
fn observable_column(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let [loose, tight] = PRESSURE;
    let (blue, x) = (Palette::blue(), OBSERVABLE_X);
    let caption = "the flow, and the error against the exact flow";
    let mut out = vec![shown(kit, stage, caption, 21.0, blue, v(x + 1.0, -2.72))?];
    for (dx, run) in [(-0.12, loose), (0.12, tight)] {
        out.push(disc(stage, [x + dx, log_y(run.error)], 0.09, blue)?);
    }
    let error = format!("error {} at both budgets", sci(tight.error, 3));
    out.push(shown_left(
        kit,
        stage,
        &error,
        18.0,
        blue,
        v(x + 0.3, log_y(tight.error)),
    )?);
    let moved = log_y(PRESSURE_FLOW_CHANGE);
    out.push(ring(stage, [x, moved], 0.09, blue, 2.0)?);
    let flow = format!("flow moved {} of its size", sci(PRESSURE_FLOW_CHANGE, 1));
    out.push(shown_left(kit, stage, &flow, 18.0, blue, v(x + 0.3, moved))?);
    Ok(out)
}

/// The ledger's name for the steady cylinder run, "none: Re 20 cylinder, 2D-1".
fn steady_change() -> String {
    format!("none: Re {STEADY_REYNOLDS} cylinder, {STEADY_CASE}")
}

fn cylinder_title() -> String {
    let case = format!("Schäfer–Turek {STEADY_CASE}, Re = {STEADY_REYNOLDS}");
    format!(
        "Steady cylinder, {case}: {CYLINDER_STEPS} steps, the last {} s of {} s",
        CYLINDER_WINDOW[1] - CYLINDER_WINDOW[0],
        CYLINDER_WINDOW[1]
    )
}

/// The one budget check. The residual is the leftover imbalance (10_11): the solver's scaled
/// pressure residual, −Δt² times the residual of the pressure equation, equals Δt × divergence
/// cell for cell, so its two records are one number, not two checks.
fn cylinder_check() -> fmn::Result<String> {
    let (residual, imbalance) = (sci(CYLINDER_PRESSURE_RESIDUAL, 3), sci(CYLINDER_VOLUME_DEFECT, 3));
    if residual != imbalance {
        return Err(fail("13_3: the cylinder's residual and imbalance records differ"));
    }
    if CYLINDER_PRESSURE_RESIDUAL.max(CYLINDER_VOLUME_DEFECT) > CYLINDER_BUDGET {
        return Err(fail("13_3: the cylinder left more than its budget"));
    }
    Ok(format!(
        "one check: scaled residual = Δt × divergence, {imbalance} ≤ {}",
        budget_power(CYLINDER_BUDGET)
    ))
}

/// The frozen rule for the force ranges, and each check that overlaps its range but leaves it.
fn range_rule() -> String {
    let partial: Vec<String> = CYLINDER_CHECKS
        .iter()
        .filter(|c| !c.inside() && c.observed[0] <= c.published[1] && c.published[0] <= c.observed[1])
        .map(|c| {
            let d = range_decimals(c);
            let (ours, theirs, sign) = if c.observed[1] > c.published[1] {
                (c.observed[1], c.published[1], ">")
            } else {
                (c.observed[0], c.published[0], "<")
            };
            format!(
                "{} overlaps but reaches {} {sign} {}",
                c.name,
                digits(ours, d),
                digits(theirs, d)
            )
        })
        .collect();
    let rule = "Frozen rule: pass only if this run's whole range lies inside the published one";
    if partial.is_empty() {
        rule.to_owned()
    } else {
        format!("{rule}; {}", partial.join("; "))
    }
}

/// Lift's settling test, beside its range.
fn lift_note() -> fmn::Result<String> {
    let [drift, allowed] = LIFT_DRIFT;
    if drift <= allowed {
        return Err(fail("13_3: the lift note says not settled, but its drift is allowed"));
    }
    Ok(format!("not settled: drift {} > {allowed}", digits(drift, 4)))
}

/// 13_3, sentence 4: the Re = 20 steady cylinder, its one budget met, lift not settled, and the
/// forces outside the published ranges.
fn cylinder_card(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let title = cylinder_title();
    let mut out = vec![shown(kit, stage, &title, 24.0, Palette::ink(), v(0.0, 1.78))?];
    let check = cylinder_check()?;
    let x = -0.3;
    out.push(shown(kit, stage, &check, 19.0, Palette::teal(), v(x, 1.18))?);
    out.extend(verdict(stage, [x + text_width(&check, 19.0) / 2.0 + 0.35, 1.18], true)?);
    let lift = CYLINDER_CHECKS
        .iter()
        .position(|c| c.name == "lift")
        .ok_or_else(|| fail("13_3: no lift row for its settling note"))?;
    for (k, check) in CYLINDER_CHECKS.iter().enumerate() {
        let y = 0.38 - 0.92 * k as f64;
        out.extend(range_row(stage, kit, check, y)?);
        if k == lift {
            out.push(shown_left(
                kit,
                stage,
                &lift_note()?,
                16.0,
                Palette::red(),
                v(3.55, y - 0.43),
            )?);
        }
    }
    out.push(shown(kit, stage, &range_rule(), 16.0, Palette::dim(), v(0.0, -1.95))?);
    let source = "Published ranges: Schäfer and Turek 1996, table 3, as frozen in the benchmark plan";
    out.push(shown(kit, stage, source, 16.0, Palette::dim(), v(0.0, -2.32))?);
    out.push(shown(
        kit,
        stage,
        CYLINDER_VERDICT,
        20.0,
        Palette::gold(),
        v(0.0, -2.78),
    )?);
    Ok(out)
}

/// Decimals for a force range: four below one, three above.
fn range_decimals(check: &RangeCheck) -> usize {
    if check.published[1] < 1.0 { 4 } else { 3 }
}

/// One force check: the published range as a teal band, this run's range as a blue bar.
fn range_row(stage: &mut Stage<'_>, kit: &Kit, check: &RangeCheck, y: f64) -> fmn::Result<Vec<Mob>> {
    let low = check.observed[0].min(check.published[0]);
    let high = check.observed[1].max(check.published[1]);
    let margin = 0.15 * (high - low);
    let (a, b) = (low - margin, high + margin);
    let x = |value: f64| -3.2 + 6.4 * (value - a) / (b - a);
    let mut out = vec![
        shown_left(kit, stage, check.name, 21.0, Palette::ink(), v(-6.6, y))?,
        line(stage, v(x(a), y), v(x(b), y), Palette::faint(), 1.2)?,
        line(
            stage,
            v(x(check.published[0]), y + 0.1),
            v(x(check.published[1]), y + 0.1),
            Palette::teal(),
            6.0,
        )?,
        line(
            stage,
            v(x(check.observed[0]), y - 0.1),
            v(x(check.observed[1]), y - 0.1),
            Palette::blue(),
            6.0,
        )?,
    ];
    let decimals = range_decimals(check);
    let range = |r: [f64; 2]| format!("{} to {}", digits(r[0], decimals), digits(r[1], decimals));
    let published = format!("published {}", range(check.published));
    let run = format!("this run {}", range(check.observed));
    out.push(shown_left(
        kit,
        stage,
        &published,
        16.0,
        Palette::teal(),
        v(3.55, y + 0.13),
    )?);
    out.push(shown_left(kit, stage, &run, 16.0, Palette::blue(), v(3.55, y - 0.15))?);
    out.extend(verdict(stage, [-4.0, y], check.inside())?);
    Ok(out)
}

// ------------------------------------------------------------------------------- 13_4's sliders

const SLIDER_X: [f64; 2] = [-6.0, -0.6];
const CELLS_Y: f64 = 1.3;
const STEP_Y: f64 = 0.45;
const C_Y: f64 = -0.4;
const R_Y: f64 = -1.25;
/// The C track runs from 0 to 0.8, the r track from 0 to 0.5.
const C_SCALE: f64 = 0.8;
const R_SCALE: f64 = 0.5;
const CHIP_AT: [f64; 2] = [-2.4, 2.05];
const TABLE_X: f64 = 0.7;

fn cells_x(cells: usize) -> f64 {
    let s = ((cells as f64).log2() - 4.0) / 5.0;
    mix(SLIDER_X[0], SLIDER_X[1], s)
}

/// The step track runs from the ladder's largest step to its smallest, by halvings.
fn step_x(dt: f64) -> f64 {
    let rungs = ladder();
    let (big, small) = (rungs[0].dt, rungs[rungs.len() - 1].dt);
    let s = (big / dt).log2() / (big / small).log2();
    mix(SLIDER_X[0], SLIDER_X[1], s)
}

fn c_x(courant: f64) -> f64 {
    mix(SLIDER_X[0], SLIDER_X[1], courant / C_SCALE)
}

fn r_x(r: f64) -> f64 {
    mix(SLIDER_X[0], SLIDER_X[1], r.min(R_SCALE) / R_SCALE)
}

/// The four tracks, their ticks, and the solver's two limits in red.
fn slider_tracks(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let (ink, dim, red) = (Palette::ink(), Palette::dim(), Palette::red());
    let mut out = Vec::new();
    let tracks = [
        (CELLS_Y, "cells per side"),
        (STEP_Y, "step Δt, s"),
        (C_Y, "C, cells per step"),
        (R_Y, "r = νΔt/h²"),
    ];
    for (y, label) in tracks {
        out.push(line(stage, v(SLIDER_X[0], y), v(SLIDER_X[1], y), dim, 2.0)?);
        out.push(shown_left(kit, stage, label, 19.0, ink, v(SLIDER_X[0], y + 0.38))?);
    }
    for rung in ladder() {
        let x = cells_x(rung.cells);
        out.push(line(stage, v(x, CELLS_Y - 0.08), v(x, CELLS_Y + 0.08), dim, 1.4)?);
        out.push(shown(
            kit,
            stage,
            &rung.cells.to_string(),
            16.0,
            dim,
            v(x, CELLS_Y - 0.27),
        )?);
        let s = step_x(rung.dt);
        out.push(line(stage, v(s, STEP_Y - 0.08), v(s, STEP_Y + 0.08), dim, 1.4)?);
    }
    for dt in [0.08, 0.04, 0.02, 0.01] {
        out.push(shown(
            kit,
            stage,
            &dt.to_string(),
            16.0,
            dim,
            v(step_x(dt), STEP_Y - 0.27),
        )?);
    }
    let limits = [
        (
            c_x(TG_COURANT_CHECK),
            C_Y,
            format!("{TG_COURANT_CHECK}: this solver's check"),
        ),
        (r_x(TG_DIFFUSION_CHECK), R_Y, "¼: this solver's check".to_owned()),
    ];
    for (x, y, label) in limits {
        out.push(line(stage, v(x, y - 0.2), v(x, y + 0.2), red, 2.6)?);
        out.push(shown(kit, stage, &label, 16.0, red, v(x, y - 0.36))?);
    }
    let checks = format!("this solver checks every run: C < {TG_COURANT_CHECK}, r < ¼");
    out.push(shown_left(kit, stage, &checks, 18.0, dim, v(SLIDER_X[0], -2.05))?);
    Ok(out)
}

/// Where the four knobs sit.
#[derive(Clone, Copy, Debug)]
struct Knobs {
    cells: f64,
    dt: f64,
    courant: f64,
    r: f64,
}

impl Knobs {
    fn of(rung: &Rung) -> Self {
        Self {
            cells: cells_x(rung.cells),
            dt: step_x(rung.dt),
            courant: c_x(rung.courant),
            r: r_x(rung.diffusion_weight),
        }
    }

    /// A measured run, at its largest measured C.
    fn of_run(run: &Run) -> Self {
        Self {
            cells: cells_x(run.cells),
            dt: step_x(run.dt),
            courant: c_x(run.max_courant),
            r: r_x(run.diffusion_weight()),
        }
    }

    fn mix(self, to: Self, s: f64) -> Self {
        Self {
            cells: mix(self.cells, to.cells, s),
            dt: mix(self.dt, to.dt, s),
            courant: mix(self.courant, to.courant, s),
            r: mix(self.r, to.r, s),
        }
    }
}

fn knobs(stage: &mut Stage<'_>, at: Knobs, cells_fixed: bool) -> fmn::Result<Vec<Mob>> {
    let cells_color = if cells_fixed { Palette::dim() } else { Palette::gold() };
    Ok(vec![
        disc(stage, [at.cells, CELLS_Y], 0.13, cells_color)?,
        disc(stage, [at.dt, STEP_Y], 0.13, Palette::gold())?,
        disc(stage, [at.courant, C_Y], 0.11, Palette::green())?,
        disc(stage, [at.r, R_Y], 0.11, Palette::purple())?,
    ])
}

/// Slide the knobs from one place to another.
fn slide(stage: &mut Stage<'_>, drawn: &mut Vec<Mob>, from: Knobs, to: Knobs, cells_fixed: bool) -> fmn::Result<()> {
    let (count, frames) = ticks(stage, pacing::SLIDE);
    for k in 1..=count {
        clear(stage, drawn);
        *drawn = knobs(stage, from.mix(to, ease(k as f64 / count as f64)), cells_fixed)?;
        wait_frames(stage, frames)?;
    }
    Ok(())
}

/// 13_4, sentence 1: the measured step study's table, title and headers.
const MEASURED_X: [f64; 4] = [TABLE_X, 1.75, 2.75, 3.95];

fn measured_head(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let title = format!("measured: cells fixed at {0} × {0}", TIME[0].cells);
    let reference = format!("difference from the {TIME_REFERENCE_STEPS}-step run");
    let mut out = vec![shown_left(kit, stage, &title, 20.0, ink, v(TABLE_X, 1.75))?];
    for (x, header) in MEASURED_X.iter().zip(["Δt, s", "C", "r", reference.as_str()]) {
        out.push(shown_left(kit, stage, header, 17.0, dim, v(*x, 1.3))?);
    }
    Ok(out)
}

fn measured_row(stage: &mut Stage<'_>, kit: &Kit, n: usize) -> fmn::Result<Vec<Mob>> {
    let run = TIME[n];
    let y = 0.9 - 0.4 * n as f64;
    let values = [
        (step_text(run.dt), Palette::ink()),
        (two(run.max_courant), Palette::green()),
        (digits(run.diffusion_weight(), 4), Palette::purple()),
        (sci(TIME_DIFFERENCES[n], 1), Palette::blue()),
    ];
    MEASURED_X
        .iter()
        .zip(values)
        .map(|(x, (text, color))| shown_left(kit, stage, &text, 19.0, color, v(*x, y)))
        .collect()
}

/// 13_4, sentences 2 to 4: the planned ladder's table.
const LADDER_X: [f64; 5] = [TABLE_X, 1.6, 2.55, 3.9, 4.75];

fn ladder_y(n: usize) -> f64 {
    0.92 - 0.34 * n as f64
}

fn ladder_head(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let title = "planned ladder: the solver's own checks, not runs";
    let mut out = vec![shown_left(kit, stage, title, 20.0, Palette::gold(), v(TABLE_X, 1.75))?];
    for (x, header) in LADDER_X.iter().zip(["cells", "steps", "Δt, s", "C", "r"]) {
        out.push(shown_left(kit, stage, header, 17.0, dim, v(*x, 1.3))?);
    }
    Ok(out)
}

fn ladder_row(stage: &mut Stage<'_>, kit: &Kit, n: usize, rung: &Rung) -> fmn::Result<Vec<Mob>> {
    let ink = if rung.locked { Palette::gold() } else { Palette::ink() };
    let values = [
        (rung.cells.to_string(), ink),
        (rung.steps.to_string(), ink),
        (step_text(rung.dt), ink),
        (
            two(rung.courant),
            if rung.locked { Palette::gold() } else { Palette::green() },
        ),
        (digits(rung.diffusion_weight, 4), Palette::purple()),
    ];
    LADDER_X
        .iter()
        .zip(values)
        .map(|(x, (text, color))| shown_left(kit, stage, &text, 19.0, color, v(*x, ladder_y(n))))
        .collect()
}

// ------------------------------------------------------------------------- 13_14's step chart

fn chart_x(courant: f64) -> f64 {
    -5.0 + 13.0 * courant
}

fn chart_y(spread: f64) -> f64 {
    -2.1 + 0.155 * spread
}

fn step_chart(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let title = format!("Smearing over the same {TRAVEL}-cell trip, in square cells");
    let mut out = vec![
        line(
            stage,
            v(chart_x(0.0), chart_y(0.0)),
            v(chart_x(0.6), chart_y(0.0)),
            dim,
            1.6,
        )?,
        line(
            stage,
            v(chart_x(0.0), chart_y(0.0)),
            v(chart_x(0.0), chart_y(25.0)),
            dim,
            1.6,
        )?,
        shown(kit, stage, "cells per step, C", 22.0, dim, v(chart_x(0.3), -2.85))?,
        shown(kit, stage, &title, 22.0, dim, v(chart_x(0.3), 2.02))?,
    ];
    for spread in [0.0, 6.0, 12.0, 18.0, 24.0] {
        let y = chart_y(spread);
        out.push(line(stage, v(chart_x(0.0) - 0.08, y), v(chart_x(0.0), y), dim, 1.4)?);
        out.push(shown(kit, stage, &whole(spread), 19.0, dim, v(chart_x(0.0) - 0.35, y))?);
    }
    for courant in STEP_ONLY_COURANT {
        let x = chart_x(courant);
        out.push(line(stage, v(x, chart_y(0.0)), v(x, chart_y(0.0) - 0.08), dim, 1.4)?);
        out.push(shown(
            kit,
            stage,
            &fraction(courant),
            21.0,
            ink,
            v(x, chart_y(0.0) - 0.36),
        )?);
    }
    Ok(out)
}

/// 13_14's Precisely, built from the computed spreading.
fn precisely_13_14() -> [String; 2] {
    let spread: Vec<String> = STEP_ONLY_COURANT
        .iter()
        .map(|&c| whole(step_only_spreading(c)))
        .collect();
    let steps: Vec<String> = STEP_ONLY_COURANT.iter().map(|&c| fraction(c)).collect();
    [
        format!(
            "Halving only the step on fixed cells: {}, {}, then {} square cells at {}, {} and {} cell per step.",
            spread[0], spread[1], spread[2], steps[0], steps[1], steps[2]
        ),
        format!(
            "That is {0}(1 − C), which heads to {0} as the step shrinks, while the exact pulse spreads 0.",
            whole(STEP_ONLY_LIMIT)
        ),
    ]
}

// ------------------------------------------------------------------------------- the tunnel

fn tunnel_point(centre: [f64; 2], scale: f64, at: [f64; 2]) -> [f64; 2] {
    [
        centre[0] + scale * (at[0] - TUNNEL_CENTRE[0]),
        centre[1] + scale * (at[1] - TUNNEL_CENTRE[1]),
    ]
}

/// The tunnel's walls and inlet arrows, drawn about `centre` at `scale` of the thumbnail.
fn tunnel(stage: &mut Stage<'_>, centre: [f64; 2], scale: f64) -> fmn::Result<Vec<Mob>> {
    let [w, h] = [TUNNEL_SIZE[0] * scale, TUNNEL_SIZE[1] * scale];
    let (left, right) = (centre[0] - w / 2.0, centre[0] + w / 2.0);
    let (top, bottom) = (centre[1] + h / 2.0, centre[1] - h / 2.0);
    let mut out = vec![
        line(stage, v(left, top), v(right, top), Palette::ink(), 2.0)?,
        line(stage, v(left, bottom), v(right, bottom), Palette::ink(), 2.0)?,
    ];
    for k in 0..4 {
        let y = bottom + h * (f64::from(k) + 0.5) / 4.0;
        out.push(arrow(
            stage,
            [left - 0.55 * scale, y],
            [left + 0.05, y],
            Palette::blue(),
            1.4,
        )?);
    }
    Ok(out)
}

fn cylinder(stage: &mut Stage<'_>, centre: [f64; 2], scale: f64, at: [f64; 2]) -> fmn::Result<Mob> {
    let p = tunnel_point(centre, scale, at);
    stage.add(
        Circle::new()
            .radius(TUNNEL_D * scale / 2.0)
            .style(
                Style::default()
                    .fill(Palette::faint(), 1.0)
                    .stroke(Palette::ink(), 1.8, 1.0),
            )
            .build()
            .moved_to(v(p[0], p[1])),
    )
}

/// Where the specified, wrong, cylinder sits: SPECIFIED_OFFSET diameters from the intended one.
fn specified_at() -> [f64; 2] {
    [
        CYLINDER_AT[0] + SPECIFIED_OFFSET[0] * TUNNEL_D,
        CYLINDER_AT[1] + SPECIFIED_OFFSET[1] * TUNNEL_D,
    ]
}

/// 13_6: one staircase panel, the specified circle as the solver sees it at `cells` per
/// diameter, with the intended circle dotted.
fn staircase_panel(stage: &mut Stage<'_>, kit: &Kit, centre: [f64; 2], cells: usize) -> fmn::Result<Vec<Mob>> {
    let d = 0.95;
    let stair = Staircase::new(cells);
    let h = d / cells as f64;
    let intended = [centre[0] - 0.25 * d, centre[1]];
    let specified = [
        intended[0] + SPECIFIED_OFFSET[0] * d,
        intended[1] + SPECIFIED_OFFSET[1] * d,
    ];
    let mut out = vec![frame(stage, centre, [2.5, 1.45], Palette::faint(), 1.2)?];
    for &(j, a, b) in &stair.runs {
        let x0 = specified[0] + a as f64 * h;
        let x1 = specified[0] + (b + 1) as f64 * h;
        let y = specified[1] + (j as f64 + 0.5) * h;
        out.push(panel(
            stage,
            [(x0 + x1) / 2.0, y],
            [x1 - x0, h],
            (Palette::faint(), 1.0),
            (Palette::ink(), 0.0),
        )?);
    }
    out.extend(dotted_circle(stage, intended, d / 2.0, Palette::ink())?);
    let label = format!("{cells} cells per diameter");
    out.push(shown(
        kit,
        stage,
        &label,
        16.0,
        Palette::dim(),
        v(centre[0], centre[1] - 0.95),
    )?);
    let offset = format!("offset {} D", two(stair.offset()));
    out.push(shown(
        kit,
        stage,
        &offset,
        16.0,
        Palette::gold(),
        v(centre[0], centre[1] + 0.92),
    )?);
    Ok(out)
}

/// 13_6: the outlet-distance outline, a shorter tunnel ending close behind the cylinder.
fn outlet_outline(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let x = specified_at()[0] + 2.0 * TUNNEL_D;
    let top = TUNNEL_CENTRE[1] + TUNNEL_SIZE[1] / 2.0;
    let bottom = TUNNEL_CENTRE[1] - TUNNEL_SIZE[1] / 2.0;
    let mut out = dotted(stage, [x, bottom], [x, top], 10, Palette::gold())?;
    out.push(shown(
        kit,
        stage,
        "an outlet here? its distance is a choice",
        18.0,
        Palette::gold(),
        v(x, top + 0.25),
    )?);
    Ok(out)
}

// ---------------------------------------------------------------------------------- the scene

/// A section of the chapter. What it leaves on screen (the question card, 13_13's patch) stays.
type Section = fn(&Chapter13, &mut Stage<'_>) -> fmn::Result<()>;

impl SceneConstruct for Chapter13 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = &self.kit;
        shown(
            k,
            stage,
            "Which mistake are we measuring?",
            46.0,
            Palette::ink(),
            v(0.0, 3.34),
        )?;
        let sections: [Section; 4] = [numerics, problem, balances, trust];
        for section in sections {
            section(self, stage)?;
        }
        k.finish(stage)
    }
}

/// 13_1 to 13_4 and 13_14. The question card it adds at 13_2 stays to the end.
fn numerics(c: &Chapter13, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let k = &c.kit;
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());

    // 13_1: the four parts of a step; two improved, and the errors that remain.
    k.say(stage, "13_1")?;
    let mut opening = Vec::new();
    for (name, x) in [("Carry", -4.5), ("Smooth", -1.5), ("Forces", 1.5), ("Correct", 4.5)] {
        opening.push(frame(stage, [x, 1.45], [2.4, 0.78], dim, 1.6)?);
        opening.push(shown(k, stage, name, 32.0, ink, v(x, 1.45))?);
    }
    k.until_sentence(stage, "13_1", 1)?;
    opening.push(frame(stage, [4.5, 1.45], [2.5, 0.88], Palette::teal(), 2.6)?);
    opening.push(shown(
        k,
        stage,
        "solved more tightly",
        20.0,
        Palette::teal(),
        v(4.5, 0.75),
    )?);
    opening.extend(mini_pulse(stage, [4.5, -0.55])?);
    opening.push(shown(k, stage, "the dye still smears", 20.0, dye(), v(4.5, -1.35))?);
    k.until_sentence(stage, "13_1", 2)?;
    opening.push(frame(stage, [-4.5, 1.45], [2.5, 0.88], Palette::blue(), 2.6)?);
    opening.push(shown(
        k,
        stage,
        "a sharper carry rule",
        20.0,
        Palette::blue(),
        v(-4.5, 0.75),
    )?);
    opening.extend(parcel_miss(stage, k, [-3.6, -0.75])?);
    k.until_sentence(stage, "13_1", 3)?;
    clear(stage, &mut opening);
    for (x, label) in [(-4.2, "cells"), (0.0, "step"), (4.2, "pressure solve")] {
        opening.extend(dial(stage, k, [x, 0.55], label)?);
    }
    opening.push(shown(
        k,
        stage,
        "three knobs, three kinds of error",
        24.0,
        gold,
        v(0.0, -1.7),
    )?);

    // 13_2: the ledger beside the question card, every row planned and not yet measured.
    k.say(stage, "13_2")?;
    clear(stage, &mut opening);
    question_card(stage, k)?;
    let entries = numerics_entries();
    let mut page = ledger_head(stage, k, PAGE_NUMERICS)?;
    k.until_sentence(stage, "13_2", 1)?;
    for (row, entry) in entries.iter().enumerate() {
        page.extend(ledger_row(stage, k, row, &entry.planned())?);
    }

    // 13_3: the unfinished solve, residual and observable apart; then the cylinder.
    k.say(stage, "13_3")?;
    let mut detail = vec![highlight(stage, 0, Some(0))?];
    k.until_sentence(stage, "13_3", 1)?;
    clear(stage, &mut page);
    clear(stage, &mut detail);
    detail.extend(residual_column(stage, k)?);
    k.until_sentence(stage, "13_3", 2)?;
    detail.extend(observable_column(stage, k)?);
    k.until_sentence(stage, "13_3", 3)?;
    detail.extend(verdict(stage, [RESIDUAL_X - 1.4, -2.72], true)?);
    detail.push(shown(
        k,
        stage,
        "useful evidence",
        18.0,
        Palette::green(),
        v(RESIDUAL_X, -3.1),
    )?);
    k.until_sentence(stage, "13_3", 4)?;
    clear(stage, &mut detail);
    detail.extend(cylinder_card(stage, k)?);

    // 13_4: the step alone, then cells and step together until r binds; record the limits.
    k.say(stage, "13_4")?;
    clear(stage, &mut detail);
    let mut written: Vec<Entry> = entries.iter().map(Entry::planned).collect();
    written[..2].clone_from_slice(&entries[..2]);
    page = ledger(stage, k, PAGE_NUMERICS, &written)?;
    page.push(highlight(stage, 2, None)?);
    k.until_sentence(stage, "13_4", 1)?;
    clear(stage, &mut page);
    let rungs = ladder();
    let mut tracks = slider_tracks(stage, k)?;
    let mut control = chip(
        stage,
        k,
        CHIP_AT,
        &format!("cells {0} × {0}", TIME[0].cells),
        Control::Fixed,
    )?;
    let mut knob = knobs(stage, Knobs::of_run(&TIME[0]), true)?;
    let mut table = measured_head(stage, k)?;
    for n in 0..TIME.len() {
        if n > 0 {
            slide(
                stage,
                &mut knob,
                Knobs::of_run(&TIME[n - 1]),
                Knobs::of_run(&TIME[n]),
                true,
            )?;
        }
        table.extend(measured_row(stage, k, n)?);
        pause(stage, pacing::RUNG_HOLD)?;
    }
    let ratios = format!("÷ {}, then ÷ {}", digits(TIME_RATIOS[0], 1), digits(TIME_RATIOS[1], 1));
    table.push(shown_left(
        k,
        stage,
        &ratios,
        19.0,
        Palette::green(),
        v(MEASURED_X[3], -0.3),
    )?);
    k.until_sentence(stage, "13_4", 2)?;
    clear(stage, &mut table);
    clear(stage, &mut control);
    control = chip(stage, k, CHIP_AT, "cells and step together", Control::Changed)?;
    table = ladder_head(stage, k)?;
    let kept = rungs.iter().take_while(|r| !r.locked).count();
    let mut previous = Knobs::of_run(&TIME[TIME.len() - 1]);
    for (n, rung) in rungs.iter().enumerate().take(kept) {
        slide(stage, &mut knob, previous, Knobs::of(rung), false)?;
        previous = Knobs::of(rung);
        table.extend(ladder_row(stage, k, n, rung)?);
        pause(stage, pacing::RUNG_HOLD)?;
    }
    k.until_sentence(stage, "13_4", 3)?;
    let span = [ladder_y(0) + 0.2, ladder_y(kept.saturating_sub(1)) - 0.2];
    let middle = (span[0] + span[1]) / 2.0;
    table.push(frame(
        stage,
        [LADDER_X[3] + 0.22, middle],
        [0.85, span[0] - span[1]],
        Palette::green(),
        1.8,
    )?);
    let same = format!("C stays {}", two(rungs[0].courant));
    table.push(shown_left(k, stage, &same, 17.0, Palette::green(), v(5.65, middle))?);
    k.until_sentence(stage, "13_4", 4)?;
    let mut ghost = Vec::new();
    for (n, rung) in rungs.iter().enumerate().skip(kept) {
        let fixed_steps = 20 * rung.cells / 32;
        let would_be = diffusion_weight(rung.cells, TG_END / fixed_steps as f64);
        clear(stage, &mut ghost);
        ghost.push(ring(stage, [r_x(would_be), R_Y], 0.11, Palette::red(), 2.0)?);
        ghost.extend(verdict(stage, [r_x(would_be), R_Y + 0.32], false)?);
        slide(stage, &mut knob, previous, Knobs::of(rung), false)?;
        previous = Knobs::of(rung);
        table.extend(ladder_row(stage, k, n, rung)?);
        let note = format!(
            "{}: {fixed_steps} steps would make r {}, so {}",
            rung.cells,
            digits(would_be, 3),
            rung.steps
        );
        let y = -1.2 - 0.32 * (n - kept) as f64;
        table.push(shown_left(k, stage, &note, 17.0, gold, v(TABLE_X, y))?);
        pause(stage, pacing::RUNG_HOLD)?;
    }
    k.until_sentence(stage, "13_4", 5)?;
    let errors: Vec<String> = TIME.iter().map(|r| digits(r.error * 1e4, 3)).collect();
    let measured = format!(
        "Measured on {0} × {0} cells: error against the exact flow {1} × {2} as the step halves",
        TIME[0].cells,
        errors.join(", "),
        power(-4)
    );
    table.push(shown(k, stage, &measured, 18.0, Palette::blue(), v(0.0, -2.5))?);
    let total = "one measured total, not split into grid and step parts";
    table.push(shown(k, stage, total, 18.0, dim, v(0.0, -2.88))?);
    k.until_sentence(stage, "13_4", 6)?;
    for group in [&mut tracks, &mut control, &mut knob, &mut table, &mut ghost] {
        clear(stage, group);
    }
    written[2..4].clone_from_slice(&entries[2..4]);
    page = ledger(stage, k, PAGE_NUMERICS, &written)?;
    page.push(highlight(stage, 2, Some(3))?);
    page.push(highlight(stage, 3, Some(3))?);

    // 13_14: the dye pulse with only the step shrunk.
    k.say(stage, "13_14")?;
    clear(stage, &mut page);
    let mut chart = step_chart(stage, k)?;
    k.until_sentence(stage, "13_14", 1)?;
    chart.extend(chip(stage, k, [4.9, 0.9], "cells", Control::Fixed)?);
    chart.extend(chip(stage, k, [4.9, 0.3], "step only", Control::Changed)?);
    k.until_sentence(stage, "13_14", 2)?;
    for courant in STEP_ONLY_COURANT {
        let spread = step_only_spreading(courant);
        if (spread - step_only_formula(courant)).abs() > 1e-9 {
            return Err(fail("13_14: the step-only spreading left 24(1 − C)"));
        }
        chart.push(disc(stage, [chart_x(courant), chart_y(spread)], 0.09, gold)?);
        chart.push(shown(
            k,
            stage,
            &whole(spread),
            22.0,
            ink,
            v(chart_x(courant), chart_y(spread) + 0.3),
        )?);
        pause(stage, pacing::POINT)?;
    }
    let settle = chart_y(STEP_ONLY_LIMIT);
    chart.push(line(
        stage,
        v(chart_x(0.0), settle),
        v(chart_x(0.6), settle),
        Palette::faint(),
        1.6,
    )?);
    let settles = "where it settles: all method-made";
    chart.push(shown_left(
        k,
        stage,
        settles,
        19.0,
        dim,
        v(chart_x(0.36), settle - 0.25),
    )?);
    k.until_sentence(stage, "13_14", 3)?;
    let zero = chart_y(0.0) + 0.04;
    chart.extend(dotted(stage, [chart_x(0.0), zero], [chart_x(0.6), zero], 24, ink)?);
    chart.push(shown_left(
        k,
        stage,
        "exact pulse: 0",
        19.0,
        ink,
        v(chart_x(0.36), chart_y(0.0) + 0.3),
    )?);
    chart.extend(precisely(k, stage, &precisely_13_14())?);
    k.hold(stage)?;
    clear(stage, &mut chart);
    page = ledger(stage, k, PAGE_NUMERICS, &entries)?;
    pause(stage, pacing::LEDGER_BEAT)?;
    clear(stage, &mut page);
    Ok(())
}

/// 13_5 to 13_8: the prediction, then the problem and the model.
fn problem(c: &Chapter13, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let k = &c.kit;
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());
    let thumb = TUNNEL_CENTRE;

    // 13_5: the frozen tunnel, then the prediction. Nothing of 13_6's answer is drawn here.
    k.say(stage, "13_5")?;
    let mut scene = tunnel(stage, thumb, 1.0)?;
    scene.push(shown(
        k,
        stage,
        "illustration",
        17.0,
        gold,
        v(thumb[0] + 3.4, thumb[1] + 1.2),
    )?);
    let mut intended = vec![cylinder(stage, thumb, 1.0, CYLINDER_AT)?];
    let suppose = "suppose refining barely changes this picture";
    let mut caption = vec![shown(k, stage, suppose, 19.0, dim, v(thumb[0], -1.65))?];
    k.until_sentence(stage, "13_5", 1)?;
    predict(
        stage,
        k,
        ["The picture hardly changes:", "does it describe the real fluid?"],
        "Yes               No",
    )?;

    // 13_6: the answer, then one change at a time: the cylinder's place, then the outlet.
    k.say(stage, "13_6")?;
    let mut answer = vec![shown(k, stage, "No.", 44.0, Palette::hero(), v(5.4, 1.2))?];
    k.until_sentence(stage, "13_6", 1)?;
    answer.push(shown(k, stage, "Not by itself.", 26.0, ink, v(5.4, 0.55))?);
    k.until_sentence(stage, "13_6", 2)?;
    clear(stage, &mut intended);
    let spec = specified_at();
    let mut marks = dotted_circle(stage, CYLINDER_AT, TUNNEL_D / 2.0, ink)?;
    marks.push(shown(
        k,
        stage,
        "intended",
        17.0,
        ink,
        v(CYLINDER_AT[0] - 0.2, CYLINDER_AT[1] + 0.62),
    )?);
    marks.push(shown(
        k,
        stage,
        "specified",
        17.0,
        gold,
        v(spec[0] + 0.3, CYLINDER_AT[1] - 0.62),
    )?);
    let mut placed = vec![cylinder(stage, thumb, 1.0, spec)?];
    answer.push(shown(k, stage, "the wrong problem,", 21.0, gold, v(5.4, -0.2))?);
    answer.push(shown(k, stage, "solved very well", 21.0, gold, v(5.4, -0.55))?);
    k.until_sentence(stage, "13_6", 3)?;
    clear(stage, &mut caption);
    let mut stairs = Vec::new();
    for (n, cells) in STAIRCASE_CELLS.into_iter().enumerate() {
        stairs.extend(staircase_panel(stage, k, [-4.4 + 3.0 * n as f64, -2.5], cells)?);
    }
    let smoother = format!(
        "finer cells smooth the steps; the center stays {} D off (illustration)",
        two(Staircase::new(STAIRCASE_CELLS[2]).offset())
    );
    stairs.push(shown(k, stage, &smoother, 17.0, gold, v(-1.4, -3.85))?);
    if let Some(narrator) = &k.narrator {
        until_share(stage, narrator, "13_6", 3, OUTLET_SHARE)?;
    }
    let mut outlet = outlet_outline(stage, k)?;

    // 13_7: the same cylinder extruded into the depth the model leaves out, then flat again.
    k.say(stage, "13_7")?;
    for group in [&mut stairs, &mut outlet, &mut answer, &mut marks] {
        clear(stage, group);
    }
    let depth = [0.9, 0.45];
    let mut extruded = Vec::new();
    for corner in [[-1.0, 1.0], [1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]] {
        let p = [
            thumb[0] + corner[0] * TUNNEL_SIZE[0] / 2.0,
            thumb[1] + corner[1] * TUNNEL_SIZE[1] / 2.0,
        ];
        extruded.push(line(
            stage,
            v(p[0], p[1]),
            v(p[0] + depth[0], p[1] + depth[1]),
            dim,
            1.4,
        )?);
    }
    let back = [thumb[0] + depth[0], thumb[1] + depth[1]];
    extruded.push(frame(stage, back, TUNNEL_SIZE, Palette::faint(), 1.4)?);
    let r = TUNNEL_D / 2.0;
    let length = depth[0].hypot(depth[1]);
    let normal = [-depth[1] / length * r, depth[0] / length * r];
    extruded.push(ring(stage, [spec[0] + depth[0], spec[1] + depth[1]], r, dim, 1.6)?);
    for side in [1.0, -1.0] {
        let a = [spec[0] + side * normal[0], spec[1] + side * normal[1]];
        extruded.push(line(
            stage,
            v(a[0], a[1]),
            v(a[0] + depth[0], a[1] + depth[1]),
            dim,
            1.6,
        )?);
    }
    extruded.push(shown(k, stage, "our flat world: a choice", 19.0, ink, v(5.5, 1.5))?);
    k.until_sentence(stage, "13_7", 1)?;
    let (from, to) = ([3.4, -1.95], [3.4 + 1.3, -1.95 + 0.65]);
    extruded.push(arrow(stage, from, to, Palette::purple(), 1.6)?);
    extruded.push(shown(
        k,
        stage,
        "third direction: left out",
        19.0,
        Palette::purple(),
        v(4.9, -0.95),
    )?);
    extruded.push(shown(
        k,
        stage,
        "more 2D cells don't restore it",
        19.0,
        dim,
        v(thumb[0], -1.65),
    )?);
    k.until_sentence(stage, "13_7", 2)?;
    clear(stage, &mut extruded);
    let mut model = vec![
        panel(stage, [5.4, 0.9], [2.4, 0.62], (card(), 1.0), (Palette::purple(), 2.0))?,
        shown(k, stage, "2D model", 26.0, Palette::purple(), v(5.4, 0.9))?,
        shown(k, stage, "a question about the model,", 20.0, ink, v(5.4, 0.15))?,
        shown(k, stage, "not just the mesh", 20.0, ink, v(5.4, -0.2))?,
    ];

    // 13_8: rounding, its own row apart from geometry; the remedy has to match the mistake.
    k.say(stage, "13_8")?;
    for group in [&mut model, &mut placed, &mut scene] {
        clear(stage, group);
    }
    let mut page = ledger(stage, k, PAGE_PROBLEM, &problem_entries(false))?;
    k.until_sentence(stage, "13_8", 1)?;
    clear(stage, &mut page);
    page = ledger(stage, k, PAGE_PROBLEM, &problem_entries(true))?;
    page.push(highlight(stage, 3, None)?);
    k.until_sentence(stage, "13_8", 2)?;
    page.push(highlight(stage, 0, Some(2))?);
    page.push(shown(
        k,
        stage,
        "more digits won't move the cylinder",
        20.0,
        Palette::red(),
        v(0.0, -1.75),
    )?);
    k.until_sentence(stage, "13_8", 3)?;
    let matched = "a misplaced cylinder needs a corrected input; rounding needs more digits";
    page.push(shown(k, stage, matched, 20.0, gold, v(0.0, -2.15))?);
    k.hold(stage)?;
    clear(stage, &mut page);
    Ok(())
}

/// 13_9 and 13_10: balances, with velocity and dye kept apart, then energy.
fn balances(c: &Chapter13, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let k = &c.kit;
    let (ink, dim, blue, red) = (Palette::ink(), Palette::dim(), Palette::blue(), Palette::red());

    // 13_9: one shared face, the pressure correction's leftover, then the dye budget.
    k.say(stage, "13_9")?;
    let mut left = vec![
        line(stage, v(0.0, 2.1), v(0.0, -2.9), Palette::faint(), 1.4)?,
        shown(k, stage, "velocity", 24.0, blue, v(-3.6, 1.95))?,
        shown(k, stage, "dye", 24.0, dye(), v(3.6, 1.95))?,
        frame(stage, [-4.4, 0.95], [1.6, 1.1], ink, 1.6)?,
        frame(stage, [-2.8, 0.95], [1.6, 1.1], ink, 1.6)?,
        arrow(stage, [-4.0, 0.95], [-3.2, 0.95], blue, 1.6)?,
        shown(
            k,
            stage,
            "flux: the rate fluid crosses a face",
            19.0,
            blue,
            v(-3.6, 0.1),
        )?,
    ];
    k.until_sentence(stage, "13_9", 1)?;
    left.push(shown(k, stage, "− F", 22.0, blue, v(-4.6, 1.2))?);
    left.push(shown(k, stage, "+ F", 22.0, blue, v(-2.6, 1.2))?);
    left.push(shown(
        k,
        stage,
        "one value per face, used by both cells",
        18.0,
        ink,
        v(-3.6, -0.3),
    )?);
    left.extend(verdict(stage, [-1.3, -0.3], true)?);
    left.push(shown(
        k,
        stage,
        "Taylor–Green's staggered grid stores it once",
        16.0,
        dim,
        v(-3.6, -0.62),
    )?);
    k.until_sentence(stage, "13_9", 2)?;
    left.push(shown(
        k,
        stage,
        "volume imbalance after projection (Δt × divergence)",
        19.0,
        red,
        v(-3.6, -1.08),
    )?);
    let same_budget: Vec<Run> = SPACE.iter().chain(&TIME).copied().collect();
    if same_budget
        .iter()
        .chain(&PRESSURE)
        .any(|r| r.max_volume_defect > r.budget)
    {
        return Err(fail("13_9: a Taylor–Green run left more than its budget"));
    }
    let largest = same_budget.iter().map(|r| r.max_volume_defect).fold(0.0, f64::max);
    let budget = same_budget.iter().map(|r| r.budget).fold(0.0, f64::max);
    let rows = [
        format!(
            "Taylor–Green, {} runs: {} ≤ {}",
            same_budget.len(),
            sci(largest, 3),
            budget_power(budget)
        ),
        format!(
            "cylinder: {} ≤ {}",
            sci(CYLINDER_VOLUME_DEFECT, 3),
            budget_power(CYLINDER_BUDGET)
        ),
    ];
    for (n, text) in rows.iter().enumerate() {
        let y = -1.5 - 0.42 * n as f64;
        left.push(shown_left(k, stage, text, 18.0, red, v(-6.6, y))?);
        left.extend(verdict(stage, [-0.5, y], true)?);
    }
    k.until_sentence(stage, "13_9", 3)?;
    let mut right = vec![shown(
        k,
        stage,
        "chapter 11's dye in the row: start → end",
        19.0,
        dye(),
        v(3.6, 1.35),
    )?];
    for (n, (name, rule)) in DYE_RULES.iter().enumerate() {
        let budget = dye_budget(*rule);
        if budget.left_the_row {
            return Err(fail("13_9: dye reached an end of chapter 11's row"));
        }
        let y = 0.8 - 0.48 * n as f64;
        let kept = (budget.end - budget.start).abs() < 1e-9;
        right.push(shown_left(k, stage, name, 19.0, ink, v(0.6, y))?);
        let change = format!("{} → {}", two(budget.start), two(budget.end));
        right.push(shown_left(
            k,
            stage,
            &change,
            19.0,
            if kept { ink } else { red },
            v(3.3, y),
        )?);
        right.extend(verdict(stage, [6.1, y], kept)?);
    }
    let ends = "end cells empty after every step: nothing entered or left";
    right.push(shown(k, stage, ends, 17.0, dim, v(3.6, -1.35))?);
    let lost = dye_budget(Transport::ClampedCurve);
    let missing = format!("clamped cubic: {} of dye unaccounted for", two(lost.start - lost.end));
    right.push(shown(k, stage, &missing, 18.0, red, v(3.6, -1.75))?);

    // 13_10: a closed box, its energy against viscosity's loss, then the open tunnel.
    k.say(stage, "13_10")?;
    clear(stage, &mut left);
    clear(stage, &mut right);
    let mut boxed = vec![shown(
        k,
        stage,
        "Energy keeps its own account",
        24.0,
        ink,
        v(0.0, 1.95),
    )?];
    k.until_sentence(stage, "13_10", 1)?;
    let centre = [-4.4, 0.0];
    boxed.push(frame(stage, centre, [2.6, 2.6], ink, 2.0)?);
    for (dx, dy, turn) in [
        (-0.65, 0.65, 1.0),
        (0.65, 0.65, -1.0),
        (-0.65, -0.65, -1.0),
        (0.65, -0.65, 1.0),
    ] {
        let at = [centre[0] + dx, centre[1] + dy];
        boxed.push(ring(stage, at, 0.42, blue, 1.6)?);
        let (from, to) = ([at[0] - 0.2 * turn, at[1] + 0.42], [at[0] + 0.2 * turn, at[1] + 0.42]);
        boxed.push(arrow(stage, from, to, blue, 1.2)?);
    }
    boxed.push(shown(
        k,
        stage,
        "Taylor–Green: a periodic box, no forces",
        18.0,
        dim,
        v(centre[0], -1.6),
    )?);
    boxed.push(shown(
        k,
        stage,
        "energy cannot rise",
        20.0,
        Palette::green(),
        v(centre[0], 1.55),
    )?);
    let law = r"\frac{dE}{dt} = -2\rho\nu \int D:D \, dV \le 0";
    boxed.push(shown_tex(k, stage, law, 26.0, ink, v(centre[0], -2.35))?);
    let mut fine = precisely(k, stage, &PRECISELY_13_10)?;
    k.until_sentence(stage, "13_10", 2)?;
    clear(stage, &mut fine);
    let mut bars = Vec::new();
    let run = SPACE[1];
    let account = energy_account(&run);
    let (base, scale) = (-1.55, 11.0);
    let computed = format!("computed, {0} × {0}", run.cells);
    let columns = [
        (1.3, account.start, "start", ink, false),
        (3.2, run.exact_energy, "viscosity predicts", ink, true),
        (5.1, run.energy, computed.as_str(), blue, false),
    ];
    for (x, energy, label, color, exact) in columns {
        let top = base + scale * energy;
        if exact {
            bars.extend(dotted(stage, [x - 0.5, top], [x + 0.5, top], 6, ink)?);
            bars.extend(dotted(stage, [x - 0.5, base], [x - 0.5, top], 10, ink)?);
            bars.extend(dotted(stage, [x + 0.5, base], [x + 0.5, top], 10, ink)?);
        } else {
            bars.push(panel(
                stage,
                [x, (base + top) / 2.0],
                [1.0, top - base],
                (color, 0.35),
                (color, 1.6),
            )?);
        }
        bars.push(shown(k, stage, &digits(energy, 4), 18.0, color, v(x, top + 0.22))?);
        bars.push(shown(k, stage, label, 16.0, dim, v(x, base - 0.25))?);
    }
    let compare = format!(
        "lost {}, viscosity predicts {}",
        digits(account.lost, 4),
        digits(account.predicted_loss, 4)
    );
    bars.push(shown(k, stage, &compare, 19.0, ink, v(3.2, -2.2))?);
    let gaps: Vec<String> = SPACE.iter().map(|r| sci(energy_account(r).gap, 1)).collect();
    let meshes: Vec<String> = SPACE.iter().map(|r| r.cells.to_string()).collect();
    let gap = format!(
        "gap {} at {} cells: ÷ 4 per halving",
        gaps.join(", "),
        meshes.join(", ")
    );
    bars.push(shown(k, stage, &gap, 16.0, dim, v(3.2, -2.58))?);
    let history = "start and end only: no energy history was recorded";
    bars.push(shown(k, stage, history, 16.0, dim, v(3.2, -2.9))?);
    fine = precisely(k, stage, &PRECISELY_13_10_GAP)?;
    k.until_sentence(stage, "13_10", 3)?;
    clear(stage, &mut boxed);
    clear(stage, &mut fine);
    let open_at = [-3.9, 0.3];
    let mut open = tunnel(stage, open_at, 0.42)?;
    open.push(cylinder(stage, open_at, 0.42, CYLINDER_AT)?);
    let edge = open_at[0] + 0.42 * TUNNEL_SIZE[0] / 2.0;
    for n in 0..3 {
        let y = open_at[1] - 0.4 + 0.4 * f64::from(n);
        open.push(arrow(stage, [edge - 0.05, y], [edge + 0.5, y], Palette::gold(), 1.3)?);
    }
    open.push(shown(
        k,
        stage,
        "energy in",
        18.0,
        blue,
        v(open_at[0] - 1.9, open_at[1] + 0.95),
    )?);
    open.push(shown(
        k,
        stage,
        "carried out",
        18.0,
        Palette::gold(),
        v(edge + 0.3, open_at[1] + 0.95),
    )?);
    open.push(shown(
        k,
        stage,
        "the tunnel: an open boundary",
        19.0,
        ink,
        v(open_at[0], 1.95),
    )?);
    open.push(frame(stage, [open_at[0], -1.35], [5.6, 0.9], dim, 1.4)?);
    open.push(shown(
        k,
        stage,
        "in − out − viscous loss = change inside",
        18.0,
        ink,
        v(open_at[0], -1.2),
    )?);
    open.push(shown(k, stage, "not measured", 18.0, dim, v(open_at[0], -1.55))?);
    fine = precisely(k, stage, &PRECISELY_13_10_OPEN)?;
    k.hold(stage)?;
    for group in [&mut open, &mut bars, &mut fine] {
        clear(stage, group);
    }
    Ok(())
}

/// 13_12's reasons, each with what it was measured against: the Taylor–Green error against the
/// exact flow per halving of the cells, and the step's own difference from the 640-step run per
/// halving of the step (the error against the exact flow hardly moves as the step halves, 13_4).
fn reasons_13_12() -> fmn::Result<[String; 3]> {
    let (space, time) = (whole(SPACE_RATIOS[0]), whole(TIME_RATIOS[0]));
    if whole(SPACE_RATIOS[1]) != space || whole(TIME_RATIOS[1]) != time {
        return Err(fail("13_12: a halving ratio rounds differently from its pair"));
    }
    let lost = dye_budget(Transport::ClampedCurve);
    Ok([
        format!("Reasons we have: Taylor–Green error against the exact flow ÷ {space} per halving of the cells;"),
        format!("the step's own difference from a {TIME_REFERENCE_STEPS}-step run ÷ {time} per halving of the step;"),
        format!(
            "budgets met in every run; a dye budget that caught a {} loss.",
            two(lost.start - lost.end)
        ),
    ])
}

/// 13_11 to 13_13: a known answer, a defined wake quantity, the declared tolerance, and the
/// next chapter's question.
fn trust(c: &Chapter13, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let k = &c.kit;
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());

    // 13_11: measure the error directly where the answer is known; define the wake's quantity.
    k.say(stage, "13_11")?;
    let mut known = vec![
        frame(stage, [-3.6, 0.35], [6.6, 3.1], dim, 1.4)?,
        shown(k, stage, "A known answer: Taylor–Green", 24.0, ink, v(-3.6, 1.6))?,
    ];
    known.extend(chip(
        stage,
        k,
        [-3.6, 1.1],
        &format!("{} steps of {} s", SPACE[0].steps, SPACE[0].dt),
        Control::Fixed,
    )?);
    for (n, run) in SPACE.iter().enumerate() {
        let text = format!("{0} × {0} cells:  error {1}", run.cells, sci(run.error, 2));
        known.push(shown_left(
            k,
            stage,
            &text,
            19.0,
            Palette::blue(),
            v(-6.4, 0.62 - 0.38 * n as f64),
        )?);
    }
    let orders: Vec<String> = SPACE_RATIOS.iter().map(|&r| digits(observed_order(r), 1)).collect();
    let ratio = format!(
        "÷ {}, then ÷ {}: order {}",
        digits(SPACE_RATIOS[0], 2),
        digits(SPACE_RATIOS[1], 2),
        orders.join(" and ")
    );
    known.push(shown_left(k, stage, &ratio, 19.0, Palette::green(), v(-6.4, -0.5))?);
    let largest_c = SPACE.iter().map(|r| r.max_courant).fold(0.0, f64::max);
    let largest_r = SPACE.iter().map(Run::diffusion_weight).fold(0.0, f64::max);
    let inside = format!(
        "the step is inside every mesh's limits: C ≤ {}, r ≤ {}",
        two(largest_c),
        digits(largest_r, 3)
    );
    known.push(shown_left(k, stage, &inside, 16.0, dim, v(-6.4, -0.85))?);
    k.until_sentence(stage, "13_11", 1)?;
    let mut wake = vec![
        frame(stage, [3.6, 0.35], [6.6, 3.1], dim, 1.4)?,
        shown(k, stage, "The wake: a defined quantity", 24.0, ink, v(3.6, 1.6))?,
    ];
    let samples = 40;
    for n in 0..samples {
        let (s0, s1) = (n as f64 / samples as f64, (n as f64 + 0.6) / samples as f64);
        let point = |s: f64| [1.0 + 5.2 * s, 0.55 + 0.42 * (TAU * 1.5 * s).sin()];
        let (a, b) = (point(s0), point(s1));
        wake.push(line(stage, v(a[0], a[1]), v(b[0], b[1]), Palette::faint(), 1.8)?);
    }
    let period = 5.2 / 1.5;
    wake.extend(bracket(
        stage,
        [1.0 + period * 0.25, 1.0 + period * 1.25],
        1.12,
        -1.0,
        gold,
    )?);
    let swing = "one swing: side to side and back";
    wake.push(shown(k, stage, swing, 17.0, gold, v(1.0 + period * 0.75, 1.3))?);
    wake.push(shown(k, stage, "illustration", 16.0, gold, v(5.6, -0.05))?);
    wake.push(shown(k, stage, "swing period: not measured", 20.0, dim, v(3.6, -0.5))?);
    k.until_sentence(stage, "13_11", 2)?;
    let same_problem = "Same problem? Each must match before agreement counts";
    let mut same = vec![shown(k, stage, same_problem, 20.0, gold, v(0.0, -1.68))?];
    let items = ["geometry", "inlet", "Reynolds number convention", "measurement window"];
    for (n, item) in items.iter().enumerate() {
        let x = -6.0 + 3.3 * n as f64;
        same.push(frame(stage, [x, -2.2], [0.26, 0.26], dim, 1.4)?);
        same.push(shown_left(k, stage, item, 18.0, ink, v(x + 0.25, -2.2))?);
    }

    // 13_12: chapter 12's declared tolerance returns as a ledger row, beside what is still open.
    k.say(stage, "13_12")?;
    for group in [&mut known, &mut wake, &mut same] {
        clear(stage, group);
    }
    let at = [-4.9, 0.6];
    let mut summary = dial(stage, k, at, "the tunnel's answer")?;
    summary.extend(tolerance_band(stage, k, at)?);
    summary.push(shown(
        k,
        stage,
        "declared before comparing",
        19.0,
        Palette::teal(),
        v(at[0], -0.85),
    )?);
    summary.push(shown(
        k,
        stage,
        "for the tunnel: not yet declared",
        17.0,
        dim,
        v(at[0], -1.25),
    )?);
    k.until_sentence(stage, "13_12", 1)?;
    clear(stage, &mut summary);
    summary = ledger(stage, k, PAGE_UNCERTAIN, &uncertain_entries())?;
    k.until_sentence(stage, "13_12", 2)?;
    let reasons = reasons_13_12()?;
    for (n, text) in reasons.iter().enumerate() {
        summary.push(shown(
            k,
            stage,
            text,
            18.0,
            Palette::green(),
            v(1.5, -2.3 - 0.34 * n as f64),
        )?);
    }

    // 13_13: put detail where it is needed, with every shared face still balanced.
    k.say(stage, "13_13")?;
    clear(stage, &mut summary);
    let tunnel_at = [-2.6, 0.2];
    let scale = 0.62;
    let mut grid = tunnel(stage, tunnel_at, scale)?;
    let (w, h) = (TUNNEL_SIZE[0] * scale, TUNNEL_SIZE[1] * scale);
    let (columns, rows) = (12, 4);
    let (left, bottom) = (tunnel_at[0] - w / 2.0, tunnel_at[1] - h / 2.0);
    for n in 1..columns {
        let x = left + w * n as f64 / columns as f64;
        grid.push(line(stage, v(x, bottom), v(x, bottom + h), Palette::faint(), 1.0)?);
    }
    for n in 1..rows {
        let y = bottom + h * n as f64 / rows as f64;
        grid.push(line(stage, v(left, y), v(left + w, y), Palette::faint(), 1.0)?);
    }
    let solid = cylinder(stage, tunnel_at, scale, CYLINDER_AT)?;
    stage.set_z_index(solid, 1, true);
    grid.push(solid);
    k.until_sentence(stage, "13_13", 1)?;
    let cell_size = [w / columns as f64, h / rows as f64];
    let patch = tunnel_point(tunnel_at, scale, CYLINDER_AT);
    let column = ((patch[0] - left) / cell_size[0]).floor();
    let corner = [left + column * cell_size[0], tunnel_at[1] - cell_size[1]];
    let teal = Palette::teal();
    for n in [1.0, 3.0] {
        let x = corner[0] + cell_size[0] * 0.5 * n;
        grid.push(line(
            stage,
            v(x, corner[1]),
            v(x, corner[1] + 2.0 * cell_size[1]),
            teal,
            1.0,
        )?);
        let y = corner[1] + cell_size[1] * 0.5 * n;
        grid.push(line(
            stage,
            v(corner[0], y),
            v(corner[0] + 2.0 * cell_size[0], y),
            teal,
            1.0,
        )?);
    }
    let patch_centre = [corner[0] + cell_size[0], corner[1] + cell_size[1]];
    grid.push(frame(
        stage,
        patch_centre,
        [2.0 * cell_size[0], 2.0 * cell_size[1]],
        teal,
        2.0,
    )?);
    let zoom = [4.9, 0.25];
    grid.push(frame(stage, [zoom[0] - 0.8, zoom[1]], [1.6, 1.6], ink, 1.8)?);
    grid.push(frame(stage, [zoom[0] + 0.4, zoom[1] + 0.4], [0.8, 0.8], teal, 1.6)?);
    grid.push(frame(stage, [zoom[0] + 0.4, zoom[1] - 0.4], [0.8, 0.8], teal, 1.6)?);
    grid.push(line(
        stage,
        v(zoom[0], zoom[1] - 0.8),
        v(zoom[0], zoom[1] + 0.8),
        gold,
        3.0,
    )?);
    grid.push(arrow(
        stage,
        [zoom[0] - 0.55, zoom[1]],
        [zoom[0] - 0.05, zoom[1]],
        Palette::blue(),
        1.4,
    )?);
    grid.push(shown_tex(
        k,
        stage,
        r"F = F_1 + F_2",
        26.0,
        ink,
        v(zoom[0], zoom[1] - 1.2),
    )?);
    grid.push(shown(
        k,
        stage,
        "where big cells meet small ones",
        17.0,
        gold,
        v(zoom[0], zoom[1] + 1.15),
    )?);
    let prospective = "a prospective patch: no saving claimed";
    grid.push(shown(k, stage, prospective, 17.0, dim, v(tunnel_at[0], -1.55))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 13_14's Precisely line in the full script (navier-full-script03.md).
    const PRECISELY_13_14: [&str; 2] = [
        "Halving only the step on fixed cells: 12, 18, then 21 square cells at ½, ¼ and ⅛ cell per step.",
        "That is 24(1 − C), which heads to 24 as the step shrinks, while the exact pulse spreads 0.",
    ];

    #[test]
    fn precisely_13_14_is_the_scripts_line() {
        assert_eq!(precisely_13_14(), PRECISELY_13_14.map(str::to_owned));
    }

    #[test]
    fn ledger_cells_fit_their_columns() {
        let mut entries: Vec<Entry> = numerics_entries().to_vec();
        entries.extend(problem_entries(true));
        entries.extend(uncertain_entries());
        for entry in &entries {
            for cell in &entry.cells {
                assert!(cell.lines.len() <= 2);
                for words in &cell.lines {
                    assert!(words.chars().count() <= CELL_CHARS, "{words}");
                }
            }
        }
    }

    #[test]
    fn numbers_read_as_measured() {
        assert_eq!(sci(2.1171317851735848e-07, 1), "2.1 × 10⁻⁷");
        assert_eq!(sci(9.9992380339964e-10, 3), "9.999 × 10⁻¹⁰");
        assert_eq!(sci(9.997609280197072e-12, 3), "9.998 × 10⁻¹²");
        assert_eq!(sci(9.99e-5, 1), "1.0 × 10⁻⁴");
        assert_eq!(power(-10), "10⁻¹⁰");
        assert_eq!(fraction(0.125), "⅛");
        assert_eq!(step_text(TG_END / 266.0), "0.00301");
        let entries = numerics_entries();
        assert_eq!(entries[0].cells[2].lines, ["imbalance ÷ 99", "error 2.567 × 10⁻⁴ both"]);
        assert_eq!(
            entries[0].cells[3].lines,
            ["32 × 32, Δt 0.01 s, fixed", "C 0.10, r 0.013"]
        );
        assert_eq!(
            entries[2].cells[0].lines,
            ["Δt 0.04 → 0.02 → 0.01 s", "cells 32 × 32 fixed"]
        );
        assert_eq!(
            entries[2].cells[2].lines,
            ["÷ 4.0, then ÷ 4.0", "2.1 × 10⁻⁷ → 1.3 × 10⁻⁸"]
        );
        assert_eq!(
            entries[2].cells[3].lines,
            ["C ≤ 0.41, r ≤ 0.052", "the step's own error"]
        );
        assert_eq!(
            entries[3].cells[3].lines,
            ["r < ¼ binds at 256 cells", "C falls to 0.25, 0.12"]
        );
        assert_eq!(entries[4].cells[2].lines, ["12, 18, 21 → 24", "exact pulse: 0"]);
        assert_eq!(
            problem_entries(true)[0].cells[2].lines,
            ["offset stays 0.50 D", "at 8, 16, 32 cells per D"]
        );
    }

    #[test]
    fn the_cylinder_rows_name_the_case_and_the_unsettled_lift() {
        let entries = numerics_entries();
        assert_eq!(
            entries[1].cells[0].lines,
            ["none: Re 20 cylinder, 2D-1", "832 steps, last 0.5 s"]
        );
        assert_eq!(
            entries[1].cells[2].lines,
            ["budget met; lift not settled;", "all three miss their ranges"]
        );
        let uncertain = uncertain_entries();
        assert_eq!(uncertain[1].cells[0].lines, ["none: Re 20 cylinder, 2D-1"]);
        assert_eq!(
            uncertain[1].cells[2].lines,
            ["lift not settled;", "all three miss their ranges"]
        );
    }

    #[test]
    fn the_dye_row_is_an_illustration() {
        let dye_row = &numerics_entries()[4];
        assert_eq!(dye_row.cells[2].tone, Tone::Illustration);
        assert_eq!(dye_row.cells[3].lines, ["illustration: uniform wind", "C = ½, ¼, ⅛"]);
    }

    #[test]
    fn the_cylinder_card_reads_its_records() {
        assert_eq!(
            cylinder_title(),
            "Steady cylinder, Schäfer–Turek 2D-1, Re = 20: 832 steps, the last 0.5 s of 2 s"
        );
        assert_eq!(
            cylinder_check().unwrap(),
            "one check: scaled residual = Δt × divergence, 9.999 × 10⁻¹⁰ ≤ 10⁻⁹"
        );
        assert_eq!(
            range_rule(),
            "Frozen rule: pass only if this run's whole range lies inside the published one; \
             drag overlaps but reaches 5.593 > 5.590"
        );
        assert_eq!(lift_note().unwrap(), "not settled: drift 0.0045 > 0.003");
        let lines = [
            (cylinder_title(), 24.0),
            (cylinder_check().unwrap(), 19.0),
            (range_rule(), 16.0),
            (CYLINDER_VERDICT.to_owned(), 20.0),
        ];
        for (words, size) in &lines {
            assert!(text_width(words, *size) < 14.4, "{words}");
        }
    }

    #[test]
    fn the_13_12_reasons_name_each_comparison() {
        assert_eq!(
            reasons_13_12().unwrap(),
            [
                "Reasons we have: Taylor–Green error against the exact flow ÷ 4 per halving of the cells;",
                "the step's own difference from a 640-step run ÷ 4 per halving of the step;",
                "budgets met in every run; a dye budget that caught a 0.21 loss.",
            ]
        );
        for words in reasons_13_12().unwrap() {
            // Centred at x = 1.5, clear of the page name that ends near x = −3.9.
            assert!(text_width(&words, 18.0) < 10.4, "{words}");
        }
    }

    #[test]
    fn the_planned_ladder_rows_read_as_the_solvers_checks() {
        let rungs = ladder();
        let c: Vec<String> = rungs.iter().map(|r| two(r.courant)).collect();
        let r: Vec<String> = rungs.iter().map(|r| digits(r.diffusion_weight, 4)).collect();
        assert_eq!(c, ["0.41", "0.41", "0.41", "0.41", "0.25", "0.12"]);
        assert_eq!(r, ["0.0259", "0.0519", "0.1038", "0.2075", "0.2496", "0.2499"]);
        let would_be: Vec<String> = rungs
            .iter()
            .filter(|r| r.locked)
            .map(|r| digits(diffusion_weight(r.cells, TG_END / (20 * r.cells / 32) as f64), 3))
            .collect();
        assert_eq!(would_be, ["0.415", "0.830"]);
    }

    #[test]
    fn the_13_10_precisely_lines_fit_the_frame() {
        for words in PRECISELY_13_10
            .iter()
            .chain(&PRECISELY_13_10_GAP)
            .chain(&PRECISELY_13_10_OPEN)
        {
            assert!(text_width(words, 19.0) < 14.4, "{words}");
        }
    }
}
