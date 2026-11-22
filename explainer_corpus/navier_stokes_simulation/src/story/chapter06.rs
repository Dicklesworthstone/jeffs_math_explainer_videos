//! Chapter 06, "The law, written down": cues 06_1 to 06_12, native and
//! exact.
//!
//! LavenderPike `lavender-law01` for root's `navier_stokes_simulation` crate
//! (Mail 36269). Cue text and pictures: `navier-full-script03.md` chapter 06
//! (sha256 31a83a2b…), the crate's `script.tsv` rows 06_1 to 06_12. Facts:
//! `navier-opening-script01/runs/facts01-run02.txt` (08e3870a…):
//! ns.unknowns.2d, ns.equations.2d, units.all.accel and units.N.per.kg.
//! `check_law01.py` recomputes each by its own route.
//!
//! The chapter opens on chapter 05's last frame, rebuilt from
//! `nothing_piles_up01::Continuity`: the title, the full tunnel, chapter 04's
//! law glided under it with ∇·u = 0 over its pressure term, and the white
//! link. At frame 0, Du/Dt and −∇p/ρ are swapped for twins typeset with span
//! maps: the same Tex, in the same place, so they can morph glyph by glyph.
//! - 06_1: the tunnel goes and the pieces slide into one line, with ∇·u = 0
//!   under it, its "=" under the line's. Every place is read from a hidden
//!   copy of the whole line typeset at once, so each piece stands where TeX
//!   sets it.
//! - 06_2 and 06_3: Du/Dt becomes ∂u/∂t + (u·∇)u and −∇p/ρ becomes −(1/ρ)∇p
//!   by `TransformMatchingTex`, one term at a time, so a ∇ or a u is never
//!   paired across terms. The rest of the line moves right as TeX sets it.
//!   The moving part is underlined gold, as in chapter 03.
//! - 06_4 to 06_8:
//!   - the rule;
//!   - the brace and the name, with the model's scope (constant density and
//!     viscosity, Newtonian);
//!   - the count: DIM + 1 equations for DIM + 1 unknowns;
//!   - pressure's missing ∂p/∂t;
//!   - m/s² under each of the five terms, from exponent algebra over kg, m
//!     and s.
//! - 06_9 to 06_11: the system glides under the tunnel, which is drawn from
//!   the `TunnelShape` given (root's saved geometry). Then the boundary roles
//!   in turn: the start, the inlet, the walls and cylinder (no slip, with a
//!   near-wall profile labelled schematic) and the outlet, framed together.
//! - 06_12: the tunnel goes and the equations shrink to the top of the side
//!   panel. Chapter 07's patch, ledger and title places stay clear.
//!
//! Root's tunnel thumbnail goes through [`TunnelHandoff`]. It comes back
//! after the tunnel leaves at 06_1, goes before the tunnel returns at 06_9,
//! and comes back at 06_12. Nothing here enters its corner while it shows.

use std::ops::Range;

use explainer_kit::{Kit, Palette, Timed, children};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::chapter03::{
    ADVECTION, ADVECTION_GAP, FADE, FINE_PT, HIGHLIGHT_W, LABEL_PT, MOVING, OPEN, QUICK, READOUT_PT, STRIKE_W,
    TITLE_AT, TITLE_PT, UNDERLINE_GAP, UNDERLINE_W, Z_FIELD, Z_HIGHLIGHT, Z_MARKS, Z_TEXT, Z_TRAIL, add, create_all,
    fade_in_all, fade_out_all, fail, line_style, swap, text_at, v, white,
};
use super::chapter05::{
    Continuity, FLOW_THICK, GLIDE_DY, PRECISELY_Y, SHADE_ALPHA, TunnelHandoff, TunnelShape, bounds,
};
#[cfg(test)]
use super::chapter05::{NoThumbnail, shown};
use super::chapter04::{EQ_PT, Forces, LHS, PLUS, TERMS, orange};

// ------------------------------------------------------------------ facts

/// A unit as exponents of kilogram, metre and second.
pub type Dim = [i32; 3];
pub const BASE_UNITS: [&str; 3] = ["kg", "m", "s"];

pub const fn times(a: Dim, b: Dim) -> Dim {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub const fn per(a: Dim, b: Dim) -> Dim {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub const KILOGRAM: Dim = [1, 0, 0];
pub const METRE: Dim = [0, 1, 0];
pub const SECOND: Dim = [0, 0, 1];
pub const VELOCITY: Dim = per(METRE, SECOND);
pub const NEWTON: Dim = per(times(KILOGRAM, METRE), times(SECOND, SECOND));
/// ρ, kg/m³.
pub const DENSITY: Dim = per(KILOGRAM, times(METRE, times(METRE, METRE)));
/// p, Pa = N/m².
pub const PASCAL: Dim = per(NEWTON, times(METRE, METRE));
/// ν, m²/s.
pub const KINEMATIC_VISCOSITY: Dim = per(times(METRE, METRE), SECOND);
#[cfg_attr(not(test), allow(dead_code))]
pub const ACCELERATION: Dim = per(VELOCITY, SECOND);

/// The units of the five terms, in order: ∂u/∂t (per second), (u·∇)u (u
/// times a per-metre change of u), (1/ρ)∇p (a per-metre change of p over ρ),
/// ν∇²u (ν times a per-metre-squared change of u), and f (a force per
/// kilogram). units.all.accel: each is m/s².
pub fn term_units() -> [Dim; 5] {
    [
        per(VELOCITY, SECOND),
        times(VELOCITY, per(VELOCITY, METRE)),
        per(per(PASCAL, METRE), DENSITY),
        times(KINEMATIC_VISCOSITY, per(VELOCITY, times(METRE, METRE))),
        per(NEWTON, KILOGRAM),
    ]
}

/// The numerator and denominator of `d`, each base unit with its power.
fn unit_parts(d: Dim, power: impl Fn(&str, i32) -> String, join: &str) -> (String, String) {
    let side = |sign: i32| {
        let parts: Vec<String> = BASE_UNITS
            .iter()
            .zip(d)
            .filter(|&(_, e)| e * sign > 0)
            .map(|(&u, e)| {
                if e * sign == 1 {
                    u.to_owned()
                } else {
                    power(u, e * sign)
                }
            })
            .collect();
        parts.join(join)
    };
    let top = side(1);
    (if top.is_empty() { "1".to_owned() } else { top }, side(-1))
}

/// `d` as Tex, `\mathrm{m/s^{2}}` for an acceleration.
pub fn unit_tex(d: Dim) -> String {
    let (top, bottom) = unit_parts(d, |u, e| format!("{u}^{{{e}}}"), r"\,");
    if bottom.is_empty() {
        format!(r"\mathrm{{{top}}}")
    } else {
        format!(r"\mathrm{{{top}/{bottom}}}")
    }
}

/// `d` in words, "m/s^2" for an acceleration.
#[cfg_attr(not(test), allow(dead_code))]
pub fn unit_words(d: Dim) -> String {
    let (top, bottom) = unit_parts(d, |u, e| format!("{u}^{e}"), "·");
    if bottom.is_empty() {
        top
    } else {
        format!("{top}/{bottom}")
    }
}

/// The flat tunnel's dimensions.
pub const DIM: u32 = 2;

/// Unknowns at every point: `dim` parts of the velocity and the pressure
/// (ns.unknowns.2d).
pub fn unknowns(dim: u32) -> u32 {
    dim + 1
}

/// Equations at every point: `dim` momentum rows and the divergence row
/// (ns.equations.2d).
pub fn equations(dim: u32) -> u32 {
    dim + 1
}

pub fn chip_words() -> [String; 2] {
    [format!("{DIM} equations (x and y)"), "1 equation".to_owned()]
}

pub fn total_words() -> String {
    format!("{} equations for {} unknowns", equations(DIM), unknowns(DIM))
}

// ------------------------------------------------------------------ words

pub const TITLE: &str = "The law, written down";
/// 06_2: what Du/Dt becomes.
pub const ACCEL: &str = r"\partial u/\partial t+(u\cdot\nabla)u";
pub const LOCAL: &str = r"\partial u/\partial t";
/// 06_3: what −∇p/ρ becomes.
pub const PRESSURE: &str = r"-(1/\rho)\nabla p";
/// The momentum row's five terms as they stand in its Tex; 06_8's units go
/// under each.
pub const FIVE: [&str; 5] = [LOCAL, MOVING[1], r"(1/\rho)\nabla p", TERMS[1], TERMS[2]];
/// The momentum row piece by piece: chapter 04's (06_1), after 06_2, after
/// 06_3. Piece 0 is the left side, 1 the "=", 2 the pressure term, 3 and 5
/// the pluses, 4 the viscous term, 6 the body force.
pub const ROWS: [[&str; 7]; 3] = [
    [LHS[0], LHS[1], TERMS[0], PLUS, TERMS[1], PLUS, TERMS[2]],
    [ACCEL, LHS[1], TERMS[0], PLUS, TERMS[1], PLUS, TERMS[2]],
    [ACCEL, LHS[1], PRESSURE, PLUS, TERMS[1], PLUS, TERMS[2]],
];
/// Chapter 05's ∇·u = 0 (`RULE`), split at its "=".
pub const RULE_PIECES: [&str; 3] = [r"\nabla\cdot u", "=", "0"];
/// 06_4, chapter 05's title.
pub const PILES: &str = "nothing piles up";
pub const NAME: &str = "incompressible Navier–Stokes equations";
/// 06_5: the model's scope under the name.
pub const SCOPE: [&str; 2] = [
    r"\text{constant density }\rho\text{ and viscosity }\nu",
    "Newtonian: neighboring layers drag in proportion to their speed difference",
];
pub const AT_EVERY_POINT: &str = "at every point:";
/// 06_7: the slot pressure lacks.
pub const NO_DPDT: &str = r"\partial p/\partial t";
/// 06_10 and 06_11, in the order they appear.
pub const TAGS: [&str; 4] = [
    "start: velocity everywhere",
    "inlet: our choice",
    "walls and cylinder: no slip",
    "outlet: our choice",
];
pub const SCHEMATIC: &str = "(schematic)";
/// The 06_10 lower third: line one, then line two in three pieces (words,
/// Tex, words). Joined with spaces, and with u = u_solid typed as the
/// source types it, they are the source's text.
pub const PRECISELY06: [&str; 4] = [
    "Without viscosity, only 'no passing through' applies.",
    "With viscosity, the no-slip rule",
    r"u=u_{\mathrm{solid}}",
    "fixes the sliding part too.",
];

// ----------------------------------------------------------------- layout

/// The finished momentum row is centred at this x. Its left edge stays put
/// from 06_1 on while the row grows to the right.
pub const LINE_CX: f64 = -0.4;
/// Baselines of the momentum row and the divergence row, 06_1 to 06_8.
pub const BASE: [f64; 2] = [1.45, 0.35];
/// Their baselines under the full tunnel, 06_9 to 06_11.
pub const LOW_BASE: [f64; 2] = [-1.95, -2.65];
/// 06_12: the system at this scale, its momentum row's left edge and
/// baseline here (the top of the side panel).
pub const PANEL_SCALE: f64 = 0.6;
pub const PANEL_LEFT: f64 = 1.15;
pub const PANEL_BASE: f64 = 2.5;
/// The 06_1 slide and the 06_2/06_3 morphs.
pub const SLIDE: f64 = 1.5;
pub const MORPH: f64 = 1.5;
pub const RING_PAD: f64 = 0.1;
pub const BRACE_GAP: f64 = 0.12;
pub const NAME_Y: f64 = -0.45;
pub const SCOPE_Y: [f64; 2] = [-1.05, -1.5];
pub const UNKNOWN_BASE: f64 = -1.12;
/// Gaps after each piece of the 06_6 unknowns line.
pub const UNKNOWN_GAPS: [f64; 5] = [0.2, 0.02, 0.14, 0.14, 0.14];
pub const CHIP_GAP: f64 = 0.35;
pub const PILES_GAP: f64 = 0.4;
pub const UNIT_GAP: f64 = 0.12;
pub const LINK_GAP: f64 = 0.06;
pub const LINK_INSET: f64 = 0.1;
pub const SLOT_PAD: f64 = 0.1;
pub const STRIKE_OVER: f64 = 0.05;
/// The boundary highlights' stroke.
pub const EDGE_W: f64 = 6.0;
/// Baselines of the two rows of tags under the tunnel.
pub const TAG_BASE: [f64; 2] = [-0.5, -1.13];
const END_TAG_INSET: f64 = 0.8;
pub const FRAME_PAD: f64 = 0.15;
/// The schematic near-wall profile: at this share of the tunnel's length,
/// arrows at these shares of the height above the bottom wall, each
/// `PROFILE_L` × s(2 − s) long with s its height over `PROFILE_DEPTH` of the
/// height: zero at the wall, steepest there.
pub const PROFILE_SHARE: f64 = 0.75;
pub const PROFILE_ROWS: [f64; 4] = [0.05, 0.1, 0.15, 0.2];
pub const PROFILE_DEPTH: f64 = 0.25;
pub const PROFILE_L: f64 = 0.9;
pub const PROFILE_DOT: f64 = 0.06;
pub const SCHEMATIC_GAP: f64 = 0.15;
/// A lower-third line's baseline sits this far under its centre line.
pub const LINE_DROP: f64 = 0.07;
pub const WORD_GAP: f64 = 0.12;

/// The profile's shape, 0 at the wall and 1 at `PROFILE_DEPTH`.
pub fn profile_shape(s: f64) -> f64 {
    s * (2.0 - s)
}

// ---------------------------------------------------------------- helpers

/// Left, right, bottom, top.
pub type Box4 = (f64, f64, f64, f64);

fn union(a: Box4, b: Box4) -> Box4 {
    (a.0.min(b.0), a.1.max(b.1), a.2.min(b.2), a.3.max(b.3))
}

/// The box of the glyphs of Tex `m` whose source spans start inside `range`
/// (entry `i` of the span map is child `i`, as chapter 03's `term_extent`).
fn glyph_box(stage: &Stage<'_>, m: Mob, spans: &SpanMapData, range: Range<usize>) -> fmn::Result<Box4> {
    let glyphs = children(stage, m);
    if glyphs.len() != spans.entries.len() {
        return Err(fail("a Tex's children do not match its span map"));
    }
    let mut found: Option<Box4> = None;
    for (entry, &glyph) in spans.entries.iter().zip(&glyphs) {
        if range.contains(&entry.start) {
            let b = bounds(stage, glyph);
            found = Some(found.map_or(b, |f| union(f, b)));
        }
    }
    found.ok_or_else(|| fail("a piece draws no glyph"))
}

/// The box of `term`, its first occurrence in the source of Tex `m`.
fn term_box(stage: &Stage<'_>, m: Mob, spans: &SpanMapData, term: &str) -> fmn::Result<Box4> {
    let start = spans
        .source
        .find(term)
        .ok_or_else(|| fail("a term is not in its Tex"))?;
    glyph_box(stage, m, spans, start..start + term.len())
}

/// A hidden line typeset whole, with a flat-bottomed "2" appended to read
/// its baseline: where each piece stands when the pieces are set together.
struct Line {
    probe: Mob,
    spans: SpanMapData,
    ranges: Vec<Range<usize>>,
}

impl Line {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pieces: &[&str]) -> fmn::Result<Self> {
        let mut source = String::new();
        let mut ranges = Vec::with_capacity(pieces.len());
        for piece in pieces {
            ranges.push(source.len()..source.len() + piece.len());
            source.push_str(piece);
        }
        let (probe, spans) = kit.tex_spans(stage, &format!("{source}2"), EQ_PT, Palette::ink(), &[], ORIGIN)?;
        Ok(Self { probe, spans, ranges })
    }

    fn piece(&self, stage: &Stage<'_>, i: usize) -> fmn::Result<Box4> {
        glyph_box(stage, self.probe, &self.spans, self.ranges[i].clone())
    }

    fn whole(&self, stage: &Stage<'_>) -> fmn::Result<Box4> {
        let end = self.ranges.last().map_or(0, |r| r.end);
        glyph_box(stage, self.probe, &self.spans, 0..end)
    }

    /// Left edge at x, baseline at `base`.
    fn put(&self, stage: &mut Stage<'_>, x: f64, base: f64) -> fmn::Result<()> {
        let (l, ..) = self.whole(stage)?;
        let two = children(stage, self.probe)
            .last()
            .copied()
            .ok_or_else(|| fail("an empty line"))?;
        let b = stage.get_bottom(two)[1];
        stage.shift(self.probe, v(x - l, base - b));
        Ok(())
    }
}

/// Moves `m` so its left edge and bottom are the slot's.
fn put_on(stage: &mut Stage<'_>, m: Mob, slot: Box4) {
    let (l, _, b, _) = bounds(stage, m);
    stage.shift(m, v(slot.0 - l, slot.2 - b));
    stage.set_z_index(m, Z_TEXT, true);
}

/// The shift that takes `m` to the slot.
fn toward(stage: &Stage<'_>, m: Mob, slot: Box4) -> (Mob, Vec3) {
    let (l, _, b, _) = bounds(stage, m);
    (m, v(slot.0 - l, slot.2 - b))
}

/// Detached display math centred at `at`, then moved so its `edge` is there.
fn tex_at(
    kit: &Kit,
    stage: &mut Stage<'_>,
    src: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, size, color, &[], at)?;
    stage.move_to(m, at, edge);
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

/// A white teaching outline round a box.
fn outline_box(stage: &mut Stage<'_>, (l, r, b, t): Box4, pad: f64) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(r - l + 2.0 * pad)
        .height(t - b + 2.0 * pad)
        .corner_radius(0.1)
        .style(line_style(white(), HIGHLIGHT_W, 1.0))
        .build()?
        .moved_to(v((l + r) / 2.0, (b + t) / 2.0));
    Ok(add(stage, frame, Z_HIGHLIGHT))
}

fn arrow(stage: &mut Stage<'_>, tail: Vec3, tip: Vec3, color: Srgb, z: i32) -> fmn::Result<Mob> {
    let a = Arrow::new(tail, tip)
        .buff(0.0)
        .thickness(FLOW_THICK)
        .color(color)
        .build()?;
    Ok(add(stage, a, z))
}

/// Words or math.
#[derive(Clone, Copy)]
pub enum Kind {
    Words,
    Math,
}

/// One piece of a line laid out on a common baseline.
#[derive(Clone, Copy)]
pub struct Piece<'a> {
    pub kind: Kind,
    pub src: &'a str,
    pub size: f64,
    pub color: Srgb,
}

fn make(kit: &Kit, stage: &mut Stage<'_>, p: &Piece<'_>, src: &str) -> fmn::Result<Mob> {
    match p.kind {
        Kind::Words => kit.text(stage, src, p.size, p.color, ORIGIN),
        Kind::Math => kit.tex(stage, src, p.size, p.color, &[], ORIGIN),
    }
}

/// How far a piece's first glyph sits above the baseline, from an unshown
/// copy with a flat-bottomed 2 appended (chapter 04's `first_glyph_lift`).
fn first_glyph_lift(stage: &Stage<'_>, probe: Mob) -> f64 {
    let glyphs = children(stage, probe);
    match (glyphs.first(), glyphs.last()) {
        (Some(&first), Some(&two)) if glyphs.len() >= 2 => stage.get_bottom(first)[1] - stage.get_bottom(two)[1],
        _ => 0.0,
    }
}

/// Moves `m` so its `edge` is at x and its baseline at `base` (chapter 04's
/// `sit_on_baseline`).
fn sit_on_baseline(stage: &mut Stage<'_>, m: Mob, lift: f64, x: f64, base: f64, edge: Vec3) {
    let head = children(stage, m).first().copied().unwrap_or(m);
    stage.move_to(m, v(x, base), edge);
    let dy = base + lift - stage.get_bottom(head)[1];
    stage.shift(m, v(0.0, dy));
    stage.set_z_index(m, Z_TEXT, true);
}

/// A piece with its `edge` at x and its baseline at `base`.
#[allow(clippy::too_many_arguments)]
fn on_baseline(
    kit: &Kit,
    stage: &mut Stage<'_>,
    kind: Kind,
    src: &str,
    size: f64,
    color: Srgb,
    x: f64,
    base: f64,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let p = Piece { kind, src, size, color };
    let probe = make(kit, stage, &p, &format!("{src}2"))?;
    let lift = first_glyph_lift(stage, probe);
    let m = make(kit, stage, &p, src)?;
    sit_on_baseline(stage, m, lift, x, base, edge);
    Ok(m)
}

/// Pieces left to right on one baseline, `gaps[i]` after piece i, the whole
/// centred at x.
fn row_of(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pieces: &[Piece<'_>],
    gaps: &[f64],
    x: f64,
    base: f64,
) -> fmn::Result<Vec<Mob>> {
    if gaps.len() + 1 != pieces.len() {
        return Err(fail("a row needs one gap between each two pieces"));
    }
    let mut made = Vec::with_capacity(pieces.len());
    for p in pieces {
        let probe = make(kit, stage, p, &format!("{}2", p.src))?;
        let lift = first_glyph_lift(stage, probe);
        let m = make(kit, stage, p, p.src)?;
        let width = stage.get_right(m)[0] - stage.get_left(m)[0];
        made.push((m, lift, width));
    }
    let total: f64 = made.iter().map(|&(_, _, w)| w).sum::<f64>() + gaps.iter().sum::<f64>();
    let mut left = x - total / 2.0;
    for (i, &(m, lift, width)) in made.iter().enumerate() {
        sit_on_baseline(stage, m, lift, left, base, LEFT);
        left += width + gaps.get(i).copied().unwrap_or(0.0);
    }
    Ok(made.into_iter().map(|(m, _, _)| m).collect())
}

/// One play of `rt` seconds.
#[derive(Default)]
struct Beat<'a> {
    out: &'a [Mob],
    into: &'a [Mob],
    /// Drawn along their paths.
    drawn: &'a [Mob],
    moves: &'a [(Mob, Vec3)],
    /// Source, target and their span maps: the target replaces the source.
    morphs: &'a [(Mob, Mob, &'a SpanMapData, &'a SpanMapData)],
}

fn beat(stage: &mut Stage<'_>, b: Beat<'_>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in b.out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in b.into {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in b.drawn {
        anims.push(stage.prepare(show_creation(m).rt(rt))?);
    }
    for &(m, delta) in b.moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(delta)?;
        anims.push(stage.prepare(a)?);
    }
    for &(from, to, from_spans, to_spans) in b.morphs {
        let a = TransformMatchingTex::new(stage.arena_mut(), from, to, from_spans, to_spans)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// `mobs` fade in one after another, `rt` each.
fn one_by_one(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    for &m in mobs {
        fade_in_all(stage, &[m], rt)?;
    }
    Ok(())
}

/// `mobs` scale by `factor` about `about` and move by `delta`, in one play.
fn shrink(stage: &mut Stage<'_>, mobs: &[Mob], about: Vec3, delta: Vec3, factor: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .scale_about(factor, Some(about), None)?
            .shift(delta)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

// ---------------------------------------------------------------- objects

/// The tunnel's boundary roles (06_10 and 06_11), from the shape given.
pub struct Boundary {
    /// The interior, tinted for "everywhere".
    pub tint: Mob,
    pub inlet: Mob,
    /// Top, bottom.
    pub walls: [Mob; 2],
    pub ring: Mob,
    pub outlet: Mob,
    /// The near-wall arrows, then the dot on the wall.
    pub profile: Vec<Mob>,
    pub schematic: Mob,
    /// In [`TAGS`] order.
    pub tags: [Mob; 4],
    pub frame: Mob,
    pub precisely: Vec<Mob>,
}

impl Boundary {
    /// Everything on the tunnel and under it but the lower third.
    pub fn marks(&self) -> Vec<Mob> {
        let mut all = vec![
            self.tint,
            self.inlet,
            self.walls[0],
            self.walls[1],
            self.ring,
            self.outlet,
        ];
        all.extend(&self.profile);
        all.push(self.schematic);
        all.extend(self.tags);
        all.push(self.frame);
        all
    }
}

/// Every object of the chapter, built once and detached.
pub struct Law {
    /// Chapter 05's title, gone at 06_1.
    pub old_title: Mob,
    /// The full tunnel: gone at 06_1, back at 06_9, gone at 06_12.
    pub tank: Mob,
    pub cylinder: Mob,
    /// Chapter 05's white link, gone at 06_1.
    pub old_link: Mob,
    pub title: Mob,
    /// The momentum row in [`ROWS`] order before 06_2 (chapter 04's pieces,
    /// with the span-mapped twins of Du/Dt and −∇p/ρ) and after 06_3.
    pub before: [Mob; 7],
    pub after: [Mob; 7],
    accel_from_spans: SpanMapData,
    accel_spans: SpanMapData,
    pressure_from_spans: SpanMapData,
    pressure_spans: SpanMapData,
    /// Chapter 05's ∇·u = 0.
    pub rule: Mob,
    /// Each piece's place in the line after 06_1, 06_2 and 06_3, and the
    /// rule's under it.
    pub slots: [[Box4; 7]; 3],
    pub rule_slots: [Box4; 3],
    /// The finished row's left edge.
    pub left: f64,
    pub underline: Mob,
    pub advection: Mob,
    /// 06_2 round Du/Dt, 06_3 round the right side, 06_4 round the rule.
    pub focus: [Mob; 3],
    pub piles: Mob,
    pub brace: Mob,
    pub name: Mob,
    pub scope: [Mob; 2],
    pub chips: [Mob; 2],
    pub total: Mob,
    pub unknowns: Vec<Mob>,
    pub pressure_ring: Mob,
    pub local_ring: Mob,
    pub empty_slot: Mob,
    pub no_dpdt: Mob,
    pub strike: Mob,
    pub link: Mob,
    /// Under the five terms, in [`FIVE`] order.
    pub units: Vec<Mob>,
    pub boundary: Boundary,
}

impl Law {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity, tunnel: TunnelShape) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let dim = Palette::dim();
        let gold = Palette::gold();
        let blue = Palette::blue();
        let title = kit.text_left(stage, TITLE, TITLE_PT, ink, v(TITLE_AT.0, TITLE_AT.1))?;
        stage.set_z_index(title, Z_TEXT, true);

        // Where the pieces stand: the finished row centred at LINE_CX, its
        // left edge the same at every stage, the rule's "=" under the row's.
        let lines = [
            Line::build(kit, stage, &ROWS[0])?,
            Line::build(kit, stage, &ROWS[1])?,
            Line::build(kit, stage, &ROWS[2])?,
        ];
        let (l, r, ..) = lines[2].whole(stage)?;
        let left = LINE_CX - (r - l) / 2.0;
        let rule_line = Line::build(kit, stage, &RULE_PIECES)?;
        let mut slots = [[(0.0, 0.0, 0.0, 0.0); 7]; 3];
        let mut rule_slots = [(0.0, 0.0, 0.0, 0.0); 3];
        for (k, line) in lines.iter().enumerate() {
            line.put(stage, left, BASE[0])?;
            for (i, slot) in slots[k].iter_mut().enumerate() {
                *slot = line.piece(stage, i)?;
            }
            let eq = slots[k][1];
            rule_line.put(stage, 0.0, BASE[1])?;
            let rule_eq = rule_line.piece(stage, 1)?;
            stage.shift(
                rule_line.probe,
                v((eq.0 + eq.1) / 2.0 - (rule_eq.0 + rule_eq.1) / 2.0, 0.0),
            );
            rule_slots[k] = rule_line.whole(stage)?;
        }

        // Frame 0: twins of Du/Dt and −∇p/ρ, with span maps, where chapter
        // 05 leaves the originals. Law order: Du/Dt, =, the three terms, the
        // two pluses.
        let law = &c.from04.law;
        let (accel_from, accel_from_spans) = kit.tex_spans(stage, LHS[0], EQ_PT, ink, &[], ORIGIN)?;
        put_on(stage, accel_from, bounds(stage, law[0]));
        let (pressure_from, pressure_from_spans) = kit.tex_spans(stage, TERMS[0], EQ_PT, orange(), &[], ORIGIN)?;
        put_on(stage, pressure_from, bounds(stage, law[2]));
        let (accel, accel_spans) = kit.tex_spans(stage, ACCEL, EQ_PT, ink, &[], ORIGIN)?;
        put_on(stage, accel, slots[1][0]);
        let (pressure, pressure_spans) = kit.tex_spans(stage, PRESSURE, EQ_PT, orange(), &[], ORIGIN)?;
        put_on(stage, pressure, slots[2][2]);
        let before = [accel_from, law[1], pressure_from, law[5], law[3], law[6], law[4]];
        let after = [accel, law[1], pressure, law[5], law[3], law[6], law[4]];

        // 06_2: advection underlined gold, as chapter 03 underlines it.
        let moving = term_box(stage, accel, &accel_spans, FIVE[1])?;
        let line_y = moving.2 - UNDERLINE_GAP;
        let underline = add(
            stage,
            polyline(vec![v(moving.0, line_y), v(moving.1, line_y)])
                .style(line_style(gold, UNDERLINE_W, 1.0))
                .build(),
            Z_TEXT,
        );
        let advection = text_at(
            kit,
            stage,
            ADVECTION,
            LABEL_PT,
            gold,
            v((moving.0 + moving.1) / 2.0, line_y - ADVECTION_GAP),
            UP,
        )?;

        let right_side = slots[1][2..].iter().fold(slots[1][2], |a, &b| union(a, b));
        let focus = [
            outline_box(stage, slots[0][0], RING_PAD)?,
            outline_box(stage, right_side, RING_PAD)?,
            outline_box(stage, rule_slots[2], RING_PAD)?,
        ];
        let piles = on_baseline(
            kit,
            stage,
            Kind::Words,
            PILES,
            LABEL_PT,
            dim,
            rule_slots[2].1 + PILES_GAP,
            BASE[1],
            LEFT,
        )?;

        // 06_5: the brace round both rows, the name and the scope.
        let top = slots[2].iter().fold(f64::MIN, |t, s| t.max(s.3));
        let brace_x = left - BRACE_GAP;
        let span = polyline(vec![v(brace_x, rule_slots[2].2), v(brace_x, top)]).build();
        let brace = add(
            stage,
            Brace::around(&span, LEFT).buff(0.0).color(white()).build(),
            Z_MARKS,
        );
        let name = text_at(kit, stage, NAME, READOUT_PT, ink, v(LINE_CX, NAME_Y), ORIGIN)?;
        let scope = [
            tex_at(kit, stage, SCOPE[0], LABEL_PT, dim, v(LINE_CX, SCOPE_Y[0]), ORIGIN)?,
            text_at(kit, stage, SCOPE[1], FINE_PT, dim, v(LINE_CX, SCOPE_Y[1]), ORIGIN)?,
        ];

        // 06_6: the count, right of each row, and the unknowns.
        let chip_x = slots[2][6].1 + CHIP_GAP;
        let words = chip_words();
        let chips = [
            on_baseline(kit, stage, Kind::Words, &words[0], LABEL_PT, ink, chip_x, BASE[0], LEFT)?,
            on_baseline(kit, stage, Kind::Words, &words[1], LABEL_PT, ink, chip_x, BASE[1], LEFT)?,
        ];
        let total = text_at(kit, stage, &total_words(), READOUT_PT, ink, v(LINE_CX, NAME_Y), ORIGIN)?;
        let piece = |kind, src, color| Piece {
            kind,
            src,
            size: LABEL_PT,
            color,
        };
        let unknowns = row_of(
            kit,
            stage,
            &[
                piece(Kind::Words, AT_EVERY_POINT, dim),
                piece(Kind::Math, "u", blue),
                piece(Kind::Words, ",", dim),
                piece(Kind::Math, "v", blue),
                piece(Kind::Words, "and", dim),
                piece(Kind::Math, "p", orange()),
            ],
            &UNKNOWN_GAPS,
            LINE_CX,
            UNKNOWN_BASE,
        )?;

        // 06_7: pressure, ∂u/∂t, the struck-out ∂p/∂t in the second row
        // under ∂u/∂t, and the link from pressure to the rule.
        let pressure_ring = outline_box(stage, slots[2][2], RING_PAD)?;
        let local = term_box(stage, accel, &accel_spans, FIVE[0])?;
        let local_ring = outline_box(stage, local, RING_PAD)?;
        let no_dpdt = tex_at(
            kit,
            stage,
            NO_DPDT,
            EQ_PT,
            dim,
            v((local.0 + local.1) / 2.0, (local.2 + local.3) / 2.0 + BASE[1] - BASE[0]),
            ORIGIN,
        )?;
        let (nl, nr, nb, nt) = bounds(stage, no_dpdt);
        let empty = Rectangle::new()
            .width(nr - nl + 2.0 * SLOT_PAD)
            .height(nt - nb + 2.0 * SLOT_PAD)
            .corner_radius(0.08)
            .style(line_style(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(v((nl + nr) / 2.0, (nb + nt) / 2.0));
        let empty_slot = add(stage, empty, Z_MARKS);
        let middle = (nb + nt) / 2.0;
        let strike = add(
            stage,
            polyline(vec![v(nl - STRIKE_OVER, middle), v(nr + STRIKE_OVER, middle)])
                .style(line_style(Palette::red(), STRIKE_W, 1.0))
                .build(),
            Z_HIGHLIGHT,
        );
        let (p, rule_box) = (slots[2][2], rule_slots[2]);
        let link = add(
            stage,
            polyline(vec![
                v((p.0 + p.1) / 2.0, p.2 - LINK_GAP),
                v(rule_box.1 - LINK_INSET, rule_box.3 + LINK_GAP),
            ])
            .style(line_style(white(), HIGHLIGHT_W, 1.0))
            .build(),
            Z_HIGHLIGHT,
        );

        // 06_8: each term's unit, computed, under its centre.
        let terms = [
            local,
            moving,
            term_box(stage, pressure, &pressure_spans, FIVE[2])?,
            slots[2][4],
            slots[2][6],
        ];
        let row_bottom = slots[2].iter().fold(f64::MAX, |b, s| b.min(s.2));
        let mut units = Vec::with_capacity(5);
        for (t, d) in terms.iter().zip(term_units()) {
            units.push(tex_at(
                kit,
                stage,
                &unit_tex(d),
                FINE_PT,
                ink,
                v((t.0 + t.1) / 2.0, row_bottom - UNIT_GAP),
                UP,
            )?);
        }

        let boundary = build_boundary(kit, stage, tunnel)?;
        Ok(Self {
            old_title: c.title,
            tank: c.tunnel.tank,
            cylinder: c.tunnel.cylinder,
            old_link: c.link,
            title,
            before,
            after,
            accel_from_spans,
            accel_spans,
            pressure_from_spans,
            pressure_spans,
            rule: c.rule_zero,
            slots,
            rule_slots,
            left,
            underline,
            advection,
            focus,
            piles,
            brace,
            name,
            scope,
            chips,
            total,
            unknowns,
            pressure_ring,
            local_ring,
            empty_slot,
            no_dpdt,
            strike,
            link,
            units,
            boundary,
        })
    }

    /// Chapter 05's last frame, with the twins in place of Du/Dt and −∇p/ρ.
    pub fn opening(&self) -> Vec<Mob> {
        let mut all = vec![self.old_title, self.tank, self.cylinder];
        all.extend(self.before);
        all.extend([self.rule, self.old_link]);
        all
    }

    /// The finished system: the momentum row and the rule.
    pub fn equations(&self) -> Vec<Mob> {
        let mut all = self.after.to_vec();
        all.push(self.rule);
        all
    }

    /// What is on screen when the chapter ends: the title and the system at
    /// the top of the side panel (root's thumbnail besides).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn final_picture(&self) -> Vec<Mob> {
        let mut all = vec![self.title];
        all.extend(self.equations());
        all
    }

    /// The pieces `which` of `row` toward their places at stage `k`.
    fn slide(&self, stage: &Stage<'_>, row: &[Mob; 7], k: usize, which: Range<usize>) -> Vec<(Mob, Vec3)> {
        which.map(|i| toward(stage, row[i], self.slots[k][i])).collect()
    }

    /// Every object that comes and goes inside the chapter.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn marks(&self) -> Vec<Mob> {
        let mut all = vec![
            self.old_title,
            self.tank,
            self.cylinder,
            self.old_link,
            self.before[0],
            self.before[2],
            self.underline,
            self.advection,
            self.piles,
            self.brace,
            self.name,
            self.total,
        ];
        all.extend(self.focus);
        all.extend(self.scope);
        all.extend(self.chips);
        all.extend(&self.unknowns);
        all.extend(self.marks_06_7());
        all.extend(&self.units);
        all.extend(self.boundary.marks());
        all.extend(&self.boundary.precisely);
        all
    }

    fn marks_06_7(&self) -> Vec<Mob> {
        vec![
            self.pressure_ring,
            self.local_ring,
            self.empty_slot,
            self.no_dpdt,
            self.strike,
            self.link,
        ]
    }
}

fn build_boundary(kit: &Kit, stage: &mut Stage<'_>, t: TunnelShape) -> fmn::Result<Boundary> {
    let ink = Palette::ink();
    let dim = Palette::dim();
    let blue = Palette::blue();
    let k = t.scale();
    let (length, height) = (t.length, t.height);
    let edge = |stage: &mut Stage<'_>, a: Vec3, b: Vec3| {
        add(
            stage,
            polyline(vec![a, b]).style(line_style(white(), EDGE_W, 1.0)).build(),
            Z_HIGHLIGHT,
        )
    };
    let tint = Rectangle::new()
        .width(length * k)
        .height(height * k)
        .style(Style::default().fill(white(), SHADE_ALPHA).stroke(white(), 0.0, 0.0))
        .build()?
        .moved_to(t.at(length / 2.0, height / 2.0));
    let tint = add(stage, tint, Z_FIELD);
    let inlet = edge(stage, t.at(0.0, 0.0), t.at(0.0, height));
    let walls = [
        edge(stage, t.at(0.0, height), t.at(length, height)),
        edge(stage, t.at(0.0, 0.0), t.at(length, 0.0)),
    ];
    let [cx, cy, r] = t.cylinder;
    let ring = Circle::new()
        .radius(r * k)
        .arc_center(t.at(cx, cy))
        .style(line_style(white(), EDGE_W, 1.0))
        .build();
    let ring = add(stage, ring, Z_HIGHLIGHT);
    let outlet = edge(stage, t.at(length, 0.0), t.at(length, height));

    let x = PROFILE_SHARE * length;
    let mut profile = Vec::with_capacity(PROFILE_ROWS.len() + 1);
    for &row in &PROFILE_ROWS {
        let tail = t.at(x, row * height);
        let reach = PROFILE_L * profile_shape(row / PROFILE_DEPTH);
        profile.push(arrow(stage, tail, v(tail[0] + reach, tail[1]), blue, Z_TRAIL)?);
    }
    profile.push(add(
        stage,
        Dot::new().point(t.at(x, 0.0)).radius(PROFILE_DOT).color(blue).build(),
        Z_TRAIL,
    ));
    let mid = t.at(x, (PROFILE_ROWS[1] + PROFILE_ROWS[2]) / 2.0 * height);
    let schematic = text_at(
        kit,
        stage,
        SCHEMATIC,
        FINE_PT,
        dim,
        v(mid[0] + PROFILE_L + SCHEMATIC_GAP, mid[1]),
        LEFT,
    )?;

    // Inlet under the inlet, outlet under the outlet, the start and the
    // walls centred.
    let (west, east) = (t.at(0.0, 0.0)[0], t.at(length, 0.0)[0]);
    let centre = (west + east) / 2.0;
    let tag = |stage: &mut Stage<'_>, s: &str, x: f64, base: f64, edge: Vec3| {
        on_baseline(kit, stage, Kind::Words, s, LABEL_PT, ink, x, base, edge)
    };
    let tags = [
        tag(stage, TAGS[0], centre, TAG_BASE[0], ORIGIN)?,
        tag(stage, TAGS[1], west - END_TAG_INSET, TAG_BASE[0], LEFT)?,
        tag(stage, TAGS[2], centre, TAG_BASE[1], ORIGIN)?,
        tag(stage, TAGS[3], east + END_TAG_INSET, TAG_BASE[0], RIGHT)?,
    ];
    let around = tags
        .iter()
        .map(|&m| bounds(stage, m))
        .reduce(union)
        .ok_or_else(|| fail("no tags"))?;
    let frame = outline_box(stage, around, FRAME_PAD)?;

    let line = |src| Piece {
        kind: Kind::Words,
        src,
        size: LABEL_PT,
        color: dim,
    };
    let mut precisely = row_of(
        kit,
        stage,
        &[line(PRECISELY06[0])],
        &[],
        0.0,
        PRECISELY_Y[0] - LINE_DROP,
    )?;
    precisely.extend(row_of(
        kit,
        stage,
        &[
            line(PRECISELY06[1]),
            Piece {
                kind: Kind::Math,
                ..line(PRECISELY06[2])
            },
            line(PRECISELY06[3]),
        ],
        &[WORD_GAP, WORD_GAP],
        0.0,
        PRECISELY_Y[1] - LINE_DROP,
    )?);
    Ok(Boundary {
        tint,
        inlet,
        walls,
        ring,
        outlet,
        profile,
        schematic,
        tags,
        frame,
        precisely,
    })
}

/// The law where chapter 05's 05_7 glide leaves it, under the tunnel.
pub fn as_chapter05_leaves_it(stage: &mut Stage<'_>, c: &Continuity) {
    for &m in &c.from04.law {
        stage.shift(m, v(0.0, GLIDE_DY));
    }
}

/// Chapter 05's last frame, all at once. `Forces::build` puts chapter 04's
/// walls on screen, which chapter 05 took away at 05_7, and
/// `Continuity::build` its stretch's updater, which 05_2 removes.
pub fn open_on_chapter05(stage: &mut Stage<'_>, forces: &Forces, c: &Continuity, law: &Law) -> fmn::Result<()> {
    for &w in &forces.walls {
        stage.remove_from_scene(w);
    }
    stage.remove_from_scene(c.mover);
    for m in law.opening() {
        if !stage.scene().mobjects().contains(&m) {
            stage.add_to_scene(m)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- the cues

/// 06_1 "We now have every piece, so we can write the law.": the tunnel and
/// the link go, the title changes, and the rule steps left out of the
/// pieces' way. Root's thumbnail returns. The pieces slide into one line,
/// then the rule rises under it.
pub fn cue_06_1(kit: &Kit, stage: &mut Stage<'_>, law: &Law, handoff: &mut dyn TunnelHandoff) -> fmn::Result<()> {
    kit.say(stage, "06_1")?;
    let (l, ..) = bounds(stage, law.rule);
    let aside = [(law.rule, v(law.rule_slots[0].0 - l, 0.0))];
    beat(
        stage,
        Beat {
            out: &[law.old_title, law.tank, law.cylinder, law.old_link],
            into: &[law.title],
            moves: &aside,
            ..Beat::default()
        },
        FADE,
    )?;
    handoff.return_thumbnail(kit, stage)?;
    let moves = law.slide(stage, &law.before, 0, 0..7);
    beat(
        stage,
        Beat {
            moves: &moves,
            ..Beat::default()
        },
        SLIDE,
    )?;
    let rise = [toward(stage, law.rule, law.rule_slots[0])];
    beat(
        stage,
        Beat {
            moves: &rise,
            ..Beat::default()
        },
        OPEN,
    )
}

/// 06_2 "On the left, a parcel's acceleration. …": Du/Dt outlined; with
/// sentence 1 ("It's the change at a fixed point, plus advection, the change
/// from moving.") it becomes ∂u/∂t + (u·∇)u while the rest moves right, and
/// the moving part is underlined gold.
pub fn cue_06_2(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_2")?;
    fade_in_all(stage, &law.focus[..1], FADE)?;
    kit.until_sentence(stage, "06_2", 1)?;
    let mut moves = law.slide(stage, &law.before, 1, 1..7);
    moves.push(toward(stage, law.rule, law.rule_slots[1]));
    beat(
        stage,
        Beat {
            out: &law.focus[..1],
            moves: &moves,
            morphs: &[(law.before[0], law.after[0], &law.accel_from_spans, &law.accel_spans)],
            ..Beat::default()
        },
        MORPH,
    )?;
    create_all(stage, &[law.underline], FADE)?;
    fade_in_all(stage, &[law.advection], FADE)
}

/// 06_3 "On the right, the forces on each kilogram. …": the right side
/// outlined; with sentence 1 ("Minus the pressure gradient over density, …")
/// −∇p/ρ becomes −(1/ρ)∇p while the viscous and body terms move right.
pub fn cue_06_3(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_3")?;
    swap(stage, &[law.advection], &law.focus[1..2], FADE)?;
    kit.until_sentence(stage, "06_3", 1)?;
    let moves = law.slide(stage, &law.after, 2, 3..7);
    beat(
        stage,
        Beat {
            out: &law.focus[1..2],
            moves: &moves,
            morphs: &[(
                law.before[2],
                law.after[2],
                &law.pressure_from_spans,
                &law.pressure_spans,
            )],
            ..Beat::default()
        },
        MORPH,
    )
}

/// 06_4 "And underneath, the rule that nothing piles up. …": chapter 05's
/// title beside the rule; with sentence 1 ("The divergence of the velocity
/// is zero.") the rule outlined.
pub fn cue_06_4(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_4")?;
    fade_in_all(stage, &[law.piles], FADE)?;
    kit.until_sentence(stage, "06_4", 1)?;
    fade_in_all(stage, &law.focus[2..], FADE)
}

/// 06_5 "These are the incompressible Navier-Stokes equations, for a fluid
/// of constant density and viscosity.": the white brace and the name, then
/// the scope.
pub fn cue_06_5(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_5")?;
    swap(stage, &[law.piles, law.focus[2]], &[law.brace, law.name], FADE)?;
    fade_in_all(stage, &law.scope, FADE)
}

/// 06_6 "In our flat tunnel, they're three equations for three unknowns at
/// every point. …": the count beside each row and in total; with sentence 1
/// ("Two parts of the velocity, and the pressure.") the unknowns.
pub fn cue_06_6(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_6")?;
    let mut out = vec![law.name];
    out.extend(law.scope);
    let mut into = law.chips.to_vec();
    into.push(law.total);
    swap(stage, &out, &into, FADE)?;
    kit.until_sentence(stage, "06_6", 1)?;
    fade_in_all(stage, &law.unknowns, FADE)
}

/// 06_7 "But look at pressure. …": the count goes and pressure is outlined;
/// sentence 1 outlines ∂u/∂t; sentence 2 ("Pressure has none.") shows the
/// empty ∂p/∂t slot and strikes it; sentence 3 links pressure to the rule.
pub fn cue_06_7(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_7")?;
    let mut out = law.chips.to_vec();
    out.push(law.total);
    out.extend(&law.unknowns);
    out.push(law.brace);
    swap(stage, &out, &[law.pressure_ring], FADE)?;
    kit.until_sentence(stage, "06_7", 1)?;
    fade_in_all(stage, &[law.local_ring], FADE)?;
    kit.until_sentence(stage, "06_7", 2)?;
    fade_in_all(stage, &[law.empty_slot, law.no_dpdt], FADE)?;
    create_all(stage, &[law.strike], QUICK)?;
    kit.until_sentence(stage, "06_7", 3)?;
    create_all(stage, &[law.link], FADE)
}

/// 06_8 "Check the units. …": the marks go; with sentence 1 ("Every term is
/// meters per second, per second.") m/s² under each term, left to right.
pub fn cue_06_8(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    kit.say(stage, "06_8")?;
    let mut out = law.marks_06_7();
    out.push(law.underline);
    fade_out_all(stage, &out, FADE)?;
    kit.until_sentence(stage, "06_8", 1)?;
    one_by_one(stage, &law.units, QUICK)
}

/// 06_9 "Equations alone aren't a problem we can solve. …": the units go and
/// the system glides under the tunnel's place; with sentence 1 ("We also
/// need where the flow starts, …") root's thumbnail goes and the tunnel
/// returns, static.
pub fn cue_06_9(kit: &Kit, stage: &mut Stage<'_>, law: &Law, handoff: &mut dyn TunnelHandoff) -> fmn::Result<()> {
    kit.say(stage, "06_9")?;
    let mut moves: Vec<(Mob, Vec3)> = law.after.iter().map(|&m| (m, v(0.0, LOW_BASE[0] - BASE[0]))).collect();
    moves.push((law.rule, v(0.0, LOW_BASE[1] - BASE[1])));
    beat(
        stage,
        Beat {
            out: &law.units,
            moves: &moves,
            ..Beat::default()
        },
        OPEN,
    )?;
    kit.until_sentence(stage, "06_9", 1)?;
    handoff.leave_thumbnail(kit, stage)?;
    fade_in_all(stage, &[law.tank, law.cylinder], OPEN)
}

/// 06_10 "At the start, we say what the velocity is everywhere. …": the
/// interior and the start tag; sentence 1 the inlet; sentence 2 the walls
/// and the cylinder; sentence 3 ("And because it's viscous, it doesn't slide
/// either.") the schematic profile and the no-slip tag; sentence 4 the lower
/// third.
pub fn cue_06_10(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    let b = &law.boundary;
    kit.say(stage, "06_10")?;
    fade_in_all(stage, &[b.tint, b.tags[0]], FADE)?;
    kit.until_sentence(stage, "06_10", 1)?;
    beat(
        stage,
        Beat {
            drawn: &[b.inlet],
            into: &b.tags[1..2],
            ..Beat::default()
        },
        FADE,
    )?;
    kit.until_sentence(stage, "06_10", 2)?;
    create_all(stage, &[b.walls[0], b.walls[1], b.ring], FADE)?;
    kit.until_sentence(stage, "06_10", 3)?;
    let mut profile = b.profile.clone();
    profile.push(b.schematic);
    fade_in_all(stage, &profile, FADE)?;
    fade_in_all(stage, &b.tags[2..3], FADE)?;
    kit.until_sentence(stage, "06_10", 4)?;
    fade_in_all(stage, &b.precisely, FADE)
}

/// 06_11 "At the outlet, we need a rule that lets air leave, …": the lower
/// third goes, the outlet and its tag come; with sentence 1 ("Change any of
/// these, …") the four tags framed together.
pub fn cue_06_11(kit: &Kit, stage: &mut Stage<'_>, law: &Law) -> fmn::Result<()> {
    let b = &law.boundary;
    kit.say(stage, "06_11")?;
    beat(
        stage,
        Beat {
            out: &b.precisely,
            drawn: &[b.outlet],
            into: &b.tags[3..],
            ..Beat::default()
        },
        FADE,
    )?;
    kit.until_sentence(stage, "06_11", 1)?;
    create_all(stage, &[b.frame], FADE)
}

/// 06_12 "So far this is mathematics. …": the tunnel and its marks go and
/// root's thumbnail returns; with sentence 1 ("Now we have to turn it into
/// something a computer can store.") the system shrinks to the top of the
/// side panel. Then the chapter holds.
pub fn cue_06_12(kit: &Kit, stage: &mut Stage<'_>, law: &Law, handoff: &mut dyn TunnelHandoff) -> fmn::Result<()> {
    kit.say(stage, "06_12")?;
    let mut out = law.boundary.marks();
    out.extend([law.tank, law.cylinder]);
    fade_out_all(stage, &out, FADE)?;
    handoff.return_thumbnail(kit, stage)?;
    kit.until_sentence(stage, "06_12", 1)?;
    shrink(
        stage,
        &law.equations(),
        v(law.left, LOW_BASE[0]),
        v(PANEL_LEFT - law.left, PANEL_BASE - LOW_BASE[0]),
        PANEL_SCALE,
        SLIDE,
    )?;
    kit.hold(stage)
}

/// The chapter from chapter 05's last frame. `tunnel` is root's saved
/// geometry ([`TunnelShape::BENCHMARK`] for the solver's tunnel), the same
/// shape chapter 05 drew.
pub fn chapter06(
    kit: &Kit,
    stage: &mut Stage<'_>,
    tunnel: TunnelShape,
    handoff: &mut dyn TunnelHandoff,
) -> fmn::Result<Law> {
    let forces = Forces::build(kit, stage)?;
    let c = Continuity::build(kit, stage, &forces, tunnel)?;
    as_chapter05_leaves_it(stage, &c);
    let law = Law::build(kit, stage, &c, tunnel)?;
    open_on_chapter05(stage, &forces, &c, &law)?;
    cue_06_1(kit, stage, &law, handoff)?;
    cue_06_2(kit, stage, &law)?;
    cue_06_3(kit, stage, &law)?;
    cue_06_4(kit, stage, &law)?;
    cue_06_5(kit, stage, &law)?;
    cue_06_6(kit, stage, &law)?;
    cue_06_7(kit, stage, &law)?;
    cue_06_8(kit, stage, &law)?;
    cue_06_9(kit, stage, &law, handoff)?;
    cue_06_10(kit, stage, &law)?;
    cue_06_11(kit, stage, &law)?;
    cue_06_12(kit, stage, &law, handoff)?;
    Ok(law)
}

/// The spoken lines, verbatim from `navier-full-script03.md`
/// (`check_law01.py` compares them byte for byte), as `script.tsv` rows.
#[cfg(test)]
pub const SCRIPT06: &str = concat!(
    "06_1\tWe now have every piece, so we can write the law.\n",
    "06_2\tOn the left, a parcel's acceleration. It's the change at a fixed point, plus advection, the change from moving.\n",
    "06_3\tOn the right, the forces on each kilogram. Minus the pressure gradient over density, viscosity's pull toward the average, and any outside force, f.\n",
    "06_4\tAnd underneath, the rule that nothing piles up. The divergence of the velocity is zero.\n",
    "06_5\tThese are the incompressible Navier-Stokes equations, for a fluid of constant density and viscosity.\n",
    "06_6\tIn our flat tunnel, they're three equations for three unknowns at every point. Two parts of the velocity, and the pressure.\n",
    "06_7\tBut look at pressure. The velocity has an equation that says how it changes in time. Pressure has none. It's whatever it needs to be to keep the divergence zero.\n",
    "06_8\tCheck the units. Every term is meters per second, per second.\n",
    "06_9\tEquations alone aren't a problem we can solve. We also need where the flow starts, and what happens at its edges.\n",
    "06_10\tAt the start, we say what the velocity is everywhere. At the inlet, we choose how the air comes in. At the walls and the cylinder, air can't pass through. And because it's viscous, it doesn't slide either. Right at a surface, it moves with the surface.\n",
    "06_11\tAt the outlet, we need a rule that lets air leave, and that rule is our choice, not nature's. Change any of these, and you have a different problem, with a different answer.\n",
    "06_12\tSo far this is mathematics. Now we have to turn it into something a computer can store.\n",
);

#[cfg(test)]
mod tests {
    use super::super::chapter03::FORMS;
    use super::super::chapter05::RULE;
    use super::super::chapter04::LHS_UNIT;
    use super::*;
    use explainer_kit::{Narrator, Script};

    /// facts01-run02's values for this chapter's keys.
    const FACTS: [(&str, &str); 4] = [
        ("ns.unknowns.2d", "3"),
        ("ns.equations.2d", "3"),
        ("units.all.accel", "m/s^2 for all five terms"),
        ("units.N.per.kg", "True"),
    ];

    fn computed(key: &str) -> String {
        match key {
            "ns.unknowns.2d" => format!("{}", unknowns(DIM)),
            "ns.equations.2d" => format!("{}", equations(DIM)),
            "units.all.accel" => {
                let all = term_units();
                assert!(all.iter().all(|&d| d == all[0]), "{all:?}");
                format!("{} for all five terms", unit_words(all[0]))
            }
            "units.N.per.kg" => String::from(if per(NEWTON, KILOGRAM) == ACCELERATION {
                "True"
            } else {
                "False"
            }),
            _ => unreachable!("{key}"),
        }
    }

    #[test]
    fn the_numbers_are_the_facts() {
        for (key, value) in FACTS {
            assert_eq!(computed(key), value, "{key}");
        }
        assert_eq!(chip_words(), ["2 equations (x and y)", "1 equation"].map(String::from));
        assert_eq!(total_words(), "3 equations for 3 unknowns");
    }

    /// units.all.accel by exponents; the picture shows what the algebra
    /// gives, the same Tex as chapter 04's m/s².
    #[test]
    fn the_units_are_acceleration() {
        assert_eq!(term_units(), [ACCELERATION; 5]);
        assert_eq!(ACCELERATION, [0, 1, -2]);
        assert_eq!(
            (PASCAL, DENSITY, KINEMATIC_VISCOSITY),
            ([1, -1, -2], [1, -3, 0], [0, 2, -1])
        );
        assert_eq!(unit_tex(ACCELERATION), LHS_UNIT);
        assert_eq!(unit_words(ACCELERATION), "m/s^2");
        assert_eq!(unit_words(DENSITY), "kg/m^3");
        assert_eq!(unit_tex(KINEMATIC_VISCOSITY), r"\mathrm{m^{2}/s}");
    }

    #[test]
    fn the_words_are_the_scripts() {
        assert_eq!(FORMS[1], format!("{}={ACCEL}", LHS[0]), "chapter 03's vector form");
        assert_eq!(ACCEL, format!("{LOCAL}+{}", MOVING[1]));
        assert_eq!(PRESSURE, format!("-{}", FIVE[2]));
        assert_eq!(RULE_PIECES.concat(), RULE);
        let finished = ROWS[2].concat();
        for term in FIVE {
            assert!(finished.contains(term), "{term} in {finished}");
        }
        assert_eq!(
            ROWS[0].concat(),
            format!("{}{}{}{PLUS}{}{PLUS}{}", LHS[0], LHS[1], TERMS[0], TERMS[1], TERMS[2]),
            "chapter 04's law"
        );
        let tex = PRECISELY06[2]
            .replace(r"_{\mathrm{solid}}", "_solid")
            .replace('=', " = ");
        assert_eq!(
            format!("{} {} {tex} {}", PRECISELY06[0], PRECISELY06[1], PRECISELY06[3]),
            "Without viscosity, only 'no passing through' applies. With viscosity, the no-slip rule u = u_solid \
             fixes the sliding part too."
        );
        assert_eq!(
            TAGS[..3],
            [
                "start: velocity everywhere",
                "inlet: our choice",
                "walls and cylinder: no slip"
            ]
        );
        assert_eq!(TAGS[3], "outlet: our choice");
    }

    #[test]
    fn every_sentence_gate_exists() {
        let script = Script::parse(SCRIPT06).unwrap();
        assert_eq!(script.lines().len(), 12);
        let narrator = Narrator::estimate(&script);
        for (id, j) in [
            ("06_2", 1),
            ("06_3", 1),
            ("06_4", 1),
            ("06_6", 1),
            ("06_7", 1),
            ("06_7", 2),
            ("06_7", 3),
            ("06_8", 1),
            ("06_9", 1),
            ("06_10", 1),
            ("06_10", 2),
            ("06_10", 3),
            ("06_10", 4),
            ("06_11", 1),
            ("06_12", 1),
        ] {
            assert!(narrator.sentence_starts(id).len() > j, "{id} sentence {j}");
        }
    }

    /// Runs `run` in a scene on the scratch voice at 10 fps.
    struct Probe<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>> {
        kit: Kit,
        run: F,
    }

    impl<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.run)(&self.kit, stage)
        }
    }

    fn probe(run: impl FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>) {
        let script = Script::parse(SCRIPT06).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe { kit, run }, config, 7, &mut fmn::scene::NullSceneSink).expect("the probe runs");
    }

    fn opening(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Forces, Continuity, Law)> {
        let forces = Forces::build(kit, stage)?;
        let c = Continuity::build(kit, stage, &forces, TunnelShape::BENCHMARK)?;
        as_chapter05_leaves_it(stage, &c);
        let law = Law::build(kit, stage, &c, TunnelShape::BENCHMARK)?;
        open_on_chapter05(stage, &forces, &c, &law)?;
        Ok((forces, c, law))
    }

    type Window = Box4;
    const FRAME: Window = (-7.1, 7.1, -3.15, 3.9);
    const LOWER: Window = (-7.1, 7.1, -3.95, -3.15);
    /// Root's thumbnail (x 3.9 to 6.9, y 2.97 to 3.53) and 0.1 round it.
    const THUMB: Window = (3.8, 7.1, 2.87, 3.9);
    /// Root's chapter 07: its patch (PATCH_CENTER ± 1.5 CELL_WIDTH), its
    /// right column (RIGHT ± 2.25: "Only the outer boundary remains" the
    /// widest at 0.52 em a character, "Net outflow" the highest) and its title.
    const PATCH07: Window = (-5.32, -1.18, -1.97, 2.17);
    const LEDGER07: Window = (1.1, 5.6, -2.1, 1.9);
    const TITLE07: Window = (-2.0, 2.0, 3.25, 3.65);

    fn within(b: Box4, w: Window) -> bool {
        w.0 <= b.0 && b.1 <= w.1 && w.2 <= b.2 && b.3 <= w.3
    }

    fn overlaps(a: Box4, b: Box4) -> bool {
        a.0 < b.1 && b.0 < a.1 && a.2 < b.3 && b.2 < a.3
    }

    fn apart(a: Box4, b: Box4) -> f64 {
        let dx = (a.0 - b.1).max(b.0 - a.1).max(0.0);
        let dy = (a.2 - b.3).max(b.2 - a.3).max(0.0);
        dx.hypot(dy)
    }

    fn lowered(b: Box4, dy: f64) -> Box4 {
        (b.0, b.1, b.2 + dy, b.3 + dy)
    }

    fn near(a: Box4, b: Box4) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3 && (a.2 - b.2).abs() < 1e-3 && (a.3 - b.3).abs() < 1e-3
    }

    /// Frame 0 is chapter 05's last frame: the twins stand exactly where
    /// chapter 04's Du/Dt and −∇p/ρ stood, and nothing else is on screen.
    #[test]
    fn the_chapter_opens_on_chapter_05s_last_frame() {
        probe(|kit, stage| {
            let (_, c, law) = opening(kit, stage)?;
            let on = stage.scene().mobjects().to_vec();
            let twin = |m: Mob| match m {
                m if m == c.from04.law[0] => law.before[0],
                m if m == c.from04.law[2] => law.before[2],
                m => m,
            };
            let end: Vec<Mob> = c.final_picture().into_iter().map(twin).collect();
            assert!(end.iter().all(|m| on.contains(m)), "chapter 05's last frame");
            assert!(on.iter().all(|m| end.contains(m)), "and nothing else");
            for (twin, original) in [(law.before[0], c.from04.law[0]), (law.before[2], c.from04.law[2])] {
                assert!(
                    near(bounds(stage, twin), bounds(stage, original)),
                    "a twin in its original's place"
                );
            }
            Ok(())
        });
    }

    /// The pieces typeset alone stand where the whole line sets them, at
    /// every stage; the rule's "=" is under the row's.
    #[test]
    fn the_pieces_fill_their_slots() {
        probe(|kit, stage| {
            let (_, _, law) = opening(kit, stage)?;
            cue_06_1(kit, stage, &law, &mut NoThumbnail)?;
            for (i, &m) in law.before.iter().enumerate() {
                assert!(near(bounds(stage, m), law.slots[0][i]), "06_1 piece {i}");
            }
            assert!(near(bounds(stage, law.rule), law.rule_slots[0]), "06_1 rule");
            cue_06_2(kit, stage, &law)?;
            assert!(near(bounds(stage, law.after[0]), law.slots[1][0]), "06_2 left side");
            for (i, &m) in law.before.iter().enumerate().skip(1) {
                assert!(near(bounds(stage, m), law.slots[1][i]), "06_2 piece {i}");
            }
            cue_06_3(kit, stage, &law)?;
            for (i, &m) in law.after.iter().enumerate() {
                assert!(near(bounds(stage, m), law.slots[2][i]), "06_3 piece {i}");
            }
            assert!(near(bounds(stage, law.rule), law.rule_slots[2]), "06_3 rule");
            assert!(
                shown(stage, &[law.before[0], law.before[2]]).is_empty(),
                "the twins are replaced"
            );
            let row_left = law.slots.iter().map(|s| s[0].0);
            assert!(row_left.clone().all(|x| (x - law.left).abs() < 1e-9), "one left edge");
            let (l, r) = (law.slots[2][0].0, law.slots[2][6].1);
            assert!(((l + r) / 2.0 - LINE_CX).abs() < 1e-9, "the finished row is centred");
            Ok(())
        });
    }

    /// Each object sits in the frame and out of root's thumbnail corner; the
    /// lower third in the caption slot; under the tunnel the rows clear the
    /// tags and the tank; at the end the system clears chapter 07's places.
    #[test]
    fn everything_sits_in_its_window() {
        probe(|kit, stage| {
            let (_, _, law) = opening(kit, stage)?;
            let mut upper: Vec<Box4> = law.slots[2].to_vec();
            upper.push(law.rule_slots[2]);
            upper.extend(law.slots[0]);
            upper.push(law.rule_slots[0]);
            let mut mobs = vec![
                law.after[0],
                law.after[2],
                law.underline,
                law.advection,
                law.piles,
                law.brace,
                law.name,
                law.total,
                law.pressure_ring,
                law.local_ring,
                law.empty_slot,
                law.no_dpdt,
                law.strike,
                law.link,
            ];
            mobs.extend(law.focus);
            mobs.extend(law.scope);
            mobs.extend(law.chips);
            mobs.extend(&law.unknowns);
            mobs.extend(&law.units);
            for &m in &mobs {
                upper.push(bounds(stage, m));
            }
            for b in &upper {
                assert!(within(*b, FRAME) && !overlaps(*b, THUMB), "06_1 to 06_8: {b:?}");
            }
            // The words under and beside the rows keep apart from them.
            let mut rows: Vec<Box4> = law.slots[2].to_vec();
            rows.push(law.rule_slots[2]);
            for m in [
                law.advection,
                law.piles,
                law.name,
                law.total,
                law.chips[0],
                law.chips[1],
            ] {
                let b = bounds(stage, m);
                for (i, row) in rows.iter().enumerate() {
                    assert!(apart(b, *row) > 0.1, "{m:?} near piece {i}");
                }
            }
            for &u in &law.units {
                assert!(apart(bounds(stage, u), law.rule_slots[2]) > 0.1, "a unit near the rule");
            }
            let (_, chip_r, ..) = bounds(stage, law.chips[0]);
            assert!(chip_r < 6.95, "the count stays in the panel: {chip_r}");

            let b = &law.boundary;
            for m in b.marks() {
                assert!(
                    within(bounds(stage, m), FRAME),
                    "tunnel view {m:?}: {:?}",
                    bounds(stage, m)
                );
            }
            for &m in &b.precisely {
                assert!(
                    within(bounds(stage, m), LOWER),
                    "lower third {m:?}: {:?}",
                    bounds(stage, m)
                );
            }
            let tank = bounds(stage, law.tank);
            let frame = bounds(stage, b.frame);
            for (i, &tag) in b.tags.iter().enumerate() {
                let t = bounds(stage, tag);
                assert!(t.3 < tank.2 - 0.2, "tag {i} under the tank");
                for (j, &other) in b.tags.iter().enumerate().skip(i + 1) {
                    let u = bounds(stage, other);
                    assert!(apart(t, u) > 0.3, "tags {i} and {j} too close: {t:?} and {u:?}");
                }
            }
            let schematic = bounds(stage, b.schematic);
            assert!(within(schematic, tank), "the schematic label inside the tank");
            let mut low: Vec<Box4> = law.slots[2]
                .iter()
                .map(|&s| lowered(s, LOW_BASE[0] - BASE[0]))
                .collect();
            low.push(lowered(law.rule_slots[2], LOW_BASE[1] - BASE[1]));
            for s in &low {
                assert!(within(*s, FRAME) && apart(*s, frame) > 0.2, "under the tunnel: {s:?}");
            }
            let row_bottom = low[..7].iter().fold(f64::MAX, |b, s| b.min(s.2));
            assert!(low[7].3 < row_bottom - 0.15, "the rows apart under the tunnel");

            // 06_12: scaled about the row's left baseline point, then moved.
            let about = (law.left, LOW_BASE[0]);
            let to_panel = |s: Box4| {
                let f = |x: f64, a: f64, to: f64| to + PANEL_SCALE * (x - a);
                (
                    f(s.0, about.0, PANEL_LEFT),
                    f(s.1, about.0, PANEL_LEFT),
                    f(s.2, about.1, PANEL_BASE),
                    f(s.3, about.1, PANEL_BASE),
                )
            };
            for s in low.iter().map(|&s| to_panel(s)) {
                assert!(within(s, FRAME) && !overlaps(s, THUMB), "06_12 panel: {s:?}");
                for w in [PATCH07, LEDGER07, TITLE07] {
                    assert!(!overlaps(s, w), "06_12 {s:?} in chapter 07's place {w:?}");
                }
            }
            Ok(())
        });
    }

    /// The marks are the saved geometry's: the inlet and outlet on the tank's
    /// ends, the walls on its sides, the ring on the cylinder; the profile
    /// inside the tank, falling to the wall.
    #[test]
    fn the_marks_are_the_tunnels() {
        probe(|kit, stage| {
            let (_, _, law) = opening(kit, stage)?;
            let t = TunnelShape::BENCHMARK;
            let b = &law.boundary;
            let tank = bounds(stage, law.tank);
            let close = |a: f64, c: f64| (a - c).abs() < 1e-6;
            let (inlet, outlet) = (bounds(stage, b.inlet), bounds(stage, b.outlet));
            assert!(close(inlet.0, tank.0) && close(outlet.1, tank.1));
            assert!(close(inlet.2, tank.2) && close(inlet.3, tank.3));
            let (top, bottom) = (bounds(stage, b.walls[0]), bounds(stage, b.walls[1]));
            assert!(close(top.3, tank.3) && close(bottom.2, tank.2));
            assert!(close(top.0, tank.0) && close(top.1, tank.1));
            let ring = bounds(stage, b.ring);
            let cylinder = bounds(stage, law.cylinder);
            assert!(close(ring.0, cylinder.0) && close(ring.1, cylinder.1) && close(ring.3, cylinder.3));
            let centre = t.at(t.cylinder[0], t.cylinder[1]);
            assert!(
                close((ring.0 + ring.1) / 2.0, centre[0]) && close(ring.1 - ring.0, 2.0 * t.cylinder[2] * t.scale())
            );
            let mut last = f64::MAX;
            for &m in b.profile[..PROFILE_ROWS.len()].iter().rev() {
                let a = bounds(stage, m);
                assert!(within(a, tank), "a profile arrow inside the tank");
                assert!(a.1 - a.0 < last, "the arrows shorten toward the wall");
                last = a.1 - a.0;
                assert!(
                    a.0 > centre[0] + t.cylinder[2] * t.scale(),
                    "downstream of the cylinder"
                );
            }
            assert_eq!((profile_shape(0.0), profile_shape(1.0)), (0.0, 1.0));
            Ok(())
        });
    }

    /// The whole chapter runs headless and ends on the system at the top of
    /// the side panel, everything else gone.
    #[test]
    fn the_chapter_runs_on_the_scratch_voice() {
        probe(|kit, stage| {
            let law = chapter06(kit, stage, TunnelShape::BENCHMARK, &mut NoThumbnail)?;
            let end = law.final_picture();
            assert_eq!(shown(stage, &end).len(), end.len(), "the final picture is on screen");
            assert!(shown(stage, &law.marks()).is_empty(), "every mark is gone");
            let (l, ..) = bounds(stage, law.after[0]);
            assert!((l - PANEL_LEFT).abs() < 1e-3, "the row's left edge at the panel: {l}");
            for m in law.equations() {
                let b = bounds(stage, m);
                assert!(within(b, FRAME) && !overlaps(b, THUMB), "{m:?}: {b:?}");
                for w in [PATCH07, LEDGER07, TITLE07] {
                    assert!(!overlaps(b, w), "{m:?} {b:?} in chapter 07's place");
                }
            }
            Ok(())
        });
    }
}
