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

use super::draw::{hold_band, lower_third, now};
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
/// 11_6's table of e^A: its cells' pitch (a cell is 0.92 of it), and how far
/// under the table's top row line the note of the two scores stands: three
/// pitches to the last row, half a cell, then the note's own half height and
/// room (at 3.3 the note stood on the last row's bottom edge).
const HEAT_PITCH: f64 = 0.95;
const SCORES_DROP: f64 = 3.75;
const SCORES_PT: f64 = 30.0;

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

/// How far from a node's centre a line between nodes stops: the disc's edge
/// and a hair more. The lit links, the arcs and the walks' traces were drawn
/// from centre to centre over the discs, and struck through the numerals.
pub const LINE_CLEAR: f64 = NODE_R + 0.03;

/// `pts` without the part of its start that lies within `LINE_CLEAR` of
/// `centre`, the new first point exactly on that circle.
fn trim_start(pts: Vec<Vec3>, centre: Vec3) -> Vec<Vec3> {
    let dist = |p: Vec3| (p[0] - centre[0]).hypot(p[1] - centre[1]);
    let Some(i) = pts.iter().position(|&p| dist(p) >= LINE_CLEAR) else {
        return pts;
    };
    if i == 0 {
        return pts;
    }
    let (a, b) = (pts[i - 1], pts[i]);
    let at = |t: f64| v(a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]));
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if dist(at(mid)) < LINE_CLEAR {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut out = vec![at(hi)];
    out.extend_from_slice(&pts[i..]);
    out
}

/// The points of a line from node `from` to node `to`, bowed `bulge` (see
/// [`bow`]), from the edge of one disc to the edge of the other.
pub fn link_path(from: usize, to: usize, bulge: f64, n: usize) -> Vec<Vec3> {
    let mut pts = trim_start(bow(node(from), node(to), bulge, n), node(from));
    pts.reverse();
    let mut pts = trim_start(pts, node(to));
    pts.reverse();
    pts
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

/// The radius of the gold token that walks the network (11_2's walker, 12_3's
/// hopper), and how far it stands from the node it is "on": up and to the
/// right, on the disc's rim, so it never covers the numeral (it stood on the
/// centre and hid the 4 for 9 s).
pub const TOKEN_R: f64 = 0.14;
pub const TOKEN_OFFSET: [f64; 2] = [0.26, 0.26];

/// Where the token stands when it is at the place `p` of the network (a node,
/// or a point of a link): `p` shifted by `TOKEN_OFFSET`.
pub fn token_at(p: Vec3) -> Vec3 {
    v(p[0] + TOKEN_OFFSET[0], p[1] + TOKEN_OFFSET[1])
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
            // lit from disc to disc: it comes in after the numerals, so a
            // line to the centres would be drawn over them
            let on = polyline(link_path(a, b, 0.0, 2))
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

/// A level for each cell of `grid`, row by row, then its two brackets (so
/// the brackets are pieces `n * n` and `n * n + 1`). The brackets are open
/// lines, so their level is a stroke alpha: the fill alpha that
/// `set_opacity` also writes turns an open bracket into a solid bar.
pub fn grid_levels(stage: &mut Stage<'_>, grid: &MatrixGrid) -> fmn::Result<Levels> {
    let mut pieces: Vec<Mob> = grid.cells.iter().flatten().copied().collect();
    pieces.extend(grid.brackets);
    Levels::build_with_strokes(stage, &pieces, &grid.brackets, 0.0)
}

/// How far a cell's outline stands out from the number on each side.
pub const OUTLINE_PAD: f64 = 0.15;

/// A rounded outline around the cell at (`row`, `col`) of `grid`: a square
/// of a row's pitch for a digit, as wide as the number and `OUTLINE_PAD`
/// either side when that is more (a square cut through the digits of
/// -0.839).
pub fn cell_outline(stage: &mut Stage<'_>, grid: &MatrixGrid, row: usize, col: usize, color: Srgb) -> fmn::Result<Mob> {
    let cell = grid.cells[row][col];
    let c = stage.get_center(cell);
    let wide = stage.get_width(cell) + 2.0 * OUTLINE_PAD;
    let r = Rectangle::new()
        .width((grid.pitch * 0.9).max(wide))
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
        let levels = grid_levels(stage, &a_grid)?;
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
            ins.push(stage.prepare(levels.tween(KITE.n * KITE.n + b, 1.0, 0.5)?)?);
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
        let note_up = now(stage);

        // ---- 11_2: multiply it by itself: two-step walks
        let line = say(&kit, stage, "11_2")?;
        // the band came as 11_1's zeros faded: on the finals' read the next
        // line began 2.0 s later
        hold_band(stage, note_up)?;
        fade_out_all(stage, &[adjacency_note], 0.4)?;
        let two = KITE.walks(2);
        let a2_rows: Vec<Vec<String>> = two.iter().map(|r| r.iter().map(|&x| x.to_string()).collect()).collect();
        let a2_grid = MatrixGrid::build(&kit, stage, &a2_rows, GRID_PT, gold, A2_AT)?;
        let a2_name = kit.tex(stage, r"A^2", 48.0, gold, &[], v(A2_AT[0], A2_AT[1] + 1.55))?;
        line.wait_for(stage, "A walk is a route")?;
        // a walk: one token goes 1, 3, 4 along the links
        let token = stage.arena_mut().add(
            Dot::new()
                .point(token_at(node(0)))
                .radius(TOKEN_R)
                .style(flat(gold, 1.0))
                .build(),
        );
        // (it rides the links shifted to the discs' rims, so the numerals stay)
        let stops = [token_at(node(0)), token_at(node(2)), token_at(node(3))];
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
                let a = polyline(link_path(from, to, 0.28, 14))
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
                // one line per step, each from disc to disc: a walk's trace
                // through a node would run over its numeral
                for pair in w.windows(2) {
                    let m = polyline(link_path(pair[0], pair[1], bulge, 10))
                        .style(faded_line(gold, 5.0, brightness(k)))
                        .build();
                    layer.push(stage.arena_mut().add(m));
                }
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
        let pitch = HEAT_PITCH;
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
            SCORES_PT,
            gold,
            v(A_AT[0] + 0.2, top - SCORES_DROP),
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
    use fmn::scene::{NullSceneSink, RuntimeConfig};

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

    /// 11_6 (the dense read of the scored draft): the note of the two scores
    /// stands under the table of e^A, clear of its last row's bottom edge
    /// (at the old offset the note's text stood on it), and over the lower
    /// third that comes up under it (its box tops out at -3.14).
    #[test]
    fn the_scores_stand_clear_of_the_table_and_the_lower_third() {
        let top = A_AT[1] + 1.5 * HEAT_PITCH;
        let last_row_bottom = top - 3.0 * HEAT_PITCH - HEAT_PITCH * 0.92 / 2.0;
        let half_text = SCORES_PT * 0.0125 / 2.0;
        let note = top - SCORES_DROP;
        assert!(
            note + half_text < last_row_bottom - 0.1,
            "the note's top {:.2} against the table's foot {last_row_bottom:.2}",
            note + half_text
        );
        let band_top = -4.0 + 0.2 + 0.3 + 0.36;
        assert!(
            note - half_text > band_top + 0.2,
            "{:.2} against {band_top:.2}",
            note - half_text
        );
        // inside the frame sideways: 30 pt of "1 and 3: 2.62    1 and 2: 2.42"
        let width = "1 and 3: 2.62    1 and 2: 2.42".chars().count() as f64 * SCORES_PT * 0.0045;
        assert!(A_AT[0] + 0.2 + width / 2.0 < 7.11 - 0.15, "{width}");
    }

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// 11_2 and 12_3 (GrayCave's `covered` rule): the gold token that walks
    /// the network never hides a node's numeral: wherever it is at a node (or
    /// half way along a link) its disc stands clear of every numeral's box,
    /// and at a node it still touches the node's disc, so it reads as being
    /// there. It rested on the centre of node 4 and covered the 4 for 9 s.
    #[test]
    fn the_token_never_covers_a_numeral() {
        run(|stage| {
            let kit = Kit::new()?;
            let mut spots: Vec<(String, Vec3)> = (0..KITE.n).map(|i| (format!("node {}", i + 1), node(i))).collect();
            for &(a, b) in KITE.links {
                let (p, q) = (node(a), node(b));
                spots.push((
                    format!("the middle of link {}-{}", a + 1, b + 1),
                    v((p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0),
                ));
            }
            for (name, spot) in &spots {
                let at = token_at(*spot);
                for i in 0..KITE.n {
                    let numeral = kit.text(stage, &(i + 1).to_string(), 34.0, Palette::ink(), node(i))?;
                    let (lo, hi) = (
                        [stage.get_left(numeral)[0], stage.get_bottom(numeral)[1]],
                        [stage.get_right(numeral)[0], stage.get_top(numeral)[1]],
                    );
                    // the nearest point of the numeral's box to the token's centre
                    let (nx, ny) = (at[0].clamp(lo[0], hi[0]), at[1].clamp(lo[1], hi[1]));
                    let gap = (at[0] - nx).hypot(at[1] - ny) - TOKEN_R;
                    assert!(gap >= 0.04, "{name}: the token is {gap:.2} from the numeral {}", i + 1);
                }
            }
            // at a node it still touches the disc
            for i in 0..KITE.n {
                let (c, t) = (node(i), token_at(node(i)));
                let d = (t[0] - c[0]).hypot(t[1] - c[1]);
                assert!(d - TOKEN_R <= NODE_R, "node {}: the token stands {d:.2} off", i + 1);
            }
            Ok(())
        });
    }

    /// 11_1, 11_4 and 11_5 (the dense read of the scored draft): the lit
    /// links, the arcs and the walks' traces came in after the numerals and
    /// ran from centre to centre, so each struck through the digits. A line
    /// between nodes now starts and ends on its discs' edges and comes
    /// nowhere near the centre of any node, for every link, both ways, and
    /// every bulge the chapter draws.
    #[test]
    fn a_line_between_nodes_stops_at_the_discs_and_never_crosses_a_numeral() {
        let bulges = [0.0, 0.14, -0.14, 0.28, -0.28, 0.42, 0.5];
        let from_centre = |p: Vec3, i: usize| (p[0] - node(i)[0]).hypot(p[1] - node(i)[1]);
        for &(a, b) in KITE.links {
            for (from, to) in [(a, b), (b, a)] {
                for &bulge in &bulges {
                    for n in [2, 10, 14] {
                        let path = link_path(from, to, bulge, n);
                        assert!(path.len() >= 2, "{from} to {to}: {path:?}");
                        let (first, last) = (path[0], path[path.len() - 1]);
                        assert!(
                            (from_centre(first, from) - LINE_CLEAR).abs() < 1e-6,
                            "{from} to {to}, bulge {bulge}: starts {} from its centre",
                            from_centre(first, from)
                        );
                        assert!(
                            (from_centre(last, to) - LINE_CLEAR).abs() < 1e-6,
                            "{from} to {to}, bulge {bulge}: ends {} from its centre",
                            from_centre(last, to)
                        );
                        for &p in &path {
                            for i in 0..KITE.n {
                                assert!(
                                    from_centre(p, i) > NODE_R,
                                    "{from} to {to}, bulge {bulge}: {:.3} from node {i}'s centre, inside its disc",
                                    from_centre(p, i)
                                );
                            }
                        }
                    }
                }
            }
        }
        // the dim links, drawn centre to centre under the discs, are the
        // lit ones' length and more: the lit line is shorter by two edges
        let lit = link_path(0, 2, 0.0, 2);
        let full = bow(node(0), node(2), 0.0, 2);
        let len = |p: &[Vec3]| {
            p.windows(2)
                .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                .sum::<f64>()
        };
        assert!((len(&full) - len(&lit) - 2.0 * LINE_CLEAR).abs() < 1e-6);
    }

    /// 13_3 (the dense read): the outline round a cell is as wide as its
    /// number and clear of it on both sides (a square cut through the digits
    /// of -0.839), and a digit's cell keeps the square.
    #[test]
    fn an_outline_stands_clear_of_the_number_it_rings() {
        run(|stage| {
            let kit = Kit::new()?;
            let rows = vec![
                vec!["-0.839".to_string(), "1.234".to_string()],
                vec!["10.5".to_string(), "0.003".to_string()],
            ];
            let grid = MatrixGrid::build(&kit, stage, &rows, 60.0, rule(), [0.0, 0.0, 0.0])?;
            for (r, c) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let ring = cell_outline(stage, &grid, r, c, result())?;
                let cell = grid.cells[r][c];
                let (cl, cr) = (stage.get_left(cell)[0], stage.get_right(cell)[0]);
                let (rl, rr) = (stage.get_left(ring)[0], stage.get_right(ring)[0]);
                assert!(
                    rl < cl - 0.1 && rr > cr + 0.1,
                    "cell {r},{c}: the ring spans {rl:.2} to {rr:.2}, the number {cl:.2} to {cr:.2}"
                );
                let (ct, cb) = (stage.get_top(cell)[1], stage.get_bottom(cell)[1]);
                let (rt, rb) = (stage.get_top(ring)[1], stage.get_bottom(ring)[1]);
                assert!(
                    rt > ct + 0.1 && rb < cb - 0.1,
                    "cell {r},{c}: the ring is too low or too tight"
                );
            }
            // a digit at 36 pt keeps the square of a row's pitch
            let small = MatrixGrid::build(&kit, stage, &[vec!["3".to_string()]], 36.0, result(), [0.0, 0.0, 0.0])?;
            let ring = cell_outline(stage, &small, 0, 0, result())?;
            assert!((stage.get_width(ring) - small.pitch * 0.9).abs() < 0.02);
            Ok(())
        });
    }

    /// 11_1 (the dense read): the grid's brackets come in by their stroke
    /// alone. `set_opacity` on an open bracket also writes its fill, and
    /// the fill of "[" is the solid bar between its path and its chord: A was
    /// drawn with thick green bars. At full level a bracket has no fill and a
    /// full stroke, and the cells are full.
    #[test]
    fn the_brackets_of_a_grid_stay_lines_as_its_levels_rise() {
        use explainer_kit::narration::wait_frames;
        run(|stage| {
            let kit = Kit::new()?;
            let grid = MatrixGrid::build(&kit, stage, &whole_rows(&KITE.adjacency()), GRID_PT, rule(), A_AT)?;
            let levels = grid_levels(stage, &grid)?;
            stage.add_to_scene(grid.group)?;
            let n = KITE.n * KITE.n;
            assert_eq!(levels.trackers.len(), n + 2, "the cells, then the two brackets");
            wait_frames(stage, 2)?;
            for &b in &grid.brackets {
                assert_eq!(
                    stage.arena().get_fill_opacity(b),
                    Some(0.0),
                    "hidden bracket has no fill"
                );
                assert_eq!(stage.arena().get_stroke_opacity(b), Some(0.0), "and no stroke");
            }
            for &t in &levels.trackers {
                stage.set_tracker_value(t, 1.0)?;
            }
            wait_frames(stage, 2)?;
            for &b in &grid.brackets {
                assert_eq!(
                    stage.arena().get_fill_opacity(b),
                    Some(0.0),
                    "a bracket is a stroke, not a bar"
                );
                let o = stage.arena().get_stroke_opacity(b).unwrap_or(0.0);
                assert!((o - 1.0).abs() < 1e-6, "its stroke is at its level: {o}");
            }
            // a typeset cell is a group: the most opaque of its family
            let cell = grid.cells[0][1];
            let o = stage
                .arena()
                .family(cell)
                .into_iter()
                .fold(0.0_f64, |o, m| o.max(stage.arena().get_fill_opacity(m).unwrap_or(0.0)));
            assert!((o - 1.0).abs() < 1e-6, "a cell's fill is at its level: {o}");
            Ok(())
        });
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
