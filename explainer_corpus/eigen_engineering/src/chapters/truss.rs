//! The truss (storyboard 05 to 07, and faintly behind 02_4): the model's
//! Pratt truss drawn from its own joints and members, its supports, its
//! moves and forces under a load, and the stiffness table that writes them
//! as one equation.
//!
//! Positions are the model's, in metres, placed on screen by a [`Layout`]:
//! nothing here is typed from the picture. Everything that changes is
//! rewritten by one updater from trackers, only when they change.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::props::RowStrips;
use explainer_kit::{Palette, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::arrows::mix;
use super::draw::{arrow_points, corners, faded_line, flat, group, moves, own, panel, squeezed, stretched};
use crate::model::linalg::solve;
use crate::model::truss::{Kind, Truss};

/// Where the truss sits on the screen: joint 0 (the pinned end, at the
/// bottom left) at `origin`, `scale` scene units to a metre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub origin: Vec3,
    pub scale: f64,
}

impl Layout {
    /// The screen point `dx`, `dy` metres from the pinned end.
    pub fn at(&self, dx: f64, dy: f64) -> Vec3 {
        v(self.origin[0] + self.scale * dx, self.origin[1] + self.scale * dy)
    }

    /// Joint `j` of `truss`, at rest.
    pub fn joint(&self, truss: &Truss, j: usize) -> Vec3 {
        let (x, y) = truss.joints[j];
        self.at(x, y)
    }

    /// The width the truss takes on screen.
    pub fn width(&self, truss: &Truss) -> f64 {
        self.scale * truss.span_m()
    }
}

/// How heavy a member is drawn: the chords and end posts, which carry the
/// most steel, over the posts and diagonals.
pub fn member_width(kind: Kind) -> f64 {
    match kind {
        Kind::Bottom | Kind::Top | Kind::EndPost => 6.0,
        Kind::Vertical | Kind::Diagonal => 3.4,
    }
}

/// The truss at rest as one polyline per member, in `colour`, detached.
pub fn outline(stage: &mut Stage<'_>, truss: &Truss, layout: Layout, colour: Srgb, opacity: f64) -> Vec<Mob> {
    truss
        .members
        .iter()
        .map(|m| {
            let line = polyline(vec![layout.joint(truss, m.a), layout.joint(truss, m.b)])
                .style(faded_line(colour, member_width(m.kind), opacity))
                .build();
            stage.arena_mut().add(line)
        })
        .collect()
}

// ------------------------------------------------------------ the geometry

/// The free directions in the order of the stiffness table's rows: each is a
/// joint and whether it is the up-and-down direction.
pub fn free_dirs(truss: &Truss) -> Vec<(usize, bool)> {
    truss.free().iter().map(|&d| (d / 2, d % 2 == 1)).collect()
}

/// Each member's contribution to the stiffness table, as `(row, column,
/// value)` in the free directions' numbering (the supports' rows and columns
/// struck out): the 4 by 4 block `k g gᵀ` of its spring, `g = (-c, -s, c, s)`.
pub fn member_entries(truss: &Truss) -> Vec<Vec<(usize, usize, f64)>> {
    let free = truss.free();
    let index = |d: usize| free.iter().position(|&f| f == d);
    truss
        .members
        .iter()
        .map(|m| {
            let (c, s) = truss.direction(m);
            let g = [-c, -s, c, s];
            let at = [2 * m.a, 2 * m.a + 1, 2 * m.b, 2 * m.b + 1];
            let k = truss.spring(m);
            let mut out = Vec::new();
            for (i, &di) in at.iter().enumerate() {
                for (j, &dj) in at.iter().enumerate() {
                    if let (Some(r), Some(cc)) = (index(di), index(dj)) {
                        out.push((r, cc, k * g[i] * g[j]));
                    }
                }
            }
            out
        })
        .collect()
}

/// The table with the first `upto` members written into it (a fraction of
/// the next one, `upto` rounded down being how many are in whole), `n` by
/// `n`. `upto` at the number of members is the stiffness table K.
pub fn partial_table(entries: &[Vec<(usize, usize, f64)>], upto: f64, n: usize) -> Vec<Vec<f64>> {
    let mut t = vec![vec![0.0; n]; n];
    let whole = upto.floor().max(0.0) as usize;
    for (m, es) in entries.iter().enumerate() {
        let share = if m < whole {
            1.0
        } else if m == whole {
            upto - upto.floor()
        } else {
            0.0
        };
        if share > 0.0 {
            for &(r, c, x) in es {
                t[r][c] += share * x;
            }
        }
    }
    t
}

/// K⁻¹ column by column, `n` by `n`: the moves for a unit push on each free
/// direction.
pub fn stiffness_inverse(truss: &Truss) -> Vec<Vec<f64>> {
    let k = truss.stiffness();
    let n = k.n;
    let mut inv = vec![vec![0.0; n]; n];
    for j in 0..n {
        let mut unit = vec![0.0; n];
        unit[j] = 1.0;
        let column = solve(&k, &unit).expect("a supported truss is not singular");
        for (i, x) in column.into_iter().enumerate() {
            inv[i][j] = x;
        }
    }
    inv
}

/// How many entries of `table` are not zero (to within a billionth of its
/// largest).
pub fn nonzero_count(table: &[Vec<f64>]) -> usize {
    let top = table.iter().flatten().fold(0.0_f64, |m, x| m.max(x.abs()));
    table.iter().flatten().filter(|x| x.abs() > 1e-9 * top).count()
}

/// The colour of one cell of a table: dark where it is zero, brighter the
/// larger it is against `max` (a gentle curve, so the small ones still show).
pub fn cell_colour(value: f64, max: f64) -> Srgb {
    let share = if max > 0.0 {
        (value.abs() / max).clamp(0.0, 1.0).powf(0.55)
    } else {
        0.0
    };
    let dark = Srgb {
        r: 0.105,
        g: 0.133,
        b: 0.188,
    };
    mix(dark, Palette::ink(), share)
}

/// A member's look under force `newtons` against the largest force `scale`:
/// squeezed magenta and heavy, stretched teal and heavy, near nothing thin
/// and grey (storyboard 05_8).
pub fn force_look(newtons: f64, scale: f64) -> (Srgb, f64) {
    let share = if scale > 0.0 {
        (newtons.abs() / scale).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let role = if newtons < 0.0 { squeezed() } else { stretched() };
    (mix(Palette::dim(), role, share), 2.2 + 8.0 * share)
}

/// The screen place of joint `j` when `field` (the moves of every joint
/// direction, metres) is applied `amp` of the way and magnified `mag`
/// times.
pub fn displaced(truss: &Truss, layout: Layout, field: &[f64], mag: f64, amp: f64, j: usize) -> Vec3 {
    let (x, y) = truss.joints[j];
    layout.at(x + amp * mag * field[2 * j], y + amp * mag * field[2 * j + 1])
}

// ------------------------------------------------------------- static parts

/// The pin under joint 0 and the roller under the last bottom joint, with
/// the ground line under the roller, detached.
pub fn supports(stage: &mut Stage<'_>, truss: &Truss, layout: Layout) -> Vec<Mob> {
    let ink = Palette::ink();
    let size = 0.2;
    let left = layout.joint(truss, 0);
    let right = layout.joint(truss, crate::model::truss::PANELS);
    let pin = Polygon::new(vec![
        v(left[0], left[1]),
        v(left[0] - size, left[1] - 1.7 * size),
        v(left[0] + size, left[1] - 1.7 * size),
    ])
    .style(faded_line(ink, 3.0, 1.0))
    .build();
    let pin_base = polyline(vec![
        v(left[0] - 1.5 * size, left[1] - 1.7 * size),
        v(left[0] + 1.5 * size, left[1] - 1.7 * size),
    ])
    .style(faded_line(ink, 3.0, 1.0))
    .build();
    let roller = Circle::new()
        .radius(0.7 * size)
        .arc_center(v(right[0], right[1] - 0.7 * size))
        .style(faded_line(ink, 3.0, 1.0))
        .build();
    let ground = polyline(vec![
        v(right[0] - 1.5 * size, right[1] - 1.4 * size),
        v(right[0] + 1.5 * size, right[1] - 1.4 * size),
    ])
    .style(faded_line(ink, 3.0, 1.0))
    .build();
    [pin, pin_base, roller, ground]
        .into_iter()
        .map(|m| stage.arena_mut().add(m))
        .collect()
}

/// A dimension line from `a` to `b` with a short tick across each end,
/// detached.
pub fn dimension(stage: &mut Stage<'_>, a: Vec3, b: Vec3, colour: Srgb) -> Vec<Mob> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    let (nx, ny) = (-dy / len * 0.1, dx / len * 0.1);
    let line = polyline(vec![a, b]).style(faded_line(colour, 2.5, 1.0)).build();
    let ticks = [a, b].map(|p| {
        polyline(vec![v(p[0] - nx, p[1] - ny), v(p[0] + nx, p[1] + ny)])
            .style(faded_line(colour, 2.5, 1.0))
            .build()
    });
    let mut out = vec![stage.arena_mut().add(line)];
    out.extend(ticks.into_iter().map(|t| stage.arena_mut().add(t)));
    out
}

/// A downward load arrow of `length` scene units hanging from `at`, in
/// `colour`, detached: the force, pulling the joint down.
pub fn load_arrow(stage: &mut Stage<'_>, at: Vec3, length: f64, colour: Srgb) -> Mob {
    let a = polyline(arrow_points(at, v(at[0], at[1] - length), 0.2))
        .style(faded_line(colour, 6.0, 1.0))
        .build();
    stage.arena_mut().add(a)
}

// ------------------------------------------------------------ the picture

/// The truss, drawn from the model, with the moves of one or two loads
/// (`fields`, magnified by `mags`) ready to apply.
pub struct TrussPic {
    pub truss: Truss,
    pub layout: Layout,
    pub members: Vec<Mob>,
    pub joints: Vec<Mob>,
    /// One blue arrow per free direction, in the table's order.
    pub arrows: Vec<Mob>,
    /// How much of each field is applied, 0 to 1.
    pub amp: Mob,
    /// Which field: 0 the first, 1 the second.
    pub field: Mob,
    /// The arrows: 0 a unit arrow on each free direction ("it can move
    /// here"), 1 the move itself, magnified; and how much of them shows.
    pub arrows_mode: Mob,
    pub arrows_on: Mob,
    /// How much of the members' colour and weight by force shows, 0 to 1.
    pub force_on: Mob,
    /// How many members are drawn, 0 to the number of members: member `i`
    /// is drawn while this passes `i` and `i + 1`.
    pub drawn: Mob,
    /// The member being written into the table, lit: the member whose index
    /// is this value rounded down; negative for none.
    pub lit: Mob,
    /// How much the members are tinted gold, an own pattern's colour, 0 to 1.
    pub tint: Mob,
    anchor: Mob,
}

/// The unit arrow of a free direction, in scene units.
const UNIT_ARROW: f64 = 0.3;

impl TrussPic {
    /// Build the picture, detached. `fields` are the moves of every joint
    /// direction (metres) for two loads (the second may repeat the first),
    /// `mags` how many times each is magnified on screen, `forces` the
    /// members' forces (newtons) under the second load.
    pub fn build(
        stage: &mut Stage<'_>,
        truss: &Truss,
        layout: Layout,
        fields: [Vec<f64>; 2],
        mags: [f64; 2],
        forces: Vec<f64>,
    ) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let mut members = Vec::new();
        for m in &truss.members {
            let line = polyline(vec![layout.joint(truss, m.a), layout.joint(truss, m.b)])
                .style(faded_line(ink, member_width(m.kind), 0.0))
                .build();
            members.push(stage.arena_mut().add(line));
        }
        let mut joints = Vec::new();
        for j in 0..truss.joints.len() {
            let d = Dot::new()
                .point(layout.joint(truss, j))
                .radius(0.07)
                .style(flat(ink, 0.0))
                .build();
            joints.push(stage.arena_mut().add(d));
        }
        let dirs = free_dirs(truss);
        let mut arrows = Vec::new();
        for &(j, vertical) in &dirs {
            let p = layout.joint(truss, j);
            let tip = if vertical {
                v(p[0], p[1] + UNIT_ARROW)
            } else {
                v(p[0] + UNIT_ARROW, p[1])
            };
            let a = polyline(arrow_points(p, tip, 0.12))
                .style(faded_line(moves(), 3.5, 0.0))
                .build();
            arrows.push(stage.arena_mut().add(a));
        }
        let amp = stage.add_value_tracker(0.0);
        let field = stage.add_value_tracker(0.0);
        let arrows_mode = stage.add_value_tracker(0.0);
        let arrows_on = stage.add_value_tracker(0.0);
        let force_on = stage.add_value_tracker(0.0);
        let drawn = stage.add_value_tracker(0.0);
        let lit = stage.add_value_tracker(-1.0);
        let tint = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());

        let t = truss.clone();
        let (members_m, joints_m, arrows_m) = (members.clone(), joints.clone(), arrows.clone());
        let biggest = forces.iter().fold(0.0_f64, |m, f| m.max(f.abs()));
        let mut last = [f64::NAN; 8];
        stage.add_updater(
            anchor,
            move |s, _| {
                let val = |s: &Arena, m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let now = [
                    val(s, amp),
                    val(s, field),
                    val(s, arrows_mode),
                    val(s, arrows_on),
                    val(s, force_on),
                    val(s, drawn),
                    val(s, lit),
                    val(s, tint),
                ];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let [amp_v, field_v, mode, a_on, f_on, drawn_v, lit_v, tint_v] = now;
                let pick = usize::from(field_v > 0.5);
                let (u, mag) = (&fields[pick], mags[pick]);
                // members: ends displaced, drawn as far as `drawn`, coloured by force
                let mut seen = vec![0.0_f64; t.joints.len()];
                for (i, m) in t.members.iter().enumerate() {
                    let a = displaced(&t, layout, u, mag, amp_v, m.a);
                    let b = displaced(&t, layout, u, mag, amp_v, m.b);
                    let _ = s.set_points(members_m[i], &corners(&[a, b]));
                    let shown = (drawn_v - i as f64).clamp(0.0, 1.0);
                    seen[m.a] = seen[m.a].max(shown);
                    seen[m.b] = seen[m.b].max(shown);
                    let (colour, width) = if lit_v >= 0.0 && lit_v.floor() as usize == i {
                        (Palette::ink(), member_width(m.kind) + 3.0)
                    } else {
                        let base = (Palette::ink(), member_width(m.kind));
                        let (fc, fw) = force_look(forces[i], biggest);
                        // an own pattern is drawn gold: tinted last, over the force colours
                        (mix(mix(base.0, fc, f_on), own(), tint_v), base.1 + (fw - base.1) * f_on)
                    };
                    s.set_stroke(members_m[i], Some(colour), Some(width), Some(shown), None, false);
                }
                for (j, &d) in joints_m.iter().enumerate() {
                    let p = displaced(&t, layout, u, mag, amp_v, j);
                    s.move_to(d, p, ORIGIN);
                    s.set_fill(d, None, Some(seen[j]), None, false);
                }
                // one arrow per free direction: the unit "it can move here",
                // or the move itself, magnified
                for (k, (j, vertical)) in free_dirs(&t).into_iter().enumerate() {
                    let rest = layout.joint(&t, j);
                    let comp = if vertical { u[2 * j + 1] } else { u[2 * j] };
                    let actual = mag * amp_v * comp * layout.scale;
                    let unit = UNIT_ARROW;
                    let len = unit + (actual - unit) * mode;
                    let tip = if vertical {
                        v(rest[0], rest[1] + len)
                    } else {
                        v(rest[0] + len, rest[1])
                    };
                    let _ = s.set_points(arrows_m[k], &corners(&arrow_points(rest, tip, 0.12)));
                    s.set_stroke(arrows_m[k], None, None, Some(a_on * seen[j]), None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            truss: truss.clone(),
            layout,
            members,
            joints,
            arrows,
            amp,
            field,
            arrows_mode,
            arrows_on,
            force_on,
            drawn,
            lit,
            tint,
            anchor,
        })
    }

    /// Add the updater and every part to the scene (the parts are at
    /// opacity 0 until `drawn` rises).
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        for &m in self.members.iter().chain(&self.joints).chain(&self.arrows) {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    /// Stop the updater.
    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// Every part, for fading out together.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.members.clone();
        all.extend(&self.joints);
        all.extend(&self.arrows);
        all
    }
}

// --------------------------------------------------------- the table picture

/// The 21 by 21 stiffness table as a lattice of cells (`RowStrips`: one
/// strip a row, recoloured in place): members written into it one after
/// another with `progress`, and turned into its inverse with
/// `inverse_mix`.
pub struct StiffnessTable {
    pub group: Mob,
    pub rows: Vec<Mob>,
    /// How many members are written in, 0 to the number of members; the
    /// one being written (the fraction) shows in white.
    pub progress: Mob,
    /// 0 the table K, 1 its inverse.
    pub inverse_mix: Mob,
    pub top_left: Vec3,
    pub cell: f64,
    pub n: usize,
    anchor: Mob,
}

impl StiffnessTable {
    pub fn build(stage: &mut Stage<'_>, truss: &Truss, top_left: Vec3, cell: f64) -> fmn::Result<Self> {
        let entries = member_entries(truss);
        let n = truss.free().len();
        let full = partial_table(&entries, entries.len() as f64, n);
        let inverse = stiffness_inverse(truss);
        let k_max = full.iter().flatten().fold(0.0_f64, |m, x| m.max(x.abs()));
        let i_max = inverse.iter().flatten().fold(0.0_f64, |m, x| m.max(x.abs()));
        let colours = |t: &[Vec<f64>], max: f64| -> Vec<Vec<Srgb>> {
            t.iter()
                .map(|r| r.iter().map(|&x| cell_colour(x, max)).collect())
                .collect()
        };
        let blank = vec![vec![0.0; n]; n];
        let strips = RowStrips::build(stage, top_left, (cell, cell), &colours(&blank, 1.0))?;
        let (group, rows) = (strips.group, strips.rows.clone());
        let progress = stage.add_value_tracker(0.0);
        let inverse_mix = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let members = entries.len() as f64;
        let mut last = [f64::NAN; 2];
        stage.add_updater(
            anchor,
            move |s, _| {
                let p = s.tracker_value(progress).unwrap_or(0.0);
                let mixed = s.tracker_value(inverse_mix).unwrap_or(0.0);
                if p.to_bits() == last[0].to_bits() && mixed.to_bits() == last[1].to_bits() {
                    return;
                }
                last = [p, mixed];
                let upto = p.clamp(0.0, members);
                let table = partial_table(&entries, upto, n);
                let whole = upto.floor() as usize;
                let live: Vec<Vec<Srgb>> = if whole < entries.len() && upto - upto.floor() > 0.0 {
                    // the member being written lights white, over what is there
                    let mut one = vec![vec![0.0; n]; n];
                    for &(r, c, x) in &entries[whole] {
                        one[r][c] += x;
                    }
                    let frac = upto - upto.floor();
                    (0..n)
                        .map(|r| {
                            (0..n)
                                .map(|c| {
                                    let base = cell_colour(table[r][c] - frac * one[r][c], k_max);
                                    if one[r][c] != 0.0 {
                                        mix(base, Palette::ink(), frac.max(0.3))
                                    } else {
                                        base
                                    }
                                })
                                .collect()
                        })
                        .collect()
                } else {
                    colours(&table, k_max)
                };
                let target: Vec<Vec<Srgb>> = if mixed > 0.0 {
                    let flipped = colours(&inverse, i_max);
                    live.iter()
                        .zip(&flipped)
                        .map(|(a, b)| a.iter().zip(b).map(|(&x, &y)| mix(x, y, mixed)).collect())
                        .collect()
                } else {
                    live
                };
                strips.recolor(s, &target);
            },
            true,
        )?;
        Ok(Self {
            group,
            rows,
            progress,
            inverse_mix,
            top_left,
            cell,
            n,
            anchor,
        })
    }

    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// The scene point of the cell at `row`, `col`.
    pub fn cell_at(&self, row: usize, col: usize) -> Vec3 {
        v(
            self.top_left[0] + self.cell * (col as f64 + 0.5),
            self.top_left[1] - self.cell * (row as f64 + 0.5),
        )
    }

    /// The width and height the lattice takes.
    pub fn size(&self) -> f64 {
        self.cell * self.n as f64
    }

    /// A strip down the diagonal from the top-left to the bottom-right
    /// corner: the mirror line, detached.
    pub fn diagonal(&self, stage: &mut Stage<'_>) -> Mob {
        let a = self.top_left;
        let b = v(a[0] + self.size(), a[1] - self.size());
        let l = polyline(vec![a, b]).style(faded_line(Palette::ink(), 2.5, 0.0)).build();
        stage.arena_mut().add(l)
    }
}

/// A rounded dark back for a note drawn over a picture, detached.
pub fn plate(stage: &mut Stage<'_>, w: f64, h: f64, at: Vec3, edge: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

pub fn together(stage: &mut Stage<'_>, parts: &[Mob]) -> fmn::Result<Mob> {
    group(stage, parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::{fact, numbers};
    use crate::model::truss::{DEPTH_M, PANEL_M, PANELS};

    #[test]
    fn the_layout_places_the_model_in_metres() {
        let t = Truss::pratt();
        let l = Layout {
            origin: v(-3.0, -1.0),
            scale: 0.25,
        };
        assert_eq!(l.joint(&t, 0), v(-3.0, -1.0));
        assert_eq!(l.joint(&t, PANELS), v(-3.0 + 0.25 * PANELS as f64 * PANEL_M, -1.0));
        // the first top joint is one panel in and one depth up
        let top = l.joint(&t, PANELS + 1);
        assert!((top[0] - (-3.0 + 0.25 * PANEL_M)).abs() < 1e-12 && (top[1] - (-1.0 + 0.25 * DEPTH_M)).abs() < 1e-12);
        assert!((l.width(&t) - 0.25 * 24.0).abs() < 1e-12);
    }

    #[test]
    fn chords_are_drawn_heavier_than_the_web() {
        assert!(member_width(Kind::Bottom) > member_width(Kind::Diagonal));
        assert_eq!(member_width(Kind::Top), member_width(Kind::EndPost));
        assert_eq!(member_width(Kind::Vertical), member_width(Kind::Diagonal));
    }

    /// 05_3: twelve joints, two directions each, the supports hold three:
    /// twenty-one arrows, none on a held direction (the pin has none, the
    /// roller only sideways).
    #[test]
    fn there_is_an_arrow_for_each_of_the_twenty_one_free_directions() {
        let t = Truss::pratt();
        let dirs = free_dirs(&t);
        assert_eq!(dirs.len(), 21);
        assert_eq!(fact("K-T1.free").unwrap(), "21");
        assert_eq!(fact("K-T1.joints").unwrap(), "12");
        assert_eq!(fact("K-T1.held").unwrap(), "3");
        assert!(!dirs.iter().any(|&(j, _)| j == 0), "the pin holds both ways: no arrow");
        let roller: Vec<_> = dirs.iter().filter(|&&(j, _)| j == PANELS).collect();
        assert_eq!(roller, [&(PANELS, false)], "the roller holds up and down only");
    }

    /// 05_4: the table the members write into is the model's K, with the
    /// facts' 441 entries of which 97 are not zero; it is symmetric; and
    /// its inverse has none zero.
    #[test]
    fn the_members_write_the_models_table_and_its_inverse_is_full() {
        let t = Truss::pratt();
        let entries = member_entries(&t);
        let n = t.free().len();
        let k = partial_table(&entries, entries.len() as f64, n);
        let model = t.stiffness();
        for (i, row) in k.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                assert!((x - model.at(i, j)).abs() < 1e-6 * model.at(0, 0).abs(), "{i},{j}");
            }
        }
        assert_eq!((n * n).to_string(), fact("K-T2.entries").unwrap());
        assert_eq!(nonzero_count(&k).to_string(), fact("K-T2.nonzero").unwrap());
        let inv = stiffness_inverse(&t);
        assert_eq!(nonzero_count(&inv).to_string(), fact("K-T2.inverse_nonzero").unwrap());
        // symmetric, as the fact says
        assert_eq!(fact("K-T2.symmetric").unwrap(), "true");
        for (i, row) in k.iter().enumerate() {
            for (j, &x) in row.iter().enumerate().take(i) {
                assert!((x - k[j][i]).abs() < 1e-6 * k[0][0].abs());
            }
        }
        // K times its inverse is the identity
        for (i, row) in k.iter().enumerate() {
            for j in 0..n {
                let s: f64 = row.iter().zip(&inv).map(|(&a, inv_row)| a * inv_row[j]).sum();
                let want = if i == j { 1.0 } else { 0.0 };
                assert!((s - want).abs() < 1e-6, "{i},{j}: {s}");
            }
        }
    }

    /// Members are written in one at a time: with none written the table
    /// is empty, a fraction of one member is that fraction of its block,
    /// and each member adds to the cells it touches only.
    #[test]
    fn writing_a_member_in_touches_only_its_own_joints_rows() {
        let t = Truss::pratt();
        let entries = member_entries(&t);
        let n = t.free().len();
        assert_eq!(nonzero_count(&partial_table(&entries, 0.0, n)), 0);
        let half = partial_table(&entries, 0.5, n);
        let one = partial_table(&entries, 1.0, n);
        for (a, b) in half.iter().flatten().zip(one.iter().flatten()) {
            assert!((2.0 * a - b).abs() < 1e-6 * (b.abs() + 1.0));
        }
        // the first member (a bottom member between joints 0 and 1) touches
        // only the rows of joint 1 (joint 0 is the pin: struck out)
        let free = free_dirs(&t);
        for (r, c, _) in &entries[0] {
            assert_eq!(free[*r].0, 1);
            assert_eq!(free[*c].0, 1);
        }
        // its stiffness is the fact's 300 kN per mm
        let k_per_mm = numbers("K-T1.spring_kn_per_mm").unwrap()[0];
        let along: f64 = entries[0]
            .iter()
            .filter(|&&(r, c, _)| r == c && !free[r].1)
            .map(|e| e.2)
            .sum();
        assert!((along / 1e6 - k_per_mm).abs() < 1e-6, "{along}");
    }

    /// 05_8: the colouring is by force: squeezed magenta, stretched teal,
    /// grey and thin where the member carries nothing.
    #[test]
    fn members_take_colour_and_weight_from_their_force() {
        let (squeeze, heavy) = force_look(-450e3, 450e3);
        let (pull, pulled) = force_look(400e3, 450e3);
        let (none, thin) = force_look(0.0, 450e3);
        assert!((squeeze.r - squeezed().r).abs() < 1e-12);
        // 400 of 450 is most of the way to the role's teal, from grey
        let (grey, teal) = (Palette::dim(), stretched());
        let share = 400.0 / 450.0;
        assert!((pull.g - (grey.g + (teal.g - grey.g) * share)).abs() < 1e-12);
        assert_eq!(none, Palette::dim());
        assert!(heavy > pulled && pulled > thin, "{heavy} {pulled} {thin}");
        assert!(thin < member_width(Kind::Vertical));
    }

    #[test]
    fn a_cell_is_brighter_the_stiffer_and_dark_at_zero() {
        let top = 300.0;
        let dark = cell_colour(0.0, top);
        let mid = cell_colour(100.0, top);
        let full = cell_colour(300.0, top);
        assert!(dark.r < mid.r && mid.r < full.r);
        assert!((full.r - Palette::ink().r).abs() < 1e-12);
        assert_eq!(
            cell_colour(-300.0, top),
            full,
            "a negative entry is as bright as its size"
        );
    }

    #[test]
    fn the_displaced_joint_follows_the_field_and_the_amplitude() {
        let t = Truss::pratt();
        let l = Layout {
            origin: v(0.0, 0.0),
            scale: 0.5,
        };
        let mut field = vec![0.0; 2 * t.joints.len()];
        field[2 * 3 + 1] = -0.018;
        assert_eq!(displaced(&t, l, &field, 100.0, 0.0, 3), l.joint(&t, 3));
        let p = displaced(&t, l, &field, 100.0, 1.0, 3);
        assert!((p[1] - (l.joint(&t, 3)[1] - 0.5 * 100.0 * 0.018)).abs() < 1e-12);
        assert_eq!(p[0], l.joint(&t, 3)[0]);
    }

    /// The members really follow the moves: on a bare stage (no render),
    /// a member's far end is the displaced joint, the joint's dot is on it
    /// and a free direction's arrow is as long as the magnified move; every
    /// run written is an odd quad path.
    #[test]
    fn the_members_and_arrows_really_follow_the_moves() {
        crate::chapters::carts::probe::run_scene(|stage| {
            let t = Truss::pratt();
            let l = Layout {
                origin: v(-2.0, -1.0),
                scale: 0.2,
            };
            let deck = t.all_moves(&t.moves(&t.deck_load()));
            let pic = TrussPic::build(
                stage,
                &t,
                l,
                [deck.clone(), deck.clone()],
                [100.0, 100.0],
                vec![0.0; t.members.len()],
            )?;
            pic.start(stage)?;
            stage.set_tracker_value(pic.drawn, t.members.len() as f64)?;
            stage.set_tracker_value(pic.amp, 1.0)?;
            stage.set_tracker_value(pic.arrows_mode, 1.0)?;
            stage.set_tracker_value(pic.arrows_on, 1.0)?;
            stage.wait(0.2)?;
            // member 0 runs from the pinned joint 0 to joint 1, which sags
            let pts = stage.get_points(pic.members[0]).expect("a member has points");
            assert_eq!(pts.len() % 2, 1);
            let want = displaced(&t, l, &deck, 100.0, 1.0, 1);
            let end = pts[pts.len() - 1];
            assert!(
                (end[0] - want[0]).abs() < 1e-5 && (end[1] - want[1]).abs() < 1e-5,
                "{end:?} {want:?}"
            );
            assert!(end[1] < l.joint(&t, 1)[1] - 0.01, "the joint did not sag");
            let dot = stage.get_center(pic.joints[1]);
            assert!(
                (dot[0] - want[0]).abs() < 1e-4 && (dot[1] - want[1]).abs() < 1e-4,
                "{dot:?}"
            );
            // the arrow of joint 1's up-and-down direction: as long as the magnified move
            let dirs = free_dirs(&t);
            let k = dirs
                .iter()
                .position(|&(j, vertical)| j == 1 && vertical)
                .expect("joint 1 moves vertically");
            let arrow = stage.get_points(pic.arrows[k]).expect("an arrow has points");
            let rest = l.joint(&t, 1);
            let length = arrow[2][1] - rest[1];
            assert!((length - 100.0 * deck[2 + 1] * l.scale).abs() < 1e-5, "{length}");
            Ok(())
        });
    }
}
