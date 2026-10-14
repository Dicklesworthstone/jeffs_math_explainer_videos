//! Chapter 15, what makes a proof: the two arguments side by side, one that
//! gave a famous theorem and one that gave nonsense (15_1); the question
//! asked over their two cards (15_2); the three things a picture must have,
//! each with a small picture of the proofs that leaned on it (15_3); the
//! opening square back, one layer of it with every length named, the
//! rigorous form of the opening picture (15_4); what the program's checks are
//! and are not (15_5); and the square once more, in full colour, with the
//! names to search (15_6).
//!
//! Every number is a fact or the model's: the legs `F-P.legs`, the long side
//! `F-P.long_side`, the pieces of the lying picture `model::lie`, the square
//! of the five cubes `model::cubes::square`, the reach of the program's
//! checks `F-K.tiled.through`, `F-S.filled.through`, `F-K.sum.through`, the
//! years of the book `F-X.nelsen`.

use explainer_kit::{Kit, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardLook, CardRow, ProofCard, Side, Words, aside, dim, fact, fact_int, fact_ints, fade_in_all, fade_out_all,
    fade_to, faded_line, fail, faint, fit_width_left, flat, hero, ink, lens_mark, question_card, question_pause, say,
    size, warn,
};
use super::four_triangles::{ASSUMED_CORNER, CLOSES, SHARP_SUM, SHOWN_TEX, corners_f, left_over_f, slid_f, tilted_f};
use super::the_missing_square::{EDGE_STRAIGHT, IT_BENDS, MED_AT, MED_CELL, NOTHING, P2, ZOOM, edge_of, magnify};
use crate::facts::shown;
use crate::model::{Rect, cubes, lie};

pub struct WhatMakesAProof {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// A cell of the small pictures of 15_1, and the gaps between them.
const MINI_CELL: f64 = 0.22;
const MINI_GAP: f64 = 0.45;
const GROUP_GAP: f64 = 0.9;
const MINI_Y: f64 = 0.45;
/// The cards of 15_2: the question over them, and their scale and tops. The
/// question's top stands under the lowest thumbnail over it (the halved
/// square's number line hangs below the shelf's floor), and the first card
/// moves down with it, and the second a little.
const QUESTION_AT: Vec3 = [0.0, 1.78, 0.0];
const CARD_LOOK_A: CardLook = CardLook {
    x: 0.0,
    top: 0.68,
    scale: 0.7,
};
const CARD_LOOK_B: CardLook = CardLook {
    x: 0.0,
    top: -0.95,
    scale: 0.7,
};
/// The three lines of 15_3: where they start, their rows and size, and the
/// pictures' column and half side.
const LINES_X: f64 = -6.6;
const LINE_Y: [f64; 3] = [2.05, 0.5, -1.05];
const LINE_PT: f64 = 30.0;
const THUMB_X: f64 = 5.5;
const THUMB_R: f64 = 0.75;
/// The square of the opening, from 15_4 on, and how much a layer that is not
/// the one named dims.
const SQUARE_AT: Vec3 = [-6.3, -0.9, 0.0];
const SQUARE_CELL: f64 = 0.19;
const DIMMED: f64 = 0.3;
/// The receipts of 15_5: the left column and its rows, the aside.
const RECEIPT_X: f64 = -6.4;
const RECEIPT_Y: [f64; 4] = [-1.5, -2.05, -2.6, -3.15];
const STATEMENT_Y: f64 = -3.7;
const ASIDE_AT_CH: Vec3 = [4.2, -2.1, 0.0];
/// The names to search of 15_6: the left edge and the rows.
const NAMES_X: f64 = -2.8;
const NAMES_Y: [f64; 4] = [2.0, 1.15, 0.6, 0.0];
/// The listen window the video's one resolution plays in (15_6): soundless,
/// labelled for the score (`score::identity`'s theme "15_6", resolved), which
/// writes the motif and its resolution into it. The score fits the theme's
/// last note's onset before 2.5 s of the video's end, after the voice has
/// cleared (0.35 s) and the breath before the window (0.6 s); the window, the
/// fade after it (0.6 s) and the relaxed beat of the closing resolution make
/// this the shortest that holds all six notes, with a margin.
pub(super) const RESOLVE_S: f64 = 6.0;
/// The window's label: the chapter and the line of the theme.
pub(super) const RESOLVE_LABEL: &str = "15_what_makes_a_proof:15_6";

// everything keeps under the shelf's row and inside the frame
const _: () = assert!(QUESTION_AT[1] + 0.9 <= super::draw::SHELF_FLOOR + 1e-9);
const _: () = assert!(QUESTION_AT[1] - 0.9 > CARD_LOOK_A.top);
const _: () = assert!(LINE_Y[0] + THUMB_R <= super::draw::SHELF_FLOOR + 1e-9);
const _: () = assert!(LINE_Y[2] - THUMB_R > -4.0 + 0.1);
const _: () = assert!(THUMB_X + THUMB_R < 7.1 - 0.15);
const _: () = assert!(SQUARE_AT[1] > RECEIPT_Y[0] + 0.3);
const _: () = assert!(STATEMENT_Y > -4.0 + 0.2);
const _: () = assert!(NAMES_Y[0] <= super::draw::SHELF_FLOOR);

// the chapter's fixed words
const FOUR_TRIANGLES: &str = "four triangles";
const THE_MISSING_SQUARE: &str = "the missing square";
const REBUILT: &str = "rebuilt and checked by code:";
const FOR_PROGRAMMERS: &str = "for programmers";
const TWO_IMPLEMENTATIONS: &str = "two implementations compared; neither is a proof";
const WIDTH_TEX: &str = "k";
const ARM_TEX: &str = r"1 + 2 + \cdots + (k - 1)";
const HALF_SLABS_TEX: &str = r"(k - 1)\text{ half slabs}";
const SEARCH_FOR: &str = "proofs without words";
const BY: &str = "Roger B. Nelsen,";
const BOOK: &str = "Proofs Without Words";
const BOOK_TWO: &str = "Proofs Without Words II";
const JOURNAL: &str = "and Mathematics Magazine";

// ---------------------------------------------------------------- the model

fn pf(p: (i64, i64)) -> P2 {
    (p.0 as f64, p.1 as f64)
}

/// A whole number with its thousands separated by commas (`1000` is
/// `1,000`).
fn grouped(n: &str) -> String {
    let digits: Vec<char> = n.chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    out
}

/// The pair of shorter sides from a fact (`3,4`).
fn legs(key: &str) -> fmn::Result<(f64, f64)> {
    match fact_ints(key)?.as_slice() {
        [a, b] => Ok((*a as f64, *b as f64)),
        _ => Err(fail(format!("fact {key} is not a pair of lengths"))),
    }
}

/// The layer `k` of the square: its corner, the arm that stands up on the
/// right and the arm along the top, as rectangles of cells.
fn layer_parts(k: i64) -> (Rect, Rect, Rect) {
    let s = cubes::side(k - 1);
    (Rect::new(s, s, k, k), Rect::new(s, 0, k, s), Rect::new(0, s, s, k))
}

// ----------------------------------------------------------- the drawings

/// A closed polygon through cell points of `board`, in `style`. Detached.
fn poly(stage: &mut Stage<'_>, board: &Board, pts: &[P2], style: Style) -> Mob {
    let scene: Vec<Vec3> = pts.iter().map(|p| board.p(p.0, p.1)).collect();
    stage.arena_mut().add(Polygon::new(scene).style(style).build())
}

/// One arrangement of chapter 13's four triangles on `board`: the tilted
/// square (first) or the two squares on the shorter sides (second). Detached.
fn four_mini(stage: &mut Stage<'_>, board: &Board, a: f64, b: f64, long: usize, second: bool) -> Vec<Mob> {
    let mut out = Vec::new();
    if second {
        for &(x, y, s) in &left_over_f(a, b) {
            let sq = [(x, y), (x + s, y), (x + s, y + s), (x, y + s)];
            out.push(poly(stage, board, &sq, flat(size(s as usize), 1.0)));
        }
        for t in &slid_f(a, b) {
            out.push(poly(stage, board, t, flat(dim(), 0.9)));
        }
    } else {
        out.push(poly(stage, board, &tilted_f(a, b), flat(size(long), 1.0)));
        for t in &corners_f(a, b) {
            out.push(poly(stage, board, t, flat(dim(), 0.9)));
        }
    }
    out
}

/// One arrangement of chapter 14's four pieces on `board`, with the empty
/// tile in red when it has one. Detached.
fn lie_mini(
    stage: &mut Stage<'_>,
    board: &Board,
    a: &lie::Arrangement,
    hole: Option<(i64, i64)>,
) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for t in [&a.big, &a.small] {
        let pts = t.map(pf);
        out.push(poly(stage, board, &pts, flat(dim(), 0.9)));
    }
    out.push(board.tiles(stage, &a.long_l, size(4))?);
    out.push(board.tiles(stage, &a.short_l, size(1))?);
    if let Some(h) = hole {
        out.push(board.tiles_styled(stage, &[h], flat(warn(), 0.55))?);
    }
    Ok(out)
}

/// The frame of a small picture, `THUMB_R` either side of `at`. Detached.
fn thumb_frame(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(2.0 * THUMB_R)
            .height(2.0 * THUMB_R)
            .corner_radius(0.08)
            .style(faded_line(faint(), 2.0, 1.0))
            .build()?
            .moved_to(at),
    ))
}

/// The straight edge and the edge that hangs off it, eight times closer, as
/// in chapter 14's window. Detached.
fn bend_thumb(stage: &mut Stage<'_>, at: Vec3) -> Vec<Mob> {
    let board = Board::new(MED_AT, MED_CELL);
    let first = lie::first();
    let bend = pf(first.bend);
    let half = (THUMB_R - 0.12, THUMB_R - 0.12);
    let (w, t) = (lie::WIDE as f64, lie::TALL as f64);
    let mut out = magnify(
        stage,
        &board,
        bend,
        ZOOM,
        at,
        half,
        &[(0.0, 0.0), (w, t)],
        faded_line(ink(), 3.0, 1.0),
    );
    out.extend(magnify(
        stage,
        &board,
        bend,
        ZOOM,
        at,
        half,
        &edge_of(&first),
        faded_line(warn(), 4.0, 1.0),
    ));
    out
}

/// The square of the cubes up to `n`, one slab to a piece, each cube in its
/// colour, on `board`: the cube of each piece with its slab. Detached.
fn square_of(stage: &mut Stage<'_>, board: &Board, n: i64) -> Vec<(i64, Mob)> {
    cubes::square(n)
        .iter()
        .map(|p| (p.cube, board.slab(stage, p.rect, size(p.cube as usize))))
        .collect()
}

impl SceneConstruct for WhatMakesAProof {
    fn name(&self) -> &str {
        "15_what_makes_a_proof"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 15)?;
        let (a, b) = legs("F-P.legs")?;
        let long = fact_int("F-P.long_side")?.max(1) as usize;
        let (first, second) = (lie::first(), lie::second());
        let hole = lie::holes(&second)
            .first()
            .copied()
            .ok_or_else(|| fail("the second arrangement has no empty tile".into()))?;
        let n = shown::CUBES;

        // ------------------------------------------------------------ 15_1
        // two pictures, one argument: the pieces in two arrangements, and
        // what was left
        let line = say(&kit, stage, "15_1")?;
        let w13 = (a + b) * MINI_CELL;
        let h13 = (a + b) * MINI_CELL;
        let w14 = lie::WIDE as f64 * MINI_CELL;
        let h14 = lie::TALL as f64 * MINI_CELL;
        let group13 = 2.0 * w13 + MINI_GAP;
        let group14 = 2.0 * w14 + MINI_GAP;
        let left = -(group13 + GROUP_GAP + group14) / 2.0;
        let xs = [
            left,
            left + w13 + MINI_GAP,
            left + group13 + GROUP_GAP,
            left + group13 + GROUP_GAP + w14 + MINI_GAP,
        ];
        let y14 = MINI_Y + (h13 - h14) / 2.0;
        let boards = [
            Board::new([xs[0], MINI_Y, 0.0], MINI_CELL),
            Board::new([xs[1], MINI_Y, 0.0], MINI_CELL),
            Board::new([xs[2], y14, 0.0], MINI_CELL),
            Board::new([xs[3], y14, 0.0], MINI_CELL),
        ];
        let firsts = [
            four_mini(stage, &boards[0], a, b, long, false),
            lie_mini(stage, &boards[2], &first, None)?,
        ];
        let seconds = [
            four_mini(stage, &boards[1], a, b, long, true),
            lie_mini(stage, &boards[3], &second, Some(hole))?,
        ];
        let centre13 = left + group13 / 2.0;
        let centre14 = left + group13 + GROUP_GAP + group14 / 2.0;
        let name_y = MINI_Y - 0.4;
        let mark_y = MINI_Y + h13 + 0.5;
        let name_theorem = kit.text(stage, FOUR_TRIANGLES, 26.0, dim(), v(centre13, name_y))?;
        let name_nonsense = kit.text(stage, THE_MISSING_SQUARE, 26.0, dim(), v(centre14, name_y))?;
        let tick = lens_mark(stage, v(centre13, mark_y), 0.55, true);
        let cross = lens_mark(stage, v(centre14, mark_y), 0.55, false);
        line.wait_for(stage, "Two pictures")?;
        let mut shown_first: Vec<Mob> = firsts[0].clone();
        shown_first.extend(&firsts[1]);
        fade_in_all(stage, &shown_first, 0.7)?;
        line.wait_for(stage, "Same pieces")?;
        let mut shown_second: Vec<Mob> = seconds[0].clone();
        shown_second.extend(&seconds[1]);
        fade_in_all(stage, &shown_second, 0.7)?;
        line.wait_for(stage, "famous theorem")?;
        fade_in_all(stage, &[name_theorem, tick], 0.5)?;
        line.wait_for(stage, "nonsense")?;
        fade_in_all(stage, &[name_nonsense, cross], 0.5)?;

        // ------------------------------------------------------------ 15_2
        // so what makes a picture a proof? both cards, back
        let line = say(&kit, stage, "15_2")?;
        let mut gone = shown_first.clone();
        gone.extend(&shown_second);
        gone.extend([name_theorem, name_nonsense, tick, cross]);
        fade_out_all(stage, &gone, 0.5)?;
        let words = line.sentence_texts();
        let question = question_card(&kit, stage, words.first().map_or("", String::as_str), QUESTION_AT)?;
        let mut card_theorem = ProofCard::build_at(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Text(ASSUMED_CORNER)),
                CardRow::Assumed(Words::Text(CLOSES)),
            ],
            CARD_LOOK_A,
        )?;
        let mut card_nonsense = ProofCard::build_at(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Text(NOTHING)),
                CardRow::Assumed(Words::Text(EDGE_STRAIGHT)),
            ],
            CARD_LOOK_B,
        )?;
        fade_in_all(stage, &[question], 0.5)?;
        card_theorem.show(stage, 0.6)?;
        card_nonsense.show(stage, 0.6)?;
        card_theorem.tick(&kit, stage, 1, SHARP_SUM)?;
        card_theorem.tick(&kit, stage, 2, "")?;
        card_nonsense.cross(&kit, stage, 1, IT_BENDS)?;
        question_pause(&kit, stage)?;

        // ------------------------------------------------------------ 15_3
        // three things: each with a picture of the proofs that leaned on it
        let line = say(&kit, stage, "15_3")?;
        let mut gone = card_theorem.mobs();
        gone.extend(card_nonsense.mobs());
        gone.push(question);
        fade_out_all(stage, &gone, 0.5)?;
        let sentences = line.sentence_texts();
        if sentences.len() != 4 {
            return Err(fail(format!("line 15_3 has {} sentences, not four", sentences.len())));
        }
        let room = THUMB_X - THUMB_R - 0.3 - LINES_X;
        let mut rows: Vec<Mob> = Vec::new();
        for (j, y) in LINE_Y.iter().enumerate() {
            let m = Words::Text(&sentences[j + 1]).set_left(&kit, stage, LINE_PT, ink(), v(LINES_X, *y))?;
            fit_width_left(stage, m, room);
            rows.push(m);
        }
        let thumb_at = LINE_Y.map(|y| v(THUMB_X, y));
        let mut thumbs: Vec<Vec<Mob>> = Vec::new();
        for at in thumb_at {
            thumbs.push(vec![thumb_frame(stage, at)?]);
        }
        let small = 2.0 * THUMB_R - 0.3;
        thumbs[0].extend(super::draw::shelf_copy(
            &kit,
            stage,
            super::staircases::SLOT,
            thumb_at[0],
            small,
        )?);
        thumbs[1].extend(bend_thumb(stage, thumb_at[1]));
        thumbs[2].extend(super::draw::shelf_copy(
            &kit,
            stage,
            super::every_cube::SLOT,
            thumb_at[2],
            small,
        )?);
        for j in 0..3 {
            line.wait_sentence(stage, j + 1)?;
            let mut ins = vec![rows[j]];
            ins.extend(&thumbs[j]);
            fade_in_all(stage, &ins, 0.6)?;
        }
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 15_4
        // the cubes needed a second chapter: the picture showed five cubes,
        // the reason covered every cube
        let line = say(&kit, stage, "15_4")?;
        let mut gone = rows.clone();
        for t in &thumbs {
            gone.extend(t);
        }
        fade_out_all(stage, &gone, 0.5)?;
        let sq = Board::new(SQUARE_AT, SQUARE_CELL);
        let slabs = square_of(stage, &sq, n);
        let (corner, arm_up, _) = layer_parts(n);
        let width = sq.dimension(&kit, stage, corner, Side::Above, Words::Tex(WIDTH_TEX), 34.0, hero())?;
        let arm = sq.dimension(&kit, stage, arm_up, Side::Right, Words::Tex(ARM_TEX), 30.0, ink())?;
        let arm_mid = sq.p((arm_up.x + arm_up.w) as f64, arm_up.h as f64 / 2.0);
        let half_at = v(arm_mid[0] + 0.45, arm_mid[1] - 0.6);
        let half_slabs = kit.tex(stage, HALF_SLABS_TEX, 30.0, hero(), &[], half_at)?;
        stage.move_to(half_slabs, half_at, LEFT);
        let all: Vec<Mob> = slabs.iter().map(|&(_, m)| m).collect();
        line.wait_for(stage, "That is why")?;
        fade_in_all(stage, &all, 0.8)?;
        line.wait_for(stage, "The reason covered")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &(cube, m) in &slabs {
            if cube < n {
                anims.push(stage.prepare(fade_to(m, DIMMED, 0.6)?)?);
            }
        }
        stage.play_prepared(anims)?;
        let mut labels = vec![width.brace, width.label, arm.brace, arm.label, half_slabs];
        fade_in_all(stage, &labels, 0.6)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 15_5
        // what the program's checks are, and are not
        let line = say(&kit, stage, "15_5")?;
        let cubes_through = fact("F-K.tiled.through")?;
        let levels_through = fact("F-S.filled.through")?;
        let sums_through = fact("F-K.sum.through")?;
        let rebuilt = kit.text_left(stage, REBUILT, 24.0, dim(), v(RECEIPT_X, RECEIPT_Y[0]))?;
        let r_cubes = kit.text_left(
            stage,
            &format!("cubes up to {cubes_through}"),
            28.0,
            ink(),
            v(RECEIPT_X, RECEIPT_Y[1]),
        )?;
        let r_levels = kit.text_left(
            stage,
            &format!("stacks up to {levels_through} levels"),
            28.0,
            ink(),
            v(RECEIPT_X, RECEIPT_Y[2]),
        )?;
        let r_sums = kit.text_left(
            stage,
            &format!("every sum up to {}", grouped(&sums_through)),
            28.0,
            ink(),
            v(RECEIPT_X, RECEIPT_Y[3]),
        )?;
        let said = line.sentence_texts();
        let statement = kit.text_left(
            stage,
            said.last().map_or("", String::as_str),
            28.0,
            hero(),
            v(RECEIPT_X, STATEMENT_Y),
        )?;
        let programmers = aside(&kit, stage, FOR_PROGRAMMERS, TWO_IMPLEMENTATIONS, ASIDE_AT_CH)?;
        line.wait_for(stage, "the cubes up to forty")?;
        fade_in_all(stage, &[rebuilt, r_cubes], 0.5)?;
        line.wait_for(stage, "twenty-four levels")?;
        fade_in_all(stage, &[r_levels], 0.5)?;
        line.wait_for(stage, "up to a thousand")?;
        fade_in_all(stage, &[r_sums], 0.5)?;
        line.wait_for(stage, "That is still only checking")?;
        fade_in_all(stage, &[programmers], 0.5)?;
        line.wait_for(stage, "A check up to some size")?;
        fade_in_all(stage, &[statement], 0.6)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 15_6
        // looking is not checking; the reason shows you why; the names to
        // search
        let line = say(&kit, stage, "15_6")?;
        let mut gone = vec![rebuilt, r_cubes, r_levels, r_sums, statement, programmers];
        gone.append(&mut labels);
        fade_out_all(stage, &gone, 0.5)?;
        let (from, to) = match fact_ints("F-X.nelsen")?.as_slice() {
            [x, y] => (*x, *y),
            _ => return Err(fail("F-X.nelsen is not a pair of years".into())),
        };
        let search = kit.text_left(stage, SEARCH_FOR, 36.0, hero(), v(NAMES_X, NAMES_Y[0]))?;
        let book_one = kit.text_left(
            stage,
            &format!("{BY} {BOOK}, {from}"),
            26.0,
            ink(),
            v(NAMES_X, NAMES_Y[1]),
        )?;
        let book_two = kit.text_left(
            stage,
            &format!("{BY} {BOOK_TWO}, {to}"),
            26.0,
            ink(),
            v(NAMES_X, NAMES_Y[2]),
        )?;
        let journal = kit.text_left(stage, JOURNAL, 24.0, dim(), v(NAMES_X, NAMES_Y[3]))?;
        line.wait_for(stage, "But when you can say why")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &(cube, m) in &slabs {
            if cube < n {
                anims.push(stage.prepare(fade_to(m, 1.0, 0.7)?)?);
            }
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "It shows you why")?;
        fade_in_all(stage, &[search, book_one, book_two, journal], 0.7)?;
        line.wait_end(stage)?;
        // the video's one resolution (storyboard: "pieces, then the whole"):
        // the motif's three notes and their resolution, alone, in the
        // music's own window over the names to search
        kit.listen(stage, RESOLVE_S, RESOLVE_LABEL)?;

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{SHELF_FLOOR, shelf_log, testing};
    use super::*;

    /// 15_2: the question card stands clear of the shelf's thumbnails over it:
    /// the halved square's number line and its gold "1" hang below that
    /// thumbnail's box, below the shelf's floor, and the card's top edge,
    /// which stood at the floor, crossed them (the dense read, P5).
    #[test]
    fn the_question_card_stands_clear_of_the_shelfs_thumbnails() {
        testing::bare(|kit, stage| {
            let q = question_card(kit, stage, "So what makes a picture a proof?", QUESTION_AT)?;
            let (c, w, h) = (stage.get_center(q), stage.get_width(q), stage.get_height(q));
            let (left, right, top) = (c[0] - w / 2.0, c[0] + w / 2.0, c[1] + h / 2.0);
            let mut lowest = f64::INFINITY;
            for entry in super::super::SHELF {
                let mobs = (entry.thumb)(kit, stage)?;
                let Some((lo, hi)) = shelf_log::union_box(stage, &mobs) else {
                    continue;
                };
                if hi[0] > left && lo[0] < right {
                    lowest = lowest.min(lo[1]);
                }
            }
            assert!(lowest.is_finite(), "no thumbnail stands over the card");
            assert!(
                top + 0.03 <= lowest,
                "the card's top is {top:.3}, the thumbnails over it reach down to {lowest:.3}"
            );
            Ok(())
        })
        .unwrap();
    }

    /// 15_5 and 15_6: the receipts and the names are the facts, and a number
    /// of thousands is written with its comma.
    #[test]
    fn the_receipts_and_the_names_are_the_facts() {
        assert_eq!(fact("F-K.tiled.through").unwrap(), "40");
        assert_eq!(fact("F-S.filled.through").unwrap(), "24");
        assert_eq!(fact("F-K.sum.through").unwrap(), "1000");
        assert_eq!(fact("F-X.nelsen").unwrap(), "1993,2000");
        assert_eq!(grouped("1000"), "1,000");
        assert_eq!(grouped("40"), "40");
        assert_eq!(grouped("1234567"), "1,234,567");
        assert_eq!(grouped("100"), "100");
    }

    /// 15_3 and 15_4: the square is the five cubes' slabs, and the layer
    /// named is the last: a corner k by k, two arms k wide and as long as the
    /// square before it, which is k - 1 half slabs each, for the layer of
    /// every size.
    #[test]
    fn the_square_and_its_layer_are_the_models() {
        let n = shown::CUBES;
        let pieces = cubes::square(n);
        assert_eq!(pieces.len().to_string(), fact("F-K.pieces").unwrap());
        let side = cubes::side(n);
        assert_eq!(side.to_string(), fact("F-K.side").unwrap());
        assert_eq!(pieces.iter().map(|p| p.rect.area()).sum::<i64>(), side * side);
        let (corner, up, along) = layer_parts(n);
        assert_eq!(corner, Rect::new(10, 10, 5, 5));
        assert_eq!(up, Rect::new(10, 0, 5, 10));
        assert_eq!(along, Rect::new(0, 10, 10, 5));
        // the model's pieces of that layer are exactly the corner and the arms
        let layer = cubes::layer(n);
        let area: i64 = layer.iter().map(|p| p.rect.area()).sum();
        assert_eq!(area, corner.area() + up.area() + along.area());
        assert!(layer.iter().all(|p| {
            let r = p.rect;
            let inside = |o: Rect| r.x >= o.x && r.y >= o.y && r.x + r.w <= o.x + o.w && r.y + r.h <= o.y + o.h;
            inside(corner) || inside(up) || inside(along)
        }));
        for k in 1..=40 {
            let (c, u, l) = layer_parts(k);
            // the arm is the sides laid end to end: 1 + 2 + ... + (k - 1)
            assert_eq!(u.h, (1..k).sum::<i64>());
            assert_eq!(u.w, k);
            assert_eq!((c.w, c.h), (k, k));
            assert_eq!((l.w, l.h), (u.h, k));
            // k - 1 half slabs of k by k/2 fill an arm: twice its area is
            // k - 1 slabs of k by k
            assert_eq!(2 * u.area(), (k - 1) * k * k);
            assert_eq!(u.area(), l.area());
        }
    }

    /// 15_1: the small pictures are chapter 13's and 14's arrangements: each
    /// pair of arrangements fills the same frame, and the slid one of the
    /// lying picture leaves its one empty tile.
    #[test]
    fn the_small_pictures_are_the_two_arguments() {
        let (a, b) = legs("F-P.legs").unwrap();
        let tri =
            |t: &[P2; 3]| ((t[1].0 - t[0].0) * (t[2].1 - t[0].1) - (t[2].0 - t[0].0) * (t[1].1 - t[0].1)).abs() / 2.0;
        let whole = (a + b) * (a + b);
        let first: f64 = corners_f(a, b).iter().map(tri).sum();
        let tilted = {
            let t = tilted_f(a, b);
            (t[1].0 - t[0].0).powi(2) + (t[1].1 - t[0].1).powi(2)
        };
        assert!((first + tilted - whole).abs() < 1e-9);
        let slid: f64 = slid_f(a, b).iter().map(tri).sum();
        let squares: f64 = left_over_f(a, b).iter().map(|s| s.2 * s.2).sum();
        assert!((slid + squares - whole).abs() < 1e-9);
        let long = fact_int("F-P.long_side").unwrap();
        assert!((tilted - (long * long) as f64).abs() < 1e-9);
        assert!(lie::holes(&lie::first()).is_empty());
        assert_eq!(lie::holes(&lie::second()).len(), 1);
    }

    /// The pieces keep clear of the shelf's row, of each other and of the
    /// frame.
    #[test]
    fn the_pictures_keep_to_their_rooms() {
        // 15_1: the pictures, the marks and the names
        let (a, b) = legs("F-P.legs").unwrap();
        let (w13, h13) = ((a + b) * MINI_CELL, (a + b) * MINI_CELL);
        let w14 = lie::WIDE as f64 * MINI_CELL;
        let whole = 2.0 * w13 + MINI_GAP + GROUP_GAP + 2.0 * w14 + MINI_GAP;
        assert!(whole / 2.0 < 7.1 - 0.2, "the pictures are {whole} wide");
        assert!(MINI_Y + h13 + 0.5 + 0.3 <= SHELF_FLOOR + 1e-9);
        // 15_2: the question and the two cards stack without meeting
        let a_rows = 3.0;
        let b_rows = 2.0;
        let card_h = |rows: f64, k: f64| (2.0 * 0.24 + 0.56 * rows) * k;
        let a_bottom = CARD_LOOK_A.top - card_h(a_rows, CARD_LOOK_A.scale);
        let b_bottom = CARD_LOOK_B.top - card_h(b_rows, CARD_LOOK_B.scale);
        assert!(a_bottom > CARD_LOOK_B.top, "the cards meet");
        assert!(b_bottom > -4.0 + 0.1, "the second card leaves the frame: {b_bottom}");
        // 15_3: the lines' pictures keep apart
        for w in LINE_Y.windows(2) {
            assert!(w[0] - w[1] >= 2.0 * THUMB_R, "the pictures meet");
        }
        // 15_4 and 15_5: the square, its labels, and the receipts under it
        let side = cubes::side(shown::CUBES) as f64 * SQUARE_CELL;
        assert!(
            SQUARE_AT[1] + side + 0.7 <= SHELF_FLOOR + 0.5,
            "the width label is too high"
        );
        assert!(RECEIPT_Y.windows(2).all(|w| w[0] > w[1]));
        // 15_6: the names sit right of the square
        assert!(NAMES_X > SQUARE_AT[0] + side + 0.5);
    }

    /// With the scratch voice placed each line starts within two seconds,
    /// and a breath, of the end of the one before.
    #[test]
    fn no_line_waits_long_for_its_picture() {
        let v = testing::run_voiced("15_what_makes_a_proof").unwrap();
        let mut lines = v.lines.clone();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut prev: Option<(String, f64)> = None;
        let mut late: Vec<String> = Vec::new();
        for (id, at, len) in &lines {
            if let Some((before, end)) = &prev {
                let gap = at - end;
                eprintln!("{id}: starts {at:.1} after a gap of {gap:.1} (since {before})");
                // the question's pause is two seconds
                let allowed = 2.0 + explainer_kit::animatic::HOLD_S + super::super::draw::QUESTION_HOLD_S;
                if gap > allowed {
                    late.push(format!("{id} starts {gap:.1} s after {before} ends"));
                }
            }
            prev = Some((id.clone(), at + len));
        }
        eprintln!("chapter {:.1} s", v.ran.seconds);
        assert!(late.is_empty(), "{late:?}");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("15_what_makes_a_proof").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "four triangles",
            "the missing square",
            "So what makes a picture a",
            "proof?",
            "the tilted corner sits on a straight side",
            "the long edge is straight",
            "Every piece is accounted for.",
            "Every fit has a reason, and a look is not a reason.",
            "And the reason does not depend on the size you happened to draw.",
            "1 + 2 + \\cdots + (k - 1)",
            "(k - 1)\\text{ half slabs}",
            "rebuilt and checked by code:",
            "cubes up to 40",
            "stacks up to 24 levels",
            "every sum up to 1,000",
            "A check up to some size does not prove the next size.",
            "for programmers",
            "two implementations compared;",
            "neither is a proof",
            "proofs without words",
            "Roger B. Nelsen, Proofs Without Words, 1993",
            "Roger B. Nelsen, Proofs Without Words II, 2000",
            "and Mathematics Magazine",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 30.0 && ran.seconds < 100.0, "{} s", ran.seconds);
    }
}
