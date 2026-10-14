//! Chapter 13, four triangles: the most famous picture, with its checks
//! done. A right triangle with shorter sides three and four, four copies set
//! in the corners of a square seven on a side (13_1), leave a tilted square
//! whose side is the triangle's long side (13_2); slide the triangles, and
//! what is left is two squares, on the shorter sides (13_3); the quiz, how
//! big was the tilted square (13_4, twenty-five); the same pile counted two
//! ways (13_5); every right triangle, both arrangements redrawn live, and the
//! pair of shorter sides two and five (13_6); and what the picture took for
//! granted, its corner and its two sharp angles (13_7).
//!
//! Every number is a fact or the model's: the legs `F-P.legs`, the big square
//! `F-P.big`, the triangles `F-P.triangles`, the tilted square `F-P.tilted`
//! (the quiz's answer `F-Q.tilted.answer`), the squares left over `F-P.squares`,
//! the long side `F-P.long_side`, the other triangle `F-P.other.*`. The
//! pieces drawn are `model::pythagoras` exactly; the live arrangement of
//! 13_6 uses the same formulas in floating point (a test pins them to the
//! model for every pair of legs up to the facts' reach).

use explainer_kit::{Kit, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardRow, LENS_AT, ProofCard, QuizCard, QuizLook, Words, breath, dim, fact, fact_ints, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, file_on_shelf, flat, ground, hero, ink, lens, lens_mark,
    lower_third, pause, say, size, slide, tex_row,
};

pub struct FourTriangles {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The big picture of 13_1 to 13_3: lower left corner and a cell's side.
const BIG_AT: Vec3 = [-6.2, -1.25, 0.0];
const BIG_CELL: f64 = 0.6;
/// The two small pictures of 13_4 to 13_6, side by side, and their cell.
const SMALL_FIRST: Vec3 = [-6.5, -0.9, 0.0];
const SMALL_SECOND: Vec3 = [-3.55, -0.9, 0.0];
const SMALL_CELL: f64 = 0.38;
/// The right-hand panel's centre and width.
const PANEL_X: f64 = 3.2;
const PANEL_W: f64 = 6.6;
/// How far a triangle's piece stands in from its true edges, as a share of
/// its size: the seams between pieces.
const SEAM: f64 = 0.04;
/// The slot this proof takes on the shelf: after the cut cube's.
pub const SLOT: usize = 7;

// the chapter's fixed words
const LONG_SIDE: &str = "long side";
const TILTED_SQUARE: &str = "tilted square";
const TWO_SQUARES: &str = "two squares";
const TILTED_EQ: &str = "tilted square = ? tiles";
const BIG_SQUARE: &str = "big square";
const TRIANGLES: &str = "four triangles";
const LEFT_OVER: &str = "left over";
const ON_THREE: &str = "on the side of three";
const ON_FOUR: &str = "on the side of four";
const SO_LONG_SIDE: &str = "so the long side is five";
const SHORTER: &str = "a and b: the shorter sides, c: the long side";
const NO_WHOLE: &str = "no whole long side";
const THEOREM_LABEL: &str = "Pythagoras's theorem:";
const THEOREM_TEXT: &str = "the squares on the two shorter sides add to the square on the long side";
const SHARP: &str = "sharp";
const TILTED_CORNER: &str = "tilted corner";
const RIGHT: &str = "right angle";
const STRAIGHT: &str = "a straight side";
pub(super) const SHARP_SUM: &str = "sharp angles make a right angle";
pub(super) const CLOSES: &str = "each pair of triangles closes into an a by b rectangle";
pub(super) const ASSUMED_CORNER: &str = "the tilted corner sits on a straight side";
/// The card's Shown line, which chapter 15 reads back.
pub(super) const SHOWN_TEX: &str = "\\text{for every right triangle, } a^{2}+b^{2}=c^{2}";
const NOBODY: &str = "No source we found says who first drew this picture.";
/// The earliest printing found is credited to Wheeler: the year is the
/// fact's (`F-X.wheeler`), so the sentence is built in two parts (R2, 13_7).
const EARLIEST_BEFORE: &str = "The earliest printing we found, in";
const EARLIEST_AFTER: &str = "is credited to A. D. Wheeler.";
const COUSIN: &str = "A close cousin is in a third-century Chinese commentary on the Zhoubi suanjing";
/// 13_3's lower-third, with no words said: chapter 12's cut square (a big
/// square, a small square and two equal rectangles) in letters (R2, 13_3).
const CUT_SQUARE_LABEL: &str = "the same cut square as before:";
const CUT_SQUARE_TEXT: &str = "(a + b)² = a² + 2ab + b²";

// ---------------------------------------------------------------- the model

pub type P2 = (f64, f64);

/// The four triangles in the corners of the square of side `a + b`, right
/// angle first: `model::pythagoras::corners` in floats.
pub fn corners_f(a: f64, b: f64) -> [[P2; 3]; 4] {
    let s = a + b;
    [
        [(0.0, 0.0), (a, 0.0), (0.0, b)],
        [(s, 0.0), (s, a), (a, 0.0)],
        [(s, s), (b, s), (s, a)],
        [(0.0, s), (0.0, b), (b, s)],
    ]
}

/// The slide that takes each triangle to the second arrangement.
pub fn slides_f(a: f64, b: f64) -> [P2; 4] {
    [(0.0, 0.0), (0.0, b), (-b, -a), (a, 0.0)]
}

/// The second arrangement: the triangles slid.
pub fn slid_f(a: f64, b: f64) -> [[P2; 3]; 4] {
    let (t, d) = (corners_f(a, b), slides_f(a, b));
    std::array::from_fn(|i| t[i].map(|p| (p.0 + d[i].0, p.1 + d[i].1)))
}

/// The tilted square the first arrangement leaves.
pub fn tilted_f(a: f64, b: f64) -> [P2; 4] {
    let s = a + b;
    [(a, 0.0), (s, a), (b, s), (0.0, b)]
}

/// The two squares the second arrangement leaves: on leg a, on leg b, as
/// (x, y, side).
pub fn left_over_f(a: f64, b: f64) -> [(f64, f64, f64); 2] {
    [(0.0, b, a), (a, 0.0, b)]
}

/// The lengths of the two shorter sides from the facts (`3,4`).
fn legs(key: &str) -> fmn::Result<(i64, i64)> {
    let l = fact_ints(key)?;
    match l.as_slice() {
        [a, b] => Ok((*a, *b)),
        _ => Err(fail(format!("fact {key} is not a pair of lengths"))),
    }
}

/// A colour for a length: a size colour at a whole length, a blend of the
/// two neighbouring size colours between.
fn length_colour(len: f64) -> Srgb {
    let (lo, hi) = (len.floor().max(1.0), len.ceil().max(1.0));
    let t = len - lo;
    let (c0, c1) = (size(lo as usize), size(hi as usize));
    Srgb {
        r: c0.r + (c1.r - c0.r) * t,
        g: c0.g + (c1.g - c0.g) * t,
        b: c0.b + (c1.b - c0.b) * t,
    }
}

// ----------------------------------------------------------- the drawings

/// A polygon through `pts` (cells of `board`), pulled in towards its middle
/// by [`SEAM`] so neighbours stay apart. Detached.
fn piece(stage: &mut Stage<'_>, board: &Board, pts: &[P2], style: Style) -> Mob {
    stage
        .arena_mut()
        .add(Polygon::new(piece_points(board, pts)).style(style).build())
}

/// The screen corners of a piece, pulled in.
fn piece_points(board: &Board, pts: &[P2]) -> Vec<Vec3> {
    let n = pts.len() as f64;
    let (cx, cy) = (
        pts.iter().map(|p| p.0).sum::<f64>() / n,
        pts.iter().map(|p| p.1).sum::<f64>() / n,
    );
    pts.iter()
        .map(|p| board.p(cx + (p.0 - cx) * (1.0 - SEAM), cy + (p.1 - cy) * (1.0 - SEAM)))
        .collect()
}

/// A polygon outline through `pts` (cells of `board`), exactly on them.
fn outline(stage: &mut Stage<'_>, board: &Board, pts: &[P2], style: Style) -> Mob {
    let scene: Vec<Vec3> = pts.iter().map(|p| board.p(p.0, p.1)).collect();
    stage.arena_mut().add(Polygon::new(scene).style(style).build())
}

/// One arrangement's pieces on a board: four triangles, then either the
/// tilted square (first) or the two squares on the legs (second).
struct Arrangement {
    triangles: [Mob; 4],
    squares: Vec<Mob>,
}

impl Arrangement {
    fn all(&self) -> Vec<Mob> {
        let mut out = self.triangles.to_vec();
        out.extend(&self.squares);
        out
    }
}

fn first_arrangement(stage: &mut Stage<'_>, board: &Board, a: f64, b: f64, fill: Srgb) -> Arrangement {
    let t = corners_f(a, b);
    let triangles = t.map(|tri| piece(stage, board, &tri, flat(dim(), 0.9)));
    let squares = vec![piece(stage, board, &tilted_f(a, b), flat(fill, 0.0))];
    Arrangement { triangles, squares }
}

fn second_arrangement(stage: &mut Stage<'_>, board: &Board, a: f64, b: f64) -> Arrangement {
    let t = slid_f(a, b);
    let triangles = t.map(|tri| piece(stage, board, &tri, flat(dim(), 0.9)));
    let squares = left_over_f(a, b)
        .iter()
        .map(|&(x, y, s)| {
            let sq = [(x, y), (x + s, y), (x + s, y + s), (x, y + s)];
            piece(stage, board, &sq, flat(length_colour(s), 1.0))
        })
        .collect();
    Arrangement { triangles, squares }
}

/// The picture of both arrangements redrawn live from one tracker, the
/// length of the first shorter side (the other is the rest of the side):
/// every point rewritten by one updater, only when the tracker moves.
struct Live {
    first: Arrangement,
    second: Arrangement,
    a: Mob,
    anchor: Mob,
}

impl Live {
    fn build(stage: &mut Stage<'_>, total: f64, a: f64, first: Board, second: Board) -> fmn::Result<Self> {
        let f = first_arrangement(stage, &first, a, total - a, dim());
        let s = second_arrangement(stage, &second, a, total - a);
        let tracker = stage.add_value_tracker(a);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (f_tri, f_sq) = (f.triangles, f.squares[0]);
        let (s_tri, s_sq) = (s.triangles, [s.squares[0], s.squares[1]]);
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |st: &mut fmn::mobject::Stage, _| {
                let a = st.tracker_value(tracker).unwrap_or(a);
                if a.to_bits() == last.to_bits() {
                    return;
                }
                last = a;
                let b = total - a;
                for (m, tri) in f_tri.iter().zip(corners_f(a, b)) {
                    let _ = st.set_points(*m, &super::draw::corners(&closed(&piece_points(&first, &tri))));
                }
                let _ = st.set_points(
                    f_sq,
                    &super::draw::corners(&closed(&piece_points(&first, &tilted_f(a, b)))),
                );
                for (m, tri) in s_tri.iter().zip(slid_f(a, b)) {
                    let _ = st.set_points(*m, &super::draw::corners(&closed(&piece_points(&second, &tri))));
                }
                for (m, &(x, y, side)) in s_sq.iter().zip(&left_over_f(a, b)) {
                    let sq = [(x, y), (x + side, y), (x + side, y + side), (x, y + side)];
                    let _ = st.set_points(*m, &super::draw::corners(&closed(&piece_points(&second, &sq))));
                    st.set_fill(*m, Some(length_colour(side)), None, None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            first: f,
            second: s,
            a: tracker,
            anchor,
        })
    }

    fn all(&self) -> Vec<Mob> {
        let mut out = self.first.all();
        out.extend(self.second.all());
        out
    }
}

/// The corners of a closed polygon with its first corner repeated at the
/// end, as `set_points` wants it once made a run by `corners`.
fn closed(pts: &[Vec3]) -> Vec<Vec3> {
    let mut out = pts.to_vec();
    if let Some(&first) = pts.first() {
        out.push(first);
    }
    out
}

/// The corner close-up: the straight side, the two hypotenuses that leave
/// it from one point, and the three angles there (sharp, tilted corner,
/// sharp), `k` times the size of a unit close-up, the point at `at`.
/// Detached; returns the mobjects in the order they come in: the side and
/// the rays, then the wedges, then the words.
fn corner_closeup(
    kit: &Kit,
    stage: &mut Stage<'_>,
    at: Vec3,
    k: f64,
    (a, b): (f64, f64),
    words: bool,
) -> fmn::Result<Vec<Mob>> {
    let r = 1.7 * k;
    let t1 = b.atan2(-a);
    let t2 = a.atan2(b);
    let tip = |ang: f64, len: f64| v(at[0] + len * ang.cos(), at[1] + len * ang.sin());
    let side = stage.arena_mut().add(
        polyline(vec![v(at[0] - 2.0 * r, at[1]), v(at[0] + 2.0 * r, at[1])])
            .style(faded_line(ink(), 4.0, 1.0))
            .build(),
    );
    let ray = |stage: &mut Stage<'_>, ang: f64| {
        stage.arena_mut().add(
            polyline(vec![at, tip(ang, 1.9 * r)])
                .style(faded_line(dim(), 4.0, 1.0))
                .build(),
        )
    };
    let (ray_a, ray_b) = (ray(stage, t1), ray(stage, t2));
    let wedge = |stage: &mut Stage<'_>, from: f64, to: f64, colour: Srgb, opacity: f64| {
        let steps = 24;
        let mut pts = vec![at];
        for i in 0..=steps {
            let t = from + (to - from) * f64::from(i) / f64::from(steps);
            pts.push(tip(t, r));
        }
        stage
            .arena_mut()
            .add(Polygon::new(pts).style(flat(colour, opacity)).build())
    };
    let left = wedge(stage, t1, std::f64::consts::PI, size(3), 0.55);
    let right = wedge(stage, 0.0, t2, size(4), 0.55);
    let middle = wedge(stage, t2, t1, hero(), 0.5);
    let mut out = vec![side, ray_a, ray_b, left, right, middle];
    if words {
        let at_angle = |ang: f64, len: f64| tip(ang, len);
        out.push(kit.text(
            stage,
            SHARP,
            26.0,
            ink(),
            at_angle((t1 + std::f64::consts::PI) / 2.0, 1.35 * r),
        )?);
        out.push(kit.text(stage, SHARP, 26.0, ink(), at_angle(t2 / 2.0, 1.35 * r))?);
        out.push(kit.text(stage, TILTED_CORNER, 26.0, hero(), at_angle((t1 + t2) / 2.0, 0.62 * r))?);
    }
    Ok(out)
}

/// The picture this card files, rebuilt as it stands on the shelf (the
/// chapters after this one open with the shelf so far): both arrangements
/// of the four triangles in their frames, the tilted square violet, at
/// thumbnail size in its slot. Detached.
pub fn thumb(_kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (la, lb) = legs("F-P.legs")?;
    let (a, b) = (la as f64, lb as f64);
    let total = a + b;
    let first_small = Board::new(SMALL_FIRST, SMALL_CELL);
    let second_small = Board::new(SMALL_SECOND, SMALL_CELL);
    let live = Live::build(stage, total, a, first_small, second_small)?;
    stage.set_fill(live.first.squares[0], Some(size(5)), Some(1.0), None, false);
    let frame_pts = [(0.0, 0.0), (total, 0.0), (total, total), (0.0, total)];
    let mut picture = live.all();
    picture.push(outline(stage, &first_small, &frame_pts, faded_line(ink(), 2.5, 1.0)));
    picture.push(outline(stage, &second_small, &frame_pts, faded_line(ink(), 2.5, 1.0)));
    let from = v(
        (SMALL_FIRST[0] + SMALL_SECOND[0] + total * SMALL_CELL) / 2.0,
        SMALL_FIRST[1] + total * SMALL_CELL / 2.0,
    );
    super::draw::shelve(stage, &picture, from, 2.0 * total * SMALL_CELL + 0.3, SLOT);
    Ok(picture)
}

impl SceneConstruct for FourTriangles {
    fn name(&self) -> &str {
        "13_four_triangles"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 13)?;
        let (la, lb) = legs("F-P.legs")?;
        let (a, b) = (la as f64, lb as f64);
        let total = a + b;
        let (oa, ob) = legs("F-P.other.legs")?;
        if oa + ob != la + lb {
            return Err(fail(
                "the other triangle's shorter sides do not fill the same square".into(),
            ));
        }
        let big = Board::new(BIG_AT, BIG_CELL);
        let first_small = Board::new(SMALL_FIRST, SMALL_CELL);
        let second_small = Board::new(SMALL_SECOND, SMALL_CELL);

        // ------------------------------------------------------------ 13_1
        // a right triangle with shorter sides three and four; four copies
        // inside a square
        let line = say(&kit, stage, "13_1")?;
        let frame = outline(
            stage,
            &big,
            &[(0.0, 0.0), (total, 0.0), (total, total), (0.0, total)],
            faded_line(ink(), 3.0, 1.0),
        );
        let first = first_arrangement(stage, &big, a, b, hero());
        let label_a = kit.text(stage, &la.to_string(), 30.0, ground(), big.p(a / 2.0, 0.42))?;
        let label_b = kit.text(stage, &lb.to_string(), 30.0, ground(), big.p(0.42, b / 2.0))?;
        fade_in_all(stage, &[frame], 0.6)?;
        line.wait_for(stage, "A right triangle")?;
        fade_in_all(stage, &[first.triangles[0], label_a, label_b], 0.7)?;
        line.wait_for(stage, "four copies")?;
        fade_in_lagged(stage, &first.triangles[1..], 1.2, 0.3)?;

        // ------------------------------------------------------------ 13_2
        // they leave a tilted square, the long side its side
        let line = say(&kit, stage, "13_2")?;
        let tilted = outline(stage, &big, &tilted_f(a, b), faded_line(hero(), 5.0, 1.0));
        let mid = big.p((a + total) / 2.0, a / 2.0);
        let reach = big.p(total + 0.7, a / 2.0);
        let leader = stage
            .arena_mut()
            .add(polyline(vec![mid, reach]).style(faded_line(hero(), 3.0, 1.0)).build());
        let long_side = kit.text_left(stage, LONG_SIDE, 30.0, hero(), v(reach[0] + 0.15, reach[1]))?;
        line.wait_for(stage, "tilted square")?;
        fade_in_all(stage, &[tilted], 0.8)?;
        line.wait_for(stage, "long side")?;
        fade_in_all(stage, &[leader, long_side], 0.6)?;

        // ------------------------------------------------------------ 13_3
        // slide the triangles: two squares are left, on the sides of three
        // and four
        let line = say(&kit, stage, "13_3")?;
        let squares: Vec<Mob> = left_over_f(a, b)
            .iter()
            .map(|&(x, y, s)| {
                let sq = [(x, y), (x + s, y), (x + s, y + s), (x, y + s)];
                piece(stage, &big, &sq, flat(length_colour(s), 1.0))
            })
            .collect();
        fade_out_all(stage, &[tilted, leader, long_side, label_a, label_b], 0.5)?;
        line.wait_for(stage, "Nothing turns")?;
        let steps = slides_f(a, b);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &(dx, dy)) in steps.iter().enumerate() {
            if dx != 0.0 || dy != 0.0 {
                anims.push(stage.prepare(slide(first.triangles[i], big.step(dx, dy), 1.6)?)?);
            }
        }
        stage.play_prepared(anims)?;
        // no words say it: the square's cut, as chapter 12 cut it, in letters
        let cut = lower_third(&kit, stage, CUT_SQUARE_LABEL, CUT_SQUARE_TEXT)?;
        fade_in_all(stage, &[cut], 0.5)?;
        line.wait_for(stage, "what is left is two squares")?;
        fade_in_all(stage, &squares, 0.8)?;

        // ------------------------------------------------------------ 13_4
        // your turn: both arrangements side by side; how big was the tilted
        // square?
        let line = say(&kit, stage, "13_4")?;
        let tilted_answer = fact("F-Q.tilted.answer")?;
        let card = QuizCard::build(
            &kit,
            stage,
            "tilted",
            Words::Text(TILTED_EQ),
            Words::Text(&tilted_answer),
            QuizLook::BESIDE,
        )?;
        let small_first = first_arrangement(stage, &first_small, a, b, hero());
        let small_second = second_arrangement(stage, &second_small, a, b);
        let small_frame_a = outline(
            stage,
            &first_small,
            &[(0.0, 0.0), (total, 0.0), (total, total), (0.0, total)],
            faded_line(ink(), 2.5, 1.0),
        );
        let small_frame_b = outline(
            stage,
            &second_small,
            &[(0.0, 0.0), (total, 0.0), (total, total), (0.0, total)],
            faded_line(ink(), 2.5, 1.0),
        );
        let small_tilted = outline(stage, &first_small, &tilted_f(a, b), faded_line(hero(), 4.0, 1.0));
        let name_a = kit.text(
            stage,
            TILTED_SQUARE,
            24.0,
            dim(),
            v(SMALL_FIRST[0] + total * SMALL_CELL / 2.0, SMALL_FIRST[1] - 0.35),
        )?;
        let name_b = kit.text(
            stage,
            TWO_SQUARES,
            24.0,
            dim(),
            v(SMALL_SECOND[0] + total * SMALL_CELL / 2.0, SMALL_SECOND[1] - 0.35),
        )?;
        let mut leaving: Vec<Mob> = first.triangles.to_vec();
        leaving.extend(&squares);
        leaving.push(frame);
        leaving.push(cut);
        fade_out_all(stage, &leaving, 0.5)?;
        let mut smalls = small_first.all();
        smalls.extend(small_second.all());
        smalls.extend([small_frame_a, small_frame_b, small_tilted, name_a, name_b]);
        fade_in_all(stage, &smalls, 0.7)?;
        line.wait_sentence(stage, 2)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ------------------------------------------------------------ 13_5
        // nine plus sixteen, twenty-five: the same pile counted two ways
        let line = say(&kit, stage, "13_5")?;
        fade_out_all(stage, &card.mobs(), 0.4)?;
        let big_n = fact("F-P.big")?;
        let tri_n = fact("F-P.triangles")?;
        let tilted_n = fact("F-P.tilted")?;
        let squares_n = fact_ints("F-P.squares")?;
        let long_n = fact("F-P.long_side")?;
        let violet = size(5);
        let (nine, sixteen) = (squares_n[0].to_string(), squares_n[1].to_string());
        // the tilted square fills violet, a five by five slab
        stage.set_fill(small_first.squares[0], Some(violet), Some(1.0), None, false);
        line.wait_for(stage, "Nine plus sixteen")?;
        let nine_sixteen = tex_row(
            &kit,
            stage,
            &[
                (nine.as_str(), length_colour(a)),
                ("+", ink()),
                (sixteen.as_str(), length_colour(b)),
                ("=", ink()),
                (tilted_n.as_str(), violet),
            ],
            52.0,
            v(PANEL_X, 2.3),
            0.25,
            PANEL_W,
        )?;
        let on_three = kit.text(stage, ON_THREE, 22.0, dim(), v(PANEL_X - 1.9, 1.6))?;
        let on_four = kit.text(stage, ON_FOUR, 22.0, dim(), v(PANEL_X + 1.9, 1.6))?;
        fade_in_all(stage, &nine_sixteen, 0.7)?;
        fade_in_all(stage, &[on_three, on_four], 0.5)?;
        line.wait_for(stage, "big square")?;
        let count_two = tex_row(
            &kit,
            stage,
            &[
                (big_n.as_str(), ink()),
                ("-", ink()),
                (tri_n.as_str(), dim()),
                ("=", ink()),
                (tilted_n.as_str(), violet),
            ],
            52.0,
            v(PANEL_X, 0.4),
            0.25,
            PANEL_W,
        )?;
        let cap_big = kit.text(stage, BIG_SQUARE, 22.0, dim(), v(PANEL_X - 2.3, -0.25))?;
        let cap_tri = kit.text(stage, TRIANGLES, 22.0, dim(), v(PANEL_X - 0.4, -0.25))?;
        let cap_left = kit.text(stage, LEFT_OVER, 22.0, violet, v(PANEL_X + 2.3, -0.25))?;
        fade_in_all(stage, &count_two, 0.7)?;
        fade_in_all(stage, &[cap_big, cap_tri, cap_left], 0.5)?;
        line.wait_for(stage, "five times five")?;
        let five = tex_row(
            &kit,
            stage,
            &[
                (tilted_n.as_str(), violet),
                ("=", ink()),
                (long_n.as_str(), ink()),
                ("\\times", ink()),
                (long_n.as_str(), ink()),
            ],
            52.0,
            v(PANEL_X, -1.3),
            0.25,
            PANEL_W,
        )?;
        let cap_five = kit.text(stage, SO_LONG_SIDE, 24.0, dim(), v(PANEL_X, -1.95))?;
        fade_in_all(stage, &five, 0.7)?;
        fade_in_all(stage, &[cap_five], 0.4)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 13_6
        // it holds for every right triangle: both arrangements redrawn live
        let line = say(&kit, stage, "13_6")?;
        let mut leaving = nine_sixteen.clone();
        leaving.extend(&count_two);
        leaving.extend(&five);
        leaving.extend([on_three, on_four, cap_big, cap_tri, cap_left, cap_five]);
        leaving.extend(&smalls);
        fade_out_all(stage, &leaving, 0.5)?;
        let live = Live::build(stage, total, a, first_small, second_small)?;
        let frame_pts = [(0.0, 0.0), (total, 0.0), (total, total), (0.0, total)];
        let live_frames = [
            outline(stage, &first_small, &frame_pts, faded_line(ink(), 2.5, 1.0)),
            outline(stage, &second_small, &frame_pts, faded_line(ink(), 2.5, 1.0)),
        ];
        stage.add_to_scene(live.anchor)?;
        stage.set_fill(live.first.squares[0], Some(violet), Some(1.0), None, false);
        let mut shown = live.all();
        shown.extend(live_frames);
        fade_in_all(stage, &shown, 0.6)?;
        let formula = kit.tex(stage, "a^{2} + b^{2} = c^{2}", 60.0, ink(), &[], v(PANEL_X, 2.3))?;
        let words = kit.text(stage, SHORTER, 22.0, dim(), v(PANEL_X, 1.55))?;
        fade_in_all(stage, &[formula, words], 0.6)?;
        // the live numbers: the squares on the shorter sides and the tilted
        // square, from the tracker
        let tracker = live.a;
        let sq_a = move |s: &fmn::mobject::Stage| {
            let x = s.tracker_value(tracker).unwrap_or(0.0);
            x * x
        };
        let sq_b = move |s: &fmn::mobject::Stage| {
            let y = total - s.tracker_value(tracker).unwrap_or(0.0);
            y * y
        };
        let readout = vec![
            kit.live_value(
                stage,
                sq_a,
                1,
                false,
                48.0,
                length_colour(a),
                v(PANEL_X - 3.0, 0.4),
                LEFT,
            )?,
            kit.text(stage, "+", 48.0, ink(), v(PANEL_X - 1.15, 0.4))?,
            kit.live_value(
                stage,
                sq_b,
                1,
                false,
                48.0,
                length_colour(b),
                v(PANEL_X - 0.8, 0.4),
                LEFT,
            )?,
            kit.text(stage, "=", 48.0, ink(), v(PANEL_X + 1.15, 0.4))?,
            kit.live_value(
                stage,
                move |s| sq_a(s) + sq_b(s),
                1,
                false,
                48.0,
                violet,
                v(PANEL_X + 1.5, 0.4),
                LEFT,
            )?,
        ];
        line.wait_for(stage, "right triangle")?;
        // the shorter sides slide through other lengths while the theorem is
        // stated (its name is on screen only), then come to rest at the other
        // pair, two and five, as that pair is named
        let other = oa as f64;
        for (i, &to) in [1.0, total - 1.0].iter().enumerate() {
            stage.play(tween(live.a, to, if i == 0 { 1.2 } else { 1.6 })?)?;
        }
        line.wait_for(stage, "The two small squares")?;
        let theorem = lower_third(&kit, stage, THEOREM_LABEL, THEOREM_TEXT)?;
        fade_in_all(stage, &[theorem], 0.5)?;
        line.wait_for(stage, "Shorter sides two and five")?;
        stage.play(tween(live.a, other, 1.6)?)?;
        line.wait_for(stage, "no whole long side")?;
        let none = kit.text(stage, NO_WHOLE, 30.0, hero(), v(PANEL_X, -0.7))?;
        fade_in_all(stage, &[none], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 0.4)?;

        // ------------------------------------------------------------ 13_7
        // what the picture took for granted: the corner, and the two sharp
        // angles that make a right angle
        let line = say(&kit, stage, "13_7")?;
        let mut leaving = readout.clone();
        leaving.extend([formula, words, none, theorem]);
        fade_out_all(stage, &leaving, 0.5)?;
        // back to the pair of legs the picture was drawn with
        stage.play(tween(live.a, a, 0.8)?)?;
        stage.clear_updaters(live.anchor, true);
        let close_at = v(3.0, -0.35);
        let close = corner_closeup(&kit, stage, close_at, 1.0, (a, b), true)?;
        let straight = kit.text(stage, STRAIGHT, 26.0, ink(), v(close_at[0] + 2.4, close_at[1] - 0.45))?;
        let g = super::draw::group(stage, &close)?;
        line.wait_for(stage, "straight side")?;
        let grow = fade_in(stage.arena_mut(), g, ORIGIN, 0.25)?;
        let grow = stage.prepare(grow.rt(1.0))?;
        stage.play_prepared(vec![grow])?;
        fade_in_all(stage, &[straight], 0.4)?;
        line.wait_for(stage, "make a right angle")?;
        let right_angle = kit.text(stage, RIGHT, 30.0, hero(), v(close_at[0] - 0.5, close_at[1] + 2.35))?;
        fade_in_all(stage, &[right_angle], 0.5)?;
        // the card, and its close-up on the ring
        let mut proof = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Text(ASSUMED_CORNER)),
                CardRow::Assumed(Words::Text(CLOSES)),
            ],
        )?;
        line.wait_for(stage, "So the tilted shape")?;
        fade_out_all(stage, &[straight, right_angle], 0.3)?;
        proof.show(stage, 0.6)?;
        // the corner shrinks onto the ring: its words stay behind
        let words_of = close[close.len() - 3..].to_vec();
        fade_out_all(stage, &words_of, 0.3)?;
        let ring = lens(stage);
        fade_in_all(stage, &[ring], 0.4)?;
        let shrink = stage.prepare(super::draw::fly(g, LENS_AT, 0.3, 0.9)?)?;
        stage.play_prepared(vec![shrink])?;
        proof.tick(&kit, stage, 1, SHARP_SUM)?;
        let tick_one = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick_one], 0.3)?;
        line.wait_end(stage)?;
        proof.tick(&kit, stage, 2, "")?;
        // the card holds three seconds alone
        pause(stage, 3.0)?;
        // it has passed: the picture goes to the shelf, and the source is
        // said with nothing else on the screen
        let mut done = vec![g, ring, tick_one];
        done.extend(proof.mobs());
        fade_out_all(stage, &done, 0.5)?;
        let mut filed = live.all();
        filed.extend(live_frames);
        let from = v(
            (SMALL_FIRST[0] + SMALL_SECOND[0] + total * SMALL_CELL) / 2.0,
            SMALL_FIRST[1] + total * SMALL_CELL / 2.0,
        );
        file_on_shelf(stage, &filed, from, 2.0 * total * SMALL_CELL + 0.3, SLOT, 1.0)?;
        // the earliest printing found, with its year from the facts, between
        // the "no source" sentence and the cousin (R2, 13_7)
        let wheeler = fact("F-X.wheeler")?;
        let source = lower_third(
            &kit,
            stage,
            "",
            &format!("{NOBODY} {EARLIEST_BEFORE} {wheeler}, {EARLIEST_AFTER} {COUSIN}"),
        )?;
        fade_in_all(stage, &[source], 0.6)?;
        breath(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::pythagoras;

    /// 13_1 to 13_3: the floats the pictures are drawn from are the model's
    /// exact corners, slides and squares, for every pair of legs up to the
    /// facts' reach, and the facts are the model's.
    #[test]
    fn the_drawn_pieces_are_the_models_for_every_pair_of_legs() {
        for a in 1..=12i64 {
            for b in 1..=12i64 {
                let (fa, fb) = (a as f64, b as f64);
                let as_f = |t: [(i64, i64); 3]| t.map(|p| (p.0 as f64, p.1 as f64));
                let model = pythagoras::corners(a, b);
                for (m, f) in model.iter().zip(corners_f(fa, fb)) {
                    assert_eq!(as_f(*m), f, "corners {a} {b}");
                }
                let slid = pythagoras::slid(a, b);
                for (m, f) in slid.iter().zip(slid_f(fa, fb)) {
                    assert_eq!(as_f(*m), f, "slid {a} {b}");
                }
                let t = pythagoras::tilted(a, b);
                assert_eq!(t.map(|p| (p.0 as f64, p.1 as f64)), tilted_f(fa, fb));
                assert!(pythagoras::is_square(&t));
                let (on_a, on_b) = pythagoras::left_over(a, b);
                let want = [(on_a.x, on_a.y, on_a.w), (on_b.x, on_b.y, on_b.w)];
                for (w, f) in want.iter().zip(left_over_f(fa, fb)) {
                    assert_eq!((w.0 as f64, w.1 as f64, w.2 as f64), f);
                }
                // the pieces are the big square: four triangles and two squares
                let tri = pythagoras::area2(&model[0]);
                assert_eq!(4 * tri + 2 * (a * a + b * b), 2 * (a + b) * (a + b));
            }
        }
        assert_eq!(fact("F-P.legs").unwrap(), "3,4");
        assert_eq!(fact("F-P.big").unwrap(), "49");
        assert_eq!(fact("F-P.triangles").unwrap(), "24");
        assert_eq!(fact("F-P.tilted").unwrap(), "25");
        assert_eq!(fact("F-P.squares").unwrap(), "9,16");
        assert_eq!(fact("F-P.long_side").unwrap(), "5");
        assert_eq!(fact("F-P.slides").unwrap(), "0 0,0 4,-4 -3,3 0");
        assert_eq!(fact("F-Q.tilted.answer").unwrap(), "25");
        assert_eq!(fact("F-P.other.legs").unwrap(), "2,5");
        assert_eq!(fact("F-P.other.tilted").unwrap(), "29");
        assert_eq!(fact("F-P.other.whole_side").unwrap(), "none");
        // the slides in the fact are the ones the picture plays
        let slides: Vec<String> = slides_f(3.0, 4.0).iter().map(|s| format!("{} {}", s.0, s.1)).collect();
        assert_eq!(slides.join(","), fact("F-P.slides").unwrap());
        // the same pile counted two ways: forty-nine less twenty-four, and
        // nine and sixteen, both twenty-five; five times five
        assert_eq!(49 - 24, 9 + 16);
        assert_eq!(5 * 5, 25);
        assert_eq!(2 * 2 + 5 * 5, 29);
        // 29 is not a square: no whole long side
        assert!((0..=6).all(|c| c * c != 29));
    }

    /// 13_6: the live arrangement at any length of the first shorter side
    /// is the picture of that triangle: its pieces fill the big square (the
    /// areas add up), and the colours of the squares on the legs are the
    /// size colours at whole lengths.
    #[test]
    fn the_live_pictures_fill_the_square_at_every_length() {
        let total = 7.0;
        for step in 0..=60 {
            let a = 1.0 + 5.0 * f64::from(step) / 60.0;
            let b = total - a;
            let tri: f64 = corners_f(a, b)
                .iter()
                .map(|t| ((t[1].0 - t[0].0) * (t[2].1 - t[0].1) - (t[2].0 - t[0].0) * (t[1].1 - t[0].1)).abs() / 2.0)
                .sum();
            // the tilted square's area is its side's length squared
            let t = tilted_f(a, b);
            let tilted_area = (t[1].0 - t[0].0).powi(2) + (t[1].1 - t[0].1).powi(2);
            assert!((tilted_area - (a * a + b * b)).abs() < 1e-9, "a = {a}");
            assert!((tri + tilted_area - total * total).abs() < 1e-9, "a = {a}");
            let squares: f64 = left_over_f(a, b).iter().map(|s| s.2 * s.2).sum();
            assert!((tri + squares - total * total).abs() < 1e-9, "a = {a}");
        }
        for k in 1..=5 {
            assert_eq!(length_colour(f64::from(k)), size(k as usize), "length {k}");
        }
        // between two lengths the colour is between their size colours
        let mid = length_colour(2.5);
        let (c2, c3) = (size(2), size(3));
        assert!((mid.r - (c2.r + c3.r) / 2.0).abs() < 1e-12);
    }

    /// 13_7: the three angles at the tilted corner's foot add to a straight
    /// line, the two sharp ones to a right angle, for every right triangle.
    #[test]
    fn the_two_sharp_angles_make_a_right_angle_at_every_foot() {
        for a in 1..=9i64 {
            for b in 1..=9i64 {
                let (fa, fb) = (a as f64, b as f64);
                // at the foot (a, 0): the left triangle's angle and the
                // right one's
                let left = fb.atan2(fa);
                let right = fa.atan2(fb);
                assert!((left + right - std::f64::consts::FRAC_PI_2).abs() < 1e-12, "{a} {b}");
                // the corner between them is what is left of a straight line
                let corner = std::f64::consts::PI - left - right;
                assert!((corner - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
            }
        }
    }

    /// The picture never keeps the voice waiting: with the scratch voice
    /// placed, each line starts within two seconds of the end of the one
    /// before, except after the quiz, whose countdown and landed answer
    /// fill the gap (shown with `--nocapture`).
    #[test]
    fn no_line_waits_long_for_its_picture() {
        let v = testing::run_voiced("13_four_triangles").unwrap();
        let mut lines = v.lines.clone();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        let quiz = super::super::draw::quiz_named("tilted").unwrap();
        let held = f64::from(quiz.seconds) + super::super::draw::LAND_S;
        let mut prev: Option<(String, f64)> = None;
        for (id, at, len) in &lines {
            if let Some((before, end)) = &prev {
                let gap = at - end;
                eprintln!("{id}: starts {at:.1} after a gap of {gap:.1} (since {before})");
                let allowed = if before == quiz.line { held + 2.0 } else { 2.0 };
                assert!(gap <= allowed, "{id} starts {gap:.1} s after {before} ends");
            }
            prev = Some((id.clone(), at + len));
        }
        eprintln!("chapter {:.1} s", v.ran.seconds);
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("13_four_triangles").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "long side",
            "tilted square = ? tiles",
            "25",
            "big square",
            "four triangles",
            "\\times",
            "a^{2} + b^{2} = c^{2}",
            "no whole long side",
            "Pythagoras's theorem:",
            "the tilted corner sits on a straight side",
            "each pair of triangles closes into an a by b rectangle",
            // R2, 13_3: the cut square in letters, no word said
            "the same cut square as before:",
            "(a + b)² = a² + 2ab + b²",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // R2, 13_7: the three sentences of the source line, whole and in
        // this order, the year the fact's (the lines of a wrapped lower-third
        // are registered one after another, so a phrase may be split between
        // two of them: the whole line is looked for joined)
        let said = words.join(" ");
        let f = crate::facts::facts().unwrap();
        let year = f.get("F-X.wheeler").unwrap();
        assert_eq!(year, "1859");
        let source = format!("{NOBODY} {EARLIEST_BEFORE} {year}, {EARLIEST_AFTER} {COUSIN}");
        assert!(said.contains(&source), "the source line is not whole: {words:?}");
        assert!(!said.contains(" …"), "a lower-third was cut short: {said}");
        assert!(ran.seconds > 40.0 && ran.seconds < 100.0, "{} s", ran.seconds);
    }
}
