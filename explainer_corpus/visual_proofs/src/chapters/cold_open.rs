//! Chapter 01, the cold open: the owner's first picture as a silent fit
//! (storyboard, "the payoff as a puzzle"). Five cubes, each a little bigger
//! than the last, come apart into their slabs (01_1); two slabs are cut in
//! half, everything lies flat and the pieces close up into one square with
//! no gap and nothing over (01_2); the question ("why would that ever
//! fit?", 01_3); and the title over the square, which dims (01_4).
//!
//! The pieces are `model::cubes::square(5)` exactly: the fifteen slabs and
//! the two that are cut, seventeen pieces in all, each drawn as the unit
//! cubes of one slab by the projected-cube kit (`iso`). Every count the
//! chapter relies on is read from the facts and checked against the model
//! before anything is shown.

use explainer_kit::{Kit, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    fact_int, fade_in_all, fade_out_all, fail, hero, ink, line_style, pause, question_card, question_pause, say, size,
    sound::{self, Strike},
};
use super::iso::{Axis, Iso, P3, Solid, ViewAt, box_cells};
use crate::facts::shown;
use crate::model::cubes::{self, Part, Piece};

pub struct ColdOpen {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order (01_4 is the title, with
/// no words).
pub const LINES: [&str; 3] = ["01_1", "01_2", "01_3"];

// the chapter's fixed words
const TITLE: &str = "Proofs You Can See";
const QUESTION_AT: Vec3 = [0.0, 0.0, 0.0];

// ---------------------------------------------------------------- geometry

/// The empty lattice units between two cubes of the row.
pub const CUBE_GAP: f64 = 1.3;
/// How far each slab lifts clear of the slab below it, in lattice units.
pub const LIFT: f64 = 0.55;
/// The row of cubes stands in front of the board (lattice y).
pub const ROW_Y: f64 = -9.0;
/// The board's lower left corner on the ground (lattice x and y): the
/// square fills 0 to 15 each way.
pub const BOARD: (f64, f64) = (0.0, 0.0);
/// How high the pieces hover over their places before they drop.
pub const HOVER: f64 = 3.0;
/// How far the halves of a cut slab open: each moves this far from the cut.
pub const OPEN: f64 = 0.3;
/// The drop of one piece, seconds, and the lag between one piece and the
/// next as a share of that (a lagged group starts each move `LAG` of a
/// move's length after the one before).
pub const DROP_S: f64 = 0.4;
pub const DROP_LAG: f64 = 0.22;
/// The listen window the landings play in (the last note rings out).
pub const WINDOW_S: f64 = 2.0;
/// The two views: the classic one of the row, and the flat one of the board.
const ROOM_ROW: (f64, f64) = (13.4, 6.4);
const ROOM_BOARD: (f64, f64) = (7.4, 7.0);

/// Where cube `k` (from 1) stands in the row: the lattice x of its left side.
pub fn row_x(k: i64) -> f64 {
    (1..k).map(|i| i as f64 + CUBE_GAP).sum()
}

/// Slab `j` (from 0, the bottom) of cube `k`, as the cube stands: stacked.
pub fn stacked(k: i64, j: i64) -> P3 {
    [row_x(k), ROW_Y, j as f64]
}

/// The same slab lifted clear of its neighbours.
pub fn lifted(k: i64, j: i64) -> P3 {
    [row_x(k), ROW_Y, j as f64 * (1.0 + LIFT)]
}

/// The lowest corner of a piece's place on the board, at height `z`.
pub fn board_corner(piece: &Piece, z: f64) -> P3 {
    [BOARD.0 + piece.rect.x as f64, BOARD.1 + piece.rect.y as f64, z]
}

/// The row of cubes, seen from the classic viewpoint and fitted to the
/// stage: the lattice box it fills runs from the first cube's left side to
/// the last one's right, and as high as the fifth cube's top slab, lifted.
pub fn row_view(n: i64) -> ViewAt {
    let width = row_x(n) + n as f64;
    let high = lifted(n, n - 1)[2] + 1.0;
    ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0, ROW_Y, 0.0], [width, ROW_Y + n as f64, high], (0.0, 0.1), ROOM_ROW)
}

/// The board seen from straight above, the square of side `side` centred.
pub fn board_view(side: i64) -> ViewAt {
    ViewAt::top_down((0.0, 0.0), 1.0).fitted(
        [BOARD.0, BOARD.1, 0.0],
        [BOARD.0 + side as f64, BOARD.1 + side as f64, 1.0],
        (0.0, -0.1),
        ROOM_BOARD,
    )
}

/// The seam of a half in the finished square: the edge along which its
/// slab was cut, as two lattice points on the board. The lower half keeps
/// its upper side, and the upper half, turned a quarter turn, its right
/// side.
pub fn seam(piece: &Piece) -> Option<(P3, P3)> {
    let r = piece.rect;
    let (x, y, w, h) = (r.x as f64, r.y as f64, r.w as f64, r.h as f64);
    let (a, b) = match piece.part {
        Part::Whole => return None,
        Part::HalfRight => ((x, y + h), (x + w, y + h)),
        Part::HalfTop => ((x + w, y), (x + w, y + h)),
    };
    Some(([BOARD.0 + a.0, BOARD.1 + a.1, 0.0], [BOARD.0 + b.0, BOARD.1 + b.1, 0.0]))
}

/// One slab of the row: whole, or cut into a lower and an upper half.
struct Slab {
    cube: i64,
    index: i64,
    whole: Solid,
}

/// A cut slab's two halves, hidden until the cut.
struct Cut {
    cube: i64,
    whole: Solid,
    lower: Solid,
    upper: Solid,
}

/// A piece of the square, the solid that lands as it, and whether it must be
/// turned a quarter turn on the way (the upper half of a cut slab).
struct Landing {
    piece: Piece,
    solid: Solid,
    turned: bool,
}

/// The facts this chapter shows, checked against the model: fifteen slabs,
/// two cut, seventeen pieces, a square fifteen on a side.
fn counts(n: i64) -> fmn::Result<(i64, i64, i64, i64)> {
    let want = [
        ("F-K.slabs", (1..=n).sum::<i64>()),
        ("F-K.cut_slabs", (1..=n).filter(|k| k % 2 == 0).count() as i64),
        ("F-K.pieces", cubes::square(n).len() as i64),
        ("F-K.side", cubes::side(n)),
    ];
    let mut out = [0; 4];
    for (i, (key, model)) in want.into_iter().enumerate() {
        let shown = fact_int(key)?;
        if shown != model {
            return Err(fail(format!("{key} is {shown} but the model's square has {model}")));
        }
        out[i] = shown;
    }
    Ok((out[0], out[1], out[2], out[3]))
}

/// The notes of the landings: piece `i` of `n` lands `DROP_S` after its drop
/// begins, and the pitches climb the F major pentatonic from the F below the
/// tonic, one note a piece: a finished picture is a finished chord.
pub fn landing_strikes(n: usize) -> Vec<Strike> {
    const PENTATONIC: [f64; 5] = [0.0, 2.0, 4.0, 7.0, 9.0];
    (0..n)
        .map(|i| Strike {
            at: DROP_LAG * DROP_S * i as f64 + DROP_S,
            pitch: 53.0 + 12.0 * (i / 5) as f64 + PENTATONIC[i % 5],
            velocity: if i + 1 == n { 0.8 } else { 0.55 },
        })
        .collect()
}

impl SceneConstruct for ColdOpen {
    fn name(&self) -> &str {
        "01_cold_open"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = shown::CUBES;
        let (slab_count, cut_count, piece_count, side) = counts(n)?;
        let pieces = cubes::square(n);

        let iso = Iso::new(stage, row_view(n))?;

        // ------------------------------------------------------- the solids
        // each cube is its slabs, stacked; a slab that is cut also has its
        // two halves, hidden and exactly where its halves are
        let mut slabs: Vec<Slab> = Vec::new();
        let mut cuts: Vec<Cut> = Vec::new();
        for k in 1..=n {
            for j in 0..k {
                let whole = iso.solid(stage, &box_cells(k, k, 1), size(k as usize), stacked(k, j))?;
                slabs.push(Slab {
                    cube: k,
                    index: j,
                    whole,
                });
                let cut = pieces
                    .iter()
                    .any(|p| p.cube == k && p.slab == j && p.part != Part::Whole);
                if cut {
                    let half = box_cells(k, k / 2, 1);
                    let at = lifted(k, j);
                    let lower = iso.solid(stage, &half, size(k as usize), at)?;
                    let upper = iso.solid(stage, &half, size(k as usize), [at[0], at[1] + (k / 2) as f64, at[2]])?;
                    cuts.push(Cut {
                        cube: k,
                        whole,
                        lower,
                        upper,
                    });
                }
            }
        }
        if slabs.len() as i64 != slab_count || cuts.len() as i64 != cut_count {
            return Err(fail("the slabs built are not the facts' slabs".into()));
        }
        // each piece of the square and the solid that lands as it
        let mut landings: Vec<Landing> = Vec::new();
        for piece in &pieces {
            let solid = match piece.part {
                Part::Whole => slabs
                    .iter()
                    .find(|s| s.cube == piece.cube && s.index == piece.slab)
                    .map(|s| s.whole),
                Part::HalfRight => cuts.iter().find(|c| c.cube == piece.cube).map(|c| c.lower),
                Part::HalfTop => cuts.iter().find(|c| c.cube == piece.cube).map(|c| c.upper),
            }
            .ok_or_else(|| fail(format!("no solid for piece {piece:?}")))?;
            landings.push(Landing {
                piece: *piece,
                solid,
                turned: piece.part == Part::HalfTop,
            });
        }
        if landings.len() as i64 != piece_count {
            return Err(fail("the pieces of the square are not the facts' pieces".into()));
        }

        // ---------------------------------------------------------- 01_1
        // five cubes, then each comes apart into its slabs
        let line = say(&kit, stage, LINES[0])?;
        let mut moves = Vec::new();
        for s in &slabs {
            moves.push(iso.fade(s.whole, 1.0, 0.5)?);
        }
        iso.play(stage, moves, 0.06)?;
        line.wait_for(stage, "Slice them into slabs")?;
        let mut moves = Vec::new();
        for s in slabs.iter().filter(|s| s.index > 0) {
            moves.push(iso.slide(stage, s.whole, lifted(s.cube, s.index), 0.8)?);
        }
        iso.play(stage, moves, 0.1)?;

        // ---------------------------------------------------------- 01_2
        // two slabs are cut, everything lies flat, the pieces close up
        let line = say(&kit, stage, LINES[1])?;
        let mut open: Vec<AnimBuilder> = Vec::new();
        let mut swap: Vec<AnimBuilder> = Vec::new();
        for c in &cuts {
            iso.show(stage, c.lower)?;
            iso.show(stage, c.upper)?;
            swap.push(iso.fade(c.whole, 0.0, 0.05)?);
            swap.push(iso.glow(c.lower, hero(), 1.0, 0.3)?);
            swap.push(iso.glow(c.upper, hero(), 1.0, 0.3)?);
            open.push(iso.slide_by(stage, c.lower, [0.0, -OPEN, 0.0], 0.5)?);
            open.push(iso.slide_by(stage, c.upper, [0.0, OPEN, 0.0], 0.5)?);
            open.push(iso.glow(c.lower, hero(), 0.0, 0.6)?);
            open.push(iso.glow(c.upper, hero(), 0.0, 0.6)?);
        }
        iso.play(stage, swap, 0.0)?;
        iso.play(stage, open, 0.0)?;
        line.wait_for(stage, "lay everything flat")?;
        // the view comes down over the row while every piece rises to hover
        // over its place on the board (the two halves that are turned, on
        // the way)
        let mut together = iso.view_to(board_view(side), 1.4)?;
        for l in &landings {
            let corner = board_corner(&l.piece, HOVER);
            together.push(if l.turned {
                iso.turn_landing(stage, l.solid, Axis::Z, 1, corner, 1.2)?
            } else {
                iso.land(stage, l.solid, corner, 1.2)?
            });
        }
        iso.play(stage, together, 0.0)?;
        line.wait_for(stage, "close up")?;
        // in the music's own window the pieces drop, layer by layer, one note
        // each, and close the square
        let strikes = landing_strikes(landings.len());
        let wav = sound::strikes("proofs_cold_open_landings", &strikes, WINDOW_S)?;
        kit.sonify(stage, WINDOW_S, "01_cold_open:01_2", &wav)?;
        let mut drops = Vec::new();
        for l in &landings {
            drops.push(iso.land(stage, l.solid, board_corner(&l.piece, 0.0), DROP_S)?);
        }
        iso.play(stage, drops, DROP_LAG)?;
        kit.mark(stage, "pizz:square");
        // the seams of the four halves stay visible in the finished square
        let flat = board_view(side);
        let mut seams: Vec<Mob> = Vec::new();
        for l in &landings {
            if let Some((a, b)) = seam(&l.piece) {
                seams.push(
                    stage.arena_mut().add(
                        polyline(vec![flat.project(a), flat.project(b)])
                            .style(line_style(hero(), 4.0))
                            .build(),
                    ),
                );
            }
        }
        if seams.len() as i64 != 2 * cut_count {
            return Err(fail("a cut slab has not two seams".into()));
        }
        fade_in_all(stage, &seams, 0.4)?;

        // ---------------------------------------------------------- 01_3
        // no gap and nothing over: the question sets itself over the square
        let line = say(&kit, stage, LINES[2])?;
        line.wait_last(stage)?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("01_3 has no sentences".into()))?;
        let question = question_card(&kit, stage, &words, QUESTION_AT)?;
        fade_in_all(stage, &[question], 0.5)?;
        question_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.4)?;

        // ---------------------------------------------------------- 01_4
        // the title, over the square, which dims; the motif plays alone
        let title = kit.text(stage, TITLE, 96.0, ink(), v(0.0, 0.0))?;
        let dim = iso.fade_all(0.22, 0.8)?;
        let mut dimming: Vec<Box<dyn Animation>> = vec![stage.prepare(dim)?];
        for &m in &seams {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            dimming.push(stage.prepare(a.rt(0.8))?);
        }
        let a = fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?;
        dimming.push(stage.prepare(a.rt(0.8))?);
        stage.play_prepared(dimming)?;
        kit.listen(stage, 3.0, "01_cold_open:01_4")?;
        pause(stage, 0.2)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::{Rect, tiles};

    /// The scene constructs end to end (every phrase it waits for is in its
    /// line, every fact it reads is there, every string typesets) and the
    /// layout lint of the kit finds nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("01_cold_open").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 20.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }

    /// The chapter speaks exactly the script's 01_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("01_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The counts the scene relies on are the facts' and the model's:
    /// fifteen slabs, two cut, seventeen pieces, a square fifteen on a side.
    #[test]
    fn the_counts_are_the_facts() {
        assert_eq!(counts(shown::CUBES).unwrap(), (15, 2, 17, 15));
        // and the model's own counts, for any number of cubes: n(n + 1)/2
        // slabs, one cut for every even cube and two pieces for it
        for n in 1..=12 {
            let pieces = cubes::square(n);
            let halves = pieces.iter().filter(|p| p.part != Part::Whole).count() as i64;
            let evens = (1..=n).filter(|k| k % 2 == 0).count() as i64;
            assert_eq!(halves, 2 * evens);
            assert_eq!(pieces.len() as i64, n * (n + 1) / 2 + evens);
        }
    }

    /// Where the pieces land: the board's rectangles cover the square
    /// exactly once each, and a piece turned a quarter turn has the sides it
    /// needs (the upper half of a cut slab is k/2 wide and k tall on the
    /// board, and k wide and k/2 deep as it stands in the row).
    #[test]
    fn the_landings_tile_the_square_and_the_turned_halves_fit() {
        let n = shown::CUBES;
        let pieces = cubes::square(n);
        let side = cubes::side(n);
        tiles(
            pieces.iter().flat_map(|p| p.rect.cells()),
            Rect::new(0, 0, side, side),
            &[],
        )
        .unwrap();
        for p in &pieces {
            let k = p.cube;
            let in_row = match p.part {
                Part::Whole => (k, k),
                Part::HalfRight | Part::HalfTop => (k, k / 2),
            };
            let (w, h) = (p.rect.w, p.rect.h);
            if p.part == Part::HalfTop {
                assert_eq!((h, w), in_row, "a quarter turn swaps a half's two sides");
            } else {
                assert_eq!((w, h), in_row);
            }
            // the corner a piece lands at is the corner of its rectangle
            let c = board_corner(p, 0.0);
            assert_eq!((c[0], c[1]), (p.rect.x as f64, p.rect.y as f64));
        }
        // exactly the upper halves are turned: one for each cut slab, which
        // is the facts' two
        let turned = pieces.iter().filter(|p| p.part == Part::HalfTop).count();
        let facts = crate::facts::facts().unwrap();
        assert_eq!(facts.get("F-K.cut_slabs"), Some(turned.to_string().as_str()));
    }

    /// The row stands clear: cubes do not touch (a gap between neighbours),
    /// the lifted slabs of one cube do not touch (a gap between slabs), and
    /// the whole row is inside the stage in the classic view.
    #[test]
    fn the_row_and_its_slabs_stand_clear_and_fit_the_stage() {
        for k in 1..shown::CUBES {
            assert!(row_x(k + 1) - (row_x(k) + k as f64) >= CUBE_GAP - 1e-12);
        }
        for k in 1..=shown::CUBES {
            for j in 1..k {
                assert!(lifted(k, j)[2] - lifted(k, j - 1)[2] >= 1.0 + LIFT - 1e-12);
            }
        }
        let view = row_view(shown::CUBES);
        let width = row_x(shown::CUBES) + shown::CUBES as f64;
        let high = lifted(shown::CUBES, shown::CUBES - 1)[2] + 1.0;
        let n = shown::CUBES as f64;
        let mut lo = (f64::INFINITY, f64::INFINITY);
        let mut hi = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        for corner in 0..8 {
            let p = [
                if corner & 1 == 1 { width } else { 0.0 },
                ROW_Y + if corner & 2 == 2 { n } else { 0.0 },
                if corner & 4 == 4 { high } else { 0.0 },
            ];
            let s = view.project(p);
            lo = (lo.0.min(s[0]), lo.1.min(s[1]));
            hi = (hi.0.max(s[0]), hi.1.max(s[1]));
        }
        assert!(
            lo.0 > -FRAME_X_RADIUS + 0.4 && hi.0 < FRAME_X_RADIUS - 0.4,
            "{lo:?} {hi:?}"
        );
        assert!(
            lo.1 > -FRAME_Y_RADIUS + 0.4 && hi.1 < FRAME_Y_RADIUS - 0.4,
            "{lo:?} {hi:?}"
        );
        // the board fits too, with room for the title's line
        let flat = board_view(cubes::side(shown::CUBES));
        let side = cubes::side(shown::CUBES) as f64;
        for (x, y) in [(0.0, 0.0), (side, 0.0), (0.0, side), (side, side)] {
            let s = flat.project([BOARD.0 + x, BOARD.1 + y, 0.0]);
            assert!(
                s[0].abs() < FRAME_X_RADIUS - 0.4 && s[1].abs() < FRAME_Y_RADIUS - 0.2,
                "{s:?}"
            );
        }
    }

    /// Each half's seam lies on the boundary of its own rectangle, along its
    /// long side (the side the slab was cut across), and a whole slab has none.
    #[test]
    fn a_half_s_seam_is_its_long_side_on_the_board() {
        let pieces = cubes::square(shown::CUBES);
        let mut seams = 0;
        for p in &pieces {
            match (seam(p), p.part) {
                (None, Part::Whole) => {}
                (Some((a, b)), Part::HalfRight | Part::HalfTop) => {
                    seams += 1;
                    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
                    assert_eq!(len, p.cube as f64, "the seam is a side of the cut slab");
                    let (x0, y0) = (BOARD.0 + p.rect.x as f64, BOARD.1 + p.rect.y as f64);
                    let (x1, y1) = (x0 + p.rect.w as f64, y0 + p.rect.h as f64);
                    for e in [a, b] {
                        assert!(e[0] >= x0 && e[0] <= x1 && e[1] >= y0 && e[1] <= y1);
                    }
                    let on_edge =
                        (a[0] == b[0] && (a[0] == x0 || a[0] == x1)) || (a[1] == b[1] && (a[1] == y0 || a[1] == y1));
                    assert!(on_edge, "the seam runs along an edge of its half");
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(seams, 4, "two cut slabs, two seams each");
    }

    /// One note a piece, in the order the pieces land, climbing; the last is
    /// the strongest and sounds inside the listen window.
    #[test]
    fn one_note_a_piece_climbing_and_inside_the_window() {
        let strikes = landing_strikes(17);
        assert_eq!(strikes.len(), 17);
        assert!(strikes.windows(2).all(|w| w[1].at > w[0].at && w[1].pitch > w[0].pitch));
        let last = strikes.last().unwrap();
        assert!(last.velocity > strikes[0].velocity);
        assert!(
            last.at < WINDOW_S,
            "the last piece lands at {} s in a window of {WINDOW_S}",
            last.at
        );
        // the pitches stay inside the marimba's range
        assert!(strikes.iter().all(|s| (48.0..=96.0).contains(&s.pitch)));
    }
}
