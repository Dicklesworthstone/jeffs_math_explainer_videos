//! Chapter 16, quotients. The recorded beats (representative failure,
//! conjugation, normality, the well-definedness proof, the R/F table) are
//! kept; four mechanisms are added. 16_14 multiplies every representative of
//! rH by every representative and colours the four products by coset (two
//! colours: no operation). 16_15 colours all sixty-four products of the
//! square by rotation/flip and regroups the rows and columns until four solid
//! blocks appear; 16_16 shrinks each block to one cell of the R/F table.
//! 16_17 sorts the same table by the left cosets of {e, s}: half of the
//! blocks are mixed, one of them is 16_14's failure, so nothing can shrink.
use crate::d4_group::{self as d4g, ALL, E, Elem, R, R2, R2S, R3, R3S, RS, S};
use crate::{tile_actor, tile_model};
use explainer_kit::{Kit, Palette, Timed, play, stroke_style, tween, v};
use fmn::prelude::*;

pub struct Quotients {
    pub kit: Kit,
}

fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in items {
        if !show {
            stage.clear_updaters(item, true);
        }
        let a = if show {
            fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?
        };
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// H = {e, s}, the subgroup whose cosets fail.
pub(crate) const FLIP_H: [Elem; 2] = [E, S];
/// The rows and columns of the table, sorted into the left cosets of H.
pub(crate) const COSET_ORDER: [Elem; 8] = [E, S, R, RS, R2, R2S, R3, R3S];
/// The rows and columns sorted into rotations, then flips.
pub(crate) const ROTATION_ORDER: [Elem; 8] = ALL;

const TABLE_ORIGIN: (f64, f64) = (-3.0, 2.25);
const TABLE_CELL: f64 = 0.66;
/// Where canonical 16_11 draws its two-by-two R/F table.
fn small_cell(row: usize, col: usize) -> (f64, f64) {
    (0.5 + 2.0 * col as f64, 0.4 - 1.3 * row as f64)
}

/// Shrink factor from a four-by-four block to one R/F cell.
const SHRINK: f64 = 0.42;

/// Where the entry a·b lands when its rotation/flip block shrinks onto
/// its R/F cell, keeping its offset from the block centre.
fn shrunk_position(a: Elem, b: Elem) -> (f64, f64) {
    let side = 4.0 * TABLE_CELL;
    let (bi, bj) = (usize::from(a.flip), usize::from(b.flip));
    let center = (
        TABLE_ORIGIN.0 + side * (bj as f64 + 0.5),
        TABLE_ORIGIN.1 - side * (bi as f64 + 0.5),
    );
    let here = cell_at(position(&ROTATION_ORDER, a), position(&ROTATION_ORDER, b));
    let target = small_cell(bi, bj);
    (
        target.0 + SHRINK * (here.0 - center.0),
        target.1 + SHRINK * (here.1 - center.1),
    )
}

fn cell_at(i: usize, j: usize) -> (f64, f64) {
    (
        TABLE_ORIGIN.0 + TABLE_CELL * (j as f64 + 0.5),
        TABLE_ORIGIN.1 - TABLE_CELL * (i as f64 + 0.5),
    )
}

fn position(order: &[Elem], g: Elem) -> usize {
    order.iter().position(|&x| x == g).expect("every move has a row")
}

fn rotation_color(g: Elem) -> Srgb {
    if g.flip { Palette::gold() } else { Palette::blue() }
}

/// Colour of the left coset gH of H = {e, s}: the rack column of g.
fn coset_color(g: Elem) -> Srgb {
    [Palette::hero(), Palette::blue(), Palette::green(), Palette::gold()][usize::from(g.turns)]
}

/// Which block of `size` rows a move belongs to in `order`.
fn block_of(order: &[Elem], g: Elem, size: usize) -> usize {
    position(order, g) / size
}

/// Blocks of the table (in `order`, blocks of `size`) whose products do not
/// all share one class under `class`.
pub(crate) fn mixed_blocks(order: &[Elem], size: usize, class: impl Fn(Elem) -> u8) -> Vec<(usize, usize)> {
    let blocks = order.len() / size;
    let mut out = Vec::new();
    for bi in 0..blocks {
        for bj in 0..blocks {
            let mut seen = std::collections::BTreeSet::new();
            for &a in &order[bi * size..(bi + 1) * size] {
                for &b in &order[bj * size..(bj + 1) * size] {
                    seen.insert(class(a.mul(b)));
                }
            }
            if seen.len() > 1 {
                out.push((bi, bj));
            }
        }
    }
    out
}

/// A square filled with `color` at `alpha`, edged in `edge`.
fn filled(
    stage: &mut Stage<'_>,
    at: (f64, f64),
    size: (f64, f64),
    color: Srgb,
    alpha: f64,
    edge: Srgb,
) -> fmn::Result<Mob> {
    let shape = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.06)
        .style(Style::default().fill(color, alpha).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(v(at.0, at.1));
    Ok(stage.arena_mut().add(shape))
}

/// One product on screen: the square that is painted, and the group
/// (square and name) that moves.
#[derive(Clone, Copy)]
struct Entry {
    square: Mob,
    group: Mob,
}

/// Paint every entry's square with the colour of its product.
fn paint(
    stage: &mut Stage<'_>,
    entries: &[((Elem, Elem), Entry)],
    color_of: fn(Elem) -> Srgb,
    lag: f64,
) -> fmn::Result<()> {
    let mut paints: Vec<Box<dyn Animation>> = Vec::new();
    for &((a, b), entry) in entries {
        let p = entry.square.animate().set_fill(Some(color_of(a.mul(b))), Some(0.55))?;
        paints.push(stage.prepare(p)?);
    }
    let group = explainer_kit::lagged(stage, paints, lag)?;
    play!(stage; group);
    Ok(())
}

fn outline(stage: &mut Stage<'_>, at: (f64, f64), size: (f64, f64), color: Srgb, width: f64) -> fmn::Result<Mob> {
    let shape = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.08)
        .style(stroke_style(color, width))
        .build()?
        .moved_to(v(at.0, at.1));
    Ok(stage.arena_mut().add(shape))
}

/// The sixty-four-entry table in `order`: one group per product (a filled
/// square in `color_of(product)` and the product's name), plus headers.
struct Table {
    headers: Vec<(Elem, bool, Mob)>,
    cells: Vec<((Elem, Elem), Entry)>,
}

impl Table {
    fn new(stage: &mut Stage<'_>, kit: &Kit, order: &[Elem], color_of: fn(Elem) -> Srgb) -> fmn::Result<Self> {
        let mut headers = Vec::new();
        for (index, &g) in order.iter().enumerate() {
            let (x, _) = cell_at(0, index);
            let top = kit.text(stage, g.name(), 24.0, Palette::ink(), v(x, TABLE_ORIGIN.1 + 0.3))?;
            let (_, y) = cell_at(index, 0);
            let side = kit.text(stage, g.name(), 24.0, Palette::ink(), v(TABLE_ORIGIN.0 - 0.4, y))?;
            headers.push((g, true, top));
            headers.push((g, false, side));
        }
        let mut cells = Vec::new();
        for (i, &a) in order.iter().enumerate() {
            for (j, &b) in order.iter().enumerate() {
                let product = a.mul(b);
                let at = cell_at(i, j);
                let square = filled(
                    stage,
                    at,
                    (TABLE_CELL - 0.06, TABLE_CELL - 0.06),
                    color_of(product),
                    0.0,
                    Palette::faint(),
                )?;
                let name = kit.text(stage, product.name(), 24.0, Palette::ink(), v(at.0, at.1))?;
                let group = d4g::group_of(stage, &[square, name])?;
                cells.push(((a, b), Entry { square, group }));
            }
        }
        Ok(Self { headers, cells })
    }

    fn all(&self) -> Vec<Mob> {
        let mut out: Vec<Mob> = self.headers.iter().map(|h| h.2).collect();
        out.extend(self.cells.iter().map(|c| c.1.group));
        out
    }

    /// Move every row, column and header to its place in `order`.
    fn regroup(&self, stage: &mut Stage<'_>, order: &[Elem], run_time: f64) -> fmn::Result<()> {
        let mut moves = Vec::new();
        for &((a, b), entry) in &self.cells {
            moves.push((entry.group, cell_at(position(order, a), position(order, b))));
        }
        for &(g, top, header) in &self.headers {
            let index = position(order, g);
            let at = if top {
                (cell_at(0, index).0, TABLE_ORIGIN.1 + 0.3)
            } else {
                (TABLE_ORIGIN.0 - 0.4, cell_at(index, 0).1)
            };
            moves.push((header, at));
        }
        d4g::glide_all(stage, &moves, run_time)
    }
}

fn block_outlines(stage: &mut Stage<'_>, size: usize, color: impl Fn(usize, usize) -> Srgb) -> fmn::Result<Vec<Mob>> {
    let blocks = 8 / size;
    let side = TABLE_CELL * size as f64;
    let mut out = Vec::new();
    for bi in 0..blocks {
        for bj in 0..blocks {
            let at = (
                TABLE_ORIGIN.0 + side * (bj as f64 + 0.5),
                TABLE_ORIGIN.1 - side * (bi as f64 + 0.5),
            );
            out.push(outline(stage, at, (side, side), color(bi, bj), 4.0)?);
        }
    }
    Ok(out)
}

impl SceneConstruct for Quotients {
    fn name(&self) -> &str {
        "16_quotients"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "16_1")?;
        let title = k.text(stage, "Can we multiply the pieces?", 36.0, Palette::ink(), v(0.0, 3.0))?;
        let prompt = k.text(
            stage,
            "Does the representative matter?",
            30.0,
            Palette::hero(),
            v(0.0, 2.2),
        )?;
        fade(stage, &[title, prompt], true)?;
        // Keep the actual members visible while introducing a choice of representative.
        let mut rack = Vec::new();
        for turns in 0..4 {
            let x = -4.8 + f64::from(turns) * 3.2;
            let border = stage.arena_mut().add(
                Polygon::new([v(x - 1.45, -1.2), v(x + 1.45, -1.2), v(x + 1.45, 1.5), v(x - 1.45, 1.5)])
                    .style(stroke_style(Palette::blue(), 2.))
                    .build(),
            );
            let heading = k.text(
                stage,
                ["H", "r H", "r² H", "r³ H"][turns as usize],
                28.,
                Palette::blue(),
                v(x, 1.15),
            )?;
            fade(stage, &[border, heading], true)?;
            rack.extend([border, heading]);
            for reflection in 0..2 {
                let px = x - 0.68 + f64::from(reflection) * 1.36;
                let tracker = stage.add_value_tracker(0.);
                let pose = tile_model::compose(
                    tile_model::turn(f64::from(turns)),
                    if reflection == 0 {
                        tile_model::IDENTITY
                    } else {
                        tile_model::flip(1.)
                    },
                );
                rack.extend(tile_actor::add_tile(stage, &k, tracker, px, 0.15, 0.42, move |_| pose)?);
                let word = if reflection == 0 {
                    ["e", "r", "r²", "r³"][turns as usize]
                } else {
                    ["s", "r s", "r² s", "r³ s"][turns as usize]
                };
                let label = k.text(stage, word, 23., Palette::ink(), v(px, -0.7))?;
                fade(stage, &[label], true)?;
                rack.push(label);
            }
        }
        k.say(stage, "16_2")?;
        let h = k.tex(
            stage,
            r"H=\{e,s\},\qquad eH=sH=H",
            32.0,
            Palette::blue(),
            &[],
            v(0.0, -1.8),
        )?;
        fade(stage, &[h], true)?;
        let q = k.text(
            stage,
            "Do e r and s r lie in the same coset?",
            29.0,
            Palette::gold(),
            v(0.0, -2.7),
        )?;
        fade(stage, &[q], true)?;
        k.hold(stage)?;
        stage.wait(3.0)?;
        k.say(stage, "16_3")?;
        fade(stage, &rack, false)?;
        fade(stage, &[prompt, h, q], false)?;
        let left_progress = stage.add_value_tracker(0.);
        let right_progress = stage.add_value_tracker(0.);
        let left_tile = tile_actor::add_tile(stage, &k, left_progress, -2.7, 0.4, 1., |t| {
            tile_model::turn(t.clamp(0., 1.))
        })?;
        let right_tile = tile_actor::add_tile(stage, &k, right_progress, 2.7, 0.4, 1., |t| {
            tile_model::compose(
                tile_model::flip((t - 1.).clamp(0., 1.)),
                tile_model::turn(t.clamp(0., 1.)),
            )
        })?;
        let left_label = k.tex(stage, r"er=r", 32., Palette::blue(), &[], v(-2.7, 2.4))?;
        let right_label = k.tex(stage, r"sr=r^{-1}s", 32., Palette::hero(), &[], v(2.7, 2.4))?;
        fade(stage, &[left_label, right_label], true)?;
        let lm = tween(left_progress, 1., 1.6)?;
        let rm = tween(right_progress, 1., 1.6)?;
        let lm = stage.prepare(lm)?;
        let rm = stage.prepare(rm)?;
        stage.play_prepared(vec![lm, rm])?;
        stage.wait(0.7)?;
        let rm = tween(right_progress, 2., 1.6)?;
        stage.play(rm)?;
        let coset = k.tex(
            stage,
            r"rH=\{r,rs\}\qquad r^{-1}s\ \text{is outside}",
            34.0,
            Palette::hero(),
            &[],
            v(0.0, -1.8),
        )?;
        fade(stage, &[coset], true)?;

        // 16_14: every choice at once. rH times rH, coloured by landing coset.
        k.say(stage, "16_14")?;
        let mut old = vec![left_label, right_label, coset];
        old.extend(left_tile);
        old.extend(right_tile);
        fade(stage, &old, false)?;
        let rh = d4g::left_coset(R, &FLIP_H);
        let grid_cell = 1.45;
        let grid_at = |i: usize, j: usize| (-2.2 + grid_cell * j as f64, 0.9 - grid_cell * i as f64);
        let mut grid = Vec::new();
        k.until_sentence(stage, "16_14", 1)?;
        let factor_note = k.tex(stage, r"rH=\{r,rs\}", 34.0, Palette::blue(), &[], v(-1.475, 2.45))?;
        grid.push(factor_note);
        for (index, &g) in rh.iter().enumerate() {
            let top = grid_at(0, index);
            let side = grid_at(index, 0);
            grid.push(k.tex(stage, g.tex(), 34.0, Palette::blue(), &[], v(top.0, top.1 + 0.95))?);
            grid.push(k.tex(stage, g.tex(), 34.0, Palette::blue(), &[], v(side.0 - 1.0, side.1))?);
        }
        fade(stage, &grid, true)?;
        let mut products = Vec::new();
        for (i, &a) in rh.iter().enumerate() {
            for (j, &b) in rh.iter().enumerate() {
                let at = grid_at(i, j);
                let square = filled(
                    stage,
                    at,
                    (grid_cell - 0.1, grid_cell - 0.1),
                    coset_color(a.mul(b)),
                    0.0,
                    Palette::faint(),
                )?;
                let name = k.tex(stage, a.mul(b).tex(), 38.0, Palette::ink(), &[], v(at.0, at.1))?;
                let group = d4g::group_of(stage, &[square, name])?;
                fade(stage, &[group], true)?;
                products.push(((a, b), Entry { square, group }));
            }
        }
        k.until_sentence(stage, "16_14", 2)?;
        paint(stage, &products, coset_color, 0.3)?;
        k.until_sentence(stage, "16_14", 3)?;
        let wish = k.text(
            stage,
            "Well defined would mean: one colour",
            27.0,
            Palette::ink(),
            v(3.9, 1.3),
        )?;
        fade(stage, &[wish], true)?;
        k.until_sentence(stage, "16_14", 4)?;
        let mut legend = vec![wish];
        for (y, g) in [(0.3, R2), (-0.6, E)] {
            let coset = d4g::left_coset(g, &FLIP_H);
            let names: Vec<&str> = coset.iter().map(|x| x.tex()).collect();
            let label = if g == E {
                "H".to_owned()
            } else {
                format!("{}H", g.tex())
            };
            legend.push(k.tex(
                stage,
                &format!(r"{label}=\{{{}\}}", names.join(",")),
                32.0,
                coset_color(g),
                &[],
                v(3.9, y),
            )?);
        }
        let verdict = k.text(
            stage,
            "Two colours: no coset product",
            28.0,
            Palette::gold(),
            v(3.9, -1.6),
        )?;
        legend.push(verdict);
        fade(stage, &legend, true)?;

        k.say(stage, "16_4")?;
        let mut old = grid;
        old.extend(products.iter().map(|p| p.1.group));
        old.extend(legend);
        fade(stage, &old, false)?;
        let conjugate = k.tex(stage, r"rsr^{-1}", 38., Palette::blue(), &[], v(0., 2.5))?;
        let order = k.text(stage, "Turn back, flip, turn forward", 28., Palette::ink(), v(0., -2.5))?;
        fade(stage, &[conjugate, order], true)?;
        let progress = stage.add_value_tracker(0.);
        let conjugated_tile = tile_actor::add_tile(stage, &k, progress, 0., 0., 1.3, |t| {
            let back = tile_model::turn(-t.clamp(0., 1.));
            let flip = tile_model::flip((t - 1.).clamp(0., 1.));
            let forward = tile_model::turn((t - 2.).clamp(0., 1.));
            tile_model::compose(forward, tile_model::compose(flip, back))
        })?;
        for endpoint in [1., 2., 3.] {
            let movement = tween(progress, endpoint, 1.7)?;
            stage.play(movement)?;
            stage.wait(0.5)?;
        }
        let axis = stage.arena_mut().add(
            Polygon::new([v(0., -1.8), v(0., 1.8)])
                .style(stroke_style(Palette::gold(), 3.))
                .build(),
        );
        let new_axis = k.text(
            stage,
            "A different mirror: outside H",
            28.,
            Palette::gold(),
            v(0., -2.0),
        )?;
        fade(stage, &[order], false)?;
        fade(stage, &[axis, new_axis], true)?;
        k.say(stage, "16_5")?;
        let mut old = vec![title, conjugate, axis, new_axis];
        old.extend(conjugated_tile);
        fade(stage, &old, false)?;
        let normal = k.tex(
            stage,
            r"gNg^{-1}=N\quad\Longleftrightarrow\quad gN=Ng",
            35.0,
            Palette::ink(),
            &[],
            v(0.0, 2.8),
        )?;
        let example = k.text(
            stage,
            "Example: N is the four rotations; conjugate by the flip s",
            27.,
            Palette::blue(),
            v(0., 2.0),
        )?;
        fade(stage, &[normal, example], true)?;
        let conjugation = stage.add_value_tracker(0.);
        let mut rotation_set = vec![normal, example];
        let mut before_labels = Vec::new();
        for turns in 0..4 {
            let x = -4.5 + f64::from(turns) * 3.;
            rotation_set.extend(tile_actor::add_tile(stage, &k, conjugation, x, 0.3, 0.7, move |t| {
                let mirror = tile_model::flip(t.clamp(0., 1.));
                let inverse_mirror = tile_model::flip(-t.clamp(0., 1.));
                tile_model::compose(
                    mirror,
                    tile_model::compose(tile_model::turn(f64::from(turns)), inverse_mirror),
                )
            })?);
            let label = k.text(
                stage,
                ["e", "r", "r²", "r³"][turns as usize],
                30.,
                Palette::blue(),
                v(x, -1.0),
            )?;
            before_labels.push(label);
        }
        fade(stage, &before_labels, true)?;
        k.until_sentence(stage, "16_5", 1)?;
        fade(stage, &before_labels, false)?;
        let movement = tween(conjugation, 1., 2.8)?;
        stage.play(movement)?;
        for turns in 0..4 {
            let inverse = (4 - turns) % 4;
            let label = k.text(
                stage,
                ["e", "r", "r²", "r³"][inverse as usize],
                30.,
                Palette::hero(),
                v(-4.5 + f64::from(turns) * 3., -1.0),
            )?;
            fade(stage, &[label], true)?;
            rotation_set.push(label);
        }
        let boundary = stage.arena_mut().add(
            Polygon::new([v(-5.8, -1.5), v(5.8, -1.5), v(5.8, 1.45), v(-5.8, 1.45)])
                .style(stroke_style(Palette::green(), 2.))
                .build(),
        );
        let note = k.text(
            stage,
            "Same four rotations, rearranged inside the set",
            29.0,
            Palette::green(),
            v(0.0, -2.0),
        )?;
        fade(stage, &[boundary, note], true)?;
        rotation_set.extend([boundary, note]);
        k.until_sentence(stage, "16_5", 3)?;
        let noncommuting = k.tex(stage, r"srs^{-1}=r^{-1}\ne r", 32., Palette::hero(), &[], v(0., -2.9))?;
        fade(stage, &[noncommuting], true)?;
        rotation_set.push(noncommuting);
        k.say(stage, "16_6")?;
        fade(stage, &rotation_set, false)?;
        let definition = k.tex(stage, r"(aN)(bN)=abN", 40.0, Palette::ink(), &[], v(0.0, 2.8))?;
        fade(stage, &[definition], true)?;
        k.until_sentence(stage, "16_6", 2)?;
        let choice = k.text(
            stage,
            "Change the representatives. Does the answer change?",
            28.,
            Palette::hero(),
            v(0., 1.8),
        )?;
        fade(stage, &[choice], true)?;
        k.until_sentence(stage, "16_6", 3)?;
        let mut proof = vec![definition, choice];
        let a = k.tex(stage, "a", 40., Palette::blue(), &[], v(-3., 0.5))?;
        let n1 = k.tex(stage, r"n_1", 40., Palette::green(), &[], v(-1., 0.5))?;
        let b = k.tex(stage, "b", 40., Palette::blue(), &[], v(1., 0.5))?;
        let n2 = k.tex(stage, r"n_2", 40., Palette::green(), &[], v(3., 0.5))?;
        fade(stage, &[a, n1, b, n2], true)?;
        proof.extend([a, n1, b, n2]);
        let membership = k.tex(stage, r"n_1,n_2\in N", 30., Palette::green(), &[], v(0., -1.2))?;
        fade(stage, &[membership], true)?;
        k.until_sentence(stage, "16_6", 4)?;
        fade(stage, &[membership], false)?;
        // Insert an identity; do not commute n1 through b, which is generally invalid.
        let mut moves = Vec::new();
        for (token, x) in [(a, -4.), (n1, 0.8), (b, 2.4), (n2, 4.)] {
            let movement = token
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .move_to(v(x, 0.5), ORIGIN)?;
            moves.push(stage.prepare(movement)?);
        }
        stage.play_prepared(moves)?;
        let inserted_b = k.tex(stage, "b", 40., Palette::blue(), &[], v(-2.4, 0.5))?;
        let inverse_b = k.tex(stage, r"b^{-1}", 40., Palette::gold(), &[], v(-0.8, 0.5))?;
        let identity = k.tex(
            stage,
            r"\text{Insert }bb^{-1}=e",
            31.,
            Palette::gold(),
            &[],
            v(0., -1.2),
        )?;
        fade(stage, &[inserted_b, inverse_b, identity], true)?;
        proof.extend([inserted_b, inverse_b]);
        k.say(stage, "16_7")?;
        fade(stage, &[identity, choice], false)?;
        proof.retain(|item| *item != choice);
        let conjugate_box = stage.arena_mut().add(
            Polygon::new([v(-1.5, -0.1), v(3., -0.1), v(3., 1.1), v(-1.5, 1.1)])
                .style(stroke_style(Palette::green(), 2.))
                .build(),
        );
        let normality = k.tex(
            stage,
            r"b^{-1}n_1b\in N\quad\text{by normality}",
            30.,
            Palette::green(),
            &[],
            v(0., -1.2),
        )?;
        fade(stage, &[conjugate_box, normality], true)?;
        stage.wait(1.5)?;
        let extra_box = stage.arena_mut().add(
            Polygon::new([v(-1.5, -0.1), v(4.6, -0.1), v(4.6, 1.1), v(-1.5, 1.1)])
                .style(stroke_style(Palette::green(), 2.))
                .build(),
        );
        fade(stage, &[conjugate_box, normality], false)?;
        let closure = k.tex(
            stage,
            r"(b^{-1}n_1b)n_2\in N\quad\text{by closure}",
            30.,
            Palette::green(),
            &[],
            v(0., -1.2),
        )?;
        fade(stage, &[extra_box, closure], true)?;
        proof.extend([extra_box, closure]);
        k.until_sentence(stage, "16_7", 1)?;
        let result = k.tex(stage, r"(an_1)(bn_2)N=abN", 37.0, Palette::hero(), &[], v(0.0, -2.4))?;
        fade(stage, &[result], true)?;
        proof.push(result);
        k.say(stage, "16_8")?;
        fade(stage, &proof, false)?;
        let necessary_title = k.text(
            stage,
            "Could this work without normality?",
            34.,
            Palette::ink(),
            v(0., 2.8),
        )?;
        fade(stage, &[necessary_title], true)?;
        k.until_sentence(stage, "16_8", 1)?;
        let identity_choice = k.tex(stage, r"H(aH)=(ea)H=aH", 36., Palette::blue(), &[], v(0., 1.5))?;
        fade(stage, &[identity_choice], true)?;
        k.until_sentence(stage, "16_8", 2)?;
        let other_choice = k.tex(stage, r"h\in H:\quad(ha)H=aH", 36., Palette::hero(), &[], v(0., 0.3))?;
        fade(stage, &[other_choice], true)?;
        stage.wait(1.2)?;
        let necessary_conjugate = k.tex(
            stage,
            r"ha\in aH\quad\Longrightarrow\quad a^{-1}ha\in H",
            33.,
            Palette::green(),
            &[],
            v(0., -1.0),
        )?;
        fade(stage, &[necessary_conjugate], true)?;
        k.until_sentence(stage, "16_8", 4)?;
        let all = k.text(
            stage,
            "Every a, every h: conjugation must preserve H",
            29.,
            Palette::green(),
            v(0., -2.2),
        )?;
        fade(stage, &[all], true)?;
        k.say(stage, "16_9")?;
        fade(
            stage,
            &[necessary_title, identity_choice, other_choice, necessary_conjugate, all],
            false,
        )?;
        let quotient = k.tex(stage, r"G/N=\{gN:g\in G\}", 38., Palette::ink(), &[], v(0., 2.5))?;
        fade(stage, &[quotient], true)?;
        k.until_sentence(stage, "16_9", 1)?;
        let assoc = k.tex(stage, r"((aN)(bN))(cN)=((ab)c)N", 32., Palette::blue(), &[], v(0., 1.3))?;
        fade(stage, &[assoc], true)?;
        stage.wait(1.0)?;
        let assoc_right = k.tex(stage, r"(aN)((bN)(cN))=(a(bc))N", 32., Palette::gold(), &[], v(0., 0.3))?;
        fade(stage, &[assoc_right], true)?;
        stage.wait(1.0)?;
        let inherited = k.tex(
            stage,
            r"(ab)c=a(bc)\text{ in }G",
            32.,
            Palette::green(),
            &[],
            v(0., -0.9),
        )?;
        fade(stage, &[inherited], true)?;
        stage.wait(1.8)?;
        k.until_sentence(stage, "16_9", 2)?;
        fade(stage, &[assoc, assoc_right, inherited], false)?;
        let axioms = k.tex(stage, r"(eN)(gN)=(eg)N=gN", 34.0, Palette::blue(), &[], v(0.0, 1.0))?;
        fade(stage, &[axioms], true)?;
        stage.wait(1.2)?;
        let inverse = k.tex(
            stage,
            r"(gN)(g^{-1}N)=(gg^{-1})N=N",
            34.,
            Palette::gold(),
            &[],
            v(0., -0.2),
        )?;
        fade(stage, &[inverse], true)?;
        k.until_sentence(stage, "16_9", 4)?;
        let proj = k.tex(
            stage,
            r"\pi(g)=gN,\qquad\pi(ab)=\pi(a)\pi(b)",
            33.0,
            Palette::green(),
            &[],
            v(0.0, -1.5),
        )?;
        fade(stage, &[proj], true)?;
        k.say(stage, "16_10")?;
        fade(stage, &[quotient, axioms, inverse, proj], false)?;
        let rotations = k.text(
            stage,
            "Forget the angle; keep the orientation",
            34.,
            Palette::ink(),
            v(0., 3.0),
        )?;
        fade(stage, &[rotations], true)?;
        let mut examples = Vec::new();
        for row in 0..2 {
            let y = if row == 0 { 1.2 } else { -1.2 };
            let color = if row == 0 { Palette::blue() } else { Palette::gold() };
            let boundary = stage.arena_mut().add(
                Polygon::new([v(-5.7, y - 0.95), v(5.7, y - 0.95), v(5.7, y + 0.95), v(-5.7, y + 0.95)])
                    .style(stroke_style(color, 2.))
                    .build(),
            );
            fade(stage, &[boundary], true)?;
            examples.push(boundary);
            for turns in 0..4 {
                let x = -4.5 + f64::from(turns) * 3.;
                let tracker = stage.add_value_tracker(0.);
                let pose = tile_model::compose(
                    tile_model::turn(f64::from(turns)),
                    if row == 0 {
                        tile_model::IDENTITY
                    } else {
                        tile_model::flip(1.)
                    },
                );
                examples.extend(tile_actor::add_tile(stage, &k, tracker, x, y, 0.55, move |_| pose)?);
                let word = if row == 0 {
                    ["e", "r", "r²", "r³"][turns as usize]
                } else {
                    ["s", "r s", "r² s", "r³ s"][turns as usize]
                };
                let label = k.text(stage, word, 25., color, v(x, y - 0.75))?;
                fade(stage, &[label], true)?;
                examples.push(label);
            }
        }

        // 16_15: all sixty-four products, coloured rotation/flip, regrouped.
        k.say(stage, "16_15")?;
        fade(stage, &examples, false)?;
        let table = Table::new(stage, &k, &COSET_ORDER, rotation_color)?;
        let everything = table.all();
        d4g::show(stage, &everything, 0.8)?;
        k.until_sentence(stage, "16_15", 1)?;
        paint(stage, &table.cells, rotation_color, 0.01)?;
        k.until_sentence(stage, "16_15", 2)?;
        table.regroup(stage, &ROTATION_ORDER, 2.0)?;
        let solid = block_outlines(stage, 4, |bi, bj| {
            if (bi + bj) % 2 == 0 {
                Palette::blue()
            } else {
                Palette::gold()
            }
        })?;
        let mut creation: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &solid {
            creation.push(stage.prepare(show_creation(m).rt(0.7))?);
        }
        stage.play_prepared(creation)?;
        k.until_sentence(stage, "16_15", 3)?;
        let rule = k.text(
            stage,
            "Block colour depends only on the blocks",
            26.0,
            Palette::ink(),
            v(4.9, -0.4),
        )?;
        fade(stage, &[rule], true)?;

        // 16_16: each solid block shrinks to one cell of the R/F table.
        k.say(stage, "16_16")?;
        let headers: Vec<Mob> = table.headers.iter().map(|h| h.2).collect();
        fade(stage, &headers, false)?;
        // A true shrink: each entry keeps its place inside its block.
        let mut collapse: Vec<Box<dyn Animation>> = Vec::new();
        for &((a, b), entry) in &table.cells {
            let target = shrunk_position(a, b);
            let a = entry
                .group
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.6),
                    ..AnimateArgs::default()
                })?
                .scale(SHRINK)?
                .move_to(v(target.0, target.1), ORIGIN)?;
            collapse.push(stage.prepare(a)?);
        }
        for (index, &m) in solid.iter().enumerate() {
            let target = small_cell(index / 2, index % 2);
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.6),
                    ..AnimateArgs::default()
                })?
                .scale(SHRINK)?
                .move_to(v(target.0, target.1), ORIGIN)?;
            collapse.push(stage.prepare(a)?);
        }
        stage.play_prepared(collapse)?;
        let cells: Vec<Mob> = table.cells.iter().map(|c| c.1.group).collect();
        let mut quotient_cells = Vec::new();
        let side = SHRINK * 4.0 * TABLE_CELL;
        for row in 0..2 {
            for col in 0..2 {
                let color = if (row + col) % 2 == 0 {
                    Palette::blue()
                } else {
                    Palette::gold()
                };
                quotient_cells.push(filled(stage, small_cell(row, col), (side, side), color, 0.3, color)?);
            }
        }
        fade(stage, &cells, false)?;
        fade(stage, &quotient_cells, true)?;
        k.until_sentence(stage, "16_16", 1)?;
        let small_name = k.text(stage, "The quotient group", 28.0, Palette::ink(), v(4.9, -0.4))?;
        play!(stage; fmn::animation::replacement_transform(rule, small_name).rt(0.6));

        k.say(stage, "16_11")?;
        let mut table_objects = Vec::new();
        let headings = [("R", Palette::blue()), ("F", Palette::gold())];
        for (index, (label, color)) in headings.iter().enumerate() {
            let column = k.text(stage, label, 38., *color, v(0.5 + index as f64 * 2., 1.7))?;
            let row = k.text(stage, label, 38., *color, v(-1.5, 0.4 - index as f64 * 1.3))?;
            fade(stage, &[column, row], true)?;
            table_objects.extend([column, row]);
        }
        // Four products are revealed individually, with their input row and column.
        for (row, col) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let x = 0.5 + f64::from(col) * 2.;
            let y = 0.4 - f64::from(row) * 1.3;
            let reflected = (row + col) % 2 == 1;
            let result = k.text(
                stage,
                if reflected { "F" } else { "R" },
                42.,
                if reflected { Palette::gold() } else { Palette::blue() },
                v(x, y),
            )?;
            let row_box = stage.arena_mut().add(
                Polygon::new([v(-1.9, y - 0.4), v(-1.1, y - 0.4), v(-1.1, y + 0.4), v(-1.9, y + 0.4)])
                    .style(stroke_style(Palette::green(), 2.))
                    .build(),
            );
            let col_box = stage.arena_mut().add(
                Polygon::new([v(x - 0.4, 1.3), v(x + 0.4, 1.3), v(x + 0.4, 2.1), v(x - 0.4, 2.1)])
                    .style(stroke_style(Palette::green(), 2.))
                    .build(),
            );
            fade(stage, &[row_box, col_box], true)?;
            fade(stage, &[result], true)?;
            stage.wait(0.8)?;
            fade(stage, &[row_box, col_box], false)?;
            table_objects.push(result);
        }
        let iso = k.tex(stage, r"D_4/R\cong C_2", 37.0, Palette::ink(), &[], v(0.0, -2.3))?;
        fade(stage, &[iso], true)?;

        // 16_17: the same table sorted by the left cosets of {e, s}.
        k.say(stage, "16_17")?;
        let mut old = table_objects;
        old.extend(quotient_cells);
        old.extend(solid);
        old.extend([iso, small_name]);
        fade(stage, &old, false)?;
        let coset_table = Table::new(stage, &k, &COSET_ORDER, coset_color)?;
        let coset_all = coset_table.all();
        d4g::show(stage, &coset_all, 0.8)?;
        k.until_sentence(stage, "16_17", 1)?;
        paint(stage, &coset_table.cells, coset_color, 0.01)?;
        let mixed = mixed_blocks(&COSET_ORDER, 2, |g| g.turns);
        let pairs = block_outlines(stage, 2, |bi, bj| {
            if mixed.contains(&(bi, bj)) {
                Palette::faint()
            } else {
                Palette::ink()
            }
        })?;
        let mut creation: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &pairs {
            creation.push(stage.prepare(show_creation(m).rt(0.6))?);
        }
        stage.play_prepared(creation)?;
        k.until_sentence(stage, "16_17", 2)?;
        let mut crosses = Vec::new();
        for &(bi, bj) in &mixed {
            let side = 2.0 * TABLE_CELL;
            let (x, y) = (
                TABLE_ORIGIN.0 + side * (bj as f64 + 0.5),
                TABLE_ORIGIN.1 - side * (bi as f64 + 0.5),
            );
            crosses.push(outline(
                stage,
                (x, y),
                (side - 0.08, side - 0.08),
                Palette::hero(),
                6.0,
            )?);
        }
        let mut creation: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &crosses {
            creation.push(stage.prepare(show_creation(m).rt(0.5))?);
        }
        let creation = explainer_kit::lagged(stage, creation, 0.15)?;
        play!(stage; creation);
        let count = k.text(
            stage,
            &format!("{} of {} blocks mixed", mixed.len(), (COSET_ORDER.len() / 2).pow(2)),
            28.0,
            Palette::hero(),
            v(4.9, 0.6),
        )?;
        fade(stage, &[count], true)?;
        k.until_sentence(stage, "16_17", 3)?;
        let r_block = block_of(&COSET_ORDER, R, 2);
        let side = 2.0 * TABLE_CELL;
        let failure = outline(
            stage,
            (
                TABLE_ORIGIN.0 + side * (r_block as f64 + 0.5),
                TABLE_ORIGIN.1 - side * (r_block as f64 + 0.5),
            ),
            (side + 0.12, side + 0.12),
            Palette::yellow(),
            7.0,
        )?;
        play!(stage; show_creation(failure).rt(0.7));
        let callback = k.tex(stage, r"rH\cdot rH", 34.0, Palette::yellow(), &[], v(4.9, -0.3))?;
        fade(stage, &[callback], true)?;
        k.until_sentence(stage, "16_17", 4)?;
        let nothing = k.text(
            stage,
            "No smaller table to shrink to",
            28.0,
            Palette::gold(),
            v(4.9, -1.3),
        )?;
        fade(stage, &[nothing], true)?;

        k.say(stage, "16_12")?;
        let mut old = coset_all;
        old.extend(pairs);
        old.extend(crosses);
        old.extend([count, failure, callback, nothing, rotations]);
        fade(stage, &old, false)?;
        let summary = k.text(
            stage,
            "Keep orientation; forget the particular move",
            32.0,
            Palette::hero(),
            v(0.0, 3.0),
        )?;
        fade(stage, &[summary], true)?;
        // Show the many-to-one map instead of dropping the example for a slogan.
        for row in 0..2 {
            let y = if row == 0 { 1.2 } else { -1.2 };
            let color = if row == 0 { Palette::blue() } else { Palette::gold() };
            let target = k.text(stage, if row == 0 { "R" } else { "F" }, 44., color, v(4.9, y))?;
            fade(stage, &[target], true)?;
            for turns in 0..4 {
                let x = -5.1 + f64::from(turns) * 2.;
                let tracker = stage.add_value_tracker(0.);
                let pose = tile_model::compose(
                    tile_model::turn(f64::from(turns)),
                    if row == 0 {
                        tile_model::IDENTITY
                    } else {
                        tile_model::flip(1.)
                    },
                );
                tile_actor::add_tile(stage, &k, tracker, x, y, 0.43, move |_| pose)?;
                let token = k.text(stage, if row == 0 { "R" } else { "F" }, 28., color, v(x, y - 0.75))?;
                fade(stage, &[token], true)?;
                let movement = token
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.65),
                        ..AnimateArgs::default()
                    })?
                    .move_to(v(3.6, y), ORIGIN)?;
                stage.play(movement)?;
                fade(stage, &[token], false)?;
            }
        }
        k.say(stage, "16_13")?;
        let boundary = stage.arena_mut().add(
            Polygon::new([v(-5.9, 0.25), v(1.7, 0.25), v(1.7, 2.15), v(-5.9, 2.15)])
                .style(stroke_style(Palette::green(), 3.))
                .build(),
        );
        let identity = k.text(
            stage,
            "All four become the quotient identity",
            28.,
            Palette::green(),
            v(0., -2.45),
        )?;
        fade(stage, &[boundary, identity], true)?;
        let kernel = k.tex(stage, r"\ker\pi=R", 32.0, Palette::green(), &[], v(4.8, 0.0))?;
        fade(stage, &[kernel], true)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_model::{self, CORNERS};

    fn same(a: tile_model::Matrix, b: tile_model::Matrix) -> bool {
        CORNERS.iter().all(|&p| {
            tile_model::apply(a, p)
                .iter()
                .zip(tile_model::apply(b, p))
                .all(|(x, y)| (*x - y).abs() < 1e-10)
        })
    }

    #[test]
    fn continuous_conjugation_fixes_identity_and_has_fourth_power_identity() {
        for step in 0..=20 {
            let t = f64::from(step) / 20.;
            let forward = tile_model::flip(t);
            let inverse = tile_model::flip(-t);
            assert!(same(tile_model::compose(forward, inverse), tile_model::IDENTITY));
            for turns in 0..4 {
                let conjugate = tile_model::compose(
                    forward,
                    tile_model::compose(tile_model::turn(f64::from(turns)), inverse),
                );
                let square = tile_model::compose(conjugate, conjugate);
                assert!(same(tile_model::compose(square, square), tile_model::IDENTITY));
            }
        }
    }

    #[test]
    fn reflection_conjugates_each_rotation_to_its_inverse() {
        let s = tile_model::flip(1.);
        for turns in 0..4 {
            let r = tile_model::turn(f64::from(turns));
            let conjugate = tile_model::compose(s, tile_model::compose(r, s));
            assert!(same(conjugate, tile_model::turn(-f64::from(turns))));
        }
        let r = tile_model::turn(1.);
        assert!(!same(tile_model::compose(s, r), tile_model::compose(r, s)));
    }

    #[test]
    fn animated_representative_failure_matches_displayed_coset() {
        let r = tile_model::turn(1.);
        let s = tile_model::flip(1.);
        let sr = tile_model::compose(s, r);
        assert!(same(sr, tile_model::compose(tile_model::turn(-1.), s)));
        assert!(!same(sr, r));
        assert!(!same(sr, tile_model::compose(r, s)));
    }

    #[test]
    fn conjugation_movie_ends_at_vertical_reflection_outside_h() {
        let s = tile_model::flip(1.);
        let conjugate = tile_model::compose(tile_model::turn(1.), tile_model::compose(s, tile_model::turn(-1.)));
        assert!(!same(conjugate, s));
        assert!(!same(conjugate, tile_model::IDENTITY));
        for p in CORNERS {
            let result = tile_model::apply(conjugate, p);
            assert!((result[0] + p[0]).abs() < 1e-10);
            assert!((result[1] - p[1]).abs() < 1e-10);
        }
    }

    #[test]
    fn four_products_of_rh_land_in_two_cosets_as_narrated() {
        let rh = d4g::left_coset(R, &FLIP_H);
        assert_eq!(rh, vec![R, RS]);
        let products: Vec<Elem> = rh.iter().flat_map(|&a| rh.iter().map(move |&b| a.mul(b))).collect();
        assert_eq!(products, vec![R2, R2S, S, E]);
        let r2h = d4g::left_coset(R2, &FLIP_H);
        let in_r2h = products.iter().filter(|p| r2h.contains(p)).count();
        let in_h = products.iter().filter(|p| FLIP_H.contains(p)).count();
        assert_eq!((in_r2h, in_h), (2, 2));
        // The legend colours are the colours of those cosets.
        assert!(
            products
                .iter()
                .all(|&p| coset_color(p) == coset_color(R2) || coset_color(p) == coset_color(E))
        );
    }

    #[test]
    fn rotation_flip_table_has_four_solid_blocks_that_form_the_two_element_group() {
        assert!(mixed_blocks(&ROTATION_ORDER, 4, |g| u8::from(g.flip)).is_empty());
        // Block colour = the quotient product: flip parity adds.
        for a in ALL {
            for b in ALL {
                assert_eq!(a.mul(b).flip, a.flip ^ b.flip);
            }
        }
        // The same entries in coset order are not grouped: regrouping is needed.
        assert!(!mixed_blocks(&COSET_ORDER, 4, |g| u8::from(g.flip)).is_empty());
        // Shrunk cells sit where the R/F table draws its entries.
        assert_eq!(small_cell(1, 0), (0.5, 0.4 - 1.3));
    }

    #[test]
    fn coset_sorted_table_has_eight_mixed_blocks_including_rh_times_rh() {
        let mixed = mixed_blocks(&COSET_ORDER, 2, |g| g.turns);
        assert_eq!(mixed.len(), 8);
        let r_block = block_of(&COSET_ORDER, R, 2);
        assert!(mixed.contains(&(r_block, r_block)));
        // The coset order really lists the left cosets of H, pair by pair.
        for pair in COSET_ORDER.chunks(2) {
            assert_eq!(d4g::left_coset(pair[0], &FLIP_H), pair.to_vec());
        }
        assert_eq!(R3S, RS.mul(R2));
        assert_eq!(R3, R.mul(R2));
    }
}
