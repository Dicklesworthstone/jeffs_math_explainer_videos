//! The logarithm film's shared look and props: facts for the scenes, colour
//! roles, fades, the decade scale, complex-plane panels, a matrix card, the
//! prediction hold and the oblique view used for spirals.
//!
//! Colour roles (storyboard §Roles), the same in every chapter:
//! - blue: positions, lengths along a ruler, exponents, logarithms
//! - green: numbers being multiplied, outputs, transformed grids
//! - hero (warm yellow): the answer the viewer is asked for
//! - purple: laps (decades on the dial, turns of the complex angle)
//! - teal: the sliding ruler and moving copies
//! - red: what is excluded or impossible (zero, the cut, the wrong guess)
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`. Engine rules (`explainer_kit::kit`): small redraws,
//! short strokes under motion, one flat fill per mobject.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled here.
#![allow(dead_code, reason = "shared props land before every chapter uses each")]

use std::f64::consts::TAU;
use std::sync::OnceLock;

use explainer_kit::{Facts, Kit, Palette, Timed, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::model::C;

// ------------------------------------------------------------------ facts

/// A scene error carrying `message`.
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// Every fact of the film, computed once.
pub fn facts() -> fmn::Result<&'static Facts> {
    static FACTS: OnceLock<Result<Facts, String>> = OnceLock::new();
    FACTS
        .get_or_init(crate::facts::facts)
        .as_ref()
        .map_err(|e| fail(format!("facts: {e}")))
}

/// One fact, as printed.
pub fn fact(key: &str) -> fmn::Result<&'static str> {
    facts()?.get(key).ok_or_else(|| fail(format!("no fact {key}")))
}

/// One fact that is a number.
pub fn fact_f64(key: &str) -> fmn::Result<f64> {
    fact(key)?
        .parse()
        .map_err(|_| fail(format!("fact {key} is not a number")))
}

/// One fact that is a comma-separated list of numbers.
pub fn fact_list(key: &str) -> fmn::Result<Vec<f64>> {
    fact(key)?
        .split(',')
        .map(|x| {
            x.parse::<f64>()
                .map_err(|_| fail(format!("fact {key} has a non-number {x:?}")))
        })
        .collect()
}

/// A 2×2 matrix fact written "a,b;c,d" (rows separated by `;`).
pub fn fact_matrix(key: &str) -> fmn::Result<crate::model::M2> {
    let rows: Vec<Vec<f64>> = fact(key)?
        .split(';')
        .map(|row| {
            row.split(',')
                .map(|x| x.trim().parse::<f64>())
                .collect::<Result<Vec<f64>, _>>()
        })
        .collect::<Result<_, _>>()
        .map_err(|_| fail(format!("fact {key} is not a matrix")))?;
    match rows.as_slice() {
        [a, b] if a.len() == 2 && b.len() == 2 => Ok([[a[0], a[1]], [b[0], b[1]]]),
        _ => Err(fail(format!("fact {key} is not 2×2"))),
    }
}

/// A number for the screen with a typographic minus.
pub fn shown(s: &str) -> String {
    s.replace('-', "\u{2212}")
}

// ------------------------------------------------------------------ roles

pub fn position_c() -> Srgb {
    Palette::blue()
}
pub fn number_c() -> Srgb {
    Palette::green()
}
pub fn answer_c() -> Srgb {
    Palette::hero()
}
pub fn lap_c() -> Srgb {
    Palette::purple()
}
pub fn slide_c() -> Srgb {
    Palette::teal()
}
pub fn warn_c() -> Srgb {
    Palette::red()
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

/// A stroke at an opacity, and no fill.
pub fn faded_line(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

// ------------------------------------------------------------- primitives

/// A tracker's value now.
pub fn value(s: &Arena, tracker: Mob) -> f64 {
    s.tracker_value(tracker).unwrap_or(0.0)
}

/// A filled dot.
pub fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(r).style(flat(c, 1.0)).build())
}

/// A ring (an unfilled circle).
pub fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(Circle::new().radius(r).arc_center(at).style(line_style(c, w)).build())
}

/// A straight stroke.
pub fn seg(stage: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(vec![a, b]).style(line_style(c, w)).build())
}

/// An open path through `pts`.
pub fn path(stage: &mut Stage<'_>, pts: Vec<Vec3>, c: Srgb, w: f64) -> Mob {
    stage.arena_mut().add(polyline(pts).style(line_style(c, w)).build())
}

/// A dashed straight line, as one group of short strokes.
pub fn dashed(stage: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64, dashes: usize) -> fmn::Result<Mob> {
    let mut parts = Vec::with_capacity(dashes);
    for k in 0..dashes {
        let (t0, t1) = (k as f64 / dashes as f64, (k as f64 + 0.55) / dashes as f64);
        let at = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
        parts.push(seg(stage, at(t0), at(t1), c, w));
    }
    group(stage, &parts)
}

/// A solid arrow.
pub fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> fmn::Result<Mob> {
    let m = Arrow::new(a, b).buff(0.0).style(line_style(c, w)).build()?;
    Ok(stage.arena_mut().add(m))
}

/// An arrow as a closure can rebuild it inside `always_redraw`: shaft and a
/// filled head, both short strokes.
pub fn arrow_parts(s: &mut Arena, a: Vec3, b: Vec3, c: Srgb, w: f64, head: f64) -> Mob {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let h = head.min(len * 0.5);
    let base = v(b[0] - h * ux, b[1] - h * uy);
    let shaft = s.add(polyline(vec![a, base]).style(line_style(c, w)).build());
    let tip = s.add(
        Polygon::new([
            b,
            v(base[0] - 0.5 * h * uy, base[1] + 0.5 * h * ux),
            v(base[0] + 0.5 * h * uy, base[1] - 0.5 * h * ux),
        ])
        .style(flat(c, 1.0))
        .build(),
    );
    let g = s.add(VMobject::new());
    let _ = s.attach(g, shaft);
    let _ = s.attach(g, tip);
    g
}

/// An arc of a circle as short strokes: centre, radius, from/to angles.
pub fn arc_points(centre: Vec3, r: f64, from: f64, to: f64, n: usize) -> Vec<Vec3> {
    (0..=n)
        .map(|k| {
            let t = from + (to - from) * k as f64 / n as f64;
            v(centre[0] + r * t.cos(), centre[1] + r * t.sin())
        })
        .collect()
}

/// An empty parent holding mobjects already in the arena.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

// -------------------------------------------------------------- playback

/// Bring several detached mobjects in together with a fade.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade several mobjects out together (live ones stop updating first, so
/// they fade instead of being redrawn at full opacity).
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// One picture gives way to the next in a single beat: fade `out` while
/// `inn` fades in, so the frame is never empty between them.
pub fn swap(stage: &mut Stage<'_>, out: &[Mob], inn: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in inn {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Detached mobjects in one after another.
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

/// Tween several trackers together.
pub fn tween_all(stage: &mut Stage<'_>, moves: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, to) in moves {
        anims.push(stage.prepare(tween(t, to, rt)?)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Shift several mobjects by the same vector together.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], by: Vec3, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(by)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        stage.wait(t - now)?;
    }
    Ok(())
}

// ------------------------------------------------------------ words

/// The chapter's title line.
pub fn title(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(stage, words, 40.0, Palette::ink(), v(0.0, 3.45))
}

/// A "precisely:" lower third in fine print: the exact version of a
/// simplification the narration makes (pedagogy rule).
pub fn precisely(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(
        stage,
        &format!("precisely: {words}"),
        22.0,
        Palette::dim(),
        v(0.0, -3.62),
    )
}

/// How long a prediction stays open after its question (authored hold;
/// a proposal until heard against the voice).
pub const THINK: f64 = 4.0;

/// Leave the viewer `THINK` seconds with the question: the current line
/// finishes, the music has the gap to itself, and a thin bar drains under
/// `at` while the picture holds still. Nothing is revealed before it ends.
pub fn predict(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, width: f64) -> fmn::Result<()> {
    let left = stage.add_value_tracker(1.0);
    let start = v(at[0] - width / 2.0, at[1]);
    let bar = stage.always_redraw(move |s| {
        let f = value(s, left).clamp(0.0, 1.0);
        let r = Rectangle::new()
            .width((width * f).max(1e-3))
            .height(0.07)
            .style(flat(Palette::hero(), 0.9))
            .build()
            .expect("positive extents");
        let m = s.add(r);
        s.move_to(m, start, LEFT);
        m
    });
    stage.add_to_scene(bar)?;
    // a bare probe in the scene drives the tracker by the clock, so the bar
    // drains during the music's own gap rather than in a played animation
    let probe = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(probe)?;
    let drained = stage.add_dt_updater(
        probe,
        move |s, _, dt| {
            let now = s.tracker_value(left).unwrap_or(0.0);
            let _ = s.set_tracker_value(left, now - dt / THINK);
        },
        false,
    );
    if drained.is_err() {
        return Err(fail("prediction bar: updater refused".into()));
    }
    kit.music_breath(stage, THINK)?;
    stage.clear_updaters(probe, true);
    fade_out_all(stage, &[bar, probe], 0.25)
}

// ------------------------------------------------------------ the ruler

/// A decade scale: 1 at `one` and 10 a `length` to its right; marks for
/// the whole numbers, halves up to five, and tenths between one and two.
#[derive(Clone, Copy, Debug)]
pub struct Decade {
    pub one: Vec3,
    pub length: f64,
    /// +1: ticks rise from the spine (labels above); −1: ticks hang below.
    pub side: f64,
}

impl Decade {
    pub fn x(&self, value: f64) -> f64 {
        self.one[0] + self.length * value.log10()
    }
    pub fn at(&self, value: f64) -> Vec3 {
        v(self.x(value), self.one[1])
    }

    /// The marks of the scale, the whole-number labels, and the spine, as
    /// one group (so the moving copy slides as one piece).
    pub fn build(&self, kit: &Kit, stage: &mut Stage<'_>, c: Srgb, label_size: f64) -> fmn::Result<Mob> {
        let mut parts = Vec::new();
        parts.push(seg(stage, self.at(1.0), self.at(10.0), c, 3.0));
        let tick = |k: f64| -> f64 { k * self.side };
        let mut marks: Vec<(f64, f64)> = (1..=10).map(|n| (f64::from(n), 0.32)).collect();
        marks.extend(
            (1..=9)
                .map(|n| (1.0 + f64::from(n) / 10.0, 0.12))
                .filter(|&(x, _)| (x - 1.5).abs() > 1e-9),
        );
        marks.extend((1..=4).map(|n| (f64::from(n) + 0.5, 0.2)));
        for (x, h) in marks {
            let p = self.at(x);
            parts.push(seg(stage, p, v(p[0], p[1] + tick(h)), c, 2.5));
        }
        for n in 1..=10 {
            let p = self.at(f64::from(n));
            let label = kit.text(stage, &n.to_string(), label_size, c, v(p[0], p[1] + tick(0.58)))?;
            parts.push(label);
        }
        group(stage, &parts)
    }
}

// ---------------------------------------------------------- planes

/// A complex-plane panel: `scale` scene units per unit, `centre` the
/// screen point of zero.
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub centre: Vec3,
    pub scale: f64,
    pub half_w: f64,
    pub half_h: f64,
}

impl Plane {
    pub fn p(&self, z: C) -> Vec3 {
        v(self.centre[0] + self.scale * z.re, self.centre[1] + self.scale * z.im)
    }
    /// Whether the point of `z` lies inside the panel's window.
    pub fn shows(&self, z: C) -> bool {
        (z.re * self.scale).abs() <= self.half_w && (z.im * self.scale).abs() <= self.half_h
    }
    /// The two axes, faint.
    pub fn axes(&self, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let (c, w, h) = (self.centre, self.half_w, self.half_h);
        let x = seg(stage, v(c[0] - w, c[1]), v(c[0] + w, c[1]), Palette::faint(), 2.5);
        let y = seg(stage, v(c[0], c[1] - h), v(c[0], c[1] + h), Palette::faint(), 2.5);
        group(stage, &[x, y])
    }
    /// The unit circle, dim.
    pub fn unit_circle(&self, stage: &mut Stage<'_>) -> Mob {
        ring(stage, self.centre, self.scale, Palette::dim(), 2.5)
    }
}

/// A plane's tick labels in units of π along its vertical axis, as "kπ".
pub fn pi_label(k: i64) -> String {
    match k {
        0 => "0".into(),
        1 => "π".into(),
        -1 => "\u{2212}π".into(),
        _ => format!("{}π", shown(&k.to_string())),
    }
}

// ---------------------------------------------------------- matrices

/// A 2×2 matrix card from primitives (TeX matrices are avoided at this
/// pin): four entries and two drawn brackets, centred at `at`.
pub fn matrix_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    entries: [[&str; 2]; 2],
    at: Vec3,
    size: f64,
    c: Srgb,
) -> fmn::Result<Mob> {
    // fmn text: one em is size / 144 scene units; an entry like "−0.5" is
    // about two em wide. Columns sit 3.2 em apart, rows 1.6 em.
    let (dx, dy) = (size * 0.011, size * 0.0055);
    let mut parts = Vec::new();
    for (i, row) in entries.iter().enumerate() {
        for (j, e) in row.iter().enumerate() {
            let x = at[0] + if j == 0 { -dx } else { dx };
            let y = at[1] + if i == 0 { dy } else { -dy };
            parts.push(kit.text(stage, &shown(e), size, c, v(x, y))?);
        }
    }
    let (w, h, lip) = (dx + size * 0.0095, dy + size * 0.0055, size * 0.003);
    for side in [-1.0, 1.0] {
        let x = at[0] + side * w;
        let pts = vec![
            v(x - side * lip, at[1] + h),
            v(x, at[1] + h),
            v(x, at[1] - h),
            v(x - side * lip, at[1] - h),
        ];
        parts.push(path(stage, pts, c, 2.5));
    }
    group(stage, &parts)
}

/// A grid of the plane under the matrix `m` (scene `centre`, `unit` scene
/// units per unit, lines at −n..n), drawn as short strokes for a redraw.
pub fn grid_under(s: &mut Arena, m: crate::model::M2, centre: Vec3, unit: f64, n: i32, c: Srgb, o: f64) -> Mob {
    let g = s.add(VMobject::new());
    let reach = f64::from(n);
    let samples = 8;
    for k in -n..=n {
        let fixed = f64::from(k);
        for vertical in [false, true] {
            let mut pts = Vec::with_capacity(2 * samples * n as usize + 1);
            for j in 0..=(2 * samples * n as usize) {
                let t = -reach + j as f64 / samples as f64;
                let p = if vertical { [fixed, t] } else { [t, fixed] };
                let q = crate::model::apply(m, p);
                pts.push(v(centre[0] + unit * q[0], centre[1] + unit * q[1]));
            }
            let w = if k == 0 { 3.0 } else { 1.8 };
            let line = s.add(polyline(pts).style(faded_line(c, w, o)).build());
            let _ = s.attach(g, line);
        }
    }
    g
}

/// As [`grid_under`], lines x = k and y = k for |k| ≤ n, but kept only
/// where the image lies within `clip` plane units of the centre, each line
/// split where it leaves that box: a flow that stretches far corners (an
/// ellipse flow mid-way, a spiral late on) cannot throw lines across the
/// text column.
#[allow(clippy::too_many_arguments)]
pub fn clipped_grid(
    s: &mut Arena,
    m: crate::model::M2,
    centre: Vec3,
    unit: f64,
    n: i32,
    clip: f64,
    c: Srgb,
    o: f64,
) -> Mob {
    let g = s.add(VMobject::new());
    let reach = f64::from(n);
    let samples = 12 * n.max(1) as usize;
    let at = |q: [f64; 2]| v(centre[0] + unit * q[0], centre[1] + unit * q[1]);
    for k in -n..=n {
        let w = if k == 0 { 3.0 } else { 1.8 };
        for vertical in [false, true] {
            let mut runs: Vec<Vec<Vec3>> = vec![Vec::new()];
            for j in 0..=(2 * samples) {
                let t = -reach + reach * j as f64 / samples as f64;
                let p = if vertical { [f64::from(k), t] } else { [t, f64::from(k)] };
                let q = crate::model::apply(m, p);
                if q[0].abs() <= clip && q[1].abs() <= clip {
                    if let Some(run) = runs.last_mut() {
                        run.push(at(q));
                    }
                } else if runs.last().is_some_and(|r| !r.is_empty()) {
                    runs.push(Vec::new());
                }
            }
            for run in runs.into_iter().filter(|r| r.len() >= 2) {
                let line = s.add(polyline(run).style(faded_line(c, w, o)).build());
                let _ = s.attach(g, line);
            }
        }
    }
    g
}

/// The letter F as a polygon of unit size about `p0` under `m`: a shape
/// with a handedness, so a mirror shows.
pub fn letter_f(m: crate::model::M2, centre: Vec3, unit: f64) -> Vec<Vec3> {
    let outline: [[f64; 2]; 10] = [
        [0.0, 0.0],
        [0.25, 0.0],
        [0.25, 0.55],
        [0.6, 0.55],
        [0.6, 0.78],
        [0.25, 0.78],
        [0.25, 1.05],
        [0.75, 1.05],
        [0.75, 1.3],
        [0.0, 1.3],
    ];
    outline
        .iter()
        .map(|&p| {
            let q = crate::model::apply(m, [p[0] + 0.15, p[1] - 0.65]);
            v(centre[0] + unit * q[0], centre[1] + unit * q[1])
        })
        .collect()
}

// ---------------------------------------------------------- spirals

/// An oblique view of a vertical spiral standing on a dial laid flat: the
/// dial's circle of radius `r` about `centre` is seen at a slant (vertical
/// squash `tilt`), its top (lap 0) tilted away to the back; the spiral
/// climbs `rise` scene units per lap, lap `floor_lap` touching the floor.
#[derive(Clone, Copy, Debug)]
pub struct Spiral {
    pub centre: Vec3,
    pub r: f64,
    pub tilt: f64,
    pub rise: f64,
    pub floor_lap: f64,
}

impl Spiral {
    /// The point `laps` laps round (clockwise seen from above, like the dial
    /// it stands on), raised by its lap count.
    pub fn at(&self, laps: f64) -> Vec3 {
        let s = self.shadow(laps);
        v(s[0], s[1] + self.rise * (laps - self.floor_lap))
    }
    /// The shadow of that point on the floor dial: laps forgotten.
    pub fn shadow(&self, laps: f64) -> Vec3 {
        let a = TAU * laps;
        v(
            self.centre[0] + self.r * a.sin(),
            self.centre[1] + self.r * self.tilt * a.cos(),
        )
    }
    pub fn points(&self, from: f64, to: f64, per_lap: usize) -> Vec<Vec3> {
        let n = ((to - from).abs() * per_lap as f64).ceil().max(1.0) as usize;
        (0..=n)
            .map(|k| self.at(from + (to - from) * k as f64 / n as f64))
            .collect()
    }
}
