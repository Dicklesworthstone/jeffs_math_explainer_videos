//! Shared drawing for chapters 10 to 15. Where a helper already exists in the logarithm film's `chapters/draw.rs`,
//! it is the same code, built there at this franken_manim pin. Here it is author source, not built.
use crate::turning_model::{C, on_circle};
use explainer_kit::{Kit, Palette, Timed, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}
/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}
/// A tracker's value now.
pub fn value(s: &Arena, tracker: Mob) -> f64 {
    s.tracker_value(tracker).unwrap_or(0.0)
}
pub fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(r).style(flat(c, 1.0)).build())
}
pub fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(Circle::new().radius(r).arc_center(at).style(line_style(c, w)).build())
}
pub fn seg(stage: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(vec![a, b]).style(line_style(c, w)).build())
}
pub fn path(stage: &mut Stage<'_>, pts: Vec<Vec3>, c: Srgb, w: f64) -> Mob {
    stage.arena_mut().add(polyline(pts).style(line_style(c, w)).build())
}
/// A solid arrow: a stroked shaft and a filled head.
pub fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> Mob {
    arrow_parts(stage.arena_mut(), a, b, c, w, 0.2)
}
/// The arrow as a closure can rebuild it inside `always_redraw`.
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
/// An empty parent holding mobjects already in the arena.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}
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
/// Live mobjects stop updating first, so they fade instead of being redrawn at full opacity.
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
/// One picture gives way to the next in a single beat.
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
/// Tween a tracker, eased.
pub fn run(stage: &mut Stage<'_>, tracker: Mob, to: f64, seconds: f64) -> fmn::Result<()> {
    let a = tween(tracker, to, seconds)?;
    let a = stage.prepare(a)?;
    stage.play_prepared(vec![a])?;
    Ok(())
}
/// Tween a tracker at constant speed: the walk on the circle has to look like a walk at speed one.
pub fn walk(stage: &mut Stage<'_>, tracker: Mob, to: f64, seconds: f64) -> fmn::Result<()> {
    let a = tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?;
    let a = stage.prepare(a)?;
    stage.play_prepared(vec![a])?;
    Ok(())
}
/// A mobject rebuilt from a tracker's value every frame, already on screen.
pub fn live(stage: &mut Stage<'_>, clock: Mob, draw: impl Fn(&mut Arena, f64) -> Mob + 'static) -> fmn::Result<Mob> {
    let mob = stage.always_redraw(move |s: &mut Arena| {
        let t = value(s, clock);
        draw(s, t)
    });
    stage.add_to_scene(mob)?;
    Ok(mob)
}
/// The chapter's title line.
pub fn title(k: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    k.text(stage, words, 36.0, Palette::ink(), v(0.0, 3.35))
}
/// One line of display math.
pub fn line_of_math(k: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, c: Srgb, x: f64, y: f64) -> fmn::Result<Mob> {
    k.tex(stage, src, size, c, &[], v(x, y))
}

/// A complex-plane panel: `scale` scene units per unit, `centre` the screen point of zero.
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
    pub fn axes(&self, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let (c, w, h) = (self.centre, self.half_w, self.half_h);
        let x = seg(stage, v(c[0] - w, c[1]), v(c[0] + w, c[1]), Palette::faint(), 2.5);
        let y = seg(stage, v(c[0], c[1] - h), v(c[0], c[1] + h), Palette::faint(), 2.5);
        group(stage, &[x, y])
    }
    pub fn unit_circle(&self, stage: &mut Stage<'_>) -> Mob {
        ring(stage, self.centre, self.scale, Palette::dim(), 2.5)
    }
    /// Screen points along the unit circle from angle `from` to angle `to`.
    pub fn arc(&self, from: f64, to: f64) -> Vec<Vec3> {
        let samples = ((to - from).abs() * 30.0).ceil().max(1.0) as u32;
        (0..=samples)
            .map(|j| self.p(on_circle(from + (to - from) * f64::from(j) / f64::from(samples))))
            .collect()
    }
}

/// The part of a chapter file before its tests, so a test's own needles cannot satisfy it.
#[cfg(test)]
pub fn scene_source(file: &'static str) -> &'static str {
    file.split("\n#[cfg(test)]\nmod tests {").next().unwrap_or(file)
}
/// Every needle occurs in the scene source, each after the one before.
#[cfg(test)]
pub fn assert_in_order(file: &'static str, needles: &[&str]) {
    let scene = scene_source(file);
    let mut at = 0;
    for needle in needles {
        let found = scene[at..]
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} missing or out of order"));
        at += found + needle.len();
    }
}
