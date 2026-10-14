//! Chapter 05, slabs: why the cubes wrap round each other. Each cube is its
//! slabs (05_1: five cubes, fifteen slabs, the same fifteen that wait in the
//! corner from chapter 04); the slabs lie flat as L-shaped layers, the single
//! tile first, then the two cube round it with one slab cut in half (05_2),
//! the three cube with no cut (05_3); the quiz of how many slabs the four
//! cube must cut (05_4); one, because each arm is six long and the slabs are
//! four wide (05_5); the five cube with none, the square fifteen on a side,
//! and the 15 that waited lands on `1 + 2 + 3 + 4 + 5` (05_6); the two
//! countings of the same 225 tiles (05_7).
//!
//! The pieces are `model::cubes::layer(k)` exactly: a corner slab, whole
//! slabs along each arm, and for an even cube one slab cut in two, a half to
//! each arm. Every number on screen is a fact (`F-K.*`, `F-T.count`,
//! `F-Q.cut_slabs.answer`) checked against the model before anything shows.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::{Kit, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::cold_open::{ROW_Y, lifted, row_x, stacked};
use super::draw::{
    ANSWER_PT, Board, Cell, Dimension, QuizCard, QuizLook, Side, Words, breath, dim, fact, fact_int, fact_ints,
    fade_in_all, fade_in_lagged, fade_out_all, fade_to, faded_line, fail, flash, fly, hero, ink, line_style,
    live_counter, quiz_named, say, size, slide, swap, tex_row, tween_linear,
};
use super::iso::{Iso, Solid, ViewAt, box_cells};
use super::kinetics::{together, zoom_about};
use super::staircases::WAITING_AT;
use crate::facts::shown;
use crate::model::Rect;
use crate::model::cubes::{self, Part, Piece};

pub struct Slabs {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6", "05_7"];

// the chapter's fixed words (numbers are built from facts)
const SLABS: &str = "slabs";
const SLAB: &str = "slab";
const TILES: &str = "tiles";
const WIDE: &str = "wide";
const LONG: &str = "long";
const QUESTION: &str = "slabs to cut: ?";
/// The quiz this chapter asks (`model::QUIZZES`).
const QUIZ: &str = "cut_slabs";

// ------------------------------------------------------------------ geometry

/// The board's lower left corner, scene units, and a tile's side while each
/// layer is built (the picture is pulled back as the square grows: the
/// finished layer's square and the row of slabs over it fill the room under
/// the shelf). Five cubes' fifteen tiles are 4.2 across at the last.
const BOARD_AT: Vec3 = [-6.5, -3.0, 0.0];
const CELLS: [f64; 5] = [1.2, 0.95, 0.58, 0.36, 0.28];
/// How far the halves of a cut slab open, scene units.
const OPEN: f64 = 0.12;
/// The middle of the right column, and where the sides' row is set.
const COLUMN_X: f64 = 3.4;
const SIDES_AT: Vec3 = [3.7, 2.72, 0.0];
/// How big the sides' row and the braces' words are set.
const SIDES_PT: f64 = 36.0;
const BRACE_PT: f64 = 32.0;
const ARM_PT: f64 = 28.0;
const REASON_PT: f64 = 38.0;
const TAG_PT: f64 = 30.0;
const COUNT_PT: f64 = 96.0;
const FORMULA_PT: f64 = 44.0;
/// The room of the row of cubes (05_1), centred, and the flattened row.
const ROW_AT: (f64, f64) = (0.0, -0.5);
const ROW_ROOM: (f64, f64) = (13.0, 5.4);
const FLAT_ROOM: (f64, f64) = (12.6, 3.0);

/// The board while layer `k` is built.
fn board_of(k: i64) -> Board {
    Board::new(BOARD_AT, CELLS[(k - 1) as usize])
}

/// The row of `n` cubes seen as ever (classic), fitted to its room.
pub fn row_view(n: i64) -> ViewAt {
    let width = row_x(n) + n as f64;
    let high = lifted(n, n - 1)[2] + 1.0;
    ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0, ROW_Y, 0.0], [width, ROW_Y + n as f64, high], ROW_AT, ROW_ROOM)
}

/// The same row seen from straight above, the cubes flat squares.
fn flat_row_view(n: i64) -> ViewAt {
    let width = row_x(n) + n as f64;
    ViewAt::top_down((0.0, 0.0), 1.0).fitted([0.0, ROW_Y, 0.0], [width, ROW_Y + n as f64, 1.0], ROW_AT, FLAT_ROOM)
}

/// Where slab `j` of cube `k` waits, in cells: in a row along the top of
/// the square the layer will make, one cell above it, a slab and a gap
/// apart.
fn waiting(k: i64, j: i64) -> Rect {
    Rect::new(j * (k + 1), cubes::side(k) + 1, k, k)
}

/// The cells of layer `k`: those of the square of side `side(k)` that the
/// square before it does not have.
fn layer_cells(k: i64) -> Vec<Cell> {
    let (s, t) = (cubes::side(k - 1), cubes::side(k));
    (0..t)
        .flat_map(|x| (0..t).map(move |y| (x, y)))
        .filter(|&(x, y)| x >= s || y >= s)
        .collect()
}

/// `n/d` as the video says it: a whole number, or with a half (`3/2` is
/// `1½`). Nothing else is shown.
fn mixed(value: &str) -> fmn::Result<String> {
    let (n, d) = match value.split_once('/') {
        Some((n, d)) => (n.parse::<i64>(), d.parse::<i64>()),
        None => (value.parse::<i64>(), Ok(1)),
    };
    let (Ok(n), Ok(d)) = (n, d) else {
        return Err(fail(format!("{value:?} is not a fraction")));
    };
    match (n / d, n % d, d) {
        (whole, 0, _) => Ok(whole.to_string()),
        (0, 1, 2) => Ok("½".to_owned()),
        (whole, 1, 2) => Ok(format!("{whole}½")),
        _ => Err(fail(format!("{value:?} is not shown as whole numbers and halves"))),
    }
}

/// One cube of the row: its slabs.
struct Stack {
    cube: i64,
    slabs: Vec<Solid>,
}

/// The seam of a half in the finished square, as two cell corners: along the
/// edge its slab was cut on (the lower half's upper side; the upper half,
/// turned, its right side).
fn seam_cells(piece: &Piece) -> Option<((f64, f64), (f64, f64))> {
    let r = piece.rect;
    let (x, y, w, h) = (r.x as f64, r.y as f64, r.w as f64, r.h as f64);
    match piece.part {
        Part::Whole => None,
        Part::HalfRight => Some(((x, y + h), (x + w, y + h))),
        Part::HalfTop => Some(((x + w, y), (x + w, y + h))),
    }
}

/// A slab of layer `k` (one of its `k` by `k` squares) waiting in its row,
/// in the cube's colour. Detached.
fn waiting_slab(stage: &mut Stage<'_>, board: &Board, k: i64, j: i64) -> Mob {
    board.slab(stage, waiting(k, j), size(k as usize))
}

/// The cut slab's two halves, where the slab waits: the lower one and the
/// upper one, each `k` wide and `k / 2` tall. Detached.
fn halves(stage: &mut Stage<'_>, board: &Board, k: i64) -> (Mob, Mob) {
    let at = waiting(k, k - 1);
    let colour = size(k as usize);
    let lower = board.slab(stage, Rect::new(at.x, at.y, k, k / 2), colour);
    let upper = board.slab(stage, Rect::new(at.x, at.y + k / 2, k, k / 2), colour);
    (lower, upper)
}

/// The seams of layer `k`'s halves, set on the board. Detached.
fn seams(stage: &mut Stage<'_>, board: &Board, pieces: &[Piece]) -> Vec<Mob> {
    pieces
        .iter()
        .filter_map(seam_cells)
        .map(|(a, b)| {
            stage.arena_mut().add(
                polyline(vec![board.p(a.0, a.1), board.p(b.0, b.1)])
                    .style(line_style(hero(), 3.0))
                    .build(),
            )
        })
        .collect()
}

/// A slab's move to its piece's place: its centre to the piece's centre.
fn to_place(board: &Board, slab: Mob, piece: &Piece, rt: f64) -> fmn::Result<AnimBuilder> {
    fly(slab, board.rect_centre(piece.rect), 1.0, rt)
}

/// Cut the slab `whole` (cube `k`'s last) into its two halves and carry
/// them to the feet of the arms, the upper one turned a quarter turn on the
/// way. `pieces` are the layer's pieces; the seams come back, drawn.
fn cut_and_place(
    kit: &Kit,
    stage: &mut Stage<'_>,
    board: &Board,
    whole: Mob,
    (lower, upper): (Mob, Mob),
    pieces: &[Piece],
) -> fmn::Result<Vec<Mob>> {
    let find = |part: Part| {
        pieces
            .iter()
            .find(|p| p.part == part)
            .ok_or_else(|| fail(format!("the layer has no {part:?}")))
    };
    let (right, top) = (find(Part::HalfRight)?, find(Part::HalfTop)?);
    // the slab is cut: it gives way to its halves, which open and glow
    swap(stage, &[whole], &[lower, upper], 0.1)?;
    kit.mark(stage, "pizz:split");
    together(
        stage,
        vec![slide(lower, v(0.0, -OPEN), 0.4)?, slide(upper, v(0.0, OPEN), 0.4)?],
        0.0,
    )?;
    flash(stage, &[lower, upper], hero(), 1.1, 0.5)?;
    // the upper half turns a quarter turn about its middle, the lower goes
    let turn = Rotating::new(upper).with_angle(FRAC_PI_2).rt(0.5);
    let drop = to_place(board, lower, right, 0.9)?;
    play!(stage; turn, drop);
    stage.play(to_place(board, upper, top, 0.9)?)?;
    let marks = seams(stage, board, pieces);
    fade_in_all(stage, &marks, 0.4)?;
    Ok(marks)
}

/// The picture is pulled back for the next layer: every mobject of the
/// board shrinks about its corner from the tile `from` to the board `to`'s,
/// and the brace of the square so far is made again at the new size. The new
/// brace comes back, drawn.
fn pull_back(
    kit: &Kit,
    stage: &mut Stage<'_>,
    from: f64,
    to: &Board,
    on_board: &[Mob],
    old: &Dimension,
    side: i64,
) -> fmn::Result<Dimension> {
    let factor = to.cell / from;
    let fresh = side_brace(kit, stage, to, side)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in on_board {
        anims.push(stage.prepare(zoom_about(m, factor, BOARD_AT, 0.9)?)?);
    }
    for m in old.mobs() {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.5))?);
    }
    for m in fresh.mobs() {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.5))?);
    }
    stage.play_prepared(anims)?;
    Ok(fresh)
}

/// The brace under the square of side `s` cells with its number. Detached.
fn side_brace(kit: &Kit, stage: &mut Stage<'_>, board: &Board, side: i64) -> fmn::Result<Dimension> {
    board.dimension(
        kit,
        stage,
        Rect::new(0, 0, side, side),
        Side::Below,
        Words::Text(&side.to_string()),
        BRACE_PT,
        ink(),
    )
}

/// An arm's two braces, `wide` under it and `long` beside it, in the
/// cube's colour. Detached.
fn arm_braces(
    kit: &Kit,
    stage: &mut Stage<'_>,
    board: &Board,
    k: i64,
    [wide, long]: [i64; 2],
) -> fmn::Result<Vec<Mob>> {
    let arm = Rect::new(cubes::side(k - 1), 0, wide, long);
    let under = board.dimension(
        kit,
        stage,
        arm,
        Side::Below,
        Words::Text(&format!("{wide} {WIDE}")),
        ARM_PT,
        size(k as usize),
    )?;
    let beside = board.dimension(
        kit,
        stage,
        arm,
        Side::Right,
        Words::Text(&format!("{long} {LONG}")),
        ARM_PT,
        size(k as usize),
    )?;
    let mut out = under.mobs().to_vec();
    out.extend(beside.mobs());
    Ok(out)
}

/// The words of a fixed sentence in the right column's middle.
fn reason_line(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, REASON_PT, hero(), v(COLUMN_X, 0.55))
}

/// Light item `i` of the row of sides.
fn light(row: &[Mob], i: usize) -> fmn::Result<AnimBuilder> {
    fade_to(row[i], 1.0, 0.4)
}

impl SceneConstruct for Slabs {
    fn name(&self) -> &str {
        "05_slabs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = shown::CUBES;
        if n as usize != CELLS.len() {
            return Err(fail("the pulled-back sizes are not one a cube".into()));
        }

        // the numbers, and the model they must agree with
        let sides = fact_ints("F-K.sides")?;
        let side = fact_int("F-K.side")?;
        let slab_count = fact_int("F-K.slabs")?;
        let waiting_fifteen = fact_int("F-T.count")?;
        let areas = fact_ints("F-K.layer_areas")?;
        let unit_squares = fact_int("F-K.unit_squares")?;
        let arm4 = fact_ints("F-K.arm.4")?;
        let arm4_slabs = mixed(&fact("F-K.arm.4.slabs")?)?;
        let arm5 = fact_ints("F-K.arm.5")?;
        let arm5_slabs = mixed(&fact("F-K.arm.5.slabs")?)?;
        let answer = fact_int("F-Q.cut_slabs.answer")?;
        let model_sides: Vec<i64> = (1..=n).map(cubes::side).collect();
        if sides != model_sides || side != cubes::side(n) {
            return Err(fail("F-K.sides are not the cubes' running sums".into()));
        }
        if slab_count != n * (n + 1) / 2 || slab_count != waiting_fifteen {
            return Err(fail(
                "the cubes' slabs are not the staircase's fifteen that waits".into(),
            ));
        }
        if areas != (1..=n).map(|k| k * k * k).collect::<Vec<_>>() || unit_squares != side * side {
            return Err(fail("the layers' areas are not the cubes".into()));
        }
        if arm4 != [4, cubes::side(3)] || arm5 != [5, cubes::side(4)] {
            return Err(fail("the arms of layers 4 and 5 are not the model's".into()));
        }
        let cut_in_four = cubes::layer(4).iter().filter(|p| p.part == Part::HalfRight).count() as i64;
        if answer != cut_in_four || cubes::layer(5).iter().any(|p| p.part != Part::Whole) {
            return Err(fail("the quiz's answer is not the model's cut slabs".into()));
        }

        let iso = Iso::new(stage, row_view(n))?;
        // the mobjects of each layer once it is down, for 05_7
        let mut layers: Vec<Vec<Mob>> = Vec::new();

        // ---------------------------------------------------------- 05_1
        // five cubes come apart into their slabs: fifteen, the fifteen
        // that waits
        let line = say(&kit, stage, LINES[0])?;
        let mut stacks: Vec<Stack> = Vec::new();
        for k in 1..=n {
            let mut slabs = Vec::new();
            for j in 0..k {
                slabs.push(iso.solid(stage, &box_cells(k, k, 1), size(k as usize), stacked(k, j))?);
            }
            stacks.push(Stack { cube: k, slabs });
        }
        let fifteen = kit.text(stage, &waiting_fifteen.to_string(), ANSWER_PT, hero(), WAITING_AT)?;
        fade_in_all(stage, &[fifteen], 0.4)?;
        let mut moves = Vec::new();
        for s in &stacks {
            for &solid in &s.slabs {
                moves.push(iso.fade(solid, 1.0, 0.5)?);
            }
        }
        iso.play(stage, moves, 0.05)?;
        // each cube lifts into its slabs on its own words, and a tag says
        // how many
        let mut tags: Vec<Mob> = Vec::new();
        for (phrase, ks) in [
            ("is five slabs", vec![5]),
            ("is four slabs", vec![4]),
            ("Across all five cubes", vec![3, 2, 1]),
        ] {
            line.wait_for(stage, phrase)?;
            let mut moves = Vec::new();
            let mut fresh = Vec::new();
            for k in ks {
                let s = &stacks[(k - 1) as usize];
                for (j, &solid) in s.slabs.iter().enumerate() {
                    moves.push(iso.slide(stage, solid, lifted(s.cube, j as i64), 0.8)?);
                }
                let top = lifted(k, k - 1);
                let at = iso.project(stage, [row_x(k) + k as f64 / 2.0, ROW_Y + k as f64, top[2] + 1.0]);
                let word = if k == 1 { SLAB } else { SLABS };
                fresh.push(kit.text(
                    stage,
                    &format!("{k} {word}"),
                    TAG_PT,
                    size(k as usize),
                    v(at[0], at[1] + 0.4),
                )?);
            }
            iso.play(stage, moves, 0.1)?;
            fade_in_all(stage, &fresh, 0.4)?;
            tags.extend(fresh);
        }
        line.wait_for(stage, "fifteen slabs")?;
        let slabs_word = kit.text(stage, SLABS, 34.0, dim(), v(WAITING_AT[0], WAITING_AT[1] - 0.75))?;
        fade_in_all(stage, &[slabs_word], 0.4)?;
        flash(stage, &[fifteen], hero(), 1.15, 0.6)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 05_2
        // the row tilts flat and gives way to the board: the single tile,
        // then the two cube's slabs, one of them cut in half
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, "Lay them flat")?;
        let mut flatten: Vec<Box<dyn Animation>> = Vec::new();
        for m in iso.view_to(flat_row_view(n), 1.0)? {
            flatten.push(stage.prepare(m)?);
        }
        let mut gone = tags.clone();
        gone.push(slabs_word);
        for m in gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            flatten.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(flatten)?;
        // the flat squares stay while they are told: each as wide as its
        // cube; they give way to the board as the single tile goes first
        line.wait_for(stage, "The single tile goes first")?;
        play!(stage; iso.fade_all(0.0, 0.6)?);
        // the sides' row 1, 3, 6, 10, 15, each lit as its layer lands
        let items: Vec<String> = sides
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i + 1 < sides.len() {
                    format!("{s},")
                } else {
                    s.to_string()
                }
            })
            .collect();
        let parts: Vec<(&str, Srgb)> = items.iter().map(|s| (s.as_str(), ink())).collect();
        let row = tex_row(&kit, stage, &parts, SIDES_PT, SIDES_AT, 0.22, 4.4)?;
        for &m in &row {
            stage.set_opacity(m, 0.0, true);
            stage.add_to_scene(m)?;
        }
        together(
            stage,
            row.iter().map(|&m| fade_to(m, 0.3, 0.5)).collect::<fmn::Result<_>>()?,
            0.0,
        )?;
        // layer 1: the single tile
        let one = cubes::layer(1);
        let b1 = board_of(1);
        let tile = b1.slab(stage, one[0].rect, size(1));
        fade_in_all(stage, &[tile], 0.4)?;
        layers.push(vec![tile]);
        kit.mark(stage, "pizz:layer");
        let mut brace = side_brace(&kit, stage, &b1, sides[0])?;
        fade_in_all(stage, &brace.mobs(), 0.4)?;
        together(stage, vec![light(&row, 0)?], 0.0)?;
        // layer 2: two slabs wait, one takes the corner, the other is cut
        line.wait_for(stage, "The two cube wraps around it")?;
        let b2 = board_of(2);
        let on_board: Vec<Mob> = layers.iter().flatten().copied().collect();
        brace = pull_back(&kit, stage, CELLS[0], &b2, &on_board, &brace, sides[0])?;
        let two = cubes::layer(2);
        let corner2 = waiting_slab(stage, &b2, 2, 0);
        let cut2 = waiting_slab(stage, &b2, 2, 1);
        fade_in_lagged(stage, &[corner2, cut2], 0.5, 0.3)?;
        line.wait_for(stage, "One slab fits the corner")?;
        let corner = two
            .iter()
            .find(|p| p.part == Part::Whole)
            .ok_or_else(|| fail("layer 2 has no corner".into()))?;
        stage.play(to_place(&b2, corner2, corner, 0.8)?)?;
        line.wait_for(stage, "must be cut in half")?;
        let pair = halves(stage, &b2, 2);
        let seams2 = cut_and_place(&kit, stage, &b2, cut2, pair, &two)?;
        kit.mark(stage, "pizz:layer");
        let mut mobs = vec![corner2, pair.0, pair.1];
        mobs.extend(seams2);
        layers.push(mobs);
        let next = side_brace(&kit, stage, &b2, sides[1])?;
        swap(stage, &brace.mobs(), &next.mobs(), 0.4)?;
        brace = next;
        together(stage, vec![light(&row, 1)?], 0.0)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 05_3
        // the three cube needs no cutting: a corner and a slab on each arm
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, "The three cube needs no cutting")?;
        let b3 = board_of(3);
        let on_board: Vec<Mob> = layers.iter().flatten().copied().collect();
        brace = pull_back(&kit, stage, CELLS[1], &b3, &on_board, &brace, sides[1])?;
        let three = cubes::layer(3);
        let waiting3: Vec<Mob> = (0..3).map(|j| waiting_slab(stage, &b3, 3, j)).collect();
        fade_in_lagged(stage, &waiting3, 0.5, 0.25)?;
        line.wait_for(stage, "One slab on the corner")?;
        // the pieces are the corner, then one for each arm
        let mut landing = Vec::new();
        for (slab, piece) in waiting3.iter().zip(&three) {
            landing.push(to_place(&b3, *slab, piece, 0.8)?);
        }
        let arms = landing.split_off(1);
        together(stage, landing, 0.0)?;
        line.wait_for(stage, "one on each arm")?;
        together(stage, arms, 0.3)?;
        kit.mark(stage, "pizz:layer");
        layers.push(waiting3.clone());
        line.wait_for(stage, "six on a side")?;
        let next = side_brace(&kit, stage, &b3, sides[2])?;
        swap(stage, &brace.mobs(), &next.mobs(), 0.4)?;
        brace = next;
        together(stage, vec![light(&row, 2)?], 0.0)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 05_4
        // your turn: the four cube's layer, its slabs waiting, its arms
        // marked 4 wide and 6 long
        let quiz = quiz_named(QUIZ)?;
        let line = say(&kit, stage, quiz.line)?;
        line.wait_for(stage, "The four cube has four slabs")?;
        let b4 = board_of(4);
        let on_board: Vec<Mob> = layers.iter().flatten().copied().collect();
        brace = pull_back(&kit, stage, CELLS[2], &b4, &on_board, &brace, sides[2])?;
        let four = cubes::layer(4);
        let layer4 = b4.outline(stage, &layer_cells(4), faded_line(ink(), 3.0, 0.7))?;
        let waiting4: Vec<Mob> = (0..4).map(|j| waiting_slab(stage, &b4, 4, j)).collect();
        fade_in_all(stage, &[layer4], 0.5)?;
        fade_in_lagged(stage, &waiting4, 0.5, 0.2)?;
        line.wait_for(stage, "How many of them")?;
        let arm_marks = arm_braces(&kit, stage, &b4, 4, [arm4[0], arm4[1]])?;
        fade_in_all(stage, &arm_marks, 0.5)?;
        line.wait_last(stage)?;
        let card = QuizCard::build(
            &kit,
            stage,
            QUIZ,
            Words::Text(QUESTION),
            Words::Text(&answer.to_string()),
            QuizLook::BESIDE,
        )?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ---------------------------------------------------------- 05_5
        // one: six long is one and a half slabs, so two go in whole and
        // one is cut in half
        let line = say(&kit, stage, LINES[4])?;
        line.wait_for(stage, "Just one")?;
        fade_out_all(stage, &[card.back, card.label, card.question], 0.4)?;
        let corner = four
            .iter()
            .find(|p| p.part == Part::Whole && p.slab == 0)
            .ok_or_else(|| fail("layer 4 has no corner".into()))?;
        stage.play(to_place(&b4, waiting4[0], corner, 0.8)?)?;
        line.wait_for(stage, "room for one and a half slabs")?;
        let reason = reason_line(
            &kit,
            stage,
            &format!("{} ÷ {} = {arm4_slabs} {SLABS}", arm4[1], arm4[0]),
        )?;
        fade_in_all(stage, &[reason], 0.5)?;
        line.wait_for(stage, "So two go in whole")?;
        let mut moves = Vec::new();
        for p in four.iter().filter(|p| p.part == Part::Whole && p.slab > 0) {
            moves.push(to_place(&b4, waiting4[p.slab as usize], p, 0.8)?);
        }
        together(stage, moves, 0.3)?;
        line.wait_for(stage, "one is cut in half")?;
        fade_out_all(stage, &[card.answer], 0.3)?;
        let pair = halves(stage, &b4, 4);
        let seams4 = cut_and_place(&kit, stage, &b4, waiting4[3], pair, &four)?;
        kit.mark(stage, "pizz:layer");
        let mut mobs = waiting4[..3].to_vec();
        mobs.extend([pair.0, pair.1]);
        mobs.extend(seams4);
        layers.push(mobs);
        let next = side_brace(&kit, stage, &b4, sides[3])?;
        let mut out = brace.mobs().to_vec();
        out.extend(&arm_marks);
        out.extend([layer4, reason]);
        swap(stage, &out, &next.mobs(), 0.5)?;
        brace = next;
        together(stage, vec![light(&row, 3)?], 0.0)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 05_6
        // the five cube: no cut, two whole slabs along each arm, the square
        // fifteen on a side, and the waiting fifteen lands on the sum
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, "The five cube wraps around")?;
        let b5 = board_of(5);
        let on_board: Vec<Mob> = layers.iter().flatten().copied().collect();
        brace = pull_back(&kit, stage, CELLS[3], &b5, &on_board, &brace, sides[3])?;
        let five = cubes::layer(5);
        let waiting5: Vec<Mob> = (0..5).map(|j| waiting_slab(stage, &b5, 5, j)).collect();
        let mut shown = arm_braces(&kit, stage, &b5, 5, [arm5[0], arm5[1]])?;
        shown.push(reason_line(
            &kit,
            stage,
            &format!("{} ÷ {} = {arm5_slabs} {SLABS}", arm5[1], arm5[0]),
        )?);
        fade_in_lagged(stage, &waiting5, 0.5, 0.2)?;
        fade_in_all(stage, &shown, 0.5)?;
        line.wait_for(stage, "with no cut at all")?;
        let mut moves = Vec::new();
        for p in &five {
            moves.push(to_place(&b5, waiting5[p.slab as usize], p, 0.8)?);
        }
        together(stage, moves, 0.25)?;
        kit.mark(stage, "pizz:layer");
        layers.push(waiting5.clone());
        line.wait_for(stage, "fifteen on a side")?;
        let next = side_brace(&kit, stage, &b5, sides[4])?;
        let mut out = brace.mobs().to_vec();
        out.extend(&shown);
        swap(stage, &out, &next.mobs(), 0.5)?;
        brace = next;
        together(stage, vec![light(&row, 4)?], 0.0)?;
        line.wait_for(stage, "One plus two plus three")?;
        // the side's number becomes the sum of the cubes' sizes
        let mut terms: Vec<(String, Srgb)> = Vec::new();
        for k in 1..=n {
            if k > 1 {
                terms.push(("+".to_owned(), ink()));
            }
            terms.push((k.to_string(), size(k as usize)));
        }
        terms.push(("=".to_owned(), ink()));
        let at = stage.get_center(brace.label);
        let sum = tex_row(
            &kit,
            stage,
            &terms.iter().map(|(s, c)| (s.as_str(), *c)).collect::<Vec<_>>(),
            BRACE_PT,
            v(at[0] - 0.45, at[1]),
            0.12,
            4.4,
        )?;
        swap(stage, &[brace.label], &sum, 0.5)?;
        line.wait_end(stage)?;
        // the fifteen that waited lands where the sum ends
        let last = sum[sum.len() - 1];
        let land = v(
            stage.get_center(last)[0] + stage.get_width(last) / 2.0 + 0.45,
            stage.get_center(last)[1],
        );
        play!(stage; fly(fifteen, land, BRACE_PT / ANSWER_PT, 1.0)?);
        kit.mark(stage, "pizz:total");
        flash(stage, &[fifteen], hero(), 1.15, 0.5)?;

        // ---------------------------------------------------------- 05_7
        // the same tiles counted two ways
        let line = say(&kit, stage, LINES[6])?;
        line.wait_for(stage, "The same two hundred and twenty-five tiles")?;
        let tally = stage.add_value_tracker(0.0);
        let counter = live_counter(&kit, stage, tally, COUNT_PT, ink(), v(COLUMN_X, 0.9))?;
        let tiles = kit.text(stage, TILES, 40.0, dim(), v(COLUMN_X, 0.0))?;
        fade_in_all(stage, &[counter, tiles], 0.4)?;
        line.wait_for(stage, "Cube by cube")?;
        // each layer lights while the count steps to the cubes so far:
        // 1, 9, 36, 100, 225
        let mut so_far = 0;
        for (k, mobs) in layers.iter().enumerate() {
            so_far += areas[k];
            play!(stage; tween_linear(tally, so_far as f64, 0.8)?);
            flash(stage, mobs, hero(), 1.05, 0.5)?;
        }
        if so_far != unit_squares {
            return Err(fail("the layers do not add to the square's tiles".into()));
        }
        line.wait_for(stage, "or fifteen times fifteen")?;
        let mut terms: Vec<(String, Srgb)> = Vec::new();
        for k in 1..=n {
            if k > 1 {
                terms.push(("+".to_owned(), ink()));
            }
            terms.push((format!("{k}^3"), size(k as usize)));
        }
        terms.push(("=".to_owned(), ink()));
        terms.push((format!("{side}^2"), hero()));
        let formula = tex_row(
            &kit,
            stage,
            &terms.iter().map(|(s, c)| (s.as_str(), *c)).collect::<Vec<_>>(),
            FORMULA_PT,
            v(COLUMN_X, -1.1),
            0.14,
            6.0,
        )?;
        let product = kit.text(
            stage,
            &format!("{side} × {side} = {unit_squares}"),
            40.0,
            hero(),
            v(COLUMN_X, -2.1),
        )?;
        fade_in_lagged(stage, &formula, 1.0, 0.1)?;
        fade_in_all(stage, &[product], 0.5)?;
        line.wait_end(stage)?;
        breath(&kit, stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::tiles;

    /// The chapter speaks exactly the script's 05_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("05_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The layers' pieces, cell for cell, fill the square exactly once each,
    /// for every cube the chapter draws and far beyond.
    #[test]
    fn the_layers_tile_the_square_for_every_size() {
        for n in 1..=12 {
            let side = cubes::side(n);
            let cells: Vec<Cell> = cubes::square(n).iter().flat_map(|p| p.rect.cells()).collect();
            tiles(cells, Rect::new(0, 0, side, side), &[]).unwrap();
        }
    }

    /// The cells of a layer are the piece's cells and no others.
    #[test]
    fn a_layer_is_its_pieces() {
        for k in 1..=12 {
            let mut a: Vec<Cell> = layer_cells(k);
            let mut b: Vec<Cell> = cubes::layer(k).iter().flat_map(|p| p.rect.cells()).collect();
            a.sort_unstable();
            b.sort_unstable();
            assert_eq!(a, b, "layer {k}");
            assert_eq!(a.len() as i64, k * k * k);
        }
    }

    /// An odd cube is whole slabs; an even cube's cut slab has two halves
    /// that are k wide by k/2 tall as cut and as they are laid, one of
    /// them turned.
    #[test]
    fn the_cut_slab_is_two_halves_one_turned() {
        for k in 1..=9 {
            let layer = cubes::layer(k);
            let halves: Vec<&Piece> = layer.iter().filter(|p| p.part != Part::Whole).collect();
            if k % 2 == 1 {
                assert!(halves.is_empty(), "k = {k}");
                continue;
            }
            assert_eq!(halves.len(), 2);
            let (r, t) = (halves[0].rect, halves[1].rect);
            assert_eq!((r.w, r.h), (k, k / 2));
            assert_eq!((t.w, t.h), (k / 2, k));
            assert_eq!(halves[0].slab, k - 1);
            assert_eq!(halves[1].slab, k - 1);
            assert!(seam_cells(halves[0]).is_some() && seam_cells(halves[1]).is_some());
        }
    }

    /// Slabs wait in a row above the square their layer makes: clear of it,
    /// under the shelf's floor, left of the quiz card (05_4) and within the
    /// frame, at every size the picture is pulled back to; and each layer
    /// is smaller than the last.
    #[test]
    fn the_waiting_rows_stand_above_their_square_and_in_the_frame() {
        for k in 1..=shown::CUBES {
            let b = board_of(k);
            let last = waiting(k, k - 1);
            assert!(last.y > cubes::side(k), "k = {k}: the row is over the square");
            let top = b.p(0.0, (last.y + last.h) as f64)[1];
            let right = b.p((last.x + last.w) as f64, 0.0)[0];
            assert!(top <= super::super::draw::SHELF_FLOOR + 1e-9, "k = {k}: {top}");
            assert!(right < FRAME_X_RADIUS - 0.2, "k = {k}: {right}");
            if k == 4 {
                assert!(right <= 0.8 + 1e-9, "the quiz card starts at 0.8: {right}");
            }
        }
        assert!(CELLS.windows(2).all(|w| w[1] < w[0]));
        assert_eq!(CELLS.len() as i64, shown::CUBES);
        // the braces under the board keep inside the frame
        const { assert!(BOARD_AT[1] - 0.9 > -FRAME_Y_RADIUS) };
    }

    /// Fractions the chapter shows read as the video says them.
    #[test]
    fn fractions_are_said_as_whole_numbers_and_halves() {
        assert_eq!(mixed("3/2").unwrap(), "1½");
        assert_eq!(mixed("2").unwrap(), "2");
        assert_eq!(mixed("1/2").unwrap(), "½");
        assert!(mixed("1/3").is_err());
        assert!(mixed("x").is_err());
    }

    /// The scene constructs end to end and the kit's layout lint finds
    /// nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("05_slabs").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 40.0 && ran.seconds < 120.0, "{} s", ran.seconds);
    }
}
