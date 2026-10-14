//! Chapter 11, walks: a matrix can record who is linked to whom. Four nodes,
//! a one where two are linked and a zero where they are not (11_1). A walk is
//! a route along links, and it may double back; the square of the matrix
//! counts the two-step walks between each pair (11_2). Your turn: how many
//! two-step walks bring you back to node three (11_3)? Three: out along any
//! of its three links, and straight back, and the matrix agrees (11_4). The
//! k-th power counts walks of k steps, so e to the matrix adds up every walk
//! of every length, each weighted by one over k factorial (11_5); network
//! scientists call the result communicability, and nodes one and three
//! score higher than one and two because node three has more ways round
//! (11_6).
//!
//! Every number is a fact: the links are `F-G.links`, the walk counts
//! `F-G.walks.k2` and `F-G.walks.one_to_four`, the running total at node
//! four `F-G.exp.one_to_four.running`, the grid of e^A `F-G.exp`, the two
//! scores `F-V.11_6.*`, the quiz answer `F-Q.11_3.answer`.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, QuizCard, fact, fade_in_all, fade_out_all, faded_line, fail, fit_width, flat, matrix, music_hold,
    numbers, panel, result, rule, say, show, substance, tween_linear, whole,
};
use super::flow::{Levels, fade_in_anim, fade_out_anim};
use crate::facts::cited;
use crate::model::{KITE, Mat, Network, factorial};

// ------------------------------------------------------------------ choices

/// Where the four nodes sit: a triangle of 1, 2 and 3 with a tail to 4.
pub const NODE_AT: [[f64; 2]; 4] = [[-5.5, 1.2], [-5.5, -1.5], [-3.5, -0.15], [-1.5, -0.15]];
pub const NODE_R: f64 = 0.36;
/// Where the grids of 11_1 to 11_4 sit, and their type size.
const A_AT: Vec3 = [2.1, 0.9, 0.0];
const A2_AT: Vec3 = [5.6, 0.9, 0.0];
const GRID_PT: f64 = 36.0;
/// The quiz card, over the grids' side of the frame.
const CARD_AT: Vec3 = [4.0, 0.4, 0.0];
const CARD_SIZE: (f64, f64) = (5.8, 3.9);
/// How many steps long the walks drawn at 11_5 go.
const WALK_STEPS: [u32; 3] = [2, 3, 4];

// ------------------------------------------------------------- the geometry

/// Every walk of exactly `k` links from `from` to `to`: a list of nodes,
/// `k + 1` long, each pair joined by a link (a walk may revisit nodes and
/// links).
pub fn walks_between(net: &Network, from: usize, to: usize, k: u32) -> Vec<Vec<usize>> {
    let mut walks: Vec<Vec<usize>> = vec![vec![from]];
    for _ in 0..k {
        let mut next = Vec::new();
        for w in &walks {
            let here = *w.last().expect("a walk has a node");
            for &(a, b) in net.links {
                let step = if a == here {
                    Some(b)
                } else if b == here {
                    Some(a)
                } else {
                    None
                };
                if let Some(n) = step {
                    let mut longer = w.clone();
                    longer.push(n);
                    next.push(longer);
                }
            }
        }
        walks = next;
    }
    walks.retain(|w| w.last() == Some(&to));
    walks
}

/// The points of a shallow arc from `a` to `b`, bowed `bulge` to the left of
/// the way from `a` to `b` at its middle (`n + 1` points).
pub fn bow(a: Vec3, b: Vec3, bulge: f64, n: usize) -> Vec<Vec3> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (nx, ny) = (-dy / len, dx / len);
    (0..=n)
        .map(|k| {
            let s = k as f64 / n as f64;
            let h = bulge * 4.0 * s * (1.0 - s);
            v(a[0] + s * dx + h * nx, a[1] + s * dy + h * ny)
        })
        .collect()
}

/// Where a token is when it has gone `s` links along `stops` (s from 0 to
/// the number of links): straight along each link in turn.
pub fn along(stops: &[Vec3], s: f64) -> Vec3 {
    let last = stops.len() - 1;
    let s = s.clamp(0.0, last as f64);
    let i = (s.floor() as usize).min(last - 1);
    let f = s - i as f64;
    v(
        stops[i][0] + f * (stops[i + 1][0] - stops[i][0]),
        stops[i][1] + f * (stops[i + 1][1] - stops[i][1]),
    )
}

/// The working of one entry of A·A as it is written under the grids:
/// `0 \times 0 + 1 \times 0 + 1 \times 1 + 0 \times 0 = 1`.
pub fn product_line(a: &Mat, i: usize, j: usize) -> String {
    let n = a.n();
    let terms: Vec<String> = (0..n)
        .map(|m| format!("{} \\times {}", whole(a.get(i, m)), whole(a.get(m, j))))
        .collect();
    let total: f64 = (0..n).map(|m| a.get(i, m) * a.get(m, j)).sum();
    format!("{} = {}", terms.join(" + "), whole(total))
}

/// How bright a walk of `k` steps is drawn: its weight 1/k! against the
/// weight of the shortest walk drawn, bent (a power) so the longest stays
/// readable. Steadily less for each step more.
pub fn brightness(k: u32) -> f64 {
    let weight = |k: u32| 1.0 / factorial(k) as f64;
    (weight(k) / weight(WALK_STEPS[0])).powf(0.45)
}

/// The opacity of a cell of the shaded grid of 11_6 for `value`, against the
/// largest entry: never invisible, never solid.
pub fn shade(value: f64, largest: f64) -> f64 {
    0.08 + 0.62 * (value / largest).clamp(0.0, 1.0)
}

// ------------------------------------------------------------ the network

/// The four nodes with their links, drawn at `NODE_AT`.
struct Net {
    discs: [Mob; 4],
    numbers: [Mob; 4],
    /// One line per link, dim.
    links: Vec<Mob>,
    /// The same lines again, lit in the rule's colour (bright while a link
    /// is being written into the grid).
    lit: Vec<Mob>,
}

/// Where node `i` sits on screen.
pub fn node(i: usize) -> Vec3 {
    v(NODE_AT[i][0], NODE_AT[i][1])
}

impl Net {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut links = Vec::new();
        let mut lit = Vec::new();
        for &(a, b) in KITE.links {
            let dim = polyline(vec![node(a), node(b)])
                .style(faded_line(Palette::dim(), 4.0, 0.7))
                .build();
            links.push(stage.arena_mut().add(dim));
            let on = polyline(vec![node(a), node(b)])
                .style(faded_line(rule(), 7.0, 1.0))
                .build();
            lit.push(stage.arena_mut().add(on));
        }
        let mut discs = Vec::new();
        let mut numbers = Vec::new();
        for i in 0..KITE.n {
            let c = Circle::new()
                .radius(NODE_R)
                .arc_center(node(i))
                .style(Style::default().fill(panel(), 1.0).stroke(Palette::ink(), 3.5, 1.0))
                .build();
            discs.push(stage.arena_mut().add(c));
            numbers.push(kit.text(stage, &(i + 1).to_string(), 34.0, Palette::ink(), node(i))?);
        }
        Ok(Self {
            discs: [discs[0], discs[1], discs[2], discs[3]],
            numbers: [numbers[0], numbers[1], numbers[2], numbers[3]],
            links,
            lit,
        })
    }

    /// The discs and their numbers.
    fn nodes(&self) -> Vec<Mob> {
        self.discs.iter().chain(&self.numbers).copied().collect()
    }
}

/// Rows of whole numbers as strings.
fn whole_rows(m: &Mat) -> Vec<Vec<String>> {
    (0..m.n())
        .map(|i| m.row(i).iter().map(|&x| whole(x)).collect())
        .collect()
}

/// A rounded outline around the cell at (`row`, `col`) of `grid`.
pub fn cell_outline(stage: &mut Stage<'_>, grid: &MatrixGrid, row: usize, col: usize, color: Srgb) -> fmn::Result<Mob> {
    let c = stage.get_center(grid.cells[row][col]);
    let r = Rectangle::new()
        .width(grid.pitch * 0.9)
        .height(grid.pitch * 0.9)
        .corner_radius(0.08)
        .style(faded_line(color, 5.0, 1.0))
        .build()?
        .moved_to(v(c[0], c[1]));
    Ok(stage.arena_mut().add(r))
}

// -------------------------------------------------------------- the scene

pub struct Walks {
    pub kit: Kit,
}

impl SceneConstruct for Walks {
    fn name(&self) -> &str {
        "11_walks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());
        let adjacency = KITE.adjacency();

        // ---- 11_1: four nodes, ones where two are linked, zeros where not
        let line = say(&kit, stage, "11_1")?;
        let net = Net::build(&kit, stage)?;
        let mut first = net.links.clone();
        first.extend(net.nodes());
        let a_grid = MatrixGrid::build(&kit, stage, &whole_rows(&adjacency), GRID_PT, teal, A_AT)?;
        let a_name = kit.tex(stage, "A", 48.0, teal, &[], v(A_AT[0], A_AT[1] + 1.55))?;
        // the grid is on the scene from the start, every cell and bracket at
        // level 0, and comes in piece by piece
        let mut pieces: Vec<Mob> = a_grid.cells.iter().flatten().copied().collect();
        pieces.extend(a_grid.brackets);
        let levels = Levels::build(stage, &pieces, 0.0)?;
        stage.add_to_scene(a_grid.group)?;
        let piece_of = |i: usize, j: usize| i * KITE.n + j;
        // the network is up from the line's first words, "who is linked to
        // whom" (it came in with "Four nodes", three seconds in: a black
        // frame under the sentence that names it)
        fade_in_all(stage, &first, 0.8)?;
        line.wait_for(stage, "Four nodes")?;
        line.wait_for(stage, "Write a one where two nodes are linked")?;
        let mut ins: Vec<Box<dyn Animation>> = vec![fade_in_anim(stage, a_name, 0.5)?];
        for b in 0..2 {
            ins.push(stage.prepare(levels.tween(pieces.len() - 2 + b, 1.0, 0.5)?)?);
        }
        stage.play_prepared(ins)?;
        // each link, as it lights, writes its two ones
        for (k, &(a, b)) in KITE.links.iter().enumerate() {
            let mut ins: Vec<Box<dyn Animation>> = vec![
                fade_in_anim(stage, net.lit[k], 0.35)?,
                stage.prepare(levels.tween(piece_of(a, b), 1.0, 0.35)?)?,
            ];
            if a != b {
                ins.push(stage.prepare(levels.tween(piece_of(b, a), 1.0, 0.35)?)?);
            }
            stage.play_prepared(ins)?;
        }
        line.wait_for(stage, "and a zero where they aren't")?;
        let mut zeros: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..KITE.n {
            for j in 0..KITE.n {
                if adjacency.get(i, j) == 0.0 {
                    zeros.push(stage.prepare(levels.tween(piece_of(i, j), 1.0, 0.5)?)?);
                }
            }
        }
        for &m in &net.lit {
            zeros.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(zeros)?;
        let adjacency_note = lower_third(&kit, stage, "precisely:", "the adjacency matrix")?;
        fade_in_all(stage, &[adjacency_note], 0.5)?;

        // ---- 11_2: multiply it by itself: two-step walks
        let line = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &[adjacency_note], 0.4)?;
        let two = KITE.walks(2);
        let a2_rows: Vec<Vec<String>> = two.iter().map(|r| r.iter().map(|&x| x.to_string()).collect()).collect();
        let a2_grid = MatrixGrid::build(&kit, stage, &a2_rows, GRID_PT, gold, A2_AT)?;
        let a2_name = kit.tex(stage, r"A^2", 48.0, gold, &[], v(A2_AT[0], A2_AT[1] + 1.55))?;
        line.wait_for(stage, "A walk is a route")?;
        // a walk: one token goes 1, 3, 4 along the links
        let token = stage
            .arena_mut()
            .add(Dot::new().point(node(0)).radius(0.14).style(flat(gold, 1.0)).build());
        let stops = [node(0), node(2), node(3)];
        let at = stage.add_value_tracker(0.0);
        let rider = stage.arena_mut().add(VMobject::new());
        let mut shown_at = f64::NAN;
        stage.add_updater(
            rider,
            move |s, _| {
                let u = s.tracker_value(at).unwrap_or(0.0);
                if u.to_bits() != shown_at.to_bits() {
                    s.move_to(token, along(&stops, u), ORIGIN);
                    shown_at = u;
                }
            },
            true,
        )?;
        stage.add_to_scene(rider)?;
        fade_in_all(stage, &[token], 0.4)?;
        play!(stage; tween_linear(at, 2.0, 1.8)?);
        line.wait_for(stage, "Multiply that matrix by itself")?;
        // row 1 and column 4 light; the four products sit under them
        let row_band = a_grid.row_band(stage, 0, gold)?;
        let col_x = stage.get_center(a_grid.cells[0][3])[0];
        let top = stage.get_center(a_grid.cells[0][3])[1] + a_grid.pitch * 0.55;
        let bottom = stage.get_center(a_grid.cells[3][3])[1] - a_grid.pitch * 0.55;
        let col_band = Rectangle::new()
            .width(a_grid.pitch * 0.9)
            .height(top - bottom)
            .corner_radius(0.08)
            .style(flat(gold, 0.2))
            .build()?
            .moved_to(v(col_x, (top + bottom) / 2.0));
        let col_band = stage.arena_mut().add(col_band);
        let working = kit.tex(stage, &product_line(&adjacency, 0, 3), 36.0, ink, &[], v(3.9, -1.3))?;
        fit_width(stage, working, 5.8);
        let middle = kit.text(
            stage,
            "a middle node counts when both links exist",
            26.0,
            Palette::dim(),
            v(3.9, -2.0),
        )?;
        fade_in_all(
            stage,
            &[row_band, col_band, working, middle, a2_grid.group, a2_name],
            0.7,
        )?;
        line.wait_for(stage, "each entry counts the two-step walks")?;
        kit.hold(stage)?;

        // ---- 11_3: your turn: back to node three in two steps
        let line = say(&kit, stage, "11_3")?;
        let clear = [
            row_band,
            col_band,
            working,
            middle,
            a2_grid.group,
            a2_name,
            a_grid.group,
            a_name,
            token,
        ];
        stage.clear_updaters(levels.anchor, true);
        fade_out_all(stage, &clear, 0.5)?;
        let ring = Circle::new()
            .radius(NODE_R + 0.14)
            .arc_center(node(2))
            .style(faded_line(gold, 6.0, 1.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        fade_in_all(stage, &[ring], 0.4)?;
        let card = QuizCard::build(&kit, stage, "11_3", CARD_AT, CARD_SIZE)?;
        let (middle_at, room, _) = card.content();
        let problem = kit.text(stage, "two steps, from 3 back to 3", 36.0, ink, middle_at)?;
        fit_width(stage, problem, room);
        line.wait_last(stage)?;
        card.show(stage, &[problem], 0.5)?;
        card.count(&kit, stage)?;
        let answer = kit.text(
            stage,
            fact("F-Q.11_3.answer")?,
            72.0,
            gold,
            v(middle_at[0], middle_at[1] - 0.6),
        )?;
        card.land(stage, &[answer], 0.4)?;

        // ---- 11_4: three, out along any link and straight back
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "Out along any of its three links")?;
        let neighbours: Vec<usize> = KITE
            .links
            .iter()
            .filter_map(|&(a, b)| match (a == 2, b == 2) {
                (true, _) => Some(b),
                (_, true) => Some(a),
                _ => None,
            })
            .collect();
        let mut arcs = Vec::new();
        for &nb in &neighbours {
            for (from, to) in [(2, nb), (nb, 2)] {
                let a = polyline(bow(node(from), node(to), 0.28, 14))
                    .style(faded_line(gold, 5.0, 1.0))
                    .build();
                arcs.push(stage.arena_mut().add(a));
            }
        }
        let mut drawn: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &arcs {
            drawn.push(stage.prepare(show_creation(m).rt(0.9))?);
        }
        let all = AnimationGroup::with_lag_ratio(stage.arena_mut(), drawn, 0.35)?.with_run_time(2.6);
        stage.play(all)?;
        line.wait_for(stage, "The matrix agrees")?;
        let a2_again = MatrixGrid::build(&kit, stage, &a2_rows, GRID_PT, gold, A2_AT)?;
        let a2_again_name = kit.tex(stage, r"A^2", 48.0, gold, &[], v(A2_AT[0], A2_AT[1] + 1.55))?;
        let outline = cell_outline(stage, &a2_again, 2, 2, gold)?;
        let mut leave = card.mobs();
        leave.extend([problem, answer]);
        fade_out_all(stage, &leave, 0.4)?;
        fade_in_all(stage, &[a2_again.group, a2_again_name, outline], 0.6)?;

        // ---- 11_5: every walk of every length, each at one over k factorial
        let line = say(&kit, stage, "11_5")?;
        let mut tidy = vec![a2_again.group, a2_again_name, outline, ring];
        tidy.extend(&arcs);
        fade_out_all(stage, &tidy, 0.5)?;
        let series = kit.tex(
            stage,
            r"e^{A}=I+A+\frac{A^2}{2!}+\frac{A^3}{3!}+\cdots",
            36.0,
            ink,
            &[("e", gold), ("A", teal)],
            v(2.9, 2.5),
        )?;
        fade_in_all(stage, &[series], 0.6)?;
        line.wait_for(stage, "The k-th power counts walks of k steps")?;
        // every walk from node 1 to node 4, a trace each, brighter the fewer
        // its steps; the running total piles up beside node 4
        let mut bulges = [0.0, 0.14, -0.14, 0.28, -0.28, 0.42].into_iter();
        let mut layers: Vec<Vec<Mob>> = Vec::new();
        for &k in &WALK_STEPS {
            let mut layer = Vec::new();
            for w in walks_between(&KITE, 0, 3, k) {
                let bulge = bulges.next().unwrap_or(0.5);
                let mut pts: Vec<Vec3> = Vec::new();
                for pair in w.windows(2) {
                    pts.extend(bow(node(pair[0]), node(pair[1]), bulge, 10));
                }
                let m = polyline(pts).style(faded_line(gold, 5.0, brightness(k))).build();
                layer.push(stage.arena_mut().add(m));
            }
            layers.push(layer);
        }
        let heap = stage.add_value_tracker(0.0);
        let heap_label = kit.text(stage, "walks 1 to 4:", 30.0, Palette::dim(), v(2.6, 0.9))?;
        let heap_value = kit.live_value(
            stage,
            move |s| s.tracker_value(heap).unwrap_or(0.0),
            2,
            false,
            52.0,
            gold,
            v(4.3, 0.9),
            LEFT,
        )?;
        fade_in_all(stage, &[heap_label], 0.4)?;
        let running = numbers("F-G.exp.one_to_four.running")?;
        for (layer, &k) in layers.iter().zip(&WALK_STEPS) {
            fade_in_all(stage, layer, 0.6)?;
            play!(stage; tween(heap, running[k as usize], 0.9)?);
        }
        line.wait_for(stage, "There are infinitely many walks")?;
        let settled: f64 = fact("F-V.11_5.one_to_four")?
            .parse()
            .map_err(|e| fail(format!("F-V.11_5.one_to_four: {e}")))?;
        play!(stage; tween(heap, settled, 1.2)?);
        let discount = lower_third(
            &kit,
            stage,
            "for programmers:",
            "1/k! is one choice of discount; it always settles, and it gives e^A",
        )?;
        fade_in_all(stage, &[discount], 0.5)?;

        // ---- 11_6: communicability
        let line = say(&kit, stage, "11_6")?;
        let mut leaving = vec![series, heap_label, heap_value, discount];
        for layer in &layers {
            leaving.extend(layer);
        }
        fade_out_all(stage, &leaving, 0.5)?;
        // e^A, each cell shaded by its value
        let exp = matrix("F-G.exp")?;
        let largest = exp.iter().flatten().copied().fold(0.0, f64::max);
        let pitch = 0.95;
        let (left, top) = (A_AT[0] - 1.5 * pitch, A_AT[1] + 1.5 * pitch);
        let mut grid_in = Vec::new();
        for (i, row) in exp.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                let c = v(left + pitch * j as f64, top - pitch * i as f64);
                let r = Rectangle::new()
                    .width(pitch * 0.92)
                    .height(pitch * 0.92)
                    .style(flat(substance(), shade(x, largest)))
                    .build()?
                    .moved_to(c);
                grid_in.push(stage.arena_mut().add(r));
                grid_in.push(kit.text(stage, &show(x, 2)?, 28.0, ink, c)?);
            }
        }
        let name = kit.tex(
            stage,
            "e^{A}",
            48.0,
            ink,
            &[("e", gold), ("A", teal)],
            v(A_AT[0], top + 0.95),
        )?;
        grid_in.push(name);
        fade_in_all(stage, &grid_in, 0.8)?;
        let mark = |stage: &mut Stage<'_>, j: usize| -> fmn::Result<Mob> {
            let r = Rectangle::new()
                .width(pitch * 0.92)
                .height(pitch * 0.92)
                .style(faded_line(gold, 5.0, 1.0))
                .build()?
                .moved_to(v(left + pitch * j as f64, top));
            Ok(stage.arena_mut().add(r))
        };
        line.wait_for(stage, "Nodes one and two are linked")?;
        let m12 = mark(stage, 1)?;
        fade_in_all(stage, &[m12], 0.4)?;
        line.wait_for(stage, "So are one and three")?;
        let m13 = mark(stage, 2)?;
        fade_in_all(stage, &[m13], 0.4)?;
        line.wait_for(stage, "But one and three score higher")?;
        let note = kit.text(
            stage,
            &format!(
                "1 and 3: {}    1 and 2: {}",
                fact("F-V.11_6.one_to_three")?,
                fact("F-V.11_6.one_to_two")?
            ),
            30.0,
            gold,
            v(A_AT[0] + 0.2, top - 3.3),
        )?;
        fade_in_all(stage, &[note], 0.5)?;
        let who = lower_third(
            &kit,
            stage,
            "who and when:",
            &format!("Estrada and Hatano, {}", cited::ESTRADA_HATANO_YEAR),
        )?;
        fade_in_all(stage, &[who], 0.5)?;
        music_hold(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, numbers, show, testing};
    use super::*;
    use crate::model::expm;

    /// What has piled up at the far node after walks of up to `k` steps from
    /// `from`, each counted at one over k factorial: the model the facts'
    /// running total is checked against.
    fn pile(net: &Network, from: usize, to: usize, k: u32) -> f64 {
        (0..=k)
            .map(|j| net.walks(j)[from][to] as f64 / factorial(j) as f64)
            .sum()
    }

    /// Every walk the picture draws is a walk: its steps are links, and the
    /// walks between each pair are exactly as many as the matrix power
    /// counts, for every pair and every length shown or said.
    #[test]
    fn the_walks_drawn_are_the_walks_the_matrix_counts() {
        let a = KITE.adjacency();
        for k in 0..=8_u32 {
            let power = a.pow(k);
            for from in 0..KITE.n {
                for to in 0..KITE.n {
                    let walks = walks_between(&KITE, from, to, k);
                    assert_eq!(walks.len() as f64, power.get(from, to), "k = {k}, {from} to {to}");
                    for w in &walks {
                        assert_eq!(w.len(), k as usize + 1);
                        assert!(w.windows(2).all(|p| a.get(p[0], p[1]) == 1.0));
                    }
                }
            }
        }
        // from node 1 to node 4: one walk of two steps, one of three, four of four
        let counts: Vec<usize> = WALK_STEPS
            .iter()
            .map(|&k| walks_between(&KITE, 0, 3, k).len())
            .collect();
        assert_eq!(counts, [1, 1, 4]);
        let listed = numbers("F-G.walks.one_to_four").unwrap();
        for (k, &c) in listed.iter().enumerate() {
            assert_eq!(walks_between(&KITE, 0, 3, k as u32).len() as f64, c, "k = {k}");
        }
    }

    /// 11_1: the grid's ones are the links the facts list, both ways.
    #[test]
    fn the_grid_is_the_links_the_facts_list() {
        let links: Vec<String> = KITE
            .links
            .iter()
            .map(|&(i, j)| format!("{}-{}", i + 1, j + 1))
            .collect();
        assert_eq!(fact("F-G.links").unwrap(), links.join(","));
        let a = KITE.adjacency();
        for i in 0..4 {
            for j in 0..4 {
                let linked = KITE.links.iter().any(|&(x, y)| (x, y) == (i, j) || (x, y) == (j, i));
                assert_eq!(a.get(i, j), if linked { 1.0 } else { 0.0 });
                assert_eq!(a.get(i, j), a.get(j, i), "links go both ways");
            }
        }
        assert_eq!(fact("F-G.degrees").unwrap(), "2,2,3,1");
    }

    /// 11_2 and 11_4: the working written under the grids is the matrix
    /// product's, and node three's walks back to itself are three.
    #[test]
    fn the_working_is_the_product_and_node_three_has_three_ways_back() {
        let a = KITE.adjacency();
        assert_eq!(
            product_line(&a, 0, 3),
            r"0 \times 0 + 1 \times 0 + 1 \times 1 + 0 \times 0 = 1"
        );
        for i in 0..4 {
            for j in 0..4 {
                let line = product_line(&a, i, j);
                let total: f64 = line.rsplit("= ").next().unwrap().parse().unwrap();
                assert_eq!(total, a.mul(&a).get(i, j));
            }
        }
        assert_eq!(KITE.walks(2)[2][2], 3);
        assert_eq!(fact("F-Q.11_3.answer").unwrap(), "3");
        let back = walks_between(&KITE, 2, 2, 2);
        assert_eq!(back.len(), 3);
        // out along each of node 3's three links, and straight back
        let mut firsts: Vec<usize> = back.iter().map(|w| w[1]).collect();
        firsts.sort_unstable();
        assert_eq!(firsts, [0, 1, 3]);
    }

    /// 11_5: the running total beside node four is the facts' (the sum of
    /// walks over k factorial), it climbs to what e^A has in that entry, and
    /// each step more dims a walk.
    #[test]
    fn the_pile_climbs_to_the_entry_of_the_exponential() {
        let running = numbers("F-G.exp.one_to_four.running").unwrap();
        for (k, &r) in running.iter().enumerate() {
            assert!((pile(&KITE, 0, 3, k as u32) - r).abs() < 1e-3, "k = {k}");
        }
        assert!(running.windows(2).all(|w| w[1] >= w[0]));
        let limit = expm(&KITE.adjacency()).get(0, 3);
        assert!(
            (running.last().unwrap() - limit).abs() < 0.01,
            "{running:?} against {limit}"
        );
        assert_eq!(show(limit, 2).unwrap(), fact("F-V.11_5.one_to_four").unwrap());
        assert!(brightness(2) == 1.0 && brightness(3) < brightness(2) && brightness(4) < brightness(3));
    }

    /// 11_6: the shaded grid is e^A, and one and three do score higher than
    /// one and two, by the numbers on screen.
    #[test]
    fn one_and_three_score_higher_than_one_and_two() {
        let e = expm(&KITE.adjacency());
        assert!(e.get(0, 2) > e.get(0, 1));
        assert_eq!(show(e.get(0, 2), 2).unwrap(), fact("F-V.11_6.one_to_three").unwrap());
        assert_eq!(show(e.get(0, 1), 2).unwrap(), fact("F-V.11_6.one_to_two").unwrap());
        let largest = e.entries().iter().copied().fold(0.0, f64::max);
        assert!(shade(0.0, largest) > 0.0 && shade(largest, largest) < 0.9);
        assert!(shade(e.get(0, 2), largest) > shade(e.get(0, 1), largest));
    }

    #[test]
    fn arcs_and_tokens_geometry() {
        let (a, b) = (v(0.0, 0.0), v(4.0, 0.0));
        let arc = bow(a, b, 0.5, 8);
        assert_eq!((arc[0], arc[8]), (a, b));
        assert!(
            (arc[4][1] - 0.5).abs() < 1e-12,
            "the bulge is at the middle, to the left"
        );
        assert_eq!(arc.len(), 9);
        let stops = [v(0.0, 0.0), v(2.0, 0.0), v(2.0, 4.0)];
        assert_eq!(along(&stops, 0.0), stops[0]);
        assert_eq!(along(&stops, 1.0), stops[1]);
        assert_eq!(along(&stops, 1.5), v(2.0, 2.0));
        assert_eq!(along(&stops, 9.0), stops[2], "it stops at the end");
        // the layout keeps the links readable: no two nodes closer than three radii
        for (i, a) in NODE_AT.iter().enumerate() {
            for (j, b) in NODE_AT.iter().enumerate().skip(i + 1) {
                let d = (a[0] - b[0]).hypot(a[1] - b[1]);
                assert!(d > 3.0 * NODE_R, "nodes {i} and {j}: {d}");
            }
        }
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("11_walks").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            r"0 \times 0 + 1 \times 0 + 1 \times 1 + 0 \times 0 = 1",
            "two steps, from 3 back to 3",
            "1/k! is one choice of discount; it always settles, and it gives e^A",
            "1 and 3: 2.62    1 and 2: 2.42",
            "Estrada and Hatano, 2008",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
