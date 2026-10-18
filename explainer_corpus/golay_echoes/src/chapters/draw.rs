//! The video's look, shared by its chapters (storyboard, color roles):
//! pulses and rows of pulses, the score chart and its blocks, the think-time
//! countdown, and line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups.
//!
//! The score chart draws the model's arithmetic literally: a product of +1
//! is a green block that stacks above the line, a product of −1 a red block
//! that lands on the outermost green one (both vanish) or, with none left,
//! hangs below the line. [`settle`] is that rule as plain data, and its
//! tests check that every column the chapter builds stands exactly as high
//! as the model's score.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, signed_thousands, v};
use fmn::prelude::*;

use crate::facts::SHIFTS;
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ----------------------------------------------------------------- geometry

/// One step along a row (and between two score columns).
pub const STEP: f64 = 0.75;
/// The echo's first pulse, and the score column of shift 0, are at this x.
pub const X0: f64 = -2.5;
/// The sent pattern's line (01_1): its up pulses stay inside the title-safe
/// area (±3.85).
pub const SEND_Y: f64 = 3.35;
/// The echo row's line.
pub const ECHO_Y: f64 = 2.55;
/// The copy row's line.
pub const COPY_Y: f64 = 1.35;
/// Where a shift's products appear, between the copy and the chart.
pub const STAGE_Y: f64 = 0.55;
/// The score chart's line.
pub const CHART_Y: f64 = -1.6;
/// The line of a second chart held over the first (01_8 to 01_10).
pub const HOVER_Y: f64 = 0.9;
/// The shift numbers under the chart.
pub const TICK_Y: f64 = -2.45;
/// The row and chart names, left of everything (centred here).
pub const LABEL_X: f64 = -6.2;
/// Where a longer name starts ("pattern A", "both rows added"): the
/// title-safe edge, 0.25 in from the frame's (−7.11), and a little more.
pub const NAME_X: f64 = -6.8;
/// One block's pitch in a column.
pub const BLOCK: f64 = 0.28;
const BLOCK_SIDE: f64 = 0.23;
/// A pulse's height above or below its line.
pub const PULSE_H: f64 = 0.38;
const PULSE_W: f64 = 0.14;

/// The x of the score column of shift `k` (on the main chart), which is also
/// where the copy's first pulse sits at that shift.
pub fn x_of(k: i64) -> f64 {
    X0 + STEP * k as f64
}

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// A horizontal line from `x0` to `x1` at `y`.
pub fn hline(x0: f64, x1: f64, y: f64, color: Srgb, width: f64) -> VMobject {
    polyline(vec![v(x0, y), v(x1, y)])
        .style(line_style(color, width))
        .build()
}

/// A score as the chapter writes it: "4", "1", "−1" (a typographic minus).
pub fn score_text(score: i64) -> String {
    signed_thousands(i128::from(score))
}

/// A product as the key writes it: "+1", "−1".
pub fn product_text(product: i64) -> String {
    if product > 0 {
        format!("+{product}")
    } else {
        score_text(product)
    }
}

// --------------------------------------------------------------- playback

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring detached mobjects in one after another, the whole taking `rt`.
pub fn fade_in_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Flash several mobjects (the highlight role), together.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move several mobjects together by `(dx, dy)` over `rt` seconds.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(v(dx, dy))?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade `old` out and the detached `new` in, in one move.
pub fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in old {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in new {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Wait `seconds` (a beat of stillness after a reveal).
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began and how long it runs, so a picture
/// can land on the phrase it illustrates. A phrase's moment is its share of
/// the line's characters (the voice reads at a nearly even rate); the
/// narrated draft's cue sheets check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
}

/// Speak line `id` now (after the previous line), as `Kit::say`.
pub fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Spoken> {
    kit.say(stage, id)?;
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?
        .to_owned();
    let words = text.split_whitespace().count() as f64;
    let len = kit
        .narrator
        .as_ref()
        .and_then(|n| n.length(id))
        .unwrap_or(words / SILENT_WPS + 0.4);
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
    })
}

impl Spoken {
    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let i = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let before = self.text[..i].chars().count() as f64;
        let all = self.text.chars().count().max(1) as f64;
        Ok(self.start + self.len * before / all)
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }
}

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown and "pause for more time").
pub const THINK_S: f64 = 4.0;
const THINK_R: f64 = 0.62;
pub const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause: let the question finish, mark the moment for the
/// score, bring in a large "?" at `at` inside a gold ring that drains over
/// [`THINK_S`] with "pause for more time" under it; at zero the ring and the
/// note go. Returns the "?", which the scene clears with the answer.
pub fn think(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 80.0, Palette::hero(), at)?;
    let note = kit.text(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0], at[1] - THINK_R - 0.3))?;
    // the ring: what is left of the think time, clockwise from the top
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + THINK_R * a.cos(), at[1] + THINK_R * a.sin())
            })
            .collect()
    };
    let left = stage.add_value_tracker(1.0);
    let ring = stage
        .arena_mut()
        .add(polyline(arc(1.0)).style(line_style(Palette::hero(), 4.0)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(left).unwrap_or(0.0);
            if last == Some(f) {
                return;
            }
            let _ = s.set_points(ring, polyline(arc(f)).build().points());
            last = Some(f);
        },
        true,
    )?;
    fade_in_all(stage, &[anchor, ring, q, note], 0.2)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.2)?;
    Ok(q)
}

// ------------------------------------------------------------------ pulses

/// One pulse: a bar standing up from its line for +1, hanging down for −1,
/// centred on `x`; `scale` shrinks it (the key's small pulses).
pub fn pulse(sign: i64, x: f64, line_y: f64, color: Srgb, scale: f64) -> fmn::Result<VMobject> {
    let h = PULSE_H * scale;
    let y = if sign > 0 { line_y + h / 2.0 } else { line_y - h / 2.0 };
    Ok(Rectangle::new()
        .width(PULSE_W * scale)
        .height(h)
        .corner_radius(0.04 * scale)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(v(x, y)))
}

/// A pattern drawn as a row: its line, and one pulse a step apart.
pub struct Row {
    pub line: Mob,
    pub pulses: Vec<Mob>,
}

impl Row {
    /// The line and the pulses, to move or fade together.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = vec![self.line];
        all.extend(&self.pulses);
        all
    }
}

/// The pattern `pattern` as a row with its first pulse at `first_x`, on the
/// line `line_y`. Detached.
pub fn row(stage: &mut Stage<'_>, pattern: &[i64], first_x: f64, line_y: f64, color: Srgb) -> fmn::Result<Row> {
    let last_x = first_x + STEP * pattern.len().saturating_sub(1) as f64;
    let line = stage
        .arena_mut()
        .add(hline(first_x - 0.35, last_x + 0.35, line_y, Palette::faint(), 2.0));
    let mut pulses = Vec::with_capacity(pattern.len());
    for (i, &sign) in pattern.iter().enumerate() {
        let p = pulse(sign, first_x + STEP * i as f64, line_y, color, 1.0)?;
        pulses.push(stage.arena_mut().add(p));
    }
    Ok(Row { line, pulses })
}

/// Recorded values drawn as a row: a bar `unit` tall per unit of value for
/// each nonzero value (up for positive, down for negative), a step apart
/// from `first_x`, on the line `line_y`. Detached; `pulses` holds the bars
/// of the nonzero values, in order.
pub fn value_row(
    stage: &mut Stage<'_>,
    values: &[i64],
    first_x: f64,
    line_y: f64,
    unit: f64,
    color: Srgb,
) -> fmn::Result<Row> {
    let last_x = first_x + STEP * values.len().saturating_sub(1) as f64;
    let line = stage
        .arena_mut()
        .add(hline(first_x - 0.35, last_x + 0.35, line_y, Palette::faint(), 2.0));
    let mut pulses = Vec::new();
    for (i, &value) in values.iter().enumerate() {
        if value == 0 {
            continue;
        }
        let h = unit * value.unsigned_abs() as f64;
        let y = if value > 0 { line_y + h / 2.0 } else { line_y - h / 2.0 };
        let bar = Rectangle::new()
            .width(PULSE_W)
            .height(h)
            .corner_radius(0.04_f64.min(h / 2.0))
            .style(flat(color, 1.0))
            .build()?
            .moved_to(v(first_x + STEP * i as f64, y));
        pulses.push(stage.arena_mut().add(bar));
    }
    Ok(Row { line, pulses })
}

/// A scene drawn as gold outlines on the line `line_y`: one per echo, at its
/// step's x and `unit` tall per unit of strength. Detached; `pulses` holds
/// the outlines in step order.
pub fn scene_row(stage: &mut Stage<'_>, scene: &[i64], line_y: f64, unit: f64) -> fmn::Result<Row> {
    let last = scene.len().saturating_sub(1) as i64;
    let line = stage
        .arena_mut()
        .add(hline(x_of(0) - 0.35, x_of(last) + 0.35, line_y, Palette::faint(), 2.0));
    let mut pulses = Vec::new();
    for (d, &strength) in scene.iter().enumerate() {
        if strength == 0 {
            continue;
        }
        let h = unit * strength.unsigned_abs() as f64;
        let y = if strength > 0 {
            line_y + h / 2.0
        } else {
            line_y - h / 2.0
        };
        let outline = Rectangle::new()
            .width(0.34)
            .height(h)
            .style(line_style(palette::echo(), 3.0))
            .build()?
            .moved_to(v(x_of(d as i64), y));
        pulses.push(stage.arena_mut().add(outline));
    }
    Ok(Row { line, pulses })
}

// ------------------------------------------------------------------ blocks

/// One block: green for a product of +1, red for −1, centred at `at`, its
/// side scaled by `scale`. Detached.
pub fn block(stage: &mut Stage<'_>, sign: i64, at: Vec3, scale: f64) -> fmn::Result<Mob> {
    slab(stage, sign, at, BLOCK_SIDE * scale, BLOCK_SIDE * scale)
}

/// A block for a product whose pair reaches across the middle of a doubled
/// pattern (chapter 03): a [`block`] with a ring in the across role.
pub fn block_across(stage: &mut Stage<'_>, sign: i64, at: Vec3) -> fmn::Result<Mob> {
    let color = if sign > 0 {
        palette::agree()
    } else {
        palette::disagree()
    };
    let b = Rectangle::new()
        .width(BLOCK_SIDE)
        .height(BLOCK_SIDE)
        .corner_radius(0.035)
        .style(Style::default().fill(color, 1.0).stroke(palette::across(), 3.0, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(b))
}

/// An arc over a row from `from` to `to` (two pulses' tops), rising
/// `height` at its middle: one lined-up pair. Detached-ready shape.
pub fn arc(from: Vec3, to: Vec3, height: f64, color: Srgb) -> VMobject {
    let points: Vec<Vec3> = (0..=16)
        .map(|i| {
            let t = f64::from(i) / 16.0;
            v(
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t + height * 4.0 * t * (1.0 - t),
            )
        })
        .collect();
    polyline(points).style(line_style(color, 2.5)).build()
}

/// One unit of score drawn `w` wide and `h` tall: green for +1, red for −1,
/// centred at `at` (a square [`block`], or a flat slab when a column must
/// hold dozens of units). Detached.
pub fn slab(stage: &mut Stage<'_>, sign: i64, at: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let color = if sign > 0 {
        palette::agree()
    } else {
        palette::disagree()
    };
    let b = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.035 * (h / BLOCK_SIDE))
        .style(flat(color, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(b))
}

/// Where one arriving block goes in a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing<T> {
    /// It stays: `level` blocks out from the line (1 is next to it), above
    /// the line for `sign` +1, below for −1.
    Stack { level: usize, sign: i64 },
    /// It lands on `partner`, the column's outermost block of the other
    /// sign, standing `level` out on the `sign` side, and both vanish.
    Cancel { level: usize, sign: i64, partner: T },
}

/// Settle blocks arriving at a column. `column` holds the column's blocks
/// from the line outward (all of one sign); the arrivals come +1 first, then
/// −1, each stacking on its own side or cancelling the outermost block of
/// the other sign. Afterwards the column's height (its signs summed) is the
/// old height plus the arrivals'.
pub fn settle<T: Copy>(column: &mut Vec<(T, i64)>, arriving: &[(T, i64)]) -> Vec<(T, Landing<T>)> {
    let mut out = Vec::with_capacity(arriving.len());
    for want in [1, -1] {
        for &(t, s) in arriving.iter().filter(|(_, s)| s.signum() == want) {
            match column.last().copied() {
                Some((partner, sign)) if sign == -s => {
                    let level = column.len();
                    column.pop();
                    out.push((t, Landing::Cancel { level, sign, partner }));
                }
                _ => {
                    column.push((t, s));
                    out.push((
                        t,
                        Landing::Stack {
                            level: column.len(),
                            sign: s,
                        },
                    ));
                }
            }
        }
    }
    out
}

/// A slab's pitch over a block's (chapter 02's columns hold up to 32 units).
pub const SLAB_RISE: f64 = 0.27;
/// A slab's width: most of a step, so a column reads as one bar of units.
pub const SLAB_W: f64 = 0.5;

/// Where a chart stands: the point of shift 0 on its line, its size across
/// (`scale`) and up (`rise`: a unit's pitch over [`BLOCK`]), how wide one
/// unit is drawn, and the shifts it draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub origin: Vec3,
    pub scale: f64,
    pub rise: f64,
    pub unit_w: f64,
    pub shifts: (i64, i64),
}

impl Place {
    /// Chapter 01's square blocks, `scale` both ways, over its shifts.
    pub fn blocks(origin: Vec3, scale: f64) -> Self {
        Self {
            origin,
            scale,
            rise: scale,
            unit_w: BLOCK_SIDE * scale,
            shifts: SHIFTS,
        }
    }

    /// The main chart, under the rows.
    pub fn main() -> Self {
        Self::blocks(v(X0, CHART_Y), 1.0)
    }

    /// A chart held over the main one, column above column.
    pub fn hover() -> Self {
        Self::blocks(v(X0, HOVER_Y), 1.0)
    }

    /// A small chart set aside at the upper right.
    pub fn shelf() -> Self {
        Self::blocks(v(4.9, 2.55), 0.45)
    }

    /// A chart of flat slabs on the line `y`, over `shifts`, its columns
    /// under the same steps as the rows.
    pub fn slabs(y: f64, shifts: (i64, i64)) -> Self {
        Self {
            origin: v(X0, y),
            scale: 1.0,
            rise: SLAB_RISE,
            unit_w: SLAB_W,
            shifts,
        }
    }

    pub fn column_x(&self, k: i64) -> f64 {
        self.origin[0] + STEP * self.scale * k as f64
    }

    /// The centre of the unit `level` out from the line on the `sign` side.
    pub fn slot(&self, k: i64, level: usize, sign: i64) -> Vec3 {
        let dy = (level as f64 - 0.5) * BLOCK * self.rise;
        v(self.column_x(k), self.origin[1] + if sign >= 0 { dy } else { -dy })
    }

    /// Just past a column of `height` units (above it, or below a column
    /// hanging down), `gap` further out: where its label goes.
    pub fn past(&self, k: i64, height: i64, gap: f64) -> Vec3 {
        let d = height.unsigned_abs() as f64 * BLOCK * self.rise + gap * self.scale;
        v(self.column_x(k), self.origin[1] + if height >= 0 { d } else { -d })
    }

    /// The middle of the chart's line over the drawn shifts.
    pub fn line_centre(&self) -> Vec3 {
        v(
            (self.column_x(self.shifts.0) + self.column_x(self.shifts.1)) / 2.0,
            self.origin[1],
        )
    }

    /// One detached unit of this chart's size at `at`.
    pub fn unit(&self, stage: &mut Stage<'_>, sign: i64, at: Vec3) -> fmn::Result<Mob> {
        slab(stage, sign, at, self.unit_w, BLOCK_SIDE * self.rise)
    }
}

/// A score chart: a line over the drawn shifts, and a column of units at
/// each shift.
pub struct Chart {
    pub place: Place,
    pub line: Mob,
    /// Per shift from `place.shifts.0`: the units from the line out, with
    /// signs.
    columns: Vec<Vec<(Mob, i64)>>,
}

impl Chart {
    /// An empty chart at `place`; its line is detached.
    pub fn new(stage: &mut Stage<'_>, place: Place) -> Self {
        let (first, last) = place.shifts;
        let half = (last - first) as f64 * STEP * place.scale / 2.0 + 0.45 * place.scale;
        let c = place.line_centre();
        let line = stage
            .arena_mut()
            .add(hline(c[0] - half, c[0] + half, c[1], Palette::dim(), 2.0));
        let columns = (first..=last).map(|_| Vec::new()).collect();
        Self { place, line, columns }
    }

    /// The shifts the chart draws, in order.
    fn shifts(&self) -> std::ops::RangeInclusive<i64> {
        self.place.shifts.0..=self.place.shifts.1
    }

    fn index(&self, k: i64) -> fmn::Result<usize> {
        let (first, last) = self.place.shifts;
        usize::try_from(k - first)
            .ok()
            .filter(|_| k <= last)
            .ok_or_else(|| fail(format!("shift {k} is outside the chart {:?}", self.place.shifts)))
    }

    /// A row that scores anything outside the drawn shifts cannot be drawn
    /// whole: refuse it rather than drop a column.
    fn fits(&self, scores: &crate::model::Scores, what: &str) -> fmn::Result<()> {
        let outside = (scores.first..=scores.last()).find(|&k| scores.at(k) != 0 && self.index(k).is_err());
        match outside {
            Some(k) => Err(fail(format!(
                "{what}: scores {} at shift {k}, outside the chart {:?}",
                scores.at(k),
                self.place.shifts
            ))),
            None => Ok(()),
        }
    }

    /// The column's height: its blocks' signs summed (the score it shows).
    pub fn height(&self, k: i64) -> i64 {
        self.index(k)
            .map_or(0, |i| self.columns[i].iter().map(|&(_, s)| s).sum())
    }

    /// The blocks of the column at `k`.
    pub fn column(&self, k: i64) -> Vec<Mob> {
        self.index(k)
            .map(|i| self.columns[i].iter().map(|&(m, _)| m).collect())
            .unwrap_or_default()
    }

    /// Take the blocks out of the chart (for another chart to settle); the
    /// line stays.
    pub fn take(&mut self) -> Vec<(i64, Vec<(Mob, i64)>)> {
        self.shifts()
            .zip(self.columns.iter_mut())
            .map(|(k, c)| (k, std::mem::take(c)))
            .filter(|(_, c)| !c.is_empty())
            .collect()
    }

    /// Settle arriving blocks into their columns, all at once: each moves
    /// to its slot over `rt` seconds, then every cancelled pair vanishes
    /// over `rt_cancel`. The arrivals must already be the chart's size.
    pub fn settle_all(
        &mut self,
        stage: &mut Stage<'_>,
        arrivals: &[(i64, Vec<(Mob, i64)>)],
        rt: f64,
        rt_cancel: f64,
    ) -> fmn::Result<()> {
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        let mut gone = Vec::new();
        for (k, arriving) in arrivals {
            let i = self.index(*k)?;
            for (m, landing) in settle(&mut self.columns[i], arriving) {
                let to = match landing {
                    Landing::Stack { level, sign } => self.place.slot(*k, level, sign),
                    Landing::Cancel { level, sign, partner } => {
                        gone.extend([m, partner]);
                        self.place.slot(*k, level, sign)
                    }
                };
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt),
                        ..AnimateArgs::default()
                    })?
                    .move_to(to, ORIGIN)?;
                moves.push(stage.prepare(a)?);
            }
        }
        stage.play_prepared(moves)?;
        if !gone.is_empty() {
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for m in gone {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.35)?;
                anims.push(stage.prepare(a.rt(rt_cancel))?);
            }
            stage.play_prepared(anims)?;
        }
        Ok(())
    }

    /// Move the chart (line and units) to `place` over `rt` seconds: the same
    /// shifts, scaled alike across and up (so a unit keeps its shape).
    pub fn move_to(&mut self, stage: &mut Stage<'_>, place: Place, rt: f64) -> fmn::Result<()> {
        let rel = place.scale / self.place.scale;
        if place.shifts != self.place.shifts || (place.rise / self.place.rise - rel).abs() > 1e-9 {
            return Err(fail(format!(
                "a chart over {:?} cannot move to {place:?}",
                self.place.shifts
            )));
        }
        let args = || AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        };
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = self
            .line
            .animate()
            .set_anim_args(args())?
            .scale(rel)?
            .move_to(place.line_centre(), ORIGIN)?;
        anims.push(stage.prepare(a)?);
        for (k, column) in self.shifts().zip(&self.columns) {
            for (level, &(m, sign)) in column.iter().enumerate() {
                let a = m
                    .animate()
                    .set_anim_args(args())?
                    .scale(rel)?
                    .move_to(place.slot(k, level + 1, sign), ORIGIN)?;
                anims.push(stage.prepare(a)?);
            }
        }
        stage.play_prepared(anims)?;
        self.place = place;
        Ok(())
    }

    /// Move every column `by` shifts along (its blocks slide to the new
    /// slots over `rt`); a column may not leave the drawn shifts.
    pub fn shift_columns(&mut self, stage: &mut Stage<'_>, by: i64, rt: f64) -> fmn::Result<()> {
        let mut moved: Vec<Vec<(Mob, i64)>> = self.columns.iter().map(|_| Vec::new()).collect();
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let shifts = self.shifts();
        for (k, column) in shifts.zip(std::mem::take(&mut self.columns)) {
            if column.is_empty() {
                continue;
            }
            let i = self.index(k + by)?;
            for (level, &(m, sign)) in column.iter().enumerate() {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt),
                        ..AnimateArgs::default()
                    })?
                    .move_to(self.place.slot(k + by, level + 1, sign), ORIGIN)?;
                anims.push(stage.prepare(a)?);
            }
            moved[i] = column;
        }
        self.columns = moved;
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// The chart stands exactly as the model scores (a render-time check of
    /// the picture against the arithmetic).
    pub fn check(&self, want: &crate::model::Scores, what: &str) -> fmn::Result<()> {
        self.fits(want, what)?;
        for k in self.shifts() {
            if self.height(k) != want.at(k) {
                return Err(fail(format!(
                    "{what}: the column at shift {k} stands {} blocks, the model scores {}",
                    self.height(k),
                    want.at(k)
                )));
            }
        }
        Ok(())
    }

    /// Stand a whole row of scores at once, unit by unit from the line
    /// (detached units, faded in over `rt`).
    pub fn stand(&mut self, stage: &mut Stage<'_>, scores: &crate::model::Scores, rt: f64) -> fmn::Result<()> {
        self.fits(scores, "a row to stand")?;
        let mut new = Vec::new();
        for k in self.shifts() {
            let i = self.index(k)?;
            let s = scores.at(k);
            let sign = s.signum();
            for level in 1..=s.unsigned_abs() as usize {
                let m = self.place.unit(stage, sign, self.place.slot(k, level, sign))?;
                self.columns[i].push((m, sign));
                new.push(m);
            }
        }
        fade_in_all(stage, &new, rt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts;
    use crate::model::{CODE_A, CODE_B, products};

    /// The heights of every column built by settling `arrivals` per shift.
    fn heights(columns: &[Vec<(usize, i64)>]) -> Vec<i64> {
        columns.iter().map(|c| c.iter().map(|&(_, s)| s).sum()).collect()
    }

    /// Score a pattern the way the chapter draws it: at each shift, the
    /// products arrive as blocks in an empty column.
    fn drawn_row(pattern: &[i64]) -> Vec<Vec<(usize, i64)>> {
        let mut id = 0;
        (SHIFTS.0..=SHIFTS.1)
            .map(|k| {
                let mut column = Vec::new();
                let arriving: Vec<(usize, i64)> = products(pattern, pattern, k)
                    .iter()
                    .map(|p| {
                        id += 1;
                        (id, p.value)
                    })
                    .collect();
                let landed = settle(&mut column, &arriving);
                assert_eq!(landed.len(), arriving.len(), "every block lands somewhere");
                column
            })
            .collect()
    }

    fn row_of(scores: &crate::model::Scores) -> Vec<i64> {
        (SHIFTS.0..=SHIFTS.1).map(|k| scores.at(k)).collect()
    }

    /// The picture is the arithmetic: every column the chapter builds stands
    /// as high as the model's score, for A, for B, for A dropped onto B, and
    /// for A dropped onto B moved one step.
    #[test]
    fn every_drawn_column_is_its_score() {
        let a = drawn_row(&CODE_A);
        let b = drawn_row(&CODE_B);
        assert_eq!(heights(&a), row_of(&facts::score_a()));
        assert_eq!(heights(&b), row_of(&facts::score_b()));
        // A onto B, column onto column
        let mut both = b.clone();
        for (column, arriving) in both.iter_mut().zip(&a) {
            settle(column, arriving);
        }
        assert_eq!(heights(&both), row_of(&facts::sum()));
        // A onto B moved one step right
        let mut moved: Vec<Vec<(usize, i64)>> = vec![Vec::new()];
        moved.extend(b.iter().take(b.len() - 1).cloned());
        for (column, arriving) in moved.iter_mut().zip(&a) {
            settle(column, arriving);
        }
        assert_eq!(heights(&moved), row_of(&facts::moved()));
    }

    /// A column only ever holds blocks of one sign, stacked without gaps.
    #[test]
    fn a_column_holds_one_sign_without_gaps() {
        let mut column = vec![(0, 1), (1, 1)];
        let landed = settle(&mut column, &[(2, -1), (3, 1), (4, -1), (5, -1), (6, -1)]);
        // +1 first: (3) stacks third; then four −1: three cancel, one hangs
        assert_eq!(landed[0], (3, Landing::Stack { level: 3, sign: 1 }));
        assert_eq!(
            landed[1],
            (
                2,
                Landing::Cancel {
                    level: 3,
                    sign: 1,
                    partner: 3
                }
            )
        );
        assert_eq!(landed[4], (6, Landing::Stack { level: 1, sign: -1 }));
        assert_eq!(column, [(6, -1)]);
    }

    #[test]
    fn places_put_slots_on_the_right_side() {
        let p = Place::main();
        assert_eq!(p.slot(0, 1, 1)[1], CHART_Y + BLOCK / 2.0);
        assert_eq!(p.slot(0, 1, -1)[1], CHART_Y - BLOCK / 2.0);
        assert_eq!(p.column_x(2), x_of(2));
        assert!(p.past(0, 4, 0.2)[1] > p.slot(0, 4, 1)[1]);
        assert!(p.past(3, -1, 0.2)[1] < p.slot(3, 1, -1)[1]);
        let s = Place::shelf();
        // the shelf clears the key and the rows (right of every row's end)
        assert!(s.column_x(SHIFTS.0) - 0.45 * s.scale > x_of(3) + STEP * 3.0);
    }

    #[test]
    fn numbers_print_with_a_typographic_minus() {
        assert_eq!(score_text(4), "4");
        assert_eq!(score_text(-1), "\u{2212}1");
        assert_eq!(product_text(1), "+1");
        assert_eq!(product_text(-1), "\u{2212}1");
    }
}
