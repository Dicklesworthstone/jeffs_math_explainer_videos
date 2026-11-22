//! Chapter 12, "Time is also a mesh", drawn from the update data in `timestep`.
//!
//! The chapter keeps its three limits physically apart: the Courant limit on the dye row (12_1 to
//! 12_14), the time step's own error on the channel parcel, with no cells at all (12_15 to 12_20),
//! and viscosity's step limit on the layers (12_21 to 12_25). Then come the refinement test
//! (12_26, 12_27). Every line is gated by `Kit::say` before its picture. Both predictions
//! (12_h1 after 12_4, 12_h2 after 12_17)
//! hold a five-second countdown with the answer hidden.

use std::f64::consts::PI;

use explainer_kit::{BACKGROUND, Kit, Palette, hex, v};
use fmn::prelude::*;

use super::timestep::{
    CENTRED_WINDOW, CHANNEL_TIME, CHAPTER11_COURANT, FITTED_WINDOW, PULSE_WIDTH, Row, Rule, TRAVEL, TimeRule,
    channel_exact, channel_miss, channel_path, channel_speed, dye_diffusion_weight, exact_layers, explicit_layers,
    implicit_layers, implicit_matrix, largest_safe_step, neighbor_average, trip_steps, zigzag, zigzag_left,
};

pub struct Chapter12 {
    pub kit: Kit,
}

/// Pacing, in seconds of scene time. No number on screen depends on these.
pub mod pacing {
    /// 12_3: each of the 48 overlaid steps.
    pub const OVERLAY_STEP: f64 = 0.07;
    /// 12_4 and 12_10: a pan between windows, made while the pulse is static.
    pub const PAN: f64 = 1.2;
    /// 12_5: a change of the vertical scale, made while the bars are static.
    pub const RESCALE: f64 = 0.45;
    /// 12_5: each of the ten blow-up steps.
    pub const BLOWUP_STEP: f64 = 0.5;
    /// 12_9: each of the 12 exact-copy steps, slow enough to see the grid slide one cell.
    pub const COPY_STEP: f64 = 0.25;
    /// 12_10: each of the 16 blend steps at C = 3/2.
    pub const BLEND_STEP: f64 = 0.2;
    /// 12_7: the parcel crossing one and a half cells.
    pub const PARCEL: f64 = 1.4;
    /// 12_15: the exact parcel's half second in the channel.
    pub const CHANNEL_RUN: f64 = 2.4;
    /// 12_16 and 12_18: between stepped markers.
    pub const MARKER: f64 = 0.22;
    /// 12_22: one layer state held on screen.
    pub const LAYER_HOLD: f64 = 1.1;
    /// The prediction countdown, in whole seconds.
    pub const COUNTDOWN: u32 = 5;
    /// Tick animations redraw at most this often per second, each tick a whole number of frames.
    pub const REDRAWS: f64 = 30.0;
}

// Chapter 11's row and readouts (chapter11-pulse03.rs), in the opening's 16 × 9 scene units.
const PLOT_LEFT: f64 = -6.55;
const PLOT_RIGHT: f64 = 2.70;
const PLOT_BOTTOM: f64 = -0.72;
const PLOT_HEIGHT: f64 = 2.25;
const STATS_X: f64 = 4.94;
/// The blow-up's scale bar, between the row and the readouts.
const SCALE_X: f64 = 3.12;
/// The scale bar's values: it relabels as the values pass 2, 10 and 100.
const SCALE_LADDER: [f64; 4] = [2.0, 10.0, 100.0, 200.0];
/// 12_11's two results, side by side.
const MINI_WINDOW: [f64; 2] = [-10.0, 14.0];
const PRECISELY_Y: f64 = -3.42;

// 12_7 and 12_8: a close-up strip of cells, the wind blowing left to right.
const STRIP_LEFT: f64 = -6.2;
const STRIP_CELL: f64 = 1.1;
const STRIP_Y: f64 = 0.35;
const STRIP_CELLS: usize = 6;
const TARGET: usize = 4;

// Chapter 03's physical channel, with room to its right for the magnifier.
const CHANNEL_ORIGIN: [f64; 2] = [-6.1, 0.0];
const CHANNEL_SCALE: f64 = 1.25;
const CHANNEL_LENGTH: f64 = 2.0;
/// The stretch of centre line, in metres, that the inset magnifies.
const ZOOM: [f64; 2] = [1.10, 1.32];
const INSET_CENTRE: [f64; 2] = [-1.35, 1.25];
const INSET_SIZE: [f64; 2] = [3.4, 1.15];
/// Forward Euler step counts down the ladder: steps of ½, ¼, 1/8, 1/16 and 1/32 s.
const LADDER_STEPS: [usize; 5] = [1, 2, 4, 8, 16];

// The layers: chapter 04's horizontal layers as a stack, bar length proportional to speed.
const STACK_TOP: f64 = 1.95;
const STACK_HEIGHT: f64 = 4.0;
const SPEED_ZERO: f64 = -5.0;
const SPEED_UNIT: f64 = 0.85;
/// Length of the step ruler per unit of r.
const RULER_PER_R: f64 = 2.6;

fn dye() -> Srgb {
    hex("#F49BBE")
}

fn white() -> Srgb {
    hex("#FFFFFF")
}

fn card() -> Srgb {
    hex("#182433")
}

fn background() -> Srgb {
    hex(BACKGROUND)
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

// ------------------------------------------------------------------------------------ numbers

/// Two decimals with a true minus sign; anything that rounds to zero prints as 0.00.
fn two(x: f64) -> String {
    let x = if x.abs() < 0.005 { 0.0 } else { x };
    format!("{x:.2}").replace('-', "−")
}

/// A whole number when the value is one, else two decimals.
fn plain(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 {
        (x.round() as i64).to_string().replace('-', "−")
    } else {
        two(x)
    }
}

/// A whole number when the value is one, else one decimal.
fn tenths(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 {
        plain(x)
    } else {
        format!("{x:.1}").replace('-', "−")
    }
}

/// Short fractions by name (½, ¼, ¾, 3/2, 1/5, 1/8, …); anything else as [`plain`].
fn fraction(x: f64) -> String {
    let sign = if x < 0.0 { "−" } else { "" };
    for q in [1u32, 2, 3, 4, 5, 8, 16, 32] {
        let p = x.abs() * f64::from(q);
        if (p - p.round()).abs() < 1e-9 {
            let p = p.round() as u64;
            let name = match (p, q) {
                (_, 1) => p.to_string(),
                (1, 2) => "½".to_owned(),
                (1, 4) => "¼".to_owned(),
                (3, 4) => "¾".to_owned(),
                _ => format!("{p}/{q}"),
            };
            return format!("{sign}{name}");
        }
    }
    plain(x)
}

/// A miss in centimetres, whole above 3 cm.
fn centimetres(metres: f64) -> String {
    let cm = metres * 100.0;
    if cm >= 3.0 {
        format!("{} cm", cm.round())
    } else {
        format!("{cm:.1} cm")
    }
}

fn millimetres(metres: f64) -> String {
    format!("{} mm", (metres * 1000.0).round())
}

/// A small whole number in words, as the Precisely lines write them.
fn word(x: f64) -> String {
    const WORDS: [&str; 13] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    ];
    let n = x.round();
    if (0.0..=12.0).contains(&n) {
        WORDS[n as usize].to_owned()
    } else {
        plain(n)
    }
}

fn cells(courant: f64) -> &'static str {
    if courant > 1.0 { "cells" } else { "cell" }
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
    let text = kit.text_left(stage, words, size, color, at)?;
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
    if (b[0] - a[0]).hypot(b[1] - a[1]) < 1e-3 {
        return Ok(Vec::new());
    }
    (0..dashes)
        .map(|n| {
            let (s0, s1) = (n as f64 / dashes as f64, (n as f64 + 0.63) / dashes as f64);
            let start = v(mix(a[0], b[0], s0), mix(a[1], b[1], s0));
            let end = v(mix(a[0], b[0], s1), mix(a[1], b[1], s1));
            line(stage, start, end, color, 1.45)
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

/// A hollow circle.
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
fn verdict(stage: &mut Stage<'_>, at: [f64; 2], good: bool, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let [x, y] = at;
    if good {
        Ok(vec![
            line(stage, v(x - 0.13, y), v(x - 0.04, y - 0.1), color, 2.6)?,
            line(stage, v(x - 0.04, y - 0.1), v(x + 0.14, y + 0.13), color, 2.6)?,
        ])
    } else {
        Ok(vec![
            line(stage, v(x - 0.12, y - 0.12), v(x + 0.12, y + 0.12), color, 2.6)?,
            line(stage, v(x - 0.12, y + 0.12), v(x + 0.12, y - 0.12), color, 2.6)?,
        ])
    }
}

/// A lower third with the exact form of a simplification, one text line per entry.
fn precisely(kit: &Kit, stage: &mut Stage<'_>, lines: &[&str]) -> fmn::Result<Vec<Mob>> {
    lines
        .iter()
        .enumerate()
        .map(|(i, words)| {
            shown(
                kit,
                stage,
                words,
                19.0,
                Palette::dim(),
                v(0.0, PRECISELY_Y - 0.4 * i as f64),
            )
        })
        .collect()
}

/// The prediction card, held through the countdown (chapter 11's `predict`). Whatever would give
/// the answer away is not drawn until the caller's reveal line.
fn predict(stage: &mut Stage<'_>, kit: &Kit, question: &str, options: &str) -> fmn::Result<()> {
    let asked = [
        panel(stage, [-0.85, 0.51], [8.9, 2.42], (card(), 1.0), (card(), 1.4))?,
        shown(kit, stage, question, 38.0, Palette::ink(), v(-0.85, 1.08))?,
        shown(kit, stage, options, 25.0, Palette::dim(), v(-0.85, 0.31))?,
    ];
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    for second in (1..=pacing::COUNTDOWN).rev() {
        let clock = shown(kit, stage, &second.to_string(), 34.0, Palette::hero(), v(-0.85, -0.19))?;
        pause(stage, 1.0)?;
        stage.scene_mut().remove(&[clock]);
    }
    stage.scene_mut().remove(&asked);
    Ok(())
}

// ---------------------------------------------------------------------------------- the row

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Filled,
    /// Gold outline only, so a second rule can lie over the first.
    Outlined,
}

/// Chapter 11's pulse row, generalised: any riding window, a visible vertical scale, and rows
/// drawn filled or outlined. Each cell is one `Rectangle`; a step replaces the heights at once.
struct RowView {
    window: [f64; 2],
    /// The value a bar of full plot height holds.
    scale: f64,
    scale_shown: bool,
    frame: Vec<Mob>,
    outline: Vec<Mob>,
    bars: Vec<Mob>,
    scale_bar: Vec<Mob>,
}

impl RowView {
    fn new(stage: &mut Stage<'_>, kit: &Kit, window: [f64; 2]) -> fmn::Result<Self> {
        let frame = vec![
            line(
                stage,
                v(PLOT_LEFT, PLOT_BOTTOM),
                v(PLOT_RIGHT, PLOT_BOTTOM),
                Palette::dim(),
                1.6,
            )?,
            shown(
                kit,
                stage,
                "Exact pulse in the moving window",
                24.0,
                Palette::ink(),
                v(-1.92, 2.15),
            )?,
        ];
        let mut view = Self {
            window,
            scale: 1.0,
            scale_shown: false,
            frame,
            outline: Vec::new(),
            bars: Vec::new(),
            scale_bar: Vec::new(),
        };
        view.draw_outline(stage)?;
        Ok(view)
    }

    fn x(&self, offset: f64) -> f64 {
        PLOT_LEFT + (PLOT_RIGHT - PLOT_LEFT) * (offset - self.window[0]) / (self.window[1] - self.window[0])
    }

    fn y(&self, value: f64) -> f64 {
        PLOT_BOTTOM + PLOT_HEIGHT * value / self.scale
    }

    /// The dotted exact pulse: it rides with the window, so only a pan or a rescale moves it.
    fn draw_outline(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        clear(stage, &mut self.outline);
        let (a, b, top) = (self.x(0.0), self.x(PULSE_WIDTH as f64), self.y(1.0));
        self.outline = dotted(stage, [a, top], [b, top], 8, Palette::ink())?;
        for edge in [a, b] {
            let side = dotted(stage, [edge, PLOT_BOTTOM], [edge, top], 12, Palette::ink())?;
            self.outline.extend(side);
        }
        Ok(())
    }

    fn draw(&mut self, stage: &mut Stage<'_>, rows: &[(&Row, Look)]) -> fmn::Result<()> {
        clear(stage, &mut self.bars);
        for (n, &(row, look)) in rows.iter().enumerate() {
            for (index, &value) in row.values.iter().enumerate() {
                let [a, b] = row.riding_bounds(index);
                if b <= self.window[0] || a >= self.window[1] {
                    continue;
                }
                let (x0, x1) = (self.x(a.max(self.window[0])), self.x(b.min(self.window[1])));
                if n == 0 && a >= self.window[0] {
                    let tick = line(
                        stage,
                        v(x0, PLOT_BOTTOM - 0.14),
                        v(x0, PLOT_BOTTOM),
                        Palette::dim(),
                        1.0,
                    )?;
                    self.bars.push(tick);
                }
                let bar = self.bar(stage, [x0, x1], value, look)?;
                self.bars.extend(bar);
            }
        }
        Ok(())
    }

    /// One cell's bar: rose, red below the baseline, with a white outline on the part outside
    /// 0 to 1.
    fn bar(&self, stage: &mut Stage<'_>, x: [f64; 2], value: f64, look: Look) -> fmn::Result<Vec<Mob>> {
        let (bottom, top) = (self.y(value.min(0.0)), self.y(value.max(0.0)));
        if top - bottom <= 1e-5 {
            return Ok(Vec::new());
        }
        let (centre, width) = ((x[0] + x[1]) / 2.0, (x[1] - x[0]) * 0.91);
        let size = [width, top - bottom];
        let middle = (bottom + top) / 2.0;
        if look == Look::Outlined {
            let gold = Palette::gold();
            return Ok(vec![panel(stage, [centre, middle], size, (gold, 0.0), (gold, 2.2))?]);
        }
        let color = if value < 0.0 { Palette::red() } else { dye() };
        let mut out = vec![panel(stage, [centre, middle], size, (color, 0.73), (color, 0.0))?];
        let outside = if value < 0.0 {
            Some((bottom, top))
        } else if value > 1.0 {
            Some((self.y(1.0), top))
        } else {
            None
        };
        if let Some((low, high)) = outside
            && high - low > 2e-3
        {
            let mark = panel(
                stage,
                [centre, (low + high) / 2.0],
                [width, high - low],
                (white(), 0.0),
                (white(), 1.2),
            )?;
            out.push(mark);
        }
        Ok(out)
    }

    /// The labelled scale bar: a bar this tall holds the value printed on it.
    fn draw_scale(&mut self, stage: &mut Stage<'_>, kit: &Kit, moving: bool) -> fmn::Result<()> {
        clear(stage, &mut self.scale_bar);
        if !self.scale_shown {
            return Ok(());
        }
        let (ink, top) = (Palette::ink(), PLOT_BOTTOM + PLOT_HEIGHT);
        let label = if moving { Palette::gold() } else { ink };
        self.scale_bar = vec![
            line(stage, v(SCALE_X, PLOT_BOTTOM), v(SCALE_X, top), ink, 2.0)?,
            line(
                stage,
                v(SCALE_X - 0.09, PLOT_BOTTOM),
                v(SCALE_X + 0.09, PLOT_BOTTOM),
                ink,
                2.0,
            )?,
            line(stage, v(SCALE_X - 0.09, top), v(SCALE_X + 0.09, top), ink, 2.0)?,
            shown(kit, stage, &plain(self.scale), 24.0, label, v(SCALE_X, top + 0.27))?,
            shown(kit, stage, "scale", 17.0, Palette::dim(), v(SCALE_X, PLOT_BOTTOM - 0.3))?,
        ];
        Ok(())
    }

    /// Change the vertical scale to `to` while the bars are static, in view, with the scale bar's
    /// label reading the scale as it goes.
    fn rescale(&mut self, stage: &mut Stage<'_>, kit: &Kit, rows: &[(&Row, Look)], to: f64) -> fmn::Result<()> {
        let from = self.scale;
        self.scale_shown = true;
        let (count, per) = ticks(stage, pacing::RESCALE);
        for n in 1..=count {
            self.scale = if n == count {
                to
            } else {
                mix(from, to, ease(n as f64 / count as f64))
            };
            self.draw_outline(stage)?;
            self.draw(stage, rows)?;
            self.draw_scale(stage, kit, n < count)?;
            wait_frames(stage, per)?;
        }
        Ok(())
    }

    /// Pan to another riding window while the row is static. Never called during a replay.
    fn pan(&mut self, stage: &mut Stage<'_>, rows: &[(&Row, Look)], to: [f64; 2]) -> fmn::Result<()> {
        let from = self.window;
        let (count, per) = ticks(stage, pacing::PAN);
        for n in 1..=count {
            let s = ease(n as f64 / count as f64);
            self.window = if n == count {
                to
            } else {
                [mix(from[0], to[0], s), mix(from[1], to[1], s)]
            };
            self.draw_outline(stage)?;
            self.draw(stage, rows)?;
            wait_frames(stage, per)?;
        }
        Ok(())
    }

    fn remove_all(&mut self, stage: &mut Stage<'_>) {
        for group in [&mut self.frame, &mut self.outline, &mut self.bars, &mut self.scale_bar] {
            clear(stage, group);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Total {
    Readout,
    /// The dye-total badge, ticked while every step kept the start's total.
    Badge {
        kept: bool,
    },
}

/// Chapter 11's readouts in the side panel, over every cell of the row.
fn readouts(stage: &mut Stage<'_>, kit: &Kit, row: &Row, of: usize, total: Total) -> fmn::Result<Vec<Mob>> {
    let m = row.measure();
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let low = if m.lowest < 0.0 { Palette::red() } else { dim };
    let mut out = vec![
        shown(
            kit,
            stage,
            &format!("Peak  {}", two(m.peak)),
            26.0,
            dye(),
            v(STATS_X, 1.05),
        )?,
        shown(
            kit,
            stage,
            &format!("Lowest  {}", two(m.lowest)),
            26.0,
            low,
            v(STATS_X, 0.43),
        )?,
        shown(
            kit,
            stage,
            &format!("Step {} of {of}", row.step),
            22.0,
            dim,
            v(STATS_X, -0.81),
        )?,
        shown(kit, stage, "Readouts count every cell", 18.0, dim, v(STATS_X, -1.33))?,
    ];
    match total {
        Total::Readout => {
            let text = format!("Dye total  {}", two(m.total));
            out.push(shown(kit, stage, &text, 26.0, ink, v(STATS_X, -0.19))?);
        }
        Total::Badge { kept } => {
            // A rose outline, its value, and a tick; never red or green (those are the opening's
            // imbalance badge).
            out.push(panel(stage, [STATS_X, -0.19], [2.9, 0.56], (dye(), 0.0), (dye(), 2.0))?);
            let text = format!("dye total  {}", plain(m.total));
            out.push(shown(kit, stage, &text, 24.0, ink, v(STATS_X - 0.3, -0.19))?);
            out.extend(verdict(stage, [STATS_X + 1.05, -0.19], kept, ink)?);
        }
    }
    Ok(out)
}

/// The Courant readout in the side panel.
fn courant_panel(stage: &mut Stage<'_>, kit: &Kit, courant: f64) -> fmn::Result<Vec<Mob>> {
    let value = format!("C = {} {} per step", fraction(courant), cells(courant));
    Ok(vec![
        shown_tex(
            kit,
            stage,
            r"C = u \Delta t / \Delta x",
            30.0,
            Palette::ink(),
            v(STATS_X, 2.62),
        )?,
        shown(kit, stage, "Courant number", 19.0, Palette::dim(), v(STATS_X, 2.2))?,
        shown(kit, stage, &value, 25.0, Palette::gold(), v(STATS_X, 1.8))?,
    ])
}

/// The neighbor rule's two weights at `courant`, from the rule itself.
fn weight_diagram(stage: &mut Stage<'_>, kit: &Kit, courant: f64) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let mut out = Vec::new();
    for (back, weight) in Rule::Neighbor.weights(courant) {
        // Clear of the prediction card's right edge (x = 3.6).
        let x = if back == 0 { 5.59 } else { 4.29 };
        out.push(panel(stage, [x, 0.55], [1.2, 0.75], (card(), 1.0), (dim, 1.4))?);
        let name = if back == 0 { "own cell" } else { "upstream" };
        out.push(shown(kit, stage, name, 19.0, dim, v(x, 1.15))?);
        let color = if weight < 0.0 { Palette::red() } else { Palette::gold() };
        out.push(shown(kit, stage, &fraction(weight), 30.0, color, v(x, 0.55))?);
    }
    out.push(shown_tex(
        kit,
        stage,
        r"(1 - C) q_i + C q_{i-1}",
        24.0,
        Palette::ink(),
        v(STATS_X, -0.25),
    )?);
    Ok(out)
}

/// 12_1's time ruler under the row, its ticks marking steps.
fn time_ruler(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let y = -2.05;
    let mut out = vec![line(stage, v(PLOT_LEFT, y), v(PLOT_RIGHT, y), Palette::ink(), 1.6)?];
    for n in 0..=12 {
        let x = mix(PLOT_LEFT, PLOT_RIGHT, f64::from(n) / 12.0);
        out.push(line(stage, v(x, y - 0.1), v(x, y + 0.1), Palette::ink(), 1.6)?);
    }
    let first = mix(PLOT_LEFT, PLOT_RIGHT, 0.5 / 12.0);
    out.push(shown_tex(
        kit,
        stage,
        r"\Delta t",
        24.0,
        Palette::gold(),
        v(first, y + 0.32),
    )?);
    out.push(shown(
        kit,
        stage,
        "Time, cut into steps",
        21.0,
        Palette::dim(),
        v(-1.92, y - 0.42),
    )?);
    Ok(out)
}

fn strip_centre(cell: usize) -> f64 {
    STRIP_LEFT + (cell as f64 + 0.5) * STRIP_CELL
}

/// One of 12_11's two results against its own dotted exact pulse.
fn mini_row(stage: &mut Stage<'_>, kit: &Kit, row: &Row, centre: f64) -> fmn::Result<Vec<Mob>> {
    let (left, right, bottom, height) = (centre - 3.1, centre + 3.1, -1.0, 2.0);
    let x = |offset: f64| left + (right - left) * (offset - MINI_WINDOW[0]) / (MINI_WINDOW[1] - MINI_WINDOW[0]);
    let mut out = vec![line(stage, v(left, bottom), v(right, bottom), Palette::dim(), 1.4)?];
    for (index, &value) in row.values.iter().enumerate() {
        let [a, b] = row.riding_bounds(index);
        if b <= MINI_WINDOW[0] || a >= MINI_WINDOW[1] || value.abs() * height <= 1e-5 {
            continue;
        }
        let (x0, x1) = (x(a.max(MINI_WINDOW[0])), x(b.min(MINI_WINDOW[1])));
        let at = [(x0 + x1) / 2.0, bottom + value * height / 2.0];
        out.push(panel(
            stage,
            at,
            [(x1 - x0) * 0.91, value.abs() * height],
            (dye(), 0.73),
            (dye(), 0.0),
        )?);
    }
    let (a, b, top) = (x(0.0), x(PULSE_WIDTH as f64), bottom + height);
    out.extend(dotted(stage, [a, top], [b, top], 8, Palette::ink())?);
    for edge in [a, b] {
        out.extend(dotted(stage, [edge, bottom], [edge, top], 12, Palette::ink())?);
    }
    let label = format!("C = {} {} per step", fraction(row.courant), cells(row.courant));
    out.push(shown(kit, stage, &label, 24.0, Palette::ink(), v(centre, 1.55))?);
    let peak = format!("Peak  {}", two(row.measure().peak));
    out.push(shown(kit, stage, &peak, 26.0, dye(), v(centre, -1.55))?);
    Ok(out)
}

/// 12_12's six labelled points: the blend's smearing over the same trip at each step size.
fn spread_chart(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let x = |courant: f64| -5.2 + 3.2 * courant;
    let y = |spread: f64| -1.6 + 0.17 * spread;
    let title = format!("Smearing over the same {TRAVEL}-cell trip, in square cells");
    let mut out = vec![
        line(stage, v(x(0.0), y(0.0)), v(x(2.3), y(0.0)), dim, 1.6)?,
        line(stage, v(x(0.0), y(0.0)), v(x(0.0), y(21.0)), dim, 1.6)?,
        shown(kit, stage, "Cells per step, C", 22.0, dim, v(x(1.15), -2.42))?,
        shown(kit, stage, &title, 22.0, dim, v(x(1.15), 2.35))?,
    ];
    for spread in [0.0, 6.0, 12.0, 18.0] {
        out.push(line(
            stage,
            v(x(0.0) - 0.08, y(spread)),
            v(x(0.0), y(spread)),
            dim,
            1.4,
        )?);
        out.push(shown(
            kit,
            stage,
            &plain(spread),
            19.0,
            dim,
            v(x(0.0) - 0.35, y(spread)),
        )?);
    }
    for courant in [0.25, 0.5, 0.75, 1.0, 1.5, 2.0] {
        let mut row = Row::trip(Rule::Blend, courant, 1);
        for _ in 0..trip_steps(courant, 1) {
            row.advance();
        }
        let spread = row.spread();
        out.push(line(
            stage,
            v(x(courant), y(0.0)),
            v(x(courant), y(0.0) - 0.08),
            dim,
            1.4,
        )?);
        out.push(shown(kit, stage, &fraction(courant), 21.0, ink, v(x(courant), -1.98))?);
        out.push(disc(stage, [x(courant), y(spread)], 0.09, Palette::gold())?);
        out.push(shown(
            kit,
            stage,
            &plain(spread),
            22.0,
            ink,
            v(x(courant), y(spread) + 0.3),
        )?);
    }
    Ok(out)
}

// ------------------------------------------------------------------------------ the channel

fn channel_point(x: f64, y: f64) -> [f64; 2] {
    [
        CHANNEL_ORIGIN[0] + CHANNEL_SCALE * x,
        CHANNEL_ORIGIN[1] + CHANNEL_SCALE * y,
    ]
}

/// The channel's walls are the streamlines y = ±4/(2 + x) metres.
fn wall(x: f64) -> f64 {
    4.0 / (2.0 + x)
}

fn inset_x(x: f64) -> f64 {
    INSET_CENTRE[0] - 1.55 + 3.1 * (x - ZOOM[0]) / (ZOOM[1] - ZOOM[0])
}

/// The opening's exact channel flow, u = 2 + x and v = −y: walls, a few velocity arrows and the
/// width labels.
fn channel(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for side in [-1.0, 1.0] {
        for n in 0..24 {
            let (x0, x1) = (
                CHANNEL_LENGTH * f64::from(n) / 24.0,
                CHANNEL_LENGTH * f64::from(n + 1) / 24.0,
            );
            let (a, b) = (channel_point(x0, side * wall(x0)), channel_point(x1, side * wall(x1)));
            out.push(line(stage, v(a[0], a[1]), v(b[0], b[1]), Palette::ink(), 2.4)?);
        }
        for x in [0.25, 0.75, 1.25, 1.75] {
            let y = side * 0.55 * wall(x);
            let start = channel_point(x, y);
            let (u, w) = (channel_speed(x), -y);
            out.push(arrow(
                stage,
                start,
                [start[0] + 0.11 * u, start[1] + 0.11 * w],
                Palette::blue(),
                1.2,
            )?);
        }
    }
    let (entrance, exit) = (channel_point(0.0, 0.0), channel_point(CHANNEL_LENGTH, 0.0));
    let entrance_width = format!("{} m", plain(2.0 * wall(0.0)));
    let exit_width = format!("{} m", plain(2.0 * wall(CHANNEL_LENGTH)));
    out.push(shown(
        kit,
        stage,
        &entrance_width,
        20.0,
        Palette::dim(),
        v(entrance[0] - 0.5, entrance[1]),
    )?);
    out.push(shown(
        kit,
        stage,
        &exit_width,
        20.0,
        Palette::dim(),
        v(exit[0] + 0.45, exit[1]),
    )?);
    Ok(out)
}

/// The magnifier on the last stretch of the parcel's path, with the exact parcel in it.
fn inset(stage: &mut Stage<'_>, kit: &Kit, exact: f64) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let [cx, cy] = INSET_CENTRE;
    let [width, height] = INSET_SIZE;
    let (a, b) = (channel_point(ZOOM[0], 0.0), channel_point(ZOOM[1], 0.0));
    let across = format!("zoom: {} cm across", ((ZOOM[1] - ZOOM[0]) * 100.0).round());
    Ok(vec![
        panel(stage, INSET_CENTRE, INSET_SIZE, (card(), 1.0), (dim, 1.4))?,
        line(stage, v(cx - 1.6, cy), v(cx + 1.6, cy), dim, 1.0)?,
        panel(
            stage,
            [(a[0] + b[0]) / 2.0, a[1]],
            [b[0] - a[0], 0.22],
            (dim, 0.0),
            (dim, 1.0),
        )?,
        line(
            stage,
            v(a[0], a[1] + 0.11),
            v(cx - width / 2.0, cy - height / 2.0),
            dim,
            0.8,
        )?,
        line(
            stage,
            v(b[0], b[1] + 0.11),
            v(cx + width / 2.0, cy - height / 2.0),
            dim,
            0.8,
        )?,
        // Keep the scale text below the projection lines.
        shown(kit, stage, &across, 17.0, dim, v(cx, cy - height / 2.0 - 0.78))?,
        disc(stage, [inset_x(exact), cy], 0.1, Palette::gold())?,
    ])
}

/// Hollow gold markers at the stepped positions, in the channel and, where they fall, in the
/// inset.
fn markers(stage: &mut Stage<'_>, path: &[f64], radius: f64, gap: f64) -> fmn::Result<Vec<Mob>> {
    let gold = Palette::gold();
    let mut out = Vec::new();
    for &x in &path[1..] {
        out.push(ring(stage, channel_point(x, 0.0), radius, gold, 2.0)?);
        if (ZOOM[0]..=ZOOM[1]).contains(&x) {
            out.push(ring(stage, [inset_x(x), INSET_CENTRE[1]], radius * 1.3, gold, 2.2)?);
        }
        pause(stage, gap)?;
    }
    Ok(out)
}

fn ladder_y(row: usize) -> f64 {
    1.55 - 0.5 * row as f64
}

/// A dial with tick marks and a needle; `rungs` is its refinement ladder.
fn dial(
    stage: &mut Stage<'_>,
    kit: &Kit,
    at: [f64; 2],
    label: &str,
    rungs: &[String],
    tag: Option<&str>,
) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
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
    for (j, rung) in rungs.iter().enumerate() {
        let ry = y - 1.45 - 0.42 * j as f64;
        out.push(line(stage, v(x - 1.2, ry + 0.19), v(x + 1.2, ry + 0.19), dim, 1.2)?);
        out.push(shown(kit, stage, rung, 20.0, ink, v(x, ry))?);
    }
    if let Some(tag) = tag {
        out.push(shown(kit, stage, tag, 19.0, Palette::gold(), v(x, y + 0.95))?);
    }
    Ok(out)
}

/// A narrow band on a dial's rim by its needle, labelled "declared tolerance".
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

/// The opening's question card.
fn question_card(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    Ok(vec![
        panel(stage, [0.0, 2.3], [9.6, 0.9], (card(), 1.0), (Palette::dim(), 1.4))?,
        shown(
            kit,
            stage,
            "Does halving the cells change the answer?",
            30.0,
            Palette::ink(),
            v(0.0, 2.3),
        )?,
    ])
}

// ------------------------------------------------------------------------------- the layers

fn speed_x(speed: f64) -> f64 {
    SPEED_ZERO + SPEED_UNIT * speed
}

fn layer_gap(layers: usize) -> f64 {
    STACK_HEIGHT / layers as f64
}

/// The stack: one blue bar per layer, length proportional to speed, and violet arrows from each
/// layer toward its neighbors' average when `pulls`.
fn stack(stage: &mut Stage<'_>, kit: &Kit, speeds: &[f64], pulls: bool) -> fmn::Result<Vec<Mob>> {
    let gap = layer_gap(speeds.len());
    let mut out = Vec::new();
    for (i, (&speed, target)) in speeds.iter().zip(neighbor_average(speeds)).enumerate() {
        let y = STACK_TOP - gap * i as f64;
        let (a, b) = (speed_x(0.0).min(speed_x(speed)), speed_x(0.0).max(speed_x(speed)));
        if b - a > 1e-4 {
            let blue = Palette::blue();
            out.push(panel(
                stage,
                [(a + b) / 2.0, y],
                [b - a, 0.62 * gap],
                (blue, 0.8),
                (blue, 0.0),
            )?);
        }
        if speeds.len() <= 8 {
            out.push(shown(kit, stage, &tenths(speed), 20.0, Palette::ink(), v(-6.55, y))?);
        }
        if pulls && (target - speed).abs() > 1e-9 {
            let lift = y + 0.4 * gap;
            out.push(arrow(
                stage,
                [speed_x(speed), lift],
                [speed_x(target), lift],
                Palette::purple(),
                1.2,
            )?);
        }
    }
    Ok(out)
}

fn stack_bottom(layers: usize) -> f64 {
    STACK_TOP - layer_gap(layers) * (layers as f64 - 1.0)
}

/// The speed axis under a stack of `layers`.
fn speed_axis(stage: &mut Stage<'_>, kit: &Kit, layers: usize) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let axis = stack_bottom(layers) - 0.55;
    let mut out = vec![
        line(stage, v(speed_x(-1.0), axis), v(speed_x(5.0), axis), dim, 1.2)?,
        shown(kit, stage, "m/s", 18.0, dim, v(speed_x(5.0) + 0.45, axis))?,
    ];
    for speed in -1..=5 {
        let x = speed_x(f64::from(speed));
        out.push(line(stage, v(x, axis - 0.07), v(x, axis + 0.07), dim, 1.2)?);
        out.push(shown(
            kit,
            stage,
            &plain(f64::from(speed)),
            17.0,
            dim,
            v(x, axis - 0.27),
        )?);
    }
    Ok(out)
}

/// The thin light-ink line where the stack settles: its mean.
fn settle_line(stage: &mut Stage<'_>, kit: &Kit, layers: usize, mean: f64) -> fmn::Result<Vec<Mob>> {
    let (ink, x) = (Palette::ink(), speed_x(mean));
    Ok(vec![
        line(
            stage,
            v(x, STACK_TOP + 0.4),
            v(x, stack_bottom(layers) - 0.35),
            ink,
            1.0,
        )?,
        shown(kit, stage, "where the stack settles", 20.0, ink, v(x, STACK_TOP + 0.62))?,
    ])
}

/// The speed axis and the settle line.
fn stack_frame(stage: &mut Stage<'_>, kit: &Kit, layers: usize, mean: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = speed_axis(stage, kit, layers)?;
    out.extend(settle_line(stage, kit, layers, mean)?);
    Ok(out)
}

/// The side panel for one explicit layer step: r, the step ruler and what the step did.
fn step_panel(stage: &mut Stage<'_>, kit: &Kit, r: f64, ruler: f64, result: &str) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let (left, right, y) = (STATS_X - ruler / 2.0, STATS_X + ruler / 2.0, 1.2);
    let value = format!("r = {}", fraction(r));
    Ok(vec![
        shown_tex(kit, stage, r"r = \nu \Delta t / h^2", 28.0, ink, v(STATS_X, 2.3))?,
        shown(kit, stage, &value, 26.0, Palette::gold(), v(STATS_X, 1.75))?,
        line(stage, v(left, y), v(right, y), ink, 2.0)?,
        line(stage, v(left, y - 0.1), v(left, y + 0.1), ink, 2.0)?,
        line(stage, v(right, y - 0.1), v(right, y + 0.1), ink, 2.0)?,
        shown(kit, stage, "step", 18.0, dim, v(STATS_X, y - 0.28))?,
        shown(kit, stage, result, 22.0, ink, v(STATS_X, 0.35))?,
    ])
}

/// What one layer step did, from its result.
fn landing(after: &[f64], mean: f64) -> String {
    if after.iter().all(|&x| (x - mean).abs() < 1e-9) {
        format!("every layer lands on {}", plain(mean))
    } else {
        format!("{} and {}", tenths(after[0]), tenths(after[1]))
    }
}

/// Show `before` briefly, then `after`, as two computed states with nothing in between.
fn layer_step(
    stage: &mut Stage<'_>,
    kit: &Kit,
    layers: &mut Vec<Mob>,
    before: &[f64],
    after: &[f64],
) -> fmn::Result<()> {
    clear(stage, layers);
    *layers = stack(stage, kit, before, false)?;
    pause(stage, 0.45)?;
    clear(stage, layers);
    *layers = stack(stage, kit, after, false)?;
    pause(stage, pacing::LAYER_HOLD)
}

/// 12_23's ladder: chapter 11's rows at C = ½ with the declared dye diffusion, whose explicit
/// weight r doubles at each halving and passes the limit ½.
fn r_table(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let column = |j: u32| 1.4 + 1.08 * f64::from(j);
    let mut out = vec![
        shown(
            kit,
            stage,
            "Chapter 11's rows, explicit dye step",
            19.0,
            dim,
            v(STATS_X - 0.3, -0.25),
        )?,
        shown(kit, stage, "cells", 17.0, dim, v(0.75, -0.7))?,
        shown(kit, stage, "r", 17.0, dim, v(0.75, -1.15))?,
    ];
    for halvings in 0..6 {
        let width = 1.0 / f64::from(1u32 << halvings);
        let r = dye_diffusion_weight(halvings);
        let color = if r > 0.5 { Palette::red() } else { ink };
        out.push(shown(
            kit,
            stage,
            &fraction(width),
            21.0,
            ink,
            v(column(halvings), -0.7),
        )?);
        out.push(shown(
            kit,
            stage,
            &fraction(r),
            21.0,
            color,
            v(column(halvings), -1.15),
        )?);
    }
    let divider = (column(3) + column(4)) / 2.0;
    out.extend(dotted(stage, [divider, -0.45], [divider, -1.4], 6, Palette::gold())?);
    let label = "step must shrink with the cells squared";
    out.push(shown(
        kit,
        stage,
        label,
        22.0,
        Palette::gold(),
        v(STATS_X - 0.4, -1.78),
    )?);
    Ok(out)
}

/// The implicit step's linked equations as a small banded matrix, like chapter 09's.
fn matrix_picture(stage: &mut Stage<'_>, kit: &Kit, matrix: &[Vec<f64>]) -> fmn::Result<Vec<Mob>> {
    let (ink, cell) = (Palette::ink(), 0.36);
    let (cx, cy) = (STATS_X, 0.15);
    let half = cell * matrix.len() as f64 / 2.0;
    let mut out = Vec::new();
    for (i, row) in matrix.iter().enumerate() {
        for (j, &entry) in row.iter().enumerate() {
            if entry == 0.0 {
                continue;
            }
            let at = v(cx - half + cell * (j as f64 + 0.5), cy + half - cell * (i as f64 + 0.5));
            let color = if i == j { Palette::gold() } else { ink };
            out.push(shown(kit, stage, &plain(entry), 17.0, color, at)?);
        }
    }
    for side in [-1.0, 1.0] {
        let x = cx + side * (half + 0.08);
        out.push(line(stage, v(x, cy - half), v(x, cy + half), ink, 3.2)?);
        for y in [cy - half, cy + half] {
            out.push(line(stage, v(x, y), v(x - side * 0.14, y), ink, 3.2)?);
        }
    }
    let label = format!("all {} new speeds, solved together", word(matrix.len() as f64));
    out.push(shown(
        kit,
        stage,
        &label,
        20.0,
        Palette::dim(),
        v(cx, cy + half + 0.35),
    )?);
    Ok(out)
}

// ------------------------------------------------------------------------------- sections

type Section = fn(&mut Stage<'_>, &Kit, Vec<Mob>) -> fmn::Result<Vec<Mob>>;

/// 12_1 to 12_14: the Courant limit on the dye row.
fn courant(stage: &mut Stage<'_>, k: &Kit, mut leftover: Vec<Mob>) -> fmn::Result<Vec<Mob>> {
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());
    let half = CHAPTER11_COURANT;
    let trip = trip_steps(half, 1);

    k.say(stage, "12_1")?;
    clear(stage, &mut leftover);
    let mut view = RowView::new(stage, k, CENTRED_WINDOW)?;
    view.draw(stage, &[(&Row::trip(Rule::Blend, half, 1), Look::Filled)])?;
    let wind_label = format!("Steady wind  →    {PULSE_WIDTH} cells of dye");
    let mut wind = vec![shown(k, stage, &wind_label, 23.0, Palette::blue(), v(-1.92, 2.72))?];
    k.until_sentence(stage, "12_1", 1)?;
    let mut ruler = time_ruler(stage, k)?;

    k.say(stage, "12_2")?;
    let mut readout = courant_panel(stage, k, half)?;

    // Both rules from the same start, over the whole trip: the blend filled, the neighbor rule
    // outlined over it.
    k.say(stage, "12_3")?;
    k.until_sentence(stage, "12_3", 1)?;
    let mut weights = weight_diagram(stage, k, half)?;
    k.until_sentence(stage, "12_3", 2)?;
    let legend_text = "Filled: blend      Outlined: neighbor rule";
    let mut legend = vec![shown(k, stage, legend_text, 21.0, dim, v(-1.92, -2.95))?];
    let (mut blend, mut neighbor) = (Row::trip(Rule::Blend, half, 1), Row::trip(Rule::Neighbor, half, 1));
    let (mut counter, mut largest) = (Vec::new(), 0.0_f64);
    for _ in 0..trip {
        blend.advance();
        neighbor.advance();
        largest = blend
            .values
            .iter()
            .zip(&neighbor.values)
            .fold(largest, |m, (a, b)| m.max((a - b).abs()));
        view.draw(stage, &[(&blend, Look::Filled), (&neighbor, Look::Outlined)])?;
        clear(stage, &mut counter);
        let step = format!("Step {} of {trip}", blend.step);
        counter.push(shown(k, stage, &step, 22.0, dim, v(STATS_X, -0.81))?);
        pause(stage, pacing::OVERLAY_STEP)?;
    }
    let difference = format!("Largest difference  {}", plain(largest));
    counter.push(shown(k, stage, &difference, 22.0, ink, v(STATS_X, -1.35))?);

    // Reset to step 0, pan while static to the window fitted to the blow-up, then ask.
    k.say(stage, "12_4")?;
    for group in [&mut ruler, &mut legend, &mut counter, &mut weights] {
        clear(stage, group);
    }
    let blowup = 10;
    let mut row = Row::new(Rule::Neighbor, 1.5, 1, blowup);
    view.draw(stage, &[(&row, Look::Filled)])?;
    pause(stage, 0.3)?;
    view.pan(stage, &[(&row, Look::Filled)], FITTED_WINDOW)?;
    clear(stage, &mut readout);
    readout = courant_panel(stage, k, row.courant)?;
    k.until_sentence(stage, "12_4", 1)?;
    weights = weight_diagram(stage, k, row.courant)?;
    predict(
        stage,
        k,
        "What happens at 1.5 cells per step?",
        "Smears more      Smears less      Blows up",
    )?;

    // The reveal: rescale visibly, then step, relabelling the scale as the values pass the ladder.
    // Step 1 lands with the line's second sentence, the rest with its third.
    k.say(stage, "12_5")?;
    clear(stage, &mut weights);
    let mut level = 0;
    view.rescale(stage, k, &[(&row, Look::Filled)], SCALE_LADDER[level])?;
    let mut stats = readouts(stage, k, &row, blowup, Total::Readout)?;
    let mut kept = true;
    for step in 0..blowup {
        k.until_sentence(stage, "12_5", if step == 0 { 1 } else { 2 })?;
        let mut next = row.clone();
        next.advance();
        if !next.fits(FITTED_WINDOW) {
            return Err(fail("12_5: the neighbor rule's dye left the fitted window"));
        }
        let needed = SCALE_LADDER
            .iter()
            .position(|&scale| next.largest() <= scale)
            .ok_or_else(|| fail("12_5: the values passed the scale ladder"))?;
        if needed > level {
            level = needed;
            view.rescale(stage, k, &[(&row, Look::Filled)], SCALE_LADDER[level])?;
        }
        row = next;
        kept &= (row.measure().total - PULSE_WIDTH as f64).abs() < 1e-9;
        view.draw(stage, &[(&row, Look::Filled)])?;
        clear(stage, &mut stats);
        stats = readouts(stage, k, &row, blowup, Total::Readout)?;
        pause(stage, pacing::BLOWUP_STEP)?;
    }
    let mut note = precisely(
        k,
        stage,
        &["Its shortest zig-zag doubles every step: |G| = |1 − 2C| = 2 at C = 3/2."],
    )?;

    k.say(stage, "12_6")?;
    clear(stage, &mut stats);
    stats = readouts(stage, k, &row, blowup, Total::Badge { kept })?;

    // A close-up: the dye crosses 1.5 cells in a step, the rule reads one cell back.
    k.say(stage, "12_7")?;
    view.remove_all(stage);
    for group in [&mut stats, &mut note, &mut wind] {
        clear(stage, group);
    }
    let mut strip = Vec::new();
    for cell in 0..STRIP_CELLS {
        let edge = if cell == TARGET { ink } else { dim };
        strip.push(panel(
            stage,
            [strip_centre(cell), STRIP_Y],
            [STRIP_CELL, 0.8],
            (card(), 1.0),
            (edge, 1.6),
        )?);
    }
    let reach = Rule::Neighbor.reach(row.courant);
    let (own, upstream) = (strip_centre(TARGET), strip_centre(TARGET - reach));
    let departure = own - row.courant * STRIP_CELL;
    strip.push(shown(k, stage, "own cell", 19.0, dim, v(own, -1.05))?);
    strip.push(shown(k, stage, "neighbor", 19.0, dim, v(upstream, -1.05))?);
    k.until_sentence(stage, "12_7", 1)?;
    let mut parcel = vec![disc(stage, [departure, STRIP_Y], 0.1, gold)?];
    k.until_sentence(stage, "12_7", 2)?;
    let (count, per) = ticks(stage, pacing::PARCEL);
    for n in 1..=count {
        clear(stage, &mut parcel);
        let x = mix(departure, own, ease(n as f64 / count as f64));
        parcel.push(disc(stage, [x, STRIP_Y], 0.1, gold)?);
        wait_frames(stage, per)?;
    }
    strip.extend(parcel);
    strip.push(arrow(
        stage,
        [departure, STRIP_Y - 0.62],
        [own, STRIP_Y - 0.62],
        gold,
        1.6,
    )?);
    let crossing = format!("{:.1} cells in one step", row.courant);
    strip.push(shown(
        k,
        stage,
        &crossing,
        22.0,
        gold,
        v((departure + own) / 2.0, -0.62),
    )?);
    strip.extend(bracket(stage, [upstream, own], STRIP_Y + 0.62, -1.0, ink)?);
    let reach_label = format!("the rule's reach: {reach} {}", cells(reach as f64));
    strip.push(shown(
        k,
        stage,
        &reach_label,
        22.0,
        ink,
        v((upstream + own) / 2.0, STRIP_Y + 0.95),
    )?);
    // Where the dye came from, outlined white: beyond the bracket.
    k.until_sentence(stage, "12_7", 3)?;
    strip.push(ring(stage, [departure, STRIP_Y], 0.17, white(), 2.2)?);

    k.say(stage, "12_8")?;
    strip.push(shown(
        k,
        stage,
        "Courant, Friedrichs and Lewy",
        30.0,
        ink,
        v(-3.2, 2.45),
    )?);
    strip.push(shown(k, stage, "the CFL condition", 24.0, gold, v(-3.2, 1.95))?);
    k.until_sentence(stage, "12_8", 1)?;
    let back = Rule::Blend.reach(row.courant);
    let reads = [strip_centre(TARGET - back), strip_centre(TARGET + 1 - back)];
    strip.extend(bracket(stage, reads, -1.4, 1.0, Palette::teal())?);
    strip.push(shown(
        k,
        stage,
        "reads from",
        22.0,
        Palette::teal(),
        v((reads[0] + reads[1]) / 2.0, -1.78),
    )?);
    let quote = "“a simple and very intuitive necessary condition for convergence”";
    strip.push(shown(k, stage, quote, 18.0, dim, v(-1.0, -2.5))?);
    let cite = "Bridson and Müller-Fischer, Fluid Simulation, SIGGRAPH 2007 course notes, p. 23";
    strip.push(shown(k, stage, cite, 15.0, dim, v(-1.0, -2.85))?);

    // The neighbor rule's limit, and the control at C = 1: an exact copy in the same window.
    k.say(stage, "12_9")?;
    clear(stage, &mut strip);
    let mut limit = vec![
        shown(k, stage, "neighbor rule:", 26.0, ink, v(-3.0, 2.72))?,
        shown_tex(k, stage, r"0 \le C \le 1", 30.0, gold, v(-0.95, 2.72))?,
    ];
    // Half the trip, so the gold marker on the pulse's starting cell stays in the window: the
    // window rides with the pulse, so the bars hold still and the grid slides back one cell a step.
    let copies = TRAVEL / 2;
    let mut copy = Row::new(Rule::Neighbor, 1.0, 1, copies);
    let start = copy
        .values
        .iter()
        .position(|&q| q > 0.0)
        .ok_or_else(|| fail("12_9: the row holds no dye"))?;
    view = RowView::new(stage, k, FITTED_WINDOW)?;
    view.scale_shown = true;
    view.draw_scale(stage, k, false)?;
    view.draw(stage, &[(&copy, Look::Filled)])?;
    clear(stage, &mut readout);
    readout = courant_panel(stage, k, copy.courant)?;
    stats = readouts(stage, k, &copy, copies, Total::Readout)?;
    limit.push(shown(
        k,
        stage,
        "Gold arrow: the cell where the pulse began",
        19.0,
        gold,
        v(-1.92, -1.75),
    )?);
    let (mut began, mut exact) = (Vec::new(), true);
    for step in 0..=copies {
        if step > 0 {
            copy.advance();
            exact &= copy.max_error() == 0.0;
            view.draw(stage, &[(&copy, Look::Filled)])?;
            clear(stage, &mut stats);
            stats = readouts(stage, k, &copy, copies, Total::Readout)?;
        }
        clear(stage, &mut began);
        let x = view.x(copy.riding_bounds(start)[0]);
        began.push(arrow(
            stage,
            [x, PLOT_BOTTOM - 0.55],
            [x, PLOT_BOTTOM - 0.06],
            gold,
            1.4,
        )?);
        pause(stage, pacing::COPY_STEP)?;
    }
    limit.extend(began);
    let (words, color) = if exact {
        ("Every step moves the pulse exactly one cell", Palette::green())
    } else {
        ("The copy drifted from the exact pulse", Palette::red())
    };
    limit.push(shown(k, stage, words, 22.0, color, v(-1.92, -2.2))?);

    // Keep only the C = 1 pulse, pan back while static, then the blend at C = 3/2.
    k.say(stage, "12_10")?;
    clear(stage, &mut limit);
    clear(stage, &mut stats);
    view.scale_shown = false;
    view.draw_scale(stage, k, false)?;
    view.pan(stage, &[(&copy, Look::Filled)], CENTRED_WINDOW)?;
    let big = 1.5;
    let mut long_steps = Row::trip(Rule::Blend, big, 1);
    view.draw(stage, &[(&long_steps, Look::Filled)])?;
    clear(stage, &mut readout);
    readout = courant_panel(stage, k, big)?;
    let from = PULSE_WIDTH as f64 + 0.5;
    let top = PLOT_BOTTOM + PLOT_HEIGHT + 0.2;
    let look_back = format!("look back {:.1} cells", long_steps.courant);
    k.until_sentence(stage, "12_10", 1)?;
    let mut look = vec![
        arrow(
            stage,
            [view.x(from), top],
            [view.x(from - long_steps.courant), top],
            gold,
            1.4,
        )?,
        shown_left(k, stage, &look_back, 22.0, gold, v(view.x(from) + 0.15, top))?,
    ];
    let long_trip = trip_steps(big, 1);
    stats = readouts(stage, k, &long_steps, long_trip, Total::Readout)?;
    let mut bounded = true;
    for _ in 0..long_trip {
        long_steps.advance();
        let m = long_steps.measure();
        bounded &= m.lowest >= 0.0 && m.peak <= 1.0 + 1e-12;
        view.draw(stage, &[(&long_steps, Look::Filled)])?;
        clear(stage, &mut stats);
        stats = readouts(stage, k, &long_steps, long_trip, Total::Readout)?;
        pause(stage, pacing::BLEND_STEP)?;
    }
    let (words, color) = if bounded {
        ("Every step stays between 0 and 1", Palette::green())
    } else {
        ("A step left 0 to 1", Palette::red())
    };
    k.until_sentence(stage, "12_10", 2)?;
    look.push(shown(k, stage, words, 22.0, color, v(-1.92, -2.2))?);

    k.say(stage, "12_11")?;
    view.remove_all(stage);
    for group in [&mut look, &mut stats, &mut readout] {
        clear(stage, group);
    }
    let mut short_steps = Row::trip(Rule::Blend, half, 1);
    for _ in 0..trip {
        short_steps.advance();
    }
    let mut minis = mini_row(stage, k, &short_steps, -3.7)?;
    minis.extend(mini_row(stage, k, &long_steps, 3.7)?);
    let same = format!("Same trip: {TRAVEL} cells");
    minis.push(shown(k, stage, &same, 22.0, dim, v(0.0, 2.35))?);

    k.say(stage, "12_12")?;
    clear(stage, &mut minis);
    let mut chart = spread_chart(stage, k)?;
    note = precisely(
        k,
        stage,
        &[
            "Per step the blend adds a(1 − a) square cells, where a is the fractional part of C.",
            "Per unit time that is D/(u∆x) = a(1 − a)/(2C), which is not monotone in the step.",
        ],
    )?;

    k.say(stage, "12_13")?;
    clear(stage, &mut note);
    chart.push(panel(
        stage,
        [-1.85, -0.05],
        [8.6, 5.4],
        (background(), 0.72),
        (background(), 0.0),
    )?);
    chart.push(panel(
        stage,
        [-1.85, 0.1],
        [5.6, 1.1],
        (background(), 0.0),
        (gold, 2.6),
    )?);
    chart.push(shown(k, stage, "uniform wind only", 40.0, gold, v(-1.85, 0.1))?);

    k.say(stage, "12_14")?;
    clear(stage, &mut chart);
    let mut claims = vec![panel(stage, [0.0, 0.3], [13.6, 1.5], (card(), 1.0), (dim, 1.4))?];
    for (words, x) in [
        ("stable at any step", -4.6),
        ("accurate at any step", 0.0),
        ("a safe whole solver", 4.6),
    ] {
        claims.push(shown(k, stage, words, 28.0, ink, v(x, 0.3))?);
    }
    for x in [-2.3, 2.3] {
        claims.push(shown_tex(k, stage, r"\neq", 40.0, gold, v(x, 0.3))?);
    }
    Ok(claims)
}

/// 12_15 to 12_20: the time step's own error, on the exact channel with no cells.
fn time_error(stage: &mut Stage<'_>, k: &Kit, mut leftover: Vec<Mob>) -> fmn::Result<Vec<Mob>> {
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());
    let exact = channel_exact(CHANNEL_TIME);

    k.say(stage, "12_15")?;
    clear(stage, &mut leftover);
    let mut scene = channel(stage, k)?;
    k.until_sentence(stage, "12_15", 2)?;
    let (count, per) = ticks(stage, pacing::CHANNEL_RUN);
    let mut moving = Vec::new();
    for n in 0..=count {
        clear(stage, &mut moving);
        let t = CHANNEL_TIME * n as f64 / count as f64;
        moving.push(disc(stage, channel_point(channel_exact(t), 0.0), 0.09, gold)?);
        let clock = if n == count {
            format!("t = {} s", fraction(CHANNEL_TIME))
        } else {
            format!("t = {t:.2} s")
        };
        moving.push(shown(k, stage, &clock, 26.0, ink, v(-4.85, -2.85))?);
        wait_frames(stage, per)?;
    }
    scene.extend(moving);

    // Forward Euler with steps of 1/8 s: four hollow markers, and the miss in the inset.
    k.say(stage, "12_16")?;
    let end = channel_point(exact, 0.0);
    scene.push(shown(
        k,
        stage,
        &format!("{exact:.2} m"),
        22.0,
        gold,
        v(end[0] - 0.2, end[1] + 0.48),
    )?);
    k.until_sentence(stage, "12_16", 1)?;
    let mut zoom = inset(stage, k, exact)?;
    let coarse = LADDER_STEPS[2];
    let path = channel_path(TimeRule::ForwardEuler, coarse);
    zoom.extend(markers(stage, &path, 0.075, pacing::MARKER)?);
    k.until_sentence(stage, "12_16", 2)?;
    let legend = format!("steps of {} s", fraction(CHANNEL_TIME / coarse as f64));
    let mut legend = vec![shown(k, stage, &legend, 21.0, gold, v(INSET_CENTRE[0], 2.2))?];
    let (low, cy) = (inset_x(path[coarse]), INSET_CENTRE[1]);
    zoom.extend(bracket(stage, [low, inset_x(exact)], cy - 0.27, 1.0, gold)?);
    let short = format!("{} short", centimetres(channel_miss(TimeRule::ForwardEuler, coarse)));
    zoom.push(shown(
        k,
        stage,
        &short,
        21.0,
        gold,
        v((low + inset_x(exact)) / 2.0, cy - 0.44),
    )?);

    k.say(stage, "12_17")?;
    predict(
        stage,
        k,
        "Halve the step: what happens to the miss?",
        "Halves      A quarter      Same",
    )?;

    k.say(stage, "12_18")?;
    let fine = LADDER_STEPS[3];
    clear(stage, &mut legend);
    let both = format!(
        "steps of {} s and {} s",
        fraction(CHANNEL_TIME / coarse as f64),
        fraction(CHANNEL_TIME / fine as f64)
    );
    legend.push(shown(k, stage, &both, 21.0, gold, v(INSET_CENTRE[0], 2.2))?);
    let path = channel_path(TimeRule::ForwardEuler, fine);
    zoom.extend(markers(stage, &path, 0.055, pacing::MARKER / 2.0)?);
    let low = inset_x(path[fine]);
    zoom.extend(bracket(stage, [low, inset_x(exact)], cy + 0.27, -1.0, gold)?);
    let short = format!("{} short", centimetres(channel_miss(TimeRule::ForwardEuler, fine)));
    zoom.push(shown(
        k,
        stage,
        &short,
        21.0,
        gold,
        v((low + inset_x(exact)) / 2.0, cy + 0.44),
    )?);
    k.until_sentence(stage, "12_18", 1)?;
    let mut ladder = vec![
        shown(k, stage, "step", 21.0, dim, v(1.55, 2.25))?,
        shown(k, stage, "forward Euler", 21.0, dim, v(3.6, 2.25))?,
        shown(k, stage, "midpoint", 21.0, dim, v(5.75, 2.25))?,
    ];
    for (row, &steps) in LADDER_STEPS.iter().enumerate() {
        let color = if steps == coarse || steps == fine { gold } else { ink };
        let step = format!("{} s", fraction(CHANNEL_TIME / steps as f64));
        ladder.push(shown(k, stage, &step, 23.0, ink, v(1.55, ladder_y(row)))?);
        let miss = centimetres(channel_miss(TimeRule::ForwardEuler, steps));
        ladder.push(shown(k, stage, &miss, 23.0, color, v(3.6, ladder_y(row)))?);
        pause(stage, pacing::MARKER)?;
    }
    k.until_sentence(stage, "12_18", 2)?;
    ladder.push(shown(k, stage, "first order", 22.0, gold, v(3.6, -1.15))?);

    k.say(stage, "12_19")?;
    let row_of = |steps: usize| {
        LADDER_STEPS
            .iter()
            .position(|&s| s == steps)
            .ok_or_else(|| fail("12_19: a step missing from the ladder"))
    };
    let midpoint = |steps: usize| millimetres(channel_miss(TimeRule::Midpoint, steps));
    k.until_sentence(stage, "12_19", 1)?;
    let at = v(5.75, ladder_y(row_of(coarse)?));
    ladder.push(shown(k, stage, &midpoint(coarse), 23.0, gold, at)?);
    k.until_sentence(stage, "12_19", 2)?;
    let at = v(5.75, ladder_y(row_of(fine)?));
    ladder.push(shown(k, stage, &midpoint(fine), 23.0, gold, at)?);
    k.until_sentence(stage, "12_19", 3)?;
    ladder.push(shown(k, stage, "second order", 22.0, gold, v(5.75, -1.15))?);
    let ratio = |rule| channel_miss(rule, coarse) / channel_miss(rule, fine);
    let measured = format!(
        "Measured on halving from {} s to {} s: forward Euler's miss falls by {}, the midpoint rule's by {}.",
        fraction(CHANNEL_TIME / coarse as f64),
        fraction(CHANNEL_TIME / fine as f64),
        two(ratio(TimeRule::ForwardEuler)),
        two(ratio(TimeRule::Midpoint))
    );
    let mut note = precisely(k, stage, &[measured.as_str()])?;

    // The time mesh joins the cells as a second knob.
    k.say(stage, "12_20")?;
    for group in [&mut scene, &mut zoom, &mut legend, &mut ladder, &mut note] {
        clear(stage, group);
    }
    let joint: Vec<String> = [1, 2, 4]
        .iter()
        .map(|&finer| {
            let width = fraction(1.0 / finer as f64);
            format!("cells {width}  ·  {} steps", trip_steps(CHAPTER11_COURANT, finer))
        })
        .collect();
    k.until_sentence(stage, "12_20", 1)?;
    let mut dials = dial(stage, k, [0.0, 0.7], "cells and step together", &joint, None)?;
    k.until_sentence(stage, "12_20", 2)?;
    clear(stage, &mut dials);
    let rungs = |mesh: &str| ["", " / 2", " / 4"].map(|part| format!("∆{mesh}{part}")).to_vec();
    let tag = Some("step fixed, inside every rung's limit");
    dials = dial(stage, k, [-3.3, 0.7], "cells", &rungs("x"), tag)?;
    dials.extend(dial(stage, k, [3.3, 0.7], "step", &rungs("t"), None)?);
    Ok(dials)
}

/// 12_21 to 12_25: viscosity's own step limit, on the stack of layers.
fn diffusion(stage: &mut Stage<'_>, k: &Kit, mut leftover: Vec<Mob>) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());
    let zig = zigzag(8);
    let mean = zig.iter().sum::<f64>() / zig.len() as f64;

    k.say(stage, "12_21")?;
    clear(stage, &mut leftover);
    k.until_sentence(stage, "12_21", 1)?;
    let mut frame = speed_axis(stage, k, zig.len())?;
    let mut layers = stack(stage, k, &zig, false)?;
    k.until_sentence(stage, "12_21", 2)?;
    clear(stage, &mut layers);
    layers = stack(stage, k, &zig, true)?;
    k.until_sentence(stage, "12_21", 3)?;
    frame.extend(settle_line(stage, k, zig.len(), mean)?);
    let kept = format!("Viscosity keeps the stack's mean, {}.", plain(mean));
    let mut note = precisely(
        k,
        stage,
        &[
            "The stack repeats, top joined to bottom, so every layer has two neighbors.",
            kept.as_str(),
        ],
    )?;

    // Explicit steps at r = ¼, ½ (twice: the swap) and 1, each one computed state.
    k.say(stage, "12_22")?;
    clear(stage, &mut note);
    let quarter = 0.25;
    let after = explicit_layers(&zig, quarter);
    let mut side = step_panel(stage, k, quarter, RULER_PER_R * quarter, &landing(&after, mean))?;
    layer_step(stage, k, &mut layers, &zig, &after)?;
    k.until_sentence(stage, "12_22", 1)?;
    let limit = 0.5;
    let swapped = explicit_layers(&zig, limit);
    clear(stage, &mut side);
    side = step_panel(stage, k, limit, RULER_PER_R * limit, &landing(&swapped, mean))?;
    layer_step(stage, k, &mut layers, &zig, &swapped)?;
    k.until_sentence(stage, "12_22", 2)?;
    let back = explicit_layers(&swapped, limit);
    clear(stage, &mut layers);
    layers = stack(stage, k, &back, false)?;
    pause(stage, pacing::LAYER_HOLD)?;
    let forth = explicit_layers(&back, limit);
    clear(stage, &mut layers);
    layers = stack(stage, k, &forth, false)?;
    k.until_sentence(stage, "12_22", 3)?;
    let double = 2.0 * limit;
    let jumped = explicit_layers(&zig, double);
    clear(stage, &mut side);
    side = step_panel(stage, k, double, RULER_PER_R * double, &landing(&jumped, mean))?;
    layer_step(stage, k, &mut layers, &zig, &jumped)?;
    note = precisely(
        k,
        stage,
        &[
            "With r = ν∆t/h², an explicit step multiplies the zig-zag by 1 − 4r.",
            "Past r = ¼ every layer overshoots 2; it stays bounded only for r ≤ ½.",
        ],
    )?;

    // Halve the spacing: the largest safe step shrinks to a quarter.
    k.say(stage, "12_23")?;
    clear(stage, &mut note);
    k.until_sentence(stage, "12_23", 1)?;
    for group in [&mut layers, &mut frame, &mut side] {
        clear(stage, group);
    }
    let finer = zigzag(2 * zig.len());
    frame = stack_frame(stage, k, finer.len(), mean)?;
    layers = stack(stage, k, &finer, true)?;
    let (coarse_safe, fine_safe) = (
        largest_safe_step(zig.len(), 1.0, 1.0),
        largest_safe_step(finer.len(), 0.5, 1.0),
    );
    let ratio = fine_safe / coarse_safe;
    let (count, per) = ticks(stage, pacing::PAN);
    let mut ruler = Vec::new();
    for n in 0..=count {
        clear(stage, &mut ruler);
        let length = RULER_PER_R * limit * mix(1.0, ratio, ease(n as f64 / count as f64));
        let (left, right, y) = (STATS_X - length / 2.0, STATS_X + length / 2.0, 1.2);
        ruler.push(line(stage, v(left, y), v(right, y), ink, 2.0)?);
        ruler.push(line(stage, v(left, y - 0.1), v(left, y + 0.1), ink, 2.0)?);
        ruler.push(line(stage, v(right, y - 0.1), v(right, y + 0.1), ink, 2.0)?);
        wait_frames(stage, per)?;
    }
    side = ruler;
    side.push(shown(k, stage, "largest safe step", 18.0, dim, v(STATS_X, 0.92))?);
    let shrink = format!("{} as long, at half the spacing", fraction(ratio));
    side.push(shown(k, stage, &shrink, 22.0, ink, v(STATS_X, 0.45))?);
    k.until_sentence(stage, "12_23", 2)?;
    side.extend(r_table(stage, k)?);

    // The implicit step: all the layers solved together.
    k.say(stage, "12_24")?;
    for group in [&mut layers, &mut frame, &mut side] {
        clear(stage, group);
    }
    let r = double;
    frame = stack_frame(stage, k, zig.len(), mean)?;
    layers = stack(stage, k, &zig, false)?;
    side = matrix_picture(stage, k, &implicit_matrix(zig.len(), r))?;
    side.push(shown(
        k,
        stage,
        &format!("r = {}", fraction(r)),
        24.0,
        Palette::gold(),
        v(STATS_X, 2.4),
    )?);
    k.until_sentence(stage, "12_24", 1)?;
    let implicit = implicit_layers(&zig, r);
    clear(stage, &mut layers);
    layers = stack(stage, k, &implicit, false)?;

    // What is left of the zig-zag after one step at r = 1, three ways.
    k.say(stage, "12_25")?;
    clear(stage, &mut side);
    let explicit_left = zigzag_left(&zig, &explicit_layers(&zig, r));
    let implicit_left = zigzag_left(&zig, &implicit);
    let exact_left = zigzag_left(&zig, &exact_layers(&zig, r));
    side = vec![
        shown(k, stage, "zig-zag left after one step", 22.0, ink, v(STATS_X, 2.0))?,
        shown(
            k,
            stage,
            &format!("r = {}", fraction(r)),
            22.0,
            Palette::gold(),
            v(STATS_X, 1.55),
        )?,
        shown(
            k,
            stage,
            &format!("explicit  {}", plain(explicit_left)),
            26.0,
            Palette::red(),
            v(STATS_X - 0.2, 0.9),
        )?,
        shown(
            k,
            stage,
            &format!("implicit  {}", two(implicit_left)),
            26.0,
            ink,
            v(STATS_X, 0.15),
        )?,
    ];
    side.extend(verdict(stage, [STATS_X + 1.3, 0.9], false, Palette::red())?);
    // A plain readout: exact for this eight-layer model only, never the dotted exact-answer role.
    k.until_sentence(stage, "12_25", 1)?;
    side.push(shown(
        k,
        stage,
        &format!("exact in time  {exact_left:.3}"),
        26.0,
        ink,
        v(STATS_X, -0.6),
    )?);
    let first = format!(
        "For these eight layers at r = 1, the implicit step multiplies the zig-zag by {}.",
        fraction(implicit_left)
    );
    let second = format!(
        "Solving the same eight layers exactly in time multiplies it by e⁻⁴ ≈ {exact_left:.3}, about {} times less.",
        word(implicit_left / exact_left)
    );
    note = precisely(k, stage, &[first.as_str(), second.as_str()])?;
    frame.extend(layers);
    frame.extend(side);
    frame.extend(note);
    Ok(frame)
}

/// 12_26 and 12_27: refine each knob on its own against a declared tolerance; the tunnel's test.
fn refine(stage: &mut Stage<'_>, k: &Kit, mut leftover: Vec<Mob>) -> fmn::Result<Vec<Mob>> {
    let (ink, dim) = (Palette::ink(), Palette::dim());

    k.say(stage, "12_26")?;
    clear(stage, &mut leftover);
    let mut card_mobs = question_card(stage, k)?;
    let knobs = [("cells", -4.3), ("step", 0.0), ("pressure solve", 4.3)];
    k.until_sentence(stage, "12_26", 1)?;
    let mut dials = Vec::new();
    for (label, x) in knobs {
        dials.extend(dial(stage, k, [x, 0.0], label, &[], None)?);
    }
    k.until_sentence(stage, "12_26", 2)?;
    for (_, x) in knobs {
        dials.extend(tolerance_band(stage, k, [x, 0.0])?);
    }

    // The tunnel stays static: its readouts wait for validated runs (FUTURE-SIM F12a).
    k.say(stage, "12_27")?;
    clear(stage, &mut dials);
    card_mobs.push(panel(stage, [0.0, 0.25], [7.0, 2.0], (card(), 0.0), (ink, 2.0))?);
    card_mobs.push(ring(stage, [-1.9, 0.25], 0.38, ink, 2.0)?);
    k.until_sentence(stage, "12_27", 1)?;
    for (words, x) in [("half the cells:  ?", -2.6), ("half the step:  ?", 2.6)] {
        card_mobs.push(shown(k, stage, words, 26.0, ink, v(x, -1.35))?);
        card_mobs.push(panel(
            stage,
            [x, -1.85],
            [1.4, 0.14],
            (Palette::teal(), 0.85),
            (Palette::teal(), 0.0),
        )?);
        card_mobs.push(shown(k, stage, "declared tolerance", 16.0, dim, v(x, -2.13))?);
    }
    Ok(card_mobs)
}

impl SceneConstruct for Chapter12 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = &self.kit;
        shown(k, stage, "Time is also a mesh", 46.0, Palette::ink(), v(0.0, 3.34))?;
        let sections: [Section; 4] = [courant, time_error, diffusion, refine];
        let mut on_screen = Vec::new();
        for section in sections {
            on_screen = section(stage, k, on_screen)?;
        }
        k.finish(stage)
    }
}
