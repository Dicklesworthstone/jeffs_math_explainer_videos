//! The lake and the table that Video A's chapters share: where the unit
//! square sits on screen, the lake's picture (in two layers, so tints can
//! lie on the water and under the landmarks), opacity-only fades for moving
//! swarms, the fish's motion (wandering, flying to a place; placement only),
//! the four-cell table and the mosaic it grows into, and chapter 03's 3 × 3
//! regions.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use explainer_kit::props::{Pose, Swarm};
use explainer_kit::{Kit, Palette, Timed, children, hex, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{Paint, Role, flat, line_style};
use crate::model::{GRID, SplitMix64};

/// The lake on screen: the unit square mapped onto this rectangle.
pub const LAKE_W: f64 = 10.0;
pub const LAKE_H: f64 = 6.0;
pub const LAKE_C: Vec3 = [0.0, -0.2, 0.0];
/// The water's corner radius (the bank's is a little more).
pub const WATER_R: f64 = 0.45;
/// One fish drawn per this many in the lake (the key says so on screen).
pub const PER_GLYPH: usize = 10;
/// The key's place: under the lake's south-east corner, inside the safe area.
pub const KEY_AT: Vec3 = [5.0, -3.6, 0.0];
/// The table of catches (01_2) and the mosaic it grows into (01_3).
pub const TABLE_C: Vec3 = [1.0, -0.55, 0.0];
pub const TABLE_W: f64 = 9.2;
pub const TABLE_H: f64 = 5.6;
pub const GAP: f64 = 0.06;

pub fn lake_p(x: f64, y: f64) -> Vec3 {
    v(LAKE_C[0] + (x - 0.5) * LAKE_W, LAKE_C[1] + (y - 0.5) * LAKE_H)
}

pub fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// A heading folded into (−π, π]: the same turn, without the laps.
pub fn fold(a: f64) -> f64 {
    (a + PI).rem_euclid(TAU) - PI
}

/// The outline of a rectangle from `(x0, y0)` to `(x1, y1)` whose corners
/// (south-west, south-east, north-east, north-west) are rounded by the
/// given radii, 0 for a sharp corner.
pub fn rounded_rect(x0: f64, y0: f64, x1: f64, y1: f64, radii: [f64; 4]) -> Vec<Vec3> {
    let corners = [
        (x0, y0, PI, 1.5 * PI),
        (x1, y0, 1.5 * PI, TAU),
        (x1, y1, 0.0, FRAC_PI_2),
        (x0, y1, FRAC_PI_2, PI),
    ];
    let mut out = Vec::new();
    for ((cx, cy, a0, a1), r) in corners.into_iter().zip(radii) {
        if r <= 0.0 {
            out.push(v(cx, cy));
            continue;
        }
        // the arc's centre sits r inside the corner on both axes
        let (sx, sy) = (
            if cx == x0 { cx + r } else { cx - r },
            if cy == y0 { cy + r } else { cy - r },
        );
        for i in 0..=8 {
            let a = a0 + (a1 - a0) * f64::from(i) / 8.0;
            out.push(v(sx + r * a.cos(), sy + r * a.sin()));
        }
    }
    out
}

/// The lake's picture in two layers: `under` (the bank, the water, the deep
/// hole's shading) and `over` (the hole's contour dashes, the reeds, the
/// dock). Tints go between them, fish on top.
pub struct LakeArt {
    pub under: Mob,
    pub over: Mob,
}

impl LakeArt {
    /// Both layers, detached.
    pub fn build(stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let under = stage.arena_mut().add(v_group(under_parts()?));
        let over = stage.arena_mut().add(v_group(over_parts()?));
        Ok(Self { under, over })
    }

    /// The two layers as one group (for scenes with nothing between them).
    pub fn whole(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let mut parts = under_parts()?;
        parts.extend(over_parts()?);
        Ok(stage.arena_mut().add(v_group(parts)))
    }
}

fn under_parts() -> fmn::Result<Vec<VMobject>> {
    let bank = Rectangle::new()
        .width(LAKE_W + 0.3)
        .height(LAKE_H + 0.3)
        .corner_radius(WATER_R + 0.12)
        .style(flat(Role::bank(), 1.0))
        .build()?
        .moved_to(LAKE_C);
    let water = Rectangle::new()
        .width(LAKE_W)
        .height(LAKE_H)
        .corner_radius(WATER_R)
        .style(flat(Role::water(), 1.0))
        .build()?
        .moved_to(LAKE_C);
    let mut parts = vec![bank, water];
    // the deep hole: three darker shades, deepest in the middle
    for (r, c) in [0.15, 0.1, 0.05].into_iter().zip(Role::deep()) {
        let ring: Vec<Vec3> = (0..48)
            .map(|i| {
                let a = TAU * f64::from(i) / 48.0;
                lake_p(0.75 + r * a.cos(), 0.25 + r * a.sin())
            })
            .collect();
        parts.push(Polygon::new(ring).style(flat(c, 1.0)).build());
    }
    Ok(parts)
}

fn over_parts() -> fmn::Result<Vec<VMobject>> {
    let mut parts = Vec::new();
    // the hole's contours, cut into short pieces (nothing long under fish)
    for (k, r) in [0.05, 0.1, 0.15].into_iter().enumerate() {
        let n = 16 + 8 * k;
        for i in (0..n).step_by(2) {
            let (a0, a1) = (TAU * i as f64 / n as f64, TAU * (i + 1) as f64 / n as f64);
            let p0 = lake_p(0.75 + r * a0.cos(), 0.25 + r * a0.sin());
            let p1 = lake_p(0.75 + r * a1.cos(), 0.25 + r * a1.sin());
            parts.push(Line::new(p0, p1).style(line_style(hex("#2E5276"), 2.0)).build()?);
        }
    }
    // reeds in the north-west, a few with seed heads
    let mut rng = SplitMix64::new(7);
    for i in 0..46 {
        let p = lake_p(0.05 + 0.26 * rng.uniform(), 0.79 + 0.19 * rng.uniform());
        let h = 0.14 + 0.14 * rng.uniform();
        let tip = v(p[0] + 0.05 * (rng.uniform() - 0.5), p[1] + h);
        parts.push(Line::new(p, tip).style(line_style(hex("#5B8A4E"), 2.5)).build()?);
        if i % 5 == 0 {
            parts.push(
                Ellipse::new()
                    .width(0.035)
                    .height(0.09)
                    .style(flat(hex("#6E5232"), 1.0))
                    .build()
                    .moved_to(tip),
            );
        }
    }
    // the dock on the south shore: a deck of planks on two posts
    let shore = lake_p(0.5, 0.0);
    let (w, h) = (0.34, 0.62);
    let deck_c = v(shore[0], shore[1] + h / 2.0 - 0.12);
    parts.push(
        Rectangle::new()
            .width(w)
            .height(h)
            .style(flat(hex("#7A5A33"), 1.0))
            .build()?
            .moved_to(deck_c),
    );
    for i in 1..5 {
        let y = deck_c[1] - h / 2.0 + h * f64::from(i) / 5.0;
        parts.push(
            Line::new(v(deck_c[0] - w / 2.0, y), v(deck_c[0] + w / 2.0, y))
                .style(line_style(hex("#5A4124"), 1.5))
                .build()?,
        );
    }
    for dx in [-w / 2.0 + 0.04, w / 2.0 - 0.04] {
        parts.push(
            Circle::new()
                .radius(0.035)
                .arc_center(v(deck_c[0] + dx, deck_c[1] + h / 2.0 - 0.03))
                .style(flat(hex("#4A3520"), 1.0))
                .build(),
        );
    }
    Ok(parts)
}

/// The key "each fish drawn = 10 fish", detached.
pub fn key(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(
        stage,
        &format!("each fish drawn = {PER_GLYPH} fish"),
        24.0,
        Palette::dim(),
        KEY_AT,
    )
}

/// An opacity-only fade for glyphs that updaters move. A `fade_in`
/// Transform on a moving swarm freezes it at the transform's start and
/// leaves it behind its poses when it ends; this instead holds a level
/// (0 to 1) in a tracker and scales every part's paint by it.
pub struct Shade {
    pub tracker: Mob,
}

impl Shade {
    /// `members` drawn from glyphs whose parts are painted `paints` (in
    /// child order), set to `level` now.
    pub fn new(stage: &mut Stage<'_>, members: &[Mob], paints: &[Paint], level: f64) -> fmn::Result<Self> {
        let mut leaves = Vec::new();
        for &m in members {
            let kids = children(stage, m);
            if kids.is_empty() {
                leaves.push((m, paints[0]));
            } else {
                leaves.extend(kids.into_iter().zip(paints.iter().copied()));
            }
        }
        let tracker = stage.add_value_tracker(level);
        stage.add_to_scene(tracker)?;
        paint(stage, &leaves, level);
        let mut last = level;
        stage.add_dt_updater(
            tracker,
            move |s, me, _| {
                let u = s.tracker_value(me).unwrap_or(last);
                if u != last {
                    last = u;
                    paint(s, &leaves, u);
                }
            },
            false,
        )?;
        Ok(Self { tracker })
    }

    /// The fade to `level` over `secs`, to play.
    pub fn to(&self, level: f64, secs: f64) -> fmn::Result<AnimBuilder> {
        tween(self.tracker, level, secs)
    }
}

fn paint(s: &mut fmn::mobject::Stage, leaves: &[(Mob, Paint)], u: f64) {
    for &(m, p) in leaves {
        match p {
            Paint::Fill(full) => {
                s.set_fill(m, None, Some(full * u), None, false);
            }
            Paint::Stroke(full) => {
                s.set_stroke(m, None, None, Some(full * u), None, false);
            }
        }
    }
}

/// Gentle life: each drawn fish loops slowly round its place in the lake.
#[derive(Clone, Copy)]
pub struct Wander {
    pub at: Vec3,
    pub omega: f64,
    pub phase: f64,
}

impl Wander {
    pub fn pose(&self, t: f64) -> Pose {
        let a = self.phase + self.omega * t;
        Pose {
            x: self.at[0] + 0.05 * a.cos(),
            y: self.at[1] + 0.035 * a.sin(),
            heading: a + FRAC_PI_2 * self.omega.signum(),
        }
    }

    /// A wander round each point: speeds and phases from `rng`.
    pub fn round(points: &[Vec3], rng: &mut SplitMix64) -> Vec<Self> {
        points
            .iter()
            .map(|&at| {
                let omega = (0.35 + 0.45 * rng.uniform()) * if rng.uniform() < 0.5 { -1.0 } else { 1.0 };
                Self {
                    at,
                    omega,
                    phase: TAU * rng.uniform(),
                }
            })
            .collect()
    }
}

/// Start `swarm` wandering, `t0` seconds into the shared wander clock.
pub fn wander(stage: &mut Stage<'_>, swarm: &Swarm, ws: Vec<Wander>, t0: f64) -> fmn::Result<()> {
    let poses = ws.iter().map(|w| w.pose(t0)).collect();
    let mut t = t0;
    swarm.drive(stage, poses, move |dt, ps| {
        t += dt;
        for (p, w) in ps.iter_mut().zip(&ws) {
            *p = w.pose(t);
        }
    })
}

/// Fly every member from `from` to `to` over `secs`, eased, by placement
/// only; the old motion stops first.
pub fn fly(stage: &mut Stage<'_>, swarm: &Swarm, from: Vec<Pose>, to: Vec<Pose>, secs: f64) -> fmn::Result<()> {
    stage.clear_updaters(swarm.group, false);
    let start: Vec<Pose> = from
        .iter()
        .map(|p| Pose {
            heading: fold(p.heading),
            ..*p
        })
        .collect();
    let first = start.clone();
    let mut t = 0.0;
    swarm.drive(stage, first, move |dt, ps| {
        t = (t + dt).min(secs);
        let u = smooth(t / secs);
        for ((p, a), b) in ps.iter_mut().zip(&start).zip(&to) {
            *p = Pose {
                x: a.x + (b.x - a.x) * u,
                y: a.y + (b.y - a.y) * u,
                heading: a.heading + (b.heading - a.heading) * u,
            };
        }
    })
}

/// Fade every one of `mobs` in together over `secs` (no shift, no scale).
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
    let mut v: Vec<Box<dyn Animation>> = Vec::with_capacity(mobs.len());
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
        v.push(stage.prepare(a)?);
    }
    stage.play_prepared(v)?;
    Ok(())
}

/// Fade every one of `mobs` out together over `secs`, their updaters
/// cleared first (a live readout would otherwise redraw at full opacity).
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
    let mut v: Vec<Box<dyn Animation>> = Vec::with_capacity(mobs.len());
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
        v.push(stage.prepare(a)?);
    }
    stage.play_prepared(v)?;
    Ok(())
}

/// Put `m` in the scene behind everything already there (table backs under
/// the fish that fly into them).
pub fn behind(stage: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
    stage.add_to_scene(m)?;
    stage.scene_mut().bring_to_back(&[m])?;
    Ok(())
}

/// A cell of the table or the mosaic.
#[derive(Clone, Copy)]
pub struct Cell {
    pub c: Vec3,
    pub w: f64,
    pub h: f64,
}

/// The four cells, columns marked | unmarked and rows in the second catch |
/// not, sized by the marked column's share of the width and the top row's
/// share of the height: [caught twice, first catch only, second catch only,
/// the dark cell].
pub fn table_cells(col_share: f64, row_share: f64) -> [Cell; 4] {
    let (left, top) = (TABLE_C[0] - TABLE_W / 2.0, TABLE_C[1] + TABLE_H / 2.0);
    let (wm, ht) = (TABLE_W * col_share, TABLE_H * row_share);
    let (wu, hb) = (TABLE_W - wm, TABLE_H - ht);
    let cell = |x0: f64, y1: f64, w: f64, h: f64| Cell {
        c: v(x0 + w / 2.0, y1 - h / 2.0),
        w: w - GAP,
        h: h - GAP,
    };
    [
        cell(left, top, wm, ht),
        cell(left, top - ht, wm, hb),
        cell(left + wm, top, wu, ht),
        cell(left + wm, top - ht, wu, hb),
    ]
}

/// The mosaic of a table: areas in proportion, the marked column `marked`
/// of `lake` wide and, in each column, the second catch's share on top.
/// Returns the four cells as [`table_cells`] orders them; the two columns'
/// top rows differ in height unless the table balances.
pub fn mosaic_cells(lake: f64, marked: f64, twice: f64, second: f64) -> [Cell; 4] {
    let (left, top) = (TABLE_C[0] - TABLE_W / 2.0, TABLE_C[1] + TABLE_H / 2.0);
    let wm = TABLE_W * marked / lake;
    let wu = TABLE_W - wm;
    let hm = TABLE_H * twice / marked;
    let hu = TABLE_H * (second - twice) / (lake - marked);
    let cell = |x0: f64, y1: f64, w: f64, h: f64| Cell {
        c: v(x0 + w / 2.0, y1 - h / 2.0),
        w: (w - GAP).max(0.01),
        h: (h - GAP).max(0.01),
    };
    [
        cell(left, top, wm, hm),
        cell(left, top - hm, wm, TABLE_H - hm),
        cell(left + wm, top, wu, hu),
        cell(left + wm, top - hu, wu, TABLE_H - hu),
    ]
}

/// `n` places in a cell on a jittered grid, fish facing east.
pub fn places_in(cell: Cell, n: usize, rng: &mut SplitMix64) -> Vec<Pose> {
    let pad = 0.1;
    let (w, h) = ((cell.w - 2.0 * pad).max(0.05), (cell.h - 2.0 * pad).max(0.05));
    let cols = ((n as f64 * w / h).sqrt().ceil() as usize).max(1);
    let rows = n.div_ceil(cols).max(1);
    (0..n)
        .map(|i| {
            let (cx, cy) = ((i % cols) as f64, (i / cols) as f64);
            Pose {
                x: cell.c[0] - w / 2.0 + w * (cx + 0.5 + 0.3 * (rng.uniform() - 0.5)) / cols as f64,
                y: cell.c[1] + h / 2.0 - h * (cy + 0.5 + 0.3 * (rng.uniform() - 0.5)) / rows as f64,
                heading: 0.35 * (rng.uniform() - 0.5),
            }
        })
        .collect()
}

/// A seeded shuffle of 0..n.
pub fn shuffled(n: usize, rng: &mut SplitMix64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = ((rng.uniform() * (i + 1) as f64) as usize).min(i);
        v.swap(i, j);
    }
    v
}

/// The four cells' fills in cell order: the marked column gold, the second
/// catch green, the dark cell near-black teal.
pub fn cell_tints() -> [Srgb; 4] {
    [hex("#4A4224"), hex("#2E2A1A"), hex("#1E3220"), hex("#0B1113")]
}

/// The four cell backgrounds, flat fills (the gaps between them are the
/// table's lines), as one detached group in cell order.
pub fn cell_backs(stage: &mut Stage<'_>, cells: &[Cell; 4]) -> fmn::Result<Mob> {
    let backs = cells.iter().zip(cell_tints()).map(|(c, t)| {
        Rectangle::new()
            .width(c.w)
            .height(c.h)
            .style(flat(t, 1.0))
            .build()
            .map(|r| r.moved_to(c.c))
    });
    let backs: Vec<VMobject> = backs.collect::<Result<_, _>>()?;
    Ok(stage.arena_mut().add(v_group(backs)))
}

/// Where the table's four header words sit for `cells`: the marked column's
/// header (its left edge at the table's left), the unmarked one (centred
/// over its column), and the two row words (right edges left of the table).
pub fn header_places(cells: &[Cell; 4]) -> [(Vec3, Vec3); 4] {
    let top = TABLE_C[1] + TABLE_H / 2.0 + 0.35;
    let left = TABLE_C[0] - TABLE_W / 2.0;
    [
        (v(left, top), LEFT),
        (v(cells[2].c[0], top), ORIGIN),
        (v(left - 0.25, cells[0].c[1]), RIGHT),
        (v(left - 0.25, cells[1].c[1]), RIGHT),
    ]
}

/// The table's words: column and row headers (R2 Maya M10: the same words
/// as the voice), placed by [`header_places`].
pub fn table_labels(kit: &Kit, stage: &mut Stage<'_>, cells: &[Cell; 4]) -> fmn::Result<Mob> {
    let words = [
        ("marked (first catch)", 28.0, Role::marked()),
        ("unmarked", 28.0, Palette::ink()),
        ("in the second catch", 26.0, Role::second()),
        ("not", 26.0, Palette::dim()),
    ];
    let g = stage.arena_mut().add(VMobject::new());
    for ((s, size, c), (at, edge)) in words.into_iter().zip(header_places(cells)) {
        let m = kit.text(stage, s, size, c, at)?;
        stage.move_to(m, at, edge);
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// The largest scale (at most 1) that fits a `w` × `h` label in `cell`.
pub fn fit(w: f64, h: f64, cell: Cell) -> f64 {
    (0.9 * cell.w / w).min(0.9 * cell.h / h).min(1.0)
}

/// Text on a dark rounded plate, so it reads over fish: one detached group
/// (plate, text) centred at `at`.
pub fn plated(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let text = kit.text(stage, s, size, color, at)?;
    let pad = size / 48.0 * 0.16;
    let (w, h) = (stage.get_width(text) + 2.4 * pad, stage.get_height(text) + 2.0 * pad);
    let plate = stage.arena_mut().add(
        Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(pad.min(h / 2.0))
            .style(flat(hex("#0B0F14"), 0.82))
            .build()?
            .moved_to(at),
    );
    let g = stage.arena_mut().add(VMobject::new());
    stage.attach(g, plate)?;
    stage.attach(g, text)?;
    Ok(g)
}

/// The three known cells' counts on plates, in the cell order of
/// [`table_cells`], each shrunk to fit its cell.
pub fn cell_numbers(kit: &Kit, stage: &mut Stage<'_>, cells: &[Cell; 4], counts: [usize; 3]) -> fmn::Result<Mob> {
    let colors = [Role::marked(), Role::marked(), Role::second()];
    let g = stage.arena_mut().add(VMobject::new());
    for ((cell, count), color) in cells.iter().zip(counts).zip(colors) {
        let m = plated(kit, stage, &thousands(count as u64), 44.0, color, cell.c)?;
        let f = fit(stage.get_width(m), stage.get_height(m), *cell);
        if f < 1.0 {
            stage.scale(m, f);
        }
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// A tray of fish: `cols` × `rows` slots `pitch` apart, centred at `c`,
/// filled row by row from the top left (a catch laid out to be counted).
#[derive(Clone, Copy, Debug)]
pub struct Tray {
    pub c: Vec3,
    pub cols: usize,
    pub rows: usize,
    pub pitch: (f64, f64),
}

impl Tray {
    /// Slot `i`'s centre.
    pub fn slot(&self, i: usize) -> Vec3 {
        let (col, row) = ((i % self.cols) as f64, (i / self.cols) as f64);
        v(
            self.c[0] + (col - (self.cols as f64 - 1.0) / 2.0) * self.pitch.0,
            self.c[1] + ((self.rows as f64 - 1.0) / 2.0 - row) * self.pitch.1,
        )
    }

    /// Every slot as a pose facing east.
    pub fn poses(&self, n: usize) -> Vec<Pose> {
        (0..n)
            .map(|i| {
                let p = self.slot(i);
                Pose {
                    x: p[0],
                    y: p[1],
                    heading: 0.0,
                }
            })
            .collect()
    }

    /// The card behind the slots: a margin of one pitch all round.
    pub fn card(&self, edge: Srgb) -> fmn::Result<VMobject> {
        let (w, h) = (self.cols as f64 * self.pitch.0, self.rows as f64 * self.pitch.1);
        Ok(Rectangle::new()
            .width(w + 0.6 * self.pitch.0)
            .height(h + 0.6 * self.pitch.1)
            .corner_radius(0.12)
            .style(Style::default().fill(hex("#14202E"), 1.0).stroke(edge, 2.0, 0.7))
            .build()?
            .moved_to(self.c))
    }

    /// The tray's top edge (for a label above it) and its left edge.
    pub fn top(&self) -> f64 {
        self.c[1] + (self.rows as f64 + 0.6) * self.pitch.1 / 2.0
    }

    pub fn left(&self) -> f64 {
        self.c[0] - (self.cols as f64 + 0.6) * self.pitch.0 / 2.0
    }
}

/// A dip-grid cell (column west → east, row south → north) on screen.
pub fn cell_rect(col: usize, row: usize) -> (Vec3, f64, f64) {
    let (w, h) = (LAKE_W / GRID as f64, LAKE_H / GRID as f64);
    (
        lake_p((col as f64 + 0.5) / GRID as f64, (row as f64 + 0.5) / GRID as f64),
        w,
        h,
    )
}

/// Chapter 03's 3 × 3 regions (and 13_9's callback), SW … NE as the oracle
/// orders them: flat shapes on the water, each tinted by its marked share,
/// the corner regions keeping the water's rounded corners.
pub struct Regions {
    pub group: Mob,
    pub cells: Vec<Mob>,
}

impl Regions {
    pub fn build(stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut shapes = Vec::with_capacity(9);
        for r in 0..9 {
            let (rx, ry) = (r % 3, r / 3);
            let lo = lake_p(rx as f64 / 3.0, ry as f64 / 3.0);
            let hi = lake_p((rx + 1) as f64 / 3.0, (ry + 1) as f64 / 3.0);
            let round = |a: bool| if a { WATER_R } else { 0.0 };
            let radii = [
                round(rx == 0 && ry == 0),
                round(rx == 2 && ry == 0),
                round(rx == 2 && ry == 2),
                round(rx == 0 && ry == 2),
            ];
            let pts = rounded_rect(lo[0], lo[1], hi[0], hi[1], radii);
            shapes.push(Polygon::new(pts).style(flat(Role::marked(), 0.0)).build());
        }
        let group = stage.arena_mut().add(v_group(shapes));
        let cells = children(stage, group);
        Ok(Self { group, cells })
    }

    /// Tint every region by its marked share (callable from an updater).
    pub fn tint(&self, s: &mut fmn::mobject::Stage, shares: &[f64; 9]) {
        for (&m, &share) in self.cells.iter().zip(shares) {
            s.set_fill(m, None, Some(region_alpha(share)), None, false);
        }
    }

    /// The dashed lines between the regions (short pieces), detached.
    pub fn borders(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let mut parts = Vec::new();
        let dashes = |a: Vec3, b: Vec3, parts: &mut Vec<VMobject>| -> fmn::Result<()> {
            let n = 24;
            for i in (0..n).step_by(2) {
                let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
                let p = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
                parts.push(Line::new(p(t0), p(t1)).style(line_style(hex("#3A5470"), 2.0)).build()?);
            }
            Ok(())
        };
        for k in 1..3 {
            let f = k as f64 / 3.0;
            dashes(lake_p(f, 0.0), lake_p(f, 1.0), &mut parts)?;
            dashes(lake_p(0.0, f), lake_p(1.0, f), &mut parts)?;
        }
        Ok(stage.arena_mut().add(v_group(parts)))
    }
}

/// A region's gold for its marked share: clear at none, strong at the
/// dock's starting two in three; a gamma a little below one so that one in
/// ten still reads, as a faint warm wash over the water.
pub fn region_alpha(share: f64) -> f64 {
    REGION_ALPHA_MAX * (share / 0.66).clamp(0.0, 1.0).powf(0.8)
}

/// The tint at the dock's starting share.
const REGION_ALPHA_MAX: f64 = 0.62;

/// The centre of 3 × 3 region `r` (SW … NE) on screen.
pub fn region_centre(r: usize) -> Vec3 {
    lake_p((2 * (r % 3) + 1) as f64 / 6.0, (2 * (r / 3) + 1) as f64 / 6.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_balanced_mosaic_has_one_straight_divider() {
        // Feller's table at its estimate: both columns' top rows are a tenth
        let c = mosaic_cells(10_000.0, 1_000.0, 100.0, 1_000.0);
        assert!((c[0].h - c[2].h).abs() < 1e-9, "{} vs {}", c[0].h, c[2].h);
        // a lake bigger than the estimate: the unmarked column's green share
        // (900 of N − 1,000) is the smaller one, so its divider sits higher
        let big = mosaic_cells(20_000.0, 1_000.0, 100.0, 1_000.0);
        assert!(big[2].h < big[0].h);
        // a lake smaller than the estimate: its green share is the bigger
        // one, so its divider sits lower (R2 Ruth R18)
        let small = mosaic_cells(5_000.0, 1_000.0, 100.0, 1_000.0);
        assert!(small[2].h > small[0].h);
        // the widths split the table in the marked share
        let w = c[0].w + c[2].w + 2.0 * GAP;
        assert!((w - TABLE_W).abs() < 1e-9);
        assert!(((c[0].w + GAP) / TABLE_W - 0.1).abs() < 1e-9);
    }

    #[test]
    fn rounded_corners_stay_inside_the_rectangle() {
        let pts = rounded_rect(0.0, 0.0, 2.0, 1.0, [0.3, 0.0, 0.3, 0.0]);
        // two sharp corners and two nine-point arcs
        assert_eq!(pts.len(), 2 + 2 * 9);
        for p in &pts {
            assert!((-1e-9..=2.0 + 1e-9).contains(&p[0]) && (-1e-9..=1.0 + 1e-9).contains(&p[1]));
        }
        assert!(pts.contains(&v(2.0, 0.0)) && pts.contains(&v(0.0, 1.0)));
        // the rounded south-west corner cuts the corner point off
        assert!(!pts.contains(&v(0.0, 0.0)));
    }

    #[test]
    fn region_tints_rise_with_the_marked_share() {
        assert_eq!(region_alpha(0.0), 0.0);
        assert!(region_alpha(0.05) < region_alpha(0.1));
        assert!(region_alpha(0.1) > 0.1, "one in ten must still show");
        assert!((region_alpha(0.66) - REGION_ALPHA_MAX).abs() < 1e-12);
        assert_eq!(region_alpha(0.9), region_alpha(0.66));
    }
}
