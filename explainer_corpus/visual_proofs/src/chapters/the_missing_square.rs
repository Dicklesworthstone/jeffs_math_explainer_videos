//! Chapter 14, the missing square: the same argument as chapter 13 with its
//! check left out. Four pieces, two triangles and two L-shapes, seem to make
//! a triangle thirteen wide and five tall (14_1); slid, they make the same
//! outline with one tile empty (14_2); if that were a proof it would prove
//! that thirty-two equals thirty-three (14_3); the quiz, which triangle is
//! steeper (14_4, the small one); two in five against three in eight, so the
//! long edge bends (14_5); the sliver between the two outlines is one tile,
//! cut along the grid into seventeen pieces that slide into the empty tile
//! (14_6); and the card fails, its Assumed line crossed on the bend (14_7).
//!
//! Every number is a fact or the model's exact one: the pieces `F-L.pieces`,
//! the outlines `F-L.outline.*`, the slopes `F-L.slopes*`, the bend off the
//! straight line `F-L.dip`, the hole `F-L.hole.at`, the sliver and its
//! seventeen pieces `F-L.sliver*`. The pieces drawn are `model::lie` exactly,
//! in whole cells and exact fractions; floats only place them.

use explainer_kit::{Kit, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardRow, LENS_AT, LENS_R, ProofCard, QuizCard, QuizLook, Words, breath, dim, fact, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, faint, fit_width, flash, flat, formula_third, frac_tex, hero, ink,
    lens, lens_mark, panel, pause, say, size, slide, source_line, tex_row, warn,
};
use super::sound::{self, Strike};
use crate::model::lie;

pub struct TheMissingSquare {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The big picture of 14_1 and 14_2: lower left corner and a cell's side.
const BIG_AT: Vec3 = [-4.55, -0.6, 0.0];
const BIG_CELL: f64 = 0.7;
/// The two small pictures side by side (14_2 to 14_4).
const SMALL_FIRST: Vec3 = [-6.6, 0.5, 0.0];
const SMALL_SECOND: Vec3 = [0.62, 0.5, 0.0];
const SMALL_CELL: f64 = 0.46;
/// The medium picture of 14_5 and the window that shows its bend closer.
pub(super) const MED_AT: Vec3 = [-6.9, -0.3, 0.0];
pub(super) const MED_CELL: f64 = 0.55;
const WINDOW_AT: Vec3 = [3.5, 0.8, 0.0];
const WINDOW: (f64, f64) = (5.4, 3.0);
/// How much closer the window looks, and the ring's close-up of 14_7.
pub(super) const ZOOM: f64 = 8.0;
const LENS_ZOOM: f64 = 12.0;
/// The two outlines laid over each other (14_6).
const OVER_AT: Vec3 = [-6.4, -0.55, 0.0];
const OVER_CELL: f64 = 0.62;
/// The right-hand panel's centre and width (14_6).
const PANEL_X: f64 = 4.4;
const PANEL_W: f64 = 4.8;
/// How far a triangle stands in from its true edges, as a share of its size:
/// the seams between pieces.
const SEAM: f64 = 0.04;
/// Where the card sits during the quiz: under the two pictures.
const QUIZ_LOOK: QuizLook = QuizLook {
    at: [0.0, -2.05, 0.0],
    scale: 0.62,
};
/// The shelf's slot the picture that lies would have taken.
pub const SLOT: usize = 8;
/// Silence with the cross on the screen, seconds (14_7).
pub(super) const SILENCE_S: f64 = 2.0;
/// 14_2's landing in sound (storyboard, "Sound plan": three of the four
/// bloom, the fourth is missing): the second arrangement's pieces light in
/// turn, each with the next note of the pentatonic's climb, and the empty
/// tile, the fourth, lights in silence: the climb stops one short. Seconds
/// from the window's start; the same spacing as each light's own length.
const LANDING_AT: [f64; 3] = [0.0, 0.6, 1.2];
/// A light on a piece, and the pause before the next.
const LIGHT_S: f64 = 0.5;
/// The strength of each note: soft, so the viewer counts the tiles under it
/// (the beat's words: the grid shows, the music thinned).
const LANDING_VELOCITY: f64 = 0.45;
/// The type size of the rise and run written on the edges.
const EDGE_PT: f64 = 24.0;

/// The notes of 14_2's window: the first three landings of the climb, at
/// [`LANDING_AT`]; there is no fourth.
pub fn missing_strikes() -> Vec<Strike> {
    LANDING_AT
        .iter()
        .enumerate()
        .map(|(i, &at)| Strike {
            at,
            pitch: sound::landing_pitch(i),
            velocity: LANDING_VELOCITY,
        })
        .collect()
}

// the big picture ends above the card's top edge, and so does the ring
const _: () = assert!(BIG_AT[1] >= super::draw::CARD_TOP);
const _: () = assert!(LENS_AT[1] - LENS_R > super::draw::CARD_TOP);

// the chapter's fixed words
const FIRST_PICTURE: &str = "first picture";
const SECOND_PICTURE: &str = "second picture";
const PIECES: &str = "pieces:";
const UP: &str = "up";
const ACROSS: &str = "across";
const STEEPER: &str = "steeper: big or small?";
const RULER: &str = "straight edge";
const TIMES_CLOSER: &str = "times closer";
const OFF_BY: &str = "hangs off the line by";
const A_TILE: &str = "of a tile";
const TRUE_TRIANGLE: &str = "true triangle";
const TILES: &str = "tiles";
const FILL_THE_TILE: &str = "pieces fill the empty tile";
const FIBONACCI: &str = "neighbours in the Fibonacci numbers, so";
const SLOPES_DIFFER: &str = "the slopes differ by only";
pub(super) const NOTHING: &str = "nothing";
pub(super) const EDGE_STRAIGHT: &str = "the long edge is straight";
pub(super) const IT_BENDS: &str = "it bends here";
const SOURCE: &str = "P. Curry, 1953; M. Gardner, Mathematics, Magic and Mystery, 1956";

// ---------------------------------------------------------------- the model

pub(super) type P2 = (f64, f64);
type Tri = [(i64, i64); 3];

fn pf(p: (i64, i64)) -> P2 {
    (p.0 as f64, p.1 as f64)
}

/// A fact fraction (`65/2`, `1/13`) or whole number as numerator and
/// denominator.
fn parts(s: &str) -> fmn::Result<(i64, i64)> {
    let bad = || fail(format!("{s:?} is not a fraction or a whole number"));
    match s.split_once('/') {
        Some((n, d)) => Ok((n.parse().map_err(|_| bad())?, d.parse().map_err(|_| bad())?)),
        None => Ok((s.parse().map_err(|_| bad())?, 1)),
    }
}

/// An exact fraction as TeX with its whole part first (`65/2` is
/// `32\frac{1}{2}`, `1/13` is `\frac{1}{13}`, `2` is `2`).
fn mixed_tex(s: &str) -> fmn::Result<String> {
    let (n, d) = parts(s)?;
    Ok(match (n / d, n % d) {
        (w, 0) => w.to_string(),
        (0, r) => format!("\\frac{{{r}}}{{{d}}}"),
        (w, r) => format!("{w}\\frac{{{r}}}{{{d}}}"),
    })
}

/// A fact that is a list of "x y" pairs (`5 2,-8 -3`), as whole numbers.
fn pairs(key: &str) -> fmn::Result<Vec<(i64, i64)>> {
    fact(key)?
        .split(',')
        .map(|item| {
            let mut it = item.split(' ').map(str::parse::<i64>);
            match (it.next(), it.next(), it.next()) {
                (Some(Ok(x)), Some(Ok(y)), None) => Ok((x, y)),
                _ => Err(fail(format!("fact {key}: {item:?} is not a pair of whole numbers"))),
            }
        })
        .collect()
}

/// A fact that is a list of fractions (`3/8,2/5`), as numerators and
/// denominators.
fn fractions(key: &str) -> fmn::Result<Vec<(i64, i64)>> {
    fact(key)?.split(',').map(parts).collect()
}

// ----------------------------------------------------------- the drawings

/// The screen corners of a triangle on `board`, pulled in towards its middle
/// by [`SEAM`] so the pieces stay apart.
fn tri_points(board: &Board, t: &Tri) -> Vec<Vec3> {
    let c = (
        t.iter().map(|p| p.0 as f64).sum::<f64>() / 3.0,
        t.iter().map(|p| p.1 as f64).sum::<f64>() / 3.0,
    );
    t.iter()
        .map(|&p| {
            let (x, y) = pf(p);
            board.p(c.0 + (x - c.0) * (1.0 - SEAM), c.1 + (y - c.1) * (1.0 - SEAM))
        })
        .collect()
}

/// The four pieces of an arrangement on a board: the two triangles, the L of
/// eight (blue) and the L of seven (pink). Detached.
struct Pieces {
    big: Mob,
    small: Mob,
    long_l: Mob,
    short_l: Mob,
}

impl Pieces {
    fn build(stage: &mut Stage<'_>, board: &Board, a: &lie::Arrangement) -> fmn::Result<Self> {
        let tri = |stage: &mut Stage<'_>, t: &Tri| {
            stage
                .arena_mut()
                .add(Polygon::new(tri_points(board, t)).style(flat(dim(), 0.9)).build())
        };
        Ok(Self {
            big: tri(stage, &a.big),
            small: tri(stage, &a.small),
            long_l: board.tiles(stage, &a.long_l, size(4))?,
            short_l: board.tiles(stage, &a.short_l, size(1))?,
        })
    }

    /// In the order of `lie::SLIDES`: big triangle, small triangle, long L,
    /// short L.
    fn all(&self) -> [Mob; 4] {
        [self.big, self.small, self.long_l, self.short_l]
    }
}

/// A grid of unit cells `w` by `h`, faint, as two mobjects (the rows' lines
/// zig-zag down the board, the columns' across). Detached.
fn grid(stage: &mut Stage<'_>, board: &Board, w: i64, h: i64) -> Vec<Mob> {
    let mut across = Vec::new();
    for j in 0..=h {
        let (x0, x1) = if j % 2 == 0 { (0, w) } else { (w, 0) };
        across.push(board.p(x0 as f64, j as f64));
        across.push(board.p(x1 as f64, j as f64));
    }
    let mut up = Vec::new();
    for i in 0..=w {
        let (y0, y1) = if i % 2 == 0 { (0, h) } else { (h, 0) };
        up.push(board.p(i as f64, y0 as f64));
        up.push(board.p(i as f64, y1 as f64));
    }
    let style = faded_line(faint(), 1.5, 1.0);
    vec![
        stage.arena_mut().add(polyline(across).style(style).build()),
        stage.arena_mut().add(polyline(up).style(style).build()),
    ]
}

/// A closed polygon through cell coordinates of `board`, in `style`.
/// Detached.
fn polygon(stage: &mut Stage<'_>, board: &Board, pts: &[P2], style: Style) -> Mob {
    let scene: Vec<Vec3> = pts.iter().map(|p| board.p(p.0, p.1)).collect();
    stage.arena_mut().add(Polygon::new(scene).style(style).build())
}

/// A polyline through cell coordinates of `board`, in `style`. Detached.
fn path(stage: &mut Stage<'_>, board: &Board, pts: &[P2], style: Style) -> Mob {
    let scene: Vec<Vec3> = pts.iter().map(|p| board.p(p.0, p.1)).collect();
    stage.arena_mut().add(polyline(scene).style(style).build())
}

/// The sloping edge of an arrangement as cell points: the corner, the bend,
/// the far corner.
pub(super) fn edge_of(a: &lie::Arrangement) -> [P2; 3] {
    [(0.0, 0.0), pf(a.bend), (lie::WIDE as f64, lie::TALL as f64)]
}

fn v_of(p: Vec3) -> Vec3 {
    v(p[0], p[1])
}

/// The rise and run of each of the two sloping edges, written above them
/// (left edge first), from the bend: the rise is how high the bend is, the
/// run how far across. Detached.
fn edge_words(kit: &Kit, stage: &mut Stage<'_>, board: &Board, a: &lie::Arrangement) -> fmn::Result<[Mob; 2]> {
    let (bx, by) = a.bend;
    let (w, t) = (lie::WIDE, lie::TALL);
    let left = kit.text(
        stage,
        &format!("{by} {UP}, {bx} {ACROSS}"),
        EDGE_PT,
        ink(),
        v_of(board.p(bx as f64 / 2.0, by as f64 / 2.0 + 1.9)),
    )?;
    let right = kit.text(
        stage,
        &format!("{} {UP}, {} {ACROSS}", t - by, w - bx),
        EDGE_PT,
        ink(),
        v_of(board.p((bx + w) as f64 / 2.0 - 2.0, (by + t) as f64 / 2.0 + 0.55)),
    )?;
    Ok([left, right])
}

/// The empty tile on `board`: a red tile and a cross on it. Detached.
fn hole_marks(stage: &mut Stage<'_>, board: &Board, hole: (i64, i64)) -> fmn::Result<(Mob, Mob)> {
    let tile = board.tiles_styled(stage, &[hole], flat(warn(), 0.55))?;
    let cross = lens_mark(stage, board.centre(hole), board.cell * 0.8, false);
    Ok((tile, cross))
}

/// The part of the segment `a` to `b` inside the box `±half` about the
/// origin (Liang and Barsky), if any of it is.
fn clip(a: P2, b: P2, half: P2) -> Option<(P2, P2)> {
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    let d = (b.0 - a.0, b.1 - a.1);
    for (p, q) in [
        (-d.0, a.0 + half.0),
        (d.0, half.0 - a.0),
        (-d.1, a.1 + half.1),
        (d.1, half.1 - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    (t0 <= t1).then_some(((a.0 + t0 * d.0, a.1 + t0 * d.1), (a.0 + t1 * d.0, a.1 + t1 * d.1)))
}

/// Where cell point `p` of `board` lands when the picture is magnified by
/// `zoom` about cell point `at`, relative to the magnifier's middle, in scene
/// units.
fn magnified(board: &Board, at: P2, zoom: f64, p: P2) -> P2 {
    ((p.0 - at.0) * board.cell * zoom, (p.1 - at.1) * board.cell * zoom)
}

/// The segments of the polyline `pts` (cell points of `board`) magnified by
/// `zoom` about `at`, clipped to the box `±half` and set about `centre`.
/// Detached; nothing for a segment that is outside the box.
#[allow(clippy::too_many_arguments)]
pub(super) fn magnify(
    stage: &mut Stage<'_>,
    board: &Board,
    at: P2,
    zoom: f64,
    centre: Vec3,
    half: P2,
    pts: &[P2],
    style: Style,
) -> Vec<Mob> {
    let mut out = Vec::new();
    for w in pts.windows(2) {
        let (a, b) = (magnified(board, at, zoom, w[0]), magnified(board, at, zoom, w[1]));
        if let Some((c, d)) = clip(a, b, half) {
            let line = polyline(vec![
                v(centre[0] + c.0, centre[1] + c.1),
                v(centre[0] + d.0, centre[1] + d.1),
            ])
            .style(style)
            .build();
            out.push(stage.arena_mut().add(line));
        }
    }
    out
}

/// The close-up on the magnifier ring of 14_7: the straight line and the
/// edge that hangs off it, magnified [`LENS_ZOOM`] times about the bend.
/// Detached.
fn lens_edges(stage: &mut Stage<'_>, a: &lie::Arrangement) -> Vec<Mob> {
    let board = Board::new(MED_AT, MED_CELL);
    let (w, t) = (lie::WIDE as f64, lie::TALL as f64);
    let half = (LENS_R * 0.7, LENS_R * 0.7);
    let bend = pf(a.bend);
    let mut out = magnify(
        stage,
        &board,
        bend,
        LENS_ZOOM,
        LENS_AT,
        half,
        &[(0.0, 0.0), (w, t)],
        faded_line(ink(), 3.0, 1.0),
    );
    out.extend(magnify(
        stage,
        &board,
        bend,
        LENS_ZOOM,
        LENS_AT,
        half,
        &[(0.0, 0.0), bend, (w, t)],
        faded_line(warn(), 4.0, 1.0),
    ));
    out
}

/// The slot this card leaves empty, as it stands on the shelf (the chapter
/// after this one opens with the shelf so far: the gap stays). Detached.
pub fn thumb(_kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    Ok(vec![super::draw::empty_slot(stage, SLOT)?])
}

/// How far right of the bend 14_5's two window labels start, and how far
/// under the red edge, where it stands there, the first one's top is.
const LABEL_X: f64 = 0.3;
const LABEL_UNDER: f64 = 0.15;
/// The room between the two labels.
const LABEL_PITCH: f64 = 0.1;

/// 14_5's two labels of the window, "hangs off the line by" and the dip, one
/// under the other, left-aligned, right of the bend and under the red edge.
/// Both lines of the close-up climb to the right (the white straight edge
/// above the red one), so the room under the red edge's left end is clear
/// of both; where the caption stood, centred 1.75 right of the bend, the
/// white line ran through its first words and its last word touched the
/// window's border. `steep` is the red edge's slope right of the bend.
/// Detached.
fn window_labels(kit: &Kit, stage: &mut Stage<'_>, dip: &str, steep: f64) -> fmn::Result<(Mob, Mob)> {
    let left = WINDOW_AT[0] + LABEL_X;
    let top = WINDOW_AT[1] + steep * LABEL_X - LABEL_UNDER;
    let off_by = kit.text(stage, OFF_BY, 22.0, dim(), ORIGIN)?;
    let dip_text = kit.tex(
        stage,
        &format!("{}\\;\\text{{{A_TILE}}}", frac_tex(dip)),
        34.0,
        hero(),
        &[],
        ORIGIN,
    )?;
    let h = stage.get_height(off_by);
    stage.move_to(off_by, v(left, top - h / 2.0), LEFT);
    let h2 = stage.get_height(dip_text);
    stage.move_to(dip_text, v(left, top - h - LABEL_PITCH - h2 / 2.0), LEFT);
    Ok((off_by, dip_text))
}

impl SceneConstruct for TheMissingSquare {
    fn name(&self) -> &str {
        "14_the_missing_square"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 14)?;
        let (first, second) = (lie::first(), lie::second());
        let slides = pairs("F-L.slides")?;
        let hole = pairs("F-L.hole.at")?
            .first()
            .copied()
            .ok_or_else(|| fail("F-L.hole.at is empty".into()))?;
        let (wide, tall) = (lie::WIDE, lie::TALL);
        let (w, t) = (wide as f64, tall as f64);
        let triangle = [(0.0, 0.0), (w, 0.0), (w, t)];
        let big = Board::new(BIG_AT, BIG_CELL);
        let first_board = Board::new(SMALL_FIRST, SMALL_CELL);
        let second_board = Board::new(SMALL_SECOND, SMALL_CELL);

        // ------------------------------------------------------------ 14_1
        // four pieces that seem to make a triangle thirteen wide and five
        // tall
        let line = say(&kit, stage, "14_1")?;
        let grid_big = grid(stage, &big, wide, tall);
        let apparent = polygon(stage, &big, &triangle, faded_line(ink(), 3.0, 1.0));
        let on_big = Pieces::build(stage, &big, &first)?;
        fade_in_all(stage, &grid_big, 0.6)?;
        line.wait_for(stage, "Four pieces")?;
        fade_in_lagged(stage, &on_big.all(), 1.8, 0.3)?;
        line.wait_for(stage, "thirteen wide")?;
        fade_in_all(stage, &[apparent], 0.6)?;

        // ------------------------------------------------------------ 14_2
        // slide the pieces: nothing turns, nothing flips; one tile is empty
        let line = say(&kit, stage, "14_2")?;
        line.wait_for(stage, "Nothing turns")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (m, &(dx, dy)) in on_big.all().iter().zip(&slides) {
            anims.push(stage.prepare(slide(*m, big.step(dx as f64, dy as f64), 1.8)?)?);
        }
        stage.play_prepared(anims)?;
        // both arrangements, small, side by side, the grid showing
        let small_first = Pieces::build(stage, &first_board, &first)?;
        let small_second = Pieces::build(stage, &second_board, &second)?;
        let mut small_grids = grid(stage, &first_board, wide, tall);
        small_grids.extend(grid(stage, &second_board, wide, tall));
        let frame_one = polygon(stage, &first_board, &triangle, faded_line(ink(), 2.5, 1.0));
        let frame_two = polygon(stage, &second_board, &triangle, faded_line(ink(), 2.5, 1.0));
        let (small_hole, small_cross) = hole_marks(stage, &second_board, hole)?;
        let name_one = kit.text(
            stage,
            FIRST_PICTURE,
            24.0,
            dim(),
            v(SMALL_FIRST[0] + w * SMALL_CELL / 2.0, SMALL_FIRST[1] - 0.3),
        )?;
        let name_two = kit.text(
            stage,
            SECOND_PICTURE,
            24.0,
            dim(),
            v(SMALL_SECOND[0] + w * SMALL_CELL / 2.0, SMALL_SECOND[1] - 0.3),
        )?;
        let mut leaving: Vec<Mob> = on_big.all().to_vec();
        leaving.extend(&grid_big);
        leaving.push(apparent);
        line.wait_for(stage, "The outline looks the same")?;
        fade_out_all(stage, &leaving, 0.6)?;
        let mut smalls: Vec<Mob> = small_grids.clone();
        smalls.extend(small_first.all());
        smalls.extend(small_second.all());
        smalls.extend([frame_one, frame_two, name_one, name_two]);
        fade_in_all(stage, &smalls, 0.7)?;
        line.wait_for(stage, "one tile is empty")?;
        fade_in_all(stage, &[small_hole, small_cross], 0.4)?;
        flash(stage, &[small_hole], warn(), 1.4, 0.8)?;
        smalls.extend([small_hole, small_cross]);
        line.wait_end(stage)?;
        // the grid shows for the breath's length and the viewer counts; the
        // arrangement is heard under it: three of the four bloom (the pieces
        // that make the outline light, each with its note) and the fourth,
        // the empty tile, is lit in silence
        let wav = sound::strikes(
            "proofs_missing_square_landings",
            &missing_strikes(),
            explainer_kit::animatic::HOLD_S,
        )?;
        kit.sonify(
            stage,
            explainer_kit::animatic::HOLD_S,
            "14_the_missing_square:14_2",
            &wav,
        )?;
        let lit = [small_second.big, small_second.small, small_second.long_l];
        for (k, &m) in lit.iter().enumerate() {
            flash(stage, &[m], hero(), 1.12, LIGHT_S)?;
            let next = LANDING_AT.get(k + 1).copied().unwrap_or(LANDING_AT[k] + LIGHT_S + 0.1);
            pause(stage, next - LANDING_AT[k] - LIGHT_S)?;
        }
        flash(stage, &[small_hole], warn(), 1.4, LIGHT_S + 0.2)?;

        // ------------------------------------------------------------ 14_3
        // the same pieces have room for one more tile: thirty-two against
        // thirty-three
        let line = say(&kit, stage, "14_3")?;
        let areas = fact("F-L.pieces")?;
        let total = fact("F-L.pieces.total")?;
        let outline_one = fact("F-L.outline.first")?;
        let outline_two = fact("F-L.outline.second")?;
        let hole_area = fact("F-L.holes.second")?;
        let area_list: Vec<&str> = areas.split(',').collect();
        let colours = [dim(), dim(), size(4), size(1)];
        if area_list.len() != colours.len() {
            return Err(fail("F-L.pieces is not one area for each of four pieces".into()));
        }
        let mut terms: Vec<(&str, Srgb)> = Vec::new();
        for (i, a) in area_list.iter().enumerate() {
            if i > 0 {
                terms.push(("+", ink()));
            }
            terms.push((a, colours[i]));
        }
        terms.push(("=", ink()));
        terms.push((total.as_str(), ink()));
        let row_one = tex_row(&kit, stage, &terms, 44.0, v(1.4, -0.7), 0.25, 6.4)?;
        let label_one = kit.text_left(stage, PIECES, 26.0, dim(), v(-6.4, -0.7))?;
        let terms_two = [
            (total.as_str(), ink()),
            ("+", ink()),
            (hole_area.as_str(), warn()),
            ("=", ink()),
            (outline_two.as_str(), ink()),
        ];
        let row_two = tex_row(&kit, stage, &terms_two, 44.0, v(1.4, -1.6), 0.25, 6.4)?;
        let label_two = kit.text_left(stage, SECOND_PICTURE, 26.0, dim(), v(-6.4, -1.6))?;
        line.wait_for(stage, "room for one more tile")?;
        fade_in_all(stage, &row_one, 0.7)?;
        fade_in_all(stage, &[label_one], 0.4)?;
        line.wait_for(stage, "thirty-two equals thirty-three")?;
        fade_in_all(stage, &row_two, 0.7)?;
        fade_in_all(stage, &[label_two], 0.4)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 14_4
        // your turn: which triangle is steeper?
        let line = say(&kit, stage, "14_4")?;
        let mut gone = row_one.clone();
        gone.extend(&row_two);
        gone.extend([label_one, label_two]);
        fade_out_all(stage, &gone, 0.5)?;
        let slopes = fractions("F-L.slopes")?;
        let ((big_up, big_run), (small_up, small_run)) = match slopes.as_slice() {
            [b, s] => (*b, *s),
            _ => return Err(fail("F-L.slopes is not the two slopes".into())),
        };
        // the sloping edges in ink, with rise and run on them
        let mut edges: Vec<Mob> = Vec::new();
        for (board, arr) in [(&first_board, &first), (&second_board, &second)] {
            edges.push(path(stage, board, &edge_of(arr), faded_line(ink(), 4.5, 1.0)));
            edges.extend(edge_words(&kit, stage, board, arr)?);
        }
        let answer = fact("F-Q.slopes.answer")?;
        let card = QuizCard::build(
            &kit,
            stage,
            "slopes",
            Words::Text(STEEPER),
            Words::Text(&answer),
            QUIZ_LOOK,
        )?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &edges, 0.6)?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ------------------------------------------------------------ 14_5
        // the small one: two in five against three in eight; the long edge
        // bends
        let line = say(&kit, stage, "14_5")?;
        let mut gone = card.mobs();
        gone.extend(&edges);
        gone.extend(&smalls);
        fade_out_all(stage, &gone, 0.6)?;
        let med = Board::new(MED_AT, MED_CELL);
        let med_pieces = Pieces::build(stage, &med, &first)?;
        let bend = pf(first.bend);
        let thousandths = fact("F-L.slopes.thousandths")?;
        let thousandths: Vec<&str> = thousandths.split(',').collect();
        let (at_big, at_small) = (med.p(4.0, 3.1), med.p(8.6, 4.6));
        let label_big = kit.text(
            stage,
            &format!("{big_up} in {big_run} = 0.{}", thousandths[0]),
            EDGE_PT,
            ink(),
            v_of(at_big),
        )?;
        let label_small = kit.text(
            stage,
            &format!("{small_up} in {small_run} = 0.{}", thousandths[1]),
            EDGE_PT,
            ink(),
            v_of(at_small),
        )?;
        let ruler = path(stage, &med, &[(0.0, 0.0), (w, t)], faded_line(ink(), 3.0, 1.0));
        let ruler_note = kit.text(stage, RULER, 22.0, ink(), v_of(med.p(2.0, 4.5)))?;
        let marker = stage.arena_mut().add(
            Circle::new()
                .radius(0.22)
                .arc_center(med.p(bend.0, bend.1))
                .style(faded_line(hero(), 3.0, 1.0))
                .build(),
        );
        line.wait_for(stage, "The small one")?;
        fade_in_all(stage, &med_pieces.all(), 0.7)?;
        line.wait_for(stage, "It climbs two in five")?;
        fade_in_all(stage, &[label_small], 0.5)?;
        line.wait_for(stage, "three in eight")?;
        fade_in_all(stage, &[label_big], 0.5)?;
        line.wait_for(stage, "not straight")?;
        fade_in_all(stage, &[ruler, ruler_note], 0.6)?;
        // the window: the bend, eight times closer
        let back = Rectangle::new()
            .width(WINDOW.0)
            .height(WINDOW.1)
            .corner_radius(0.15)
            .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.5, 1.0))
            .build()?
            .moved_to(WINDOW_AT);
        let back = stage.arena_mut().add(back);
        let title = kit.text(
            stage,
            &format!("{} {TIMES_CLOSER}", ZOOM as i64),
            24.0,
            dim(),
            v(WINDOW_AT[0], WINDOW_AT[1] + WINDOW.1 / 2.0 + 0.3),
        )?;
        let half = (WINDOW.0 / 2.0 - 0.1, WINDOW.1 / 2.0 - 0.1);
        let mut inside = magnify(
            stage,
            &med,
            bend,
            ZOOM,
            WINDOW_AT,
            half,
            &[(0.0, 0.0), (w, t)],
            faded_line(ink(), 3.0, 1.0),
        );
        inside.extend(magnify(
            stage,
            &med,
            bend,
            ZOOM,
            WINDOW_AT,
            half,
            &edge_of(&first),
            faded_line(warn(), 5.0, 1.0),
        ));
        // how far the corner hangs off the line, measured straight up
        let dip = fact("F-L.dip")?;
        let (sn, sd) = parts(&fact("F-L.straight_slope")?)?;
        let rise = (sn as f64 / sd as f64) * bend.0 - bend.1;
        let up = magnified(&med, bend, ZOOM, (bend.0, bend.1 + rise)).1;
        let bend_dot = stage.arena_mut().add(
            Circle::new()
                .radius(0.09)
                .arc_center(WINDOW_AT)
                .style(flat(warn(), 1.0))
                .build(),
        );
        let bracket = stage.arena_mut().add(
            polyline(vec![v(WINDOW_AT[0], WINDOW_AT[1]), v(WINDOW_AT[0], WINDOW_AT[1] + up)])
                .style(faded_line(hero(), 3.0, 1.0))
                .build(),
        );
        let (off_by, dip_text) = window_labels(&kit, stage, &dip, lie::slope(&first.small).to_f64())?;
        line.wait_for(stage, "It bends")?;
        let mut window_in = vec![back, title];
        window_in.extend(&inside);
        window_in.extend([bend_dot, bracket, off_by, dip_text, marker]);
        fade_in_all(stage, &window_in, 0.8)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 14_6
        // one edge sags, the other bulges: the sliver between them is one
        // tile, and its pieces slide into the empty tile
        let line = say(&kit, stage, "14_6")?;
        let mut gone = window_in.clone();
        gone.extend(med_pieces.all());
        gone.extend([ruler, ruler_note, label_big, label_small]);
        fade_out_all(stage, &gone, 0.6)?;
        let over = Board::new(OVER_AT, OVER_CELL);
        let over_grid = grid(stage, &over, wide, tall);
        let sags = polygon(
            stage,
            &over,
            &[(0.0, 0.0), (w, 0.0), (w, t), pf(first.bend)],
            faded_line(dim(), 4.0, 1.0),
        );
        let bulges = polygon(
            stage,
            &over,
            &[(0.0, 0.0), (w, 0.0), (w, t), pf(second.bend)],
            faded_line(ink(), 4.0, 1.0),
        );
        let hole_cell = over.tiles_styled(stage, &[hole], faded_line(warn(), 3.0, 1.0))?;
        let true_area = fact("F-L.true_triangle")?;
        let count_one = kit.text(
            stage,
            &format!("{FIRST_PICTURE}: {outline_one} {TILES}"),
            28.0,
            dim(),
            v(PANEL_X, 2.6),
        )?;
        let count_two = kit.text(
            stage,
            &format!("{SECOND_PICTURE}: {outline_two} {TILES}"),
            28.0,
            ink(),
            v(PANEL_X, 2.0),
        )?;
        let true_tex = kit.tex(
            stage,
            &format!("\\text{{{TRUE_TRIANGLE}}}\\; {}", mixed_tex(&true_area)?),
            34.0,
            ink(),
            &[],
            v(PANEL_X, 1.3),
        )?;
        fit_width(stage, count_one, PANEL_W);
        fit_width(stage, count_two, PANEL_W);
        fit_width(stage, true_tex, PANEL_W);
        line.wait_for(stage, "One edge sags")?;
        fade_in_all(stage, &over_grid, 0.5)?;
        fade_in_all(stage, &[sags, bulges, hole_cell, count_one, count_two, true_tex], 0.8)?;
        // the sliver, cut along the grid into its pieces
        let cut = lie::sliver_pieces();
        let folds = pairs("F-L.sliver.fold")?;
        if cut.len() != folds.len() {
            return Err(fail(
                "the sliver's pieces and their slides are not the same number".into(),
            ));
        }
        let mut sliver: Vec<Mob> = Vec::new();
        for (_, poly) in &cut {
            let pts: Vec<P2> = poly.iter().map(|&(x, y)| (x.to_f64(), y.to_f64())).collect();
            sliver.push(polygon(stage, &over, &pts, flat(warn(), 0.8)));
        }
        let thickest = fact("F-L.sliver.thickest")?;
        let note = kit.tex(
            stage,
            &format!("\\text{{thickest }}{}\\text{{ {A_TILE}}}", frac_tex(&thickest)),
            32.0,
            warn(),
            &[],
            v(PANEL_X, 0.55),
        )?;
        fit_width(stage, note, PANEL_W);
        line.wait_for(stage, "Cut that sliver")?;
        fade_in_all(stage, &sliver, 0.6)?;
        fade_in_all(stage, &[note], 0.4)?;
        // the pieces slide, lagged, by whole tiles, nothing turning, into
        // the empty tile
        line.wait_for(stage, "slide into the empty tile")?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for (m, &(dx, dy)) in sliver.iter().zip(&folds) {
            moves.push(stage.prepare(slide(*m, over.step(dx as f64, dy as f64), 0.9)?)?);
        }
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), moves, 0.12)?.with_run_time(3.0);
        stage.play(group)?;
        let pieces_n = fact("F-L.sliver.pieces")?;
        let area = fact("F-L.sliver")?;
        let count_note = kit.text(
            stage,
            &format!("{pieces_n} {FILL_THE_TILE}"),
            24.0,
            dim(),
            v(PANEL_X, -0.15),
        )?;
        fit_width(stage, count_note, PANEL_W);
        let area_tex = kit.tex(
            stage,
            &format!("\\text{{area}}\\; {area}\\text{{ tile}}"),
            40.0,
            hero(),
            &[],
            v(PANEL_X, -0.85),
        )?;
        line.wait_for(stage, "exactly one tile")?;
        fade_in_all(stage, &[count_note, area_tex], 0.5)?;
        // 2, 3, 5, 8, 13: the numbers of the slopes and the long side
        let mut nums = [small_up, big_up, small_run, big_run, wide];
        nums.sort_unstable();
        let cross = small_up * big_run - big_up * small_run;
        let differ = fact("F-L.slopes.differ")?;
        let list = format!("{}:", nums.iter().map(i64::to_string).collect::<Vec<_>>().join(", "));
        let cross_tex = format!("{small_up} \\times {big_run} - {big_up} \\times {small_run} = {cross}");
        let differ_tex = frac_tex(&differ);
        let fib = formula_third(
            &kit,
            stage,
            &list,
            &[
                Words::Text(FIBONACCI),
                Words::Tex(&cross_tex),
                Words::Text(SLOPES_DIFFER),
                Words::Tex(&differ_tex),
            ],
        )?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[fib], 0.5)?;
        breath(&kit, stage)?;

        // ------------------------------------------------------------ 14_7
        // neither picture was ever a triangle: the card fails. The music is
        // out from this sentence on, so the red cross lands on nothing, and
        // it stays out through the two seconds of silence after the line
        kit.mark(stage, "silence:cross");
        let line = say(&kit, stage, "14_7")?;
        let mut gone = sliver.clone();
        gone.extend(&over_grid);
        gone.extend([
            sags, bulges, hole_cell, count_one, count_two, true_tex, note, count_note, area_tex, fib,
        ]);
        fade_out_all(stage, &gone, 0.5)?;
        let mut proof = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Text(NOTHING)),
                CardRow::Assumed(Words::Text(EDGE_STRAIGHT)),
            ],
        )?;
        proof.show(stage, 0.6)?;
        // the ring closes on the long edge: the bend, and the cross on it
        let ring = lens(stage);
        let close = lens_edges(stage, &first);
        let mut ring_in = vec![ring];
        ring_in.extend(&close);
        fade_in_all(stage, &ring_in, 0.6)?;
        proof.cross(&kit, stage, 1, IT_BENDS)?;
        let on_bend = lens_mark(stage, LENS_AT, 0.55, false);
        fade_in_all(stage, &[on_bend], 0.3)?;
        // 14_7's source (the dense read of the narrated draft): the card
        // refuses to file, so its slot on the shelf stays empty, and the
        // source of the picture is written as the two names are said. After
        // the silence they came in 1.2 s before the chapter ended, and the
        // citation could not be read.
        line.wait_for(stage, "Martin Gardner credits")?;
        let slot = super::draw::empty_slot(stage, SLOT)?;
        #[cfg(test)]
        super::draw::shelf_log::filed(stage, SLOT, &[slot]);
        let source = source_line(&kit, stage, SOURCE, false)?;
        fade_in_all(stage, &[slot, source], 0.6)?;
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "14_7", "source_in", stage.scene().time().to_f64());
        // the narration runs on; then two seconds of silence with the cross
        // on the screen
        line.wait_end(stage)?;
        pause(stage, SILENCE_S)?;
        // the silence ends here: the music may return at the next gap (the
        // storyboard has it back at 15_1, a register lower, which is the
        // score's to do)
        kit.mark(stage, "silence_end:cross");

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{SHELF_FLOOR, fact_int, quiz_named, testing};
    use super::*;
    use crate::model::{Q, q};

    /// Twice the area of a polygon of whole corners.
    fn shoelace2(pts: &[(i64, i64)]) -> i64 {
        (0..pts.len())
            .map(|i| {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum::<i64>()
            .abs()
    }

    /// 14_1 to 14_3: the pieces, the outlines and the hole are the facts and
    /// the model's exact ones: 12 + 5 + 8 + 7 = 32 is the first outline, and
    /// the second is the same 32 and one empty tile, 33.
    #[test]
    fn the_pieces_the_outlines_and_the_hole_are_the_facts() {
        let (first, second) = (lie::first(), lie::second());
        let areas = [
            shoelace2(&first.big) / 2,
            shoelace2(&first.small) / 2,
            first.long_l.len() as i64,
            first.short_l.len() as i64,
        ];
        let listed: Vec<String> = areas.iter().map(i64::to_string).collect();
        assert_eq!(listed.join(","), fact("F-L.pieces").unwrap());
        assert_eq!(areas.iter().sum::<i64>(), fact_int("F-L.pieces.total").unwrap());
        // the same pieces slid have the same areas
        assert_eq!(shoelace2(&second.big) / 2, areas[0]);
        assert_eq!(shoelace2(&second.small) / 2, areas[1]);
        assert_eq!(second.long_l.len() as i64, areas[2]);
        assert_eq!(second.short_l.len() as i64, areas[3]);
        // the outlines: 32 and 33, the true triangle 32 and a half
        assert_eq!(
            (shoelace2(&lie::outline(&first)) / 2).to_string(),
            fact("F-L.outline.first").unwrap()
        );
        assert_eq!(
            (shoelace2(&lie::outline(&second)) / 2).to_string(),
            fact("F-L.outline.second").unwrap()
        );
        assert_eq!(fact("F-L.true_triangle").unwrap(), "65/2");
        assert_eq!(fact("F-L.frame").unwrap(), "13x5");
        // the first has no hole; the second has one, where the fact says
        assert!(lie::holes(&first).is_empty());
        assert_eq!(lie::holes(&second), vec![(7, 1)]);
        assert_eq!(pairs("F-L.hole.at").unwrap(), vec![(7, 1)]);
        assert_eq!(fact("F-L.holes.second").unwrap(), "1");
        // the slides of the fact are the model's, in order, and they take
        // the first arrangement to the second
        let slid = pairs("F-L.slides").unwrap();
        assert_eq!(slid, lie::SLIDES.to_vec());
        let by = |(dx, dy): (i64, i64)| move |p: (i64, i64)| (p.0 + dx, p.1 + dy);
        assert_eq!(first.big.map(by(slid[0])), second.big);
        assert_eq!(first.small.map(by(slid[1])), second.small);
        assert_eq!(mixed_tex("65/2").unwrap(), "32\\frac{1}{2}");
        assert_eq!(mixed_tex("1/13").unwrap(), "\\frac{1}{13}");
        assert_eq!(mixed_tex("1").unwrap(), "1");
        assert!(parts("1/x").is_err());
    }

    /// 14_4 and 14_5: the slopes, the straight edge and how far the corner
    /// hangs off it are the facts, exactly: 3/8 and 2/5 against 5/13, and a
    /// thirteenth of a tile on either side.
    #[test]
    fn the_slopes_and_the_bend_are_the_facts() {
        let (first, second) = (lie::first(), lie::second());
        assert_eq!(lie::slope(&first.big), q(3, 8));
        assert_eq!(lie::slope(&first.small), q(2, 5));
        assert_eq!(fact("F-L.slopes").unwrap(), "3/8,2/5");
        assert_eq!(fact("F-L.slopes.thousandths").unwrap(), "375,400");
        assert!(q(3, 8) < q(2, 5), "the small triangle is the steeper");
        assert_eq!(fact("F-Q.slopes.answer").unwrap(), "small");
        assert_eq!(fact_int("F-Q.slopes.seconds").unwrap(), 8);
        assert_eq!(q(2, 5) - q(3, 8), q(1, 40));
        assert_eq!(fact("F-L.slopes.differ").unwrap(), "1/40");
        // the straight edge, and how far each bend is from it (straight up)
        assert_eq!(fact("F-L.straight_slope").unwrap(), "5/13");
        let sag = lie::straight_edge_at(first.bend.0) - lie::edge_at(&first, first.bend.0);
        let bulge = lie::edge_at(&second, second.bend.0) - lie::straight_edge_at(second.bend.0);
        assert_eq!(sag, q(1, 13));
        assert_eq!(bulge, q(1, 13));
        assert_eq!(fact("F-L.dip").unwrap(), "1/13");
        assert_eq!(fact("F-L.bulge").unwrap(), "1/13");
        // the rise and run written on the edges are the bend's coordinates
        assert_eq!(first.bend, (8, 3));
        assert_eq!(second.bend, (5, 2));
        // in the window the gap is a visible share of it, and inside the ring
        let gap = sag.to_f64() * MED_CELL * ZOOM;
        assert!(
            gap > 0.2 && gap < WINDOW.1 / 2.0 - 0.3,
            "the window's gap is {gap} units"
        );
        let ring_gap = sag.to_f64() * MED_CELL * LENS_ZOOM;
        assert!(
            ring_gap > 0.2 && ring_gap < LENS_R * 0.7 - 0.1,
            "the ring's gap is {ring_gap} units"
        );
        // the slopes as decimals
        for (s, d) in [q(3, 8), q(2, 5)].iter().zip(["375", "400"]) {
            assert_eq!(*s * Q::int(1000), Q::int(d.parse().unwrap()));
        }
        // 2, 3, 5, 8, 13 are neighbours in the Fibonacci numbers
        let nums = [2, 3, 5, 8, 13];
        assert!(nums.windows(3).all(|w| w[0] + w[1] == w[2]));
        assert_eq!(nums[0] * nums[3] - nums[1] * nums[2], 1);
        let slopes = fractions("F-L.slopes").unwrap();
        assert_eq!(slopes, vec![(3, 8), (2, 5)]);
        assert_eq!(lie::WIDE, nums[4]);
    }

    /// 14_6: the sliver is one tile; its pieces, slid by the facts' whole
    /// tiles, lie in the empty tile without overlapping and fill it.
    #[test]
    fn the_sliver_folds_into_the_empty_tile_exactly() {
        let cut = lie::sliver_pieces();
        let folds = pairs("F-L.sliver.fold").unwrap();
        assert_eq!(cut.len().to_string(), fact("F-L.sliver.pieces").unwrap());
        assert_eq!(cut.len(), folds.len());
        // total area one
        let area = cut.iter().fold(Q::ZERO, |s, (_, p)| s + lie::polygon_area(p));
        assert_eq!(area, Q::ONE);
        assert_eq!(fact("F-L.sliver").unwrap(), "1");
        // slid, every piece is inside the empty tile (7, 1)
        let slid: Vec<Vec<(Q, Q)>> = cut
            .iter()
            .zip(&folds)
            .map(|((_, poly), &(dx, dy))| {
                poly.iter()
                    .map(|&(x, y)| (x + Q::int(dx), y + Q::int(dy)))
                    .collect::<Vec<_>>()
            })
            .collect();
        for piece in &slid {
            for &(x, y) in piece {
                assert!(
                    x >= Q::int(7) && x <= Q::int(8) && y >= Q::int(1) && y <= Q::int(2),
                    "{x:?} {y:?}"
                );
            }
        }
        // and they do not overlap
        for i in 0..slid.len() {
            for j in i + 1..slid.len() {
                assert_eq!(lie::overlap_area(&slid[i], &slid[j]), Q::ZERO, "pieces {i} and {j}");
            }
        }
        // the grid cut: each piece lies inside the unit cell it is named by
        for ((i, j), poly) in &cut {
            for &(x, y) in poly {
                assert!(x >= Q::int(*i) && x <= Q::int(i + 1) && y >= Q::int(*j) && y <= Q::int(j + 1));
            }
        }
        // the thickest the sliver is
        assert_eq!(fact("F-L.sliver.thickest").unwrap(), "1/8");
    }

    /// 14_5 (the dense read of the scored draft, mx-e-a-dza.10): the window's
    /// two labels stand clear of both lines of the close-up and of its
    /// border. The white line ran through "hangs off the line by", and its
    /// last word touched the window's right border.
    #[test]
    fn the_window_labels_stand_clear_of_both_lines_and_the_border() {
        use fmn::scene::{NullSceneSink, RuntimeConfig};
        struct Probe;
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let kit = Kit::new()?;
                let first = lie::first();
                let steep = lie::slope(&first.small).to_f64();
                let (sn, sd) = parts(&fact("F-L.straight_slope")?)?;
                let straight = sn as f64 / sd as f64;
                let (off_by, dip) = window_labels(&kit, stage, &fact("F-L.dip")?, steep)?;
                // the two lines of the close-up, as the scene sets them: the
                // red edge through the bend, steeper to its right, and the
                // white straight edge above it by the bracket's height
                let sag = lie::straight_edge_at(first.bend.0) - lie::edge_at(&first, first.bend.0);
                let up = sag.to_f64() * MED_CELL * ZOOM;
                let red = |x: f64| WINDOW_AT[1] + steep * (x - WINDOW_AT[0]);
                let white = |x: f64| WINDOW_AT[1] + up + straight * (x - WINDOW_AT[0]);
                let (right_edge, floor) = (WINDOW_AT[0] + WINDOW.0 / 2.0, WINDOW_AT[1] - WINDOW.1 / 2.0);
                let mut below = f64::INFINITY;
                for (what, m) in [("caption", off_by), ("dip", dip)] {
                    let (l, r) = (stage.get_left(m)[0], stage.get_right(m)[0]);
                    let (t, b) = (stage.get_top(m)[1], stage.get_bottom(m)[1]);
                    eprintln!(
                        "{what}: x {l:.2}..{r:.2}, y {b:.2}..{t:.2}; red {:.2} at its left",
                        red(l)
                    );
                    // right of the bracket, a hand inside the border and the floor
                    assert!(l > WINDOW_AT[0] + 0.1, "{what} reaches the bracket: {l:.2}");
                    assert!(
                        r <= right_edge - 0.3,
                        "{what} ends at {r:.2}, the border is at {right_edge:.2}"
                    );
                    assert!(b >= floor + 0.2, "{what} ends at {b:.2}, the border is at {floor:.2}");
                    // both lines climb right: under the red edge's left end is under both
                    assert!(white(l) > red(l));
                    assert!(
                        t <= red(l) - 0.1,
                        "{what}'s top {t:.2} meets the red edge ({:.2})",
                        red(l)
                    );
                    assert!(t <= below, "the labels are one over the other");
                    below = b - 0.02;
                }
                Ok(())
            }
        }
        fmn::run_scene(&mut Probe, RuntimeConfig::default(), 7, &mut NullSceneSink).expect("the probe runs");
    }

    /// The clipping of a window and the magnifier: a segment through the
    /// middle is clipped to the box, one outside is dropped, and a point of
    /// the picture is magnified about the bend.
    #[test]
    fn the_window_clips_and_magnifies() {
        let half = (2.0, 1.0);
        let (a, b) = clip((-3.0, 0.0), (3.0, 0.0), half).unwrap();
        assert_eq!((a, b), ((-2.0, 0.0), (2.0, 0.0)));
        assert!(clip((-3.0, 2.0), (3.0, 2.0), half).is_none());
        let (a, b) = clip((0.0, 0.0), (1.0, 0.5), half).unwrap();
        assert_eq!((a, b), ((0.0, 0.0), (1.0, 0.5)));
        let board = Board::new([0.0; 3], 0.5);
        assert_eq!(magnified(&board, (8.0, 3.0), 8.0, (9.0, 3.0)), (4.0, 0.0));
        assert_eq!(magnified(&board, (8.0, 3.0), 8.0, (8.0, 3.0)), (0.0, 0.0));
    }

    /// The pictures keep clear of the shelf's row and of the card's top, and
    /// the two small pictures and the quiz card clear one another.
    #[test]
    fn the_pictures_keep_to_their_rooms() {
        let (w, t) = (lie::WIDE as f64, lie::TALL as f64);
        for (at, cell, what) in [
            (BIG_AT, BIG_CELL, "big"),
            (SMALL_FIRST, SMALL_CELL, "small first"),
            (SMALL_SECOND, SMALL_CELL, "small second"),
            (MED_AT, MED_CELL, "medium"),
            (OVER_AT, OVER_CELL, "overlay"),
        ] {
            assert!(
                at[1] + cell * t <= SHELF_FLOOR + 1e-9,
                "{what} reaches {}",
                at[1] + cell * t
            );
            assert!(
                at[0] >= -7.1 + 0.15 && at[0] + cell * w <= 7.1 - 0.15,
                "{what} leaves the frame"
            );
        }
        // the small pictures do not meet
        assert!(SMALL_FIRST[0] + SMALL_CELL * w < SMALL_SECOND[0]);
        // the quiz card sits under their names
        let card_top = QUIZ_LOOK.at[1] + QUIZ_LOOK.size().1 / 2.0;
        assert!(card_top < SMALL_FIRST[1] - 0.3 - 0.2, "the card's top is {card_top}");
        // the window and the medium picture do not meet, and the window's
        // title stays below the shelf
        assert!(MED_AT[0] + MED_CELL * w < WINDOW_AT[0] - WINDOW.0 / 2.0);
        assert!(WINDOW_AT[1] + WINDOW.1 / 2.0 + 0.5 <= SHELF_FLOOR + 1e-9);
        // the overlay does not meet the panel
        assert!(OVER_AT[0] + OVER_CELL * w < PANEL_X - PANEL_W / 2.0);
    }

    /// 14_2's window (storyboard, "Sound plan": three of the four bloom, the
    /// fourth is missing): three notes, the first three of the pentatonic's
    /// climb that the landings use, spaced as the lights on the pieces are,
    /// all inside the window of the breath they replace; the fourth step of
    /// the climb never sounds.
    #[test]
    fn three_of_the_four_bloom_and_the_fourth_is_missing() {
        let strikes = missing_strikes();
        assert_eq!(strikes.len(), 3);
        for (i, s) in strikes.iter().enumerate() {
            assert_eq!(s.pitch, sound::landing_pitch(i), "note {i} of the climb");
        }
        assert!(strikes.iter().all(|s| s.pitch != sound::landing_pitch(3)));
        assert!(strikes.windows(2).all(|w| w[1].at - w[0].at >= LIGHT_S + 0.05));
        // the last note and its ring, and the silent light on the empty tile
        // after it, end inside the window that took the breath's 2.5 s
        let window = explainer_kit::animatic::HOLD_S;
        let last = strikes.last().unwrap().at;
        assert!(
            last + LIGHT_S + 0.1 + LIGHT_S + 0.2 <= window + 0.05,
            "{last} s of {window}"
        );
        // quiet: the viewer is counting tiles under it
        assert!(strikes.iter().all(|s| s.velocity <= 0.5));
    }

    /// With the scratch voice placed each line starts within two seconds,
    /// and a breath, of the end of the one before, except after the quiz,
    /// whose countdown and landed answer fill the gap.
    #[test]
    fn no_line_waits_long_for_its_picture() {
        let v = testing::run_voiced("14_the_missing_square").unwrap();
        let mut lines = v.lines.clone();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        let quiz = quiz_named("slopes").unwrap();
        let held = f64::from(quiz.seconds) + super::super::draw::LAND_S;
        let mut prev: Option<(String, f64)> = None;
        let mut late: Vec<String> = Vec::new();
        for (id, at, len) in &lines {
            if let Some((before, end)) = &prev {
                let gap = at - end;
                eprintln!("{id}: starts {at:.1} after a gap of {gap:.1} (since {before})");
                let allowed = if before == quiz.line {
                    held + 2.0
                } else {
                    2.0 + explainer_kit::animatic::HOLD_S
                };
                if gap > allowed {
                    late.push(format!("{id} starts {gap:.1} s after {before} ends"));
                }
            }
            prev = Some((id.clone(), at + len));
        }
        eprintln!("chapter {:.1} s", v.ran.seconds);
        assert!(late.is_empty(), "{late:?}");
    }

    /// 14_7 (the dense read of the narrated draft): the citation and the
    /// empty slot come in as the two names are said and stand through the
    /// silence, long enough to read: at least five seconds before the
    /// chapter's closing fade and well before the voice has finished (they
    /// came in after the silence, 1.2 s before the end).
    #[test]
    fn the_source_comes_in_with_the_names_and_can_be_read() {
        use super::super::draw::quiz_log;
        let v = testing::run_voiced("14_the_missing_square").unwrap();
        let (_, at, len) = v.lines.iter().find(|l| l.0 == "14_7").cloned().unwrap();
        let events = quiz_log::take(&v.tag, "14_7");
        let source_in = events.iter().find(|e| e.0 == "source_in").map(|e| e.1).unwrap();
        eprintln!(
            "source in at {source_in:.1}, voice {at:.1}..{:.1}, chapter {:.1}",
            at + len,
            v.ran.seconds
        );
        // after the first sentence, while the names are being said
        assert!(source_in > at && source_in < at + 0.5 * len, "{source_in:.1}");
        let stands = v.ran.seconds - 0.6 - source_in;
        assert!(stands >= 5.0, "the citation stands {stands:.1} s");
        // the two seconds of silence are still the cross's own
        assert!((SILENCE_S - 2.0).abs() < 1e-9);
        let end = v
            .marks
            .iter()
            .find(|m| m.0 == "silence_end:cross")
            .map(|m| m.1)
            .unwrap();
        let start = v.marks.iter().find(|m| m.0 == "silence:cross").map(|m| m.1).unwrap();
        assert!(
            end - (at + len) >= SILENCE_S - 0.2 && start <= at + 0.1,
            "{start:.1} {end:.1}"
        );
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("14_the_missing_square").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "pieces:",
            "first picture",
            "second picture",
            "steeper: big or small?",
            "small",
            "3 up, 8 across",
            "2 up, 5 across",
            "3 in 8 = 0.375",
            "2 in 5 = 0.400",
            "8 times closer",
            "\\frac{1}{13}",
            "thickest",
            "17 pieces fill the empty tile",
            "neighbours in the Fibonacci numbers",
            "2 \\times 8 - 3 \\times 5 = 1",
            "\\frac{1}{40}",
            "the long edge is straight",
            "it bends here",
            "P. Curry, 1953; M. Gardner, Mathematics, Magic and Mystery, 1956",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 110.0, "{} s", ran.seconds);
    }
}
