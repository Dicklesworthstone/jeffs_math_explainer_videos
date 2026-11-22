//! Crossing against running along on one cell (19), a finite-volume step on four cells (20), and one open
//! face with the scheme's assumptions (21). CC3 scratch draft: uncompiled, unrendered, sentence timing
//! provisional until Frederick takes exist.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow, fade, line};
use crate::facts::{FV_BEFORE, FV_INFLOW, Q, cell, fv_step, hundredths, spreading, turning};

pub struct OneCell { pub kit: Kit }
pub struct FiniteVolumes { pub kit: Kit }
pub struct OpenBoundary { pub kit: Kit }

// 19: the same unit cell drawn twice, spreading field on the left, turning field on the right.
const SPREAD_AT: (f64, f64) = (-3.6, 0.3);
const TURN_AT: (f64, f64) = (2.6, 0.3);
const HALF: f64 = 1.25;
const SHOW: f64 = 4.0; // screen units per unit of field strength for the sample arrows
// Label offsets: beside the outward arrows of the spreading cell, outside the running arrows of the turning cell.
const BESIDE_NORMALS: [(f64, f64); 4] = [(HALF + 0.85, 0.0), (0.4, HALF + 0.3), (-HALF - 0.85, 0.0), (0.4, -HALF - 0.3)];
const OUTSIDE_TANGENTS: [(f64, f64); 4] = [(HALF + 0.6, 0.0), (0.0, HALF + 0.55), (-HALF - 0.6, 0.0), (0.0, -HALF - 0.55)];

fn square(stage: &mut Stage<'_>, c: (f64, f64), h: f64, color: Srgb, width: f64) -> fmn::Result<Vec<Mob>> {
    let k = [(-h, -h), (h, -h), (h, h), (-h, h)];
    (0..4).map(|i| line(stage, (c.0 + k[i].0, c.1 + k[i].1), (c.0 + k[(i + 1) % 4].0, c.1 + k[(i + 1) % 4].1), color, width)).collect()
}

/// Sample arrows of `field` at eight points of the cell, skipping its centre, where both fields vanish.
fn samples(stage: &mut Stage<'_>, c: (f64, f64), field: fn(Q, Q) -> (Q, Q)) -> fmn::Result<Vec<Mob>> {
    let f = |x: Q| x.0 as f64 / x.1 as f64;
    let mut mobs = Vec::new();
    for (i, j) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        let (x, y) = (Q::new(i, 4), Q::new(j, 4));
        let (fx, fy) = field(x, y);
        let a = (c.0 + 2.0 * HALF * f(x), c.1 + 2.0 * HALF * f(y));
        mobs.extend(arrow(stage, a, (a.0 + SHOW * f(fx), a.1 + SHOW * f(fy)), Palette::dim())?);
    }
    Ok(mobs)
}

/// Value labels for the four edges (right, top, left, bottom) at offsets `at` from the cell centre.
fn edge_labels(kit: &Kit, stage: &mut Stage<'_>, c: (f64, f64), values: &[Q], at: [(f64, f64); 4], color: Srgb) -> fmn::Result<Vec<Mob>> {
    values.iter().zip(at).map(|(q, d)| kit.text(stage, &q.to_string(), 22.0, color, v(c.0 + d.0, c.1 + d.1))).collect()
}

fn sum(v: &[Q]) -> Q {
    v.iter().fold(Q::new(0, 1), |a, &b| a.add(b))
}

impl SceneConstruct for OneCell {
    fn name(&self) -> &str { "19_one_cell" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (s_cross, s_along) = cell(spreading);
        let (t_cross, t_along) = cell(turning);
        let title = kit.text(stage, "Crossing an edge, or running along it", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut first = vec![title];
        first.extend(square(stage, SPREAD_AT, HALF, Palette::blue(), 2.5)?);
        first.extend(square(stage, TURN_AT, HALF, Palette::blue(), 2.5)?);
        fade(stage, &first, true)?;
        kit.say(stage, "19_1")?;
        kit.until_sentence(stage, "19_1", 2)?;
        let mut spread = samples(stage, SPREAD_AT, spreading)?;
        spread.push(kit.text(stage, "spreading: (x/2, y/2)", 24.0, Palette::hero(), v(SPREAD_AT.0, -1.85))?);
        fade(stage, &spread, true)?;
        kit.until_sentence(stage, "19_1", 3)?;
        let mut turn = samples(stage, TURN_AT, turning)?;
        turn.push(kit.text(stage, "turning: (−y/2, x/2)", 24.0, Palette::green(), v(TURN_AT.0, -1.85))?);
        fade(stage, &turn, true)?;
        kit.hold(stage)?;
        kit.say(stage, "19_2")?;
        kit.until_sentence(stage, "19_2", 1)?;
        let question = kit.text(stage, "crosses the edge?   runs along it?", 26.0, Palette::ink(), v(-0.5, 2.3))?;
        fade(stage, &[question], true)?;
        kit.until_sentence(stage, "19_2", 2)?;
        let (cx, cy) = SPREAD_AT;
        let mut outward = Vec::new();
        for (d, e) in [((HALF, 0.0), (0.55, 0.0)), ((0.0, HALF), (0.0, 0.55)), ((-HALF, 0.0), (-0.55, 0.0)), ((0.0, -HALF), (0.0, -0.55))] {
            outward.extend(arrow(stage, (cx + d.0, cy + d.1), (cx + d.0 + e.0, cy + d.1 + e.1), Palette::hero())?);
        }
        outward.extend(edge_labels(&kit, stage, SPREAD_AT, &s_cross, BESIDE_NORMALS, Palette::hero())?);
        fade(stage, &outward, true)?;
        kit.until_sentence(stage, "19_2", 3)?;
        let spread_count = kit.text(stage, &format!("crossing: 4 × {} = {}    along: {}", s_cross[0], sum(&s_cross), sum(&s_along)), 23.0, Palette::ink(), v(SPREAD_AT.0, -2.5))?;
        fade(stage, &[spread_count], true)?;
        kit.hold(stage)?;
        kit.say(stage, "19_3")?;
        kit.until_sentence(stage, "19_3", 1)?;
        let (tx, ty) = TURN_AT;
        let (o, m) = (HALF + 0.22, 0.85);
        let mut running = Vec::new();
        for (a, b) in [((o, -m), (o, m)), ((m, o), (-m, o)), ((-o, m), (-o, -m)), ((-m, -o), (m, -o))] {
            running.extend(arrow(stage, (tx + a.0, ty + a.1), (tx + b.0, ty + b.1), Palette::green())?);
        }
        running.extend(edge_labels(&kit, stage, TURN_AT, &t_along, OUTSIDE_TANGENTS, Palette::green())?);
        running.push(kit.text(stage, &format!("crossing: {}    along: 4 × {} = {}", sum(&t_cross), t_along[0], sum(&t_along)), 23.0, Palette::ink(), v(TURN_AT.0, -2.5))?);
        fade(stage, &running, true)?;
        kit.until_sentence(stage, "19_3", 2)?;
        let div = kit.text(stage, "net outflow per area = divergence", 23.0, Palette::hero(), v(SPREAD_AT.0, -3.15))?;
        let curl = kit.text(stage, "circulation per area = curl", 23.0, Palette::green(), v(TURN_AT.0, -3.15))?;
        fade(stage, &[div, curl], true)?;
        kit.until_sentence(stage, "19_3", 3)?;
        fade(stage, &[question], false)?;
        let parts = kit.text(stage, "divergence: crossing part     curl: along part", 25.0, Palette::ink(), v(-0.5, 2.3))?;
        fade(stage, &[parts], true)?;
        kit.finish(stage)
    }
}

// 20 and 21: four cells A B / C D, two units on a side.
const GRID: (f64, f64) = (-3.4, -0.5);
const SIDE: f64 = 2.0;
const COLUMN: f64 = 3.2;
const CENTRES: [(f64, f64); 4] = [(-4.4, 0.5), (-2.4, 0.5), (-4.4, -1.5), (-2.4, -1.5)];

fn grid(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut mobs = square(stage, GRID, SIDE, Palette::blue(), 2.5)?;
    mobs.push(line(stage, (GRID.0, GRID.1 - SIDE), (GRID.0, GRID.1 + SIDE), Palette::blue(), 2.0)?);
    mobs.push(line(stage, (GRID.0 - SIDE, GRID.1), (GRID.0 + SIDE, GRID.1), Palette::blue(), 2.0)?);
    Ok(mobs)
}

fn amounts(kit: &Kit, stage: &mut Stage<'_>, u: [i64; 4]) -> fmn::Result<Vec<Mob>> {
    u.iter().zip(CENTRES).map(|(&n, c)| kit.text(stage, &hundredths(n), 36.0, Palette::ink(), v(c.0, c.1))).collect()
}

impl SceneConstruct for FiniteVolumes {
    fn name(&self) -> &str { "20_finite_volumes" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (flux, next) = fv_step(FV_BEFORE);
        let title = kit.text(stage, "Finite volumes: one number per cell", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut first = vec![title];
        first.extend(grid(stage)?);
        fade(stage, &first, true)?;
        kit.say(stage, "20_1")?;
        kit.until_sentence(stage, "20_1", 1)?;
        let store = kit.text(stage, "each cell: one number, the amount inside", 23.0, Palette::ink(), v(COLUMN, 2.1))?;
        fade(stage, &[store], true)?;
        kit.until_sentence(stage, "20_1", 2)?;
        let (gx, gy) = GRID;
        let mut faces = vec![
            line(stage, (gx, gy), (gx, gy + SIDE), Palette::hero(), 4.0)?,
            line(stage, (gx, gy - SIDE), (gx, gy), Palette::hero(), 4.0)?,
            line(stage, (gx - SIDE, gy), (gx, gy), Palette::hero(), 4.0)?,
            line(stage, (gx, gy), (gx + SIDE, gy), Palette::hero(), 4.0)?,
        ];
        faces.push(kit.text(stage, "each shared face: one flux, moved across it", 23.0, Palette::hero(), v(COLUMN, 1.45))?);
        fade(stage, &faces, true)?;
        kit.hold(stage)?;
        kit.say(stage, "20_2")?;
        let before = amounts(&kit, stage, FV_BEFORE)?;
        fade(stage, &before, true)?;
        kit.until_sentence(stage, "20_2", 1)?;
        let rule = kit.text(stage, "flux = 0.1 × (difference across the face)", 23.0, Palette::ink(), v(COLUMN, 0.7))?;
        let closed = kit.text(stage, "outer boundary closed", 22.0, Palette::dim(), v(COLUMN, 0.1))?;
        fade(stage, &[rule, closed], true)?;
        kit.until_sentence(stage, "20_2", 2)?;
        let now = kit.text(stage, &format!("total now: {}", hundredths(FV_BEFORE.iter().sum())), 25.0, Palette::ink(), v(COLUMN, -0.6))?;
        let question = kit.text(stage, "total after one step?", 25.0, Palette::hero(), v(COLUMN, -1.25))?;
        fade(stage, &[now, question], true)?;
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        stage.wait(3.0)?;
        kit.say(stage, "20_3")?;
        // Faces in FV_FACES order: A|B (right), A/C (down), B/D (down), C|D (equal, no arrow).
        let mut moved = arrow(stage, (-3.75, 0.5), (-3.05, 0.5), Palette::hero())?;
        moved.extend(arrow(stage, (-4.4, -0.15), (-4.4, -0.85), Palette::hero())?);
        moved.extend(arrow(stage, (-2.4, -0.15), (-2.4, -0.85), Palette::hero())?);
        for (&n, at) in flux.iter().zip([(-3.0, 0.85), (-4.9, -0.2), (-1.9, -0.2)]) {
            moved.push(kit.text(stage, &hundredths(n), 22.0, Palette::hero(), v(at.0, at.1))?);
        }
        fade(stage, &moved, true)?;
        kit.until_sentence(stage, "20_3", 1)?;
        let equal = kit.text(stage, &format!("bottom pair equal: {}", hundredths(flux[3])), 22.0, Palette::dim(), v(GRID.0, -2.85))?;
        fade(stage, &[equal], true)?;
        kit.until_sentence(stage, "20_3", 2)?;
        fade(stage, &before, false)?;
        let after = amounts(&kit, stage, next)?;
        fade(stage, &after, true)?;
        kit.until_sentence(stage, "20_3", 3)?;
        fade(stage, &[question], false)?;
        let kept = kit.text(stage, &format!("total after one step: {}", hundredths(next.iter().sum())), 25.0, Palette::green(), v(COLUMN, -1.25))?;
        let signs = kit.text(stage, "each flux: + on one side, − on the other", 23.0, Palette::ink(), v(COLUMN, -1.95))?;
        fade(stage, &[kept, signs], true)?;
        kit.finish(stage)
    }
}

impl SceneConstruct for OpenBoundary {
    fn name(&self) -> &str { "21_open_boundary" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (_, next) = fv_step(FV_BEFORE);
        let title = kit.text(stage, "One open face", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut first = vec![title];
        first.extend(grid(stage)?);
        let mut cells = amounts(&kit, stage, next)?;
        first.extend(cells.iter().copied());
        fade(stage, &first, true)?;
        kit.say(stage, "21_1")?;
        let mut inflow = arrow(stage, (CENTRES[1].0, 2.4), (CENTRES[1].0, 1.0), Palette::hero())?;
        inflow.push(kit.text(stage, &format!("{} in", hundredths(FV_INFLOW)), 22.0, Palette::hero(), v(CENTRES[1].0 + 0.65, 2.05))?);
        inflow.push(kit.text(stage, &format!("open face: {} flows in", hundredths(FV_INFLOW)), 24.0, Palette::hero(), v(COLUMN, 2.1))?);
        fade(stage, &inflow, true)?;
        kit.until_sentence(stage, "21_1", 1)?;
        fade(stage, &[cells[1]], false)?;
        cells[1] = kit.text(stage, &hundredths(next[1] + FV_INFLOW), 36.0, Palette::hero(), v(CENTRES[1].0, CENTRES[1].1))?;
        let old_total: i64 = next.iter().sum();
        let total = kit.text(stage, &format!("total: {} + {} = {}", hundredths(old_total), hundredths(FV_INFLOW), hundredths(old_total + FV_INFLOW)), 26.0, Palette::green(), v(COLUMN, 1.4))?;
        fade(stage, &[cells[1], total], true)?;
        kit.until_sentence(stage, "21_1", 2)?;
        let cancel = kit.text(stage, "shared faces cancel, as the seams did", 23.0, Palette::ink(), v(COLUMN, 0.7))?;
        fade(stage, &[cancel], true)?;
        kit.until_sentence(stage, "21_1", 3)?;
        let exact = kit.text(stage, "exact up to rounding, for any flux formula", 23.0, Palette::dim(), v(COLUMN, 0.1))?;
        fade(stage, &[exact], true)?;
        kit.hold(stage)?;
        kit.say(stage, "21_2")?;
        let assumed = kit.text(stage, "assumed:", 23.0, Palette::dim(), v(COLUMN, -0.6))?;
        fade(stage, &[assumed], true)?;
        kit.until_sentence(stage, "21_2", 1)?;
        let rows = [
            kit.text(stage, "cells fixed", 22.0, Palette::ink(), v(COLUMN, -1.15))?,
            kit.text(stage, "one flux per shared face, used with both signs", 22.0, Palette::ink(), v(COLUMN, -1.65))?,
            kit.text(stage, "nothing created inside a cell", 22.0, Palette::ink(), v(COLUMN, -2.15))?,
        ];
        fade(stage, &rows, true)?;
        kit.until_sentence(stage, "21_2", 2)?;
        let accuracy = kit.text(stage, "accuracy: flux formula and time step", 22.0, Palette::hero(), v(COLUMN, -2.75))?;
        fade(stage, &[accuracy], true)?;
        kit.hold(stage)?;
        kit.say(stage, "21_3")?;
        let stable = kit.text(stage, "factor 0.1: stable here", 22.0, Palette::dim(), v(GRID.0, -2.95))?;
        fade(stage, &[stable], true)?;
        kit.until_sentence(stage, "21_3", 1)?;
        let limit = kit.text(stage, "equal-grid 4-neighbor diffusion: factor ≤ 1/4", 19.0, Palette::dim(), v(GRID.0, -3.45))?;
        fade(stage, &[limit], true)?;
        kit.until_sentence(stage, "21_3", 2)?;
        let next_up = kit.text(stage, "next: circulation along edges", 24.0, Palette::green(), v(COLUMN, -3.35))?;
        fade(stage, &[next_up], true)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_arrows_stay_inside_their_cell() {
        for field in [spreading as fn(Q, Q) -> (Q, Q), turning] {
            for (i, j) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                let (fx, fy) = field(Q::new(i, 4), Q::new(j, 4));
                let tip = |p: i64, f: Q| 2.0 * HALF * p as f64 / 4.0 + SHOW * f.0 as f64 / f.1 as f64;
                assert!(tip(i, fx).abs() < HALF && tip(j, fy).abs() < HALF, "{i},{j}");
                assert!(fx != Q::new(0, 1) || fy != Q::new(0, 1), "a sample arrow would have zero length");
            }
        }
    }

    #[test]
    fn the_flux_labels_match_the_three_moving_faces() {
        let (flux, next) = fv_step(FV_BEFORE);
        assert_eq!(flux.map(hundredths), ["0.2", "0.3", "0.1", "0"]);
        assert_eq!(next.map(hundredths), ["3.5", "2.1", "1.3", "1.1"]);
        assert_eq!(hundredths(next[1] + FV_INFLOW), "2.35");
    }

    #[test]
    fn the_grid_stays_left_of_the_text_column() {
        assert!(GRID.0 + SIDE < -1.0 && GRID.1 - SIDE > -2.6 && GRID.1 + SIDE < 2.0);
        assert!(TURN_AT.0 + OUTSIDE_TANGENTS[0].0 < 6.5 && SPREAD_AT.0 + BESIDE_NORMALS[2].0 > -6.5);
        assert!(SPREAD_AT.1 + BESIDE_NORMALS[3].1 > -1.6 && TURN_AT.1 + OUTSIDE_TANGENTS[3].1 > -1.6, "edge labels clear the field names");
    }
}
