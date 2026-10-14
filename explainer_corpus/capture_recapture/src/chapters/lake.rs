//! The lake and the table that Video A's chapters share: where the unit
//! square sits on screen, the lake's picture, the four-cell table and the
//! mosaic it grows into, and the fish's motion (wandering, flying to a
//! place), all by placement only.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use explainer_kit::props::{Pose, Swarm};
use explainer_kit::{Kit, Palette, hex, thousands, v};
use fmn::prelude::*;

use super::draw::{flat, line_style};
use crate::model::{GRID, SplitMix64};

/// The lake on screen: the unit square mapped onto this rectangle.
pub const LAKE_W: f64 = 10.4;
pub const LAKE_H: f64 = 6.4;
pub const LAKE_C: Vec3 = [0.0, -0.35, 0.0];
/// One fish drawn per this many in the lake (the key says so on screen).
pub const PER_GLYPH: usize = 10;
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

/// The bank and the water: flat fills, no outlines under the fish.
pub fn water() -> fmn::Result<Vec<VMobject>> {
    let bank = Rectangle::new()
        .width(LAKE_W + 0.3)
        .height(LAKE_H + 0.3)
        .corner_radius(0.55)
        .style(flat(hex("#0A141E"), 1.0))
        .build()?
        .moved_to(LAKE_C);
    let water = Rectangle::new()
        .width(LAKE_W)
        .height(LAKE_H)
        .corner_radius(0.45)
        .style(flat(hex("#13253A"), 1.0))
        .build()?
        .moved_to(LAKE_C);
    Ok(vec![bank, water])
}

/// The landmarks: the deep hole in the south-east with its contours cut
/// into short pieces, reeds in the north-west, and the dock on the south
/// shore.
pub fn landmarks() -> fmn::Result<Vec<VMobject>> {
    let mut parts = Vec::new();
    let basin: Vec<Vec3> = (0..48)
        .map(|i| {
            let a = TAU * f64::from(i) / 48.0;
            lake_p(0.75 + 0.14 * a.cos(), 0.25 + 0.14 * a.sin())
        })
        .collect();
    parts.push(Polygon::new(basin).style(flat(hex("#0C1828"), 0.9)).build());
    for (k, r) in [0.05, 0.09, 0.13].into_iter().enumerate() {
        let n = 16 + 8 * k;
        for i in (0..n).step_by(2) {
            let (a0, a1) = (TAU * i as f64 / n as f64, TAU * (i + 1) as f64 / n as f64);
            let p0 = lake_p(0.75 + r * a0.cos(), 0.25 + r * a0.sin());
            let p1 = lake_p(0.75 + r * a1.cos(), 0.25 + r * a1.sin());
            parts.push(Line::new(p0, p1).style(line_style(hex("#2E5276"), 2.0)).build()?);
        }
    }
    let mut rng = SplitMix64::new(7);
    for _ in 0..46 {
        let p = lake_p(0.06 + 0.26 * rng.uniform(), 0.78 + 0.19 * rng.uniform());
        let h = 0.14 + 0.14 * rng.uniform();
        let tip = v(p[0] + 0.04 * (rng.uniform() - 0.5), p[1] + h);
        parts.push(Line::new(p, tip).style(line_style(hex("#5B8A4E"), 2.5)).build()?);
    }
    let d = lake_p(0.5, 0.0);
    parts.push(
        Rectangle::new()
            .width(0.42)
            .height(0.55)
            .style(flat(hex("#7A5A33"), 1.0))
            .build()?
            .moved_to(v(d[0], d[1] + 0.12)),
    );
    Ok(parts)
}

/// The lake seen from above, water and landmarks as one group.
pub fn lake_picture(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let mut parts = water()?;
    parts.extend(landmarks()?);
    Ok(stage.arena_mut().add(v_group(parts)))
}

/// Gentle life for the cold open: each drawn fish loops slowly round its
/// place in the lake.
#[derive(Clone, Copy)]
pub struct Wander {
    pub at: Vec3,
    pub omega: f64,
    pub phase: f64,
}

impl Wander {
    fn pose(&self, t: f64) -> Pose {
        let a = self.phase + self.omega * t;
        Pose {
            x: self.at[0] + 0.05 * a.cos(),
            y: self.at[1] + 0.035 * a.sin(),
            heading: a + FRAC_PI_2 * self.omega.signum(),
        }
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
                heading: 0.0,
            }
        })
        .collect()
}

/// The first `k` of a seeded shuffle of 0..n.
pub fn shuffled(n: usize, rng: &mut SplitMix64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = ((rng.uniform() * (i + 1) as f64) as usize).min(i);
        v.swap(i, j);
    }
    v
}

/// The four cell backgrounds, flat fills (the gaps between them are the
/// table's lines), as one group in cell order.
pub fn cell_backs(stage: &mut Stage<'_>, cells: &[Cell; 4]) -> fmn::Result<Mob> {
    let tints = [hex("#3A3A22"), hex("#33291A"), hex("#1E2E1E"), hex("#0B0E13")];
    let backs = cells.iter().zip(tints).map(|(c, t)| {
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

/// The table's words: column and row headers (R2 Maya M10: the same words
/// as the voice).
pub fn table_labels(kit: &Kit, stage: &mut Stage<'_>, cells: &[Cell; 4]) -> fmn::Result<Mob> {
    let top = TABLE_C[1] + TABLE_H / 2.0 + 0.35;
    let left = TABLE_C[0] - TABLE_W / 2.0 - 0.25;
    let marked = kit.text(
        stage,
        "marked (first catch)",
        28.0,
        Palette::gold(),
        v(cells[0].c[0], top),
    )?;
    let unmarked = kit.text(stage, "unmarked", 28.0, Palette::ink(), v(cells[2].c[0], top))?;
    let second = kit.text(
        stage,
        "in the second catch",
        26.0,
        Palette::green(),
        v(left, cells[0].c[1]),
    )?;
    stage.move_to(second, v(left, cells[0].c[1]), RIGHT);
    let not = kit.text(stage, "not", 26.0, Palette::dim(), v(left, cells[1].c[1]))?;
    stage.move_to(not, v(left, cells[1].c[1]), RIGHT);
    let g = stage.arena_mut().add(VMobject::new());
    for m in [marked, unmarked, second, not] {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// The three known cells' counts, in the cell order of [`table_cells`].
pub fn cell_numbers(kit: &Kit, stage: &mut Stage<'_>, cells: &[Cell; 4], counts: [usize; 3]) -> fmn::Result<Mob> {
    let colors = [Palette::hero(), Palette::gold(), Palette::green()];
    let g = stage.arena_mut().add(VMobject::new());
    for ((cell, count), color) in cells.iter().zip(counts).zip(colors) {
        let m = kit.text(stage, &thousands(count as u64), 44.0, color, cell.c)?;
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// A grid cell (column west → east, row south → north) on screen.
pub fn cell_rect(col: usize, row: usize) -> (Vec3, f64, f64) {
    let (w, h) = (LAKE_W / GRID as f64, LAKE_H / GRID as f64);
    (
        lake_p((col as f64 + 0.5) / GRID as f64, (row as f64 + 0.5) / GRID as f64),
        w,
        h,
    )
}
