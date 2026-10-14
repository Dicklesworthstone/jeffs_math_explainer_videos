//! Chapter 10, halves: a square of area one, half of it coloured, then half
//! of what is left, again and again (storyboard 10_1 to 10_6). The pieces are
//! `model::series::half(k)` exactly; every fraction on screen is a fact
//! (`F-H.*`) printed as it is written there.
//!
//! The picture has two parts. The square sits at the right, and a number
//! line from 0 to 1 as long as its side sits under it. 10_3 zooms the square
//! (about its corner, where what is left shrinks to) to show more pieces and
//! the gap still there; 10_4 zooms the line (about its right end) by 2^10 so
//! that a guess a thousandth short of one and the piece that passes it can be
//! told apart; 10_5 zooms back out and lays the crossed-out guesses down as a
//! band that grows to the edge; 10_6 lands the guess that nothing passes.
//!
//! A zoom scales fills and outlines about a fixed point and slides the
//! point-like marks (ticks, numerals, crosses) by the same rule, so a tick
//! keeps its size while its place is exactly where the zoom puts it.

use explainer_kit::{Kit, Timed, fade_all, v};
use fmn::library::exmark;
use fmn::prelude::*;

use super::draw::{
    ASIDE_AT, CardRow, LENS_AT, ProofCard, QUESTION_HOLD_S, Words, aside, dim, fact, fact_int, fail, faint,
    file_on_shelf, flash, fly, formula, ground, group, hero, ink, lens, lens_mark, line_style, lower_third, panel,
    pause, question_card, repaint, say, size, slide, sound, swap, wait_until, warn, write_formula,
};
use super::kinetics::{
    Moves, arrive, arrive_sliding, fit_in, leave, play, play_with, prepared, rectangle, slid_by_zoom, together,
    zoom_about, zoomed,
};
use crate::facts::shown;
use crate::model::series;

pub struct Halves {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["10_1", "10_2", "10_3", "10_4", "10_5", "10_6"];

// ---------------------------------------------------------------- geometry

/// Where 10_2's readout of what is filled is set: its fraction's top keeps
/// under the shelf's floor.
const FILLED_Y: f64 = 2.3;
/// Where the question of 10_3 is set: its top keeps under the shelf's floor.
const QUESTION_Y: f64 = 1.4;
/// The unit square: its side, and its lower left corner, in scene units.
pub const SIDE: f64 = 3.6;
pub const SQUARE_X: f64 = 2.5;
pub const SQUARE_Y: f64 = 0.1;
/// The dark seam between neighbouring pieces.
pub const SEAM: f64 = 0.03;
/// The number line under the square, and the half height of a tick on it.
pub const LINE_Y: f64 = -0.85;
/// How far under the line its numerals sit (the card's top is at -1.6).
pub const NUMERAL_DROP: f64 = 0.45;
/// How far a guess's cross stands from the edge of its card.
pub const CROSS_BESIDE: f64 = 0.28;
/// Where the picture goes when the card comes (the square's middle, and how
/// much the picture shrinks), and the shelf slot this proof takes (03, 04,
/// 06, 09, then this).
pub const PICTURE_AT: Vec3 = [-3.9, 1.3, 0.0];
pub const PICTURE_K: f64 = 0.8;
pub const SLOT: usize = 4;
pub const TICK: f64 = 0.12;
/// 10_3's zoom about the square's top right corner, and how many pieces are
/// drawn once it has been made.
pub const CORNER_ZOOM: f64 = 4.0;
pub const DRAWN_PIECES: u32 = 8;
/// Pieces whose passing of a guess is drawn on the number line, and the
/// band's pieces in 10_5 (the gap halves ten times, then the band is at the
/// edge as far as the picture can say).
pub const BAND_PIECES: u32 = 10;
/// How many pieces a 64-bit float sum takes to reach exactly 1.0 (the
/// programmers' tag at 10_4; a test sums them).
pub const FLOAT_PIECES: u32 = 54;
/// The guesses of 10_4, as written and as numbers (the facts give how many
/// pieces pass each).
pub const GUESSES: [(&str, f64, &str); 2] = [("0.999", 0.999, "thousandth"), ("0.999999", 0.999_999, "millionth")];
/// The words of the card at the end and the programmers' tag.
pub const CARD_SHOWN: &str = "after n pieces, for every n from 1, the gap is the last piece and goes below any size";
pub const CARD_ASSUMED: &str = "each piece is half of what was left";
pub const CARD_REASON: &str = "so it lies inside it";
pub const LOWER_LABEL: &str = "precisely:";
/// How long the "precisely" statement of 10_6 is read, seconds: the listen
/// window that holds the tonic.
pub const PRECISELY_S: f64 = 3.0;
pub const LOWER_TEXT: &str = "after n pieces the sum is below one every time, and above every number below one \
                              eventually. Geometric series";
pub const TAG: &str = "for programmers";
pub const READOUT_FILLED: &str = "filled";
pub const READOUT_GAP: &str = "gap";
pub const PIECE: &str = "piece";

/// The scene point of the square's unit coordinates (x right, y up).
pub fn at(x: f64, y: f64) -> Vec3 {
    v(SQUARE_X + x * SIDE, SQUARE_Y + y * SIDE)
}

/// The square's right edge, where the line's 1 is.
pub fn right() -> f64 {
    SQUARE_X + SIDE
}

/// Where 10_3 and 10_4 zoom: the square's top right corner, and the line's
/// right end.
pub fn corner() -> Vec3 {
    at(1.0, 1.0)
}
pub fn line_end() -> Vec3 {
    v(right(), LINE_Y)
}

/// Piece `k` (from 1) as unit coordinates: x, y, width, height.
pub fn piece_rect(k: u32) -> (f64, f64, f64, f64) {
    let (x, y, w, h) = series::half(k);
    (x.to_f64(), y.to_f64(), w.to_f64(), h.to_f64())
}

/// What is left after `k` pieces, as unit coordinates.
pub fn gap_rect(k: u32) -> (f64, f64, f64, f64) {
    let (x, y, w, h) = series::left_after_halves(k);
    (x.to_f64(), y.to_f64(), w.to_f64(), h.to_f64())
}

/// The total after `k` pieces, as a place on the line (0 to 1).
pub fn filled(k: u32) -> f64 {
    1.0 - 0.5f64.powi(k as i32)
}

/// Where value `value` of the line is when the picture is zoomed by `zoom`
/// about the line's right end.
pub fn line_x(value: f64, zoom: f64) -> f64 {
    right() - (1.0 - value) * SIDE * zoom
}

/// A fraction as a fact prints it (`31/32`, `1/1024`, `1`), in TeX, with the
/// thousands of a long denominator grouped.
pub fn tex_fraction(s: &str) -> String {
    let group = |digits: &str| -> String {
        let n = digits.len();
        let mut out = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (n - i).is_multiple_of(3) {
                out.push_str("{,}");
            }
            out.push(c);
        }
        out
    };
    match s.split_once('/') {
        Some((n, d)) => format!("\\frac{{{}}}{{{}}}", group(n), group(d)),
        None => group(s),
    }
}

/// The last of a fact's comma-separated list.
fn last_of(key: &str) -> fmn::Result<String> {
    let value = fact(key)?;
    value
        .rsplit(',')
        .next()
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("fact {key} is empty")))
}

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    /// The five pieces' areas, as written (`1/2`, `1/4`, …).
    pieces: Vec<String>,
    filled: String,
    gap: String,
    /// How many pieces pass each guess, and the gap that leaves.
    passes: [(u32, String); 2],
}

fn reads() -> fmn::Result<Reads> {
    let n = shown::HALVES;
    let pieces: Vec<String> = fact("F-H.pieces")?.split(',').map(str::to_owned).collect();
    let model: Vec<String> = (1..=n).map(|k| series::area(series::half(k)).to_string()).collect();
    if pieces != model {
        return Err(fail(format!(
            "F-H.pieces is {pieces:?} but the model's pieces are {model:?}"
        )));
    }
    let filled = last_of("F-H.filled")?;
    let gap = fact("F-H.gap")?;
    let left = series::area(series::left_after_halves(n)).to_string();
    if gap != left {
        return Err(fail(format!("F-H.gap is {gap} but the model's gap is {left}")));
    }
    let mut passes = Vec::new();
    for (_, _, name) in GUESSES {
        let k = fact_int(&format!("F-H.pieces_to.{name}"))? as u32;
        passes.push((k, fact(&format!("F-H.gap_at.{name}"))?));
    }
    let [a, b] = <[(u32, String); 2]>::try_from(passes).map_err(|_| fail("two guesses".into()))?;
    Ok(Reads {
        pieces,
        filled,
        gap,
        passes: [a, b],
    })
}

// ------------------------------------------------------------------ pieces

/// One piece of the halved square, with its label.
struct Pc {
    mob: Mob,
    label: Option<Mob>,
}

/// Piece `k` as a flat rectangle of its colour, at the square's scale
/// (`zoom` about `about` when it is drawn after a zoom).
fn piece(stage: &mut Stage<'_>, k: u32, color: Srgb, about: Vec3, zoom: f64) -> Mob {
    let (x, y, w, h) = piece_rect(k);
    let corner_at = zoomed(at(x, y), about, zoom);
    rectangle(
        stage,
        corner_at,
        w * SIDE * zoom,
        h * SIDE * zoom,
        SEAM / 2.0,
        color,
        1.0,
    )
}

/// A piece's label, centred, in the dark of the ground so it reads on blue
/// and on gold, shrunk into the piece.
fn label(kit: &Kit, stage: &mut Stage<'_>, k: u32, area: &str) -> fmn::Result<Mob> {
    let (x, y, w, h) = piece_rect(k);
    let centre = at(x + w / 2.0, y + h / 2.0);
    let big = (w.min(h) * SIDE * 20.0).clamp(18.0, 54.0);
    let m = kit.tex(stage, &tex_fraction(area), big, ground(), &[], centre)?;
    fit_in(stage, m, w * SIDE * 0.8, h * SIDE * 0.8);
    Ok(m)
}

/// The newest piece comes in gold while the one before turns blue.
fn step(stage: &mut Stage<'_>, before: Option<&Pc>, next: &Pc, rt: f64) -> fmn::Result<()> {
    let mut moves: Moves = Vec::new();
    if let Some(p) = before {
        moves.extend(prepared(stage, vec![repaint(p.mob, size(4), 1.0, rt)?])?);
    }
    moves.push(arrive(stage, next.mob, rt)?);
    if let Some(l) = next.label {
        moves.push(arrive(stage, l, rt)?);
    }
    play(stage, moves, 0.0)
}

/// An outline of what is left after `k` pieces, in `dim` (a ghost that
/// carries a question), at zoom `zoom` about `about`.
fn gap_outline(stage: &mut Stage<'_>, k: u32, about: Vec3, zoom: f64) -> Mob {
    let (x, y, w, h) = gap_rect(k);
    let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
        .map(|(cx, cy)| zoomed(at(cx, cy), about, zoom))
        .to_vec();
    stage
        .arena_mut()
        .add(Polygon::new(corners).style(line_style(dim(), 3.0)).build())
}

// the line's numerals clear the card's top edge (-1.6), and the guess cards
// sit between the line and the square's foot
const _: () = assert!(LINE_Y - NUMERAL_DROP - 0.2 > -1.6);
const _: () = assert!(LINE_Y + 0.55 + 0.23 < SQUARE_Y);

/// A rectangle as (left, bottom, right, top) in scene units.
type Rect4 = (f64, f64, f64, f64);

/// 10_3's window: where the magnified corner shows. The zoom is this window
/// opening about the square's top right corner, with the corner's picture
/// magnified inside it, so that nothing of the picture leaves the frame.
pub const WINDOW: Rect4 = (-0.4, -3.1, 6.9, 3.8);

/// `r` cut to `win`, or nothing if none of it is inside.
fn clipped(r: Rect4, win: Rect4) -> Option<Rect4> {
    let cut = (r.0.max(win.0), r.1.max(win.1), r.2.min(win.2), r.3.min(win.3));
    (cut.2 > cut.0 && cut.3 > cut.1).then_some(cut)
}

/// The window when the corner is zoomed by `z`: at 1 the patch of the
/// corner it opens from, at `CORNER_ZOOM` the whole of [`WINDOW`].
pub fn window_at(z: f64) -> Rect4 {
    let c = corner();
    let k = z / CORNER_ZOOM;
    (
        c[0] + k * (WINDOW.0 - c[0]),
        c[1] + k * (WINDOW.1 - c[1]),
        c[0] + k * (WINDOW.2 - c[0]),
        c[1] + k * (WINDOW.3 - c[1]),
    )
}

/// A rectangle of the square (x, y, width, height in unit coordinates) as it
/// is drawn with the corner zoomed by `z`, cut to the window.
pub fn windowed(r: (f64, f64, f64, f64), z: f64) -> Option<Rect4> {
    let c = corner();
    let (lo, hi) = (zoomed(at(r.0, r.1), c, z), zoomed(at(r.0 + r.2, r.1 + r.3), c, z));
    clipped((lo[0], lo[1], hi[0], hi[1]), window_at(z))
}

/// A flat piece of the magnified corner, or nothing if it is out of the
/// window. Its seam grows with the zoom, as the picture does.
fn windowed_piece(stage: &mut Stage<'_>, r: (f64, f64, f64, f64), z: f64, color: Srgb, opacity: f64) -> Option<Mob> {
    let inset = SEAM / 2.0 * z;
    let (x0, y0, x1, y1) = windowed(r, z)?;
    ((x1 - x0) > 2.0 * inset && (y1 - y0) > 2.0 * inset)
        .then(|| rectangle(stage, v(x0, y0), x1 - x0, y1 - y0, inset, color, opacity))
}

/// The square's top and right edges as they are in the window at zoom `z`
/// (its left and bottom edges are outside it).
fn window_outline(stage: &mut Stage<'_>, z: f64) -> Mob {
    let (c, win) = (corner(), window_at(z));
    stage.arena_mut().add(
        polyline(vec![v(win.0, c[1]), c, v(c[0], win.1)])
            .style(line_style(ink(), 3.0))
            .build(),
    )
}

/// The window's own edge.
fn window_frame(stage: &mut Stage<'_>, z: f64) -> Mob {
    let w = window_at(z);
    stage.arena_mut().add(
        Polygon::new(vec![v(w.0, w.1), v(w.2, w.1), v(w.2, w.3), v(w.0, w.3)])
            .style(line_style(faint(), 2.0))
            .build(),
    )
}

/// Shrink a picture by `k` about `from` and put that point at `to`: every
/// mobject of `mobs` (on the screen, not grouped) moves with it, rigidly.
fn relocate(stage: &mut Stage<'_>, mobs: &[Mob], from: Vec3, to: Vec3, k: f64, rt: f64) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for &m in mobs {
        let c = stage.get_center(m);
        let target = v(to[0] + k * (c[0] - from[0]), to[1] + k * (c[1] - from[1]));
        moves.push(fly(m, target, k, rt)?);
    }
    together(stage, moves, 0.0)
}

/// A tick, numeral or cross that slides with a zoom of the line.
#[derive(Clone, Copy)]
struct Mark {
    mob: Mob,
    /// Where it sits when the line is not zoomed.
    home: Vec3,
}

/// The slide that takes a mark from `home` to where the line zoomed by
/// `zoom` puts it. The line is stretched along itself only: a numeral under
/// it or a card above it keeps its height.
pub fn slide_in_zoom(home: Vec3, zoom: f64) -> Vec3 {
    v(slid_by_zoom(home, line_end(), zoom)[0], 0.0)
}

/// The slide that brings a mark home again from a zoom by `zoom`.
pub fn slide_out_zoom(home: Vec3, zoom: f64) -> Vec3 {
    let z = zoomed(home, line_end(), zoom);
    v(home[0] - z[0], 0.0)
}

/// A thin upright tick of `color`, centred at `x` on the line, `half` high.
fn tick(stage: &mut Stage<'_>, x: f64, half: f64, color: Srgb) -> Mob {
    rectangle(stage, v(x - 0.02, LINE_Y - half), 0.04, 2.0 * half, 0.0, color, 1.0)
}

/// A guess on the line: a small card of its number above the point it names,
/// joined to it by a leader, and half the card's width (the cross that
/// strikes the guess goes beside the card, never over its number). One
/// group. Detached.
fn guess_card(kit: &Kit, stage: &mut Stage<'_>, words: &str, x: f64) -> fmn::Result<(Mob, f64)> {
    let y = LINE_Y + 0.55;
    let text = kit.text(stage, words, 26.0, ink(), v(x, y))?;
    let wide = stage.get_width(text) + 0.35;
    let back = Rectangle::new()
        .width(wide)
        .height(0.46)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.0, 1.0))
        .build()?
        .moved_to(v(x, y));
    let back = stage.arena_mut().add(back);
    let leader = stage.arena_mut().add(
        polyline(vec![v(x, y - 0.23), v(x, LINE_Y)])
            .style(line_style(ink(), 1.5))
            .build(),
    );
    // the text is built before the back so it is measured; the back goes
    // under it in the group
    Ok((group(stage, &[back, leader, text])?, wide / 2.0))
}

/// A cross of `warn`, `height` high, centred at `at`. Detached.
fn cross(stage: &mut Stage<'_>, at: Vec3, height: f64) -> Mob {
    stage
        .arena_mut()
        .add(exmark(warn()).with_height(height, false).moved_to(at))
}

/// The picture this card files, rebuilt as it stands on the shelf (the
/// chapters after this one open with the shelf so far): the halved square
/// with its eight pieces and the gap, the number line under it with its
/// ticks, the two guesses struck, the band, and the gold 1, all at
/// thumbnail size in its slot. Detached.
pub fn thumb(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let read = reads()?;
    let c = corner();
    let mut picture: Vec<Mob> = vec![
        stage.arena_mut().add(
            Polygon::new(vec![at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)])
                .style(line_style(ink(), 3.0))
                .build(),
        ),
    ];
    for k in 1..=DRAWN_PIECES {
        picture.push(piece(stage, k, size(4), c, 1.0));
    }
    picture.push(gap_outline(stage, DRAWN_PIECES, c, 1.0));
    // the line, its two ends and numerals, and a tick for every piece up to
    // the one that passes the first guess (that one gold)
    picture.push(
        stage.arena_mut().add(
            polyline(vec![v(SQUARE_X, LINE_Y), line_end()])
                .style(line_style(ink(), 3.0))
                .build(),
        ),
    );
    picture.push(tick(stage, SQUARE_X, 0.2, ink()));
    picture.push(tick(stage, right(), 0.2, ink()));
    picture.push(kit.tex(stage, "0", 34.0, ink(), &[], v(SQUARE_X, LINE_Y - NUMERAL_DROP))?);
    picture.push(kit.tex(stage, "1", 34.0, ink(), &[], v(right(), LINE_Y - NUMERAL_DROP))?);
    let to_first = read.passes[0].0;
    for k in 1..=to_first {
        let colour = if k == to_first { hero() } else { size(4) };
        picture.push(tick(stage, line_x(filled(k), 1.0), TICK, colour));
    }
    // the two guesses, struck where they sit on the line
    for (_, value, _) in GUESSES {
        picture.push(cross(stage, v(line_x(value, 1.0), LINE_Y + 0.3), 0.4 * 0.44));
    }
    // the band: each piece's share of the line, with its cross at the end
    for j in 1..=BAND_PIECES {
        let (from, to) = (filled(j - 1), filled(j));
        picture.push(rectangle(
            stage,
            v(line_x(from, 1.0), LINE_Y - 0.08),
            (to - from) * SIDE,
            0.16,
            0.0,
            warn(),
            0.4,
        ));
        picture.push(cross(stage, v(line_x(to, 1.0), LINE_Y + 0.3), 0.22));
    }
    picture.push(gap_bar(stage, BAND_PIECES));
    // the card of the last guess turned gold: 1
    let gold_text = kit.text(stage, "1", 26.0, hero(), v(right(), LINE_Y + 0.55))?;
    let gold_back = stage.arena_mut().add(
        Rectangle::new()
            .width(0.8)
            .height(0.46)
            .corner_radius(0.1)
            .style(Style::default().fill(panel(), 0.96).stroke(hero(), 2.5, 1.0))
            .build()?
            .moved_to(v(right(), LINE_Y + 0.55)),
    );
    picture.push(group(stage, &[gold_back, gold_text])?);
    super::draw::shelve(stage, &picture, at(0.5, 0.5), SIDE, SLOT);
    Ok(picture)
}

impl SceneConstruct for Halves {
    fn name(&self) -> &str {
        "10_halves"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 10)?;
        let read = reads()?;
        let n = shown::HALVES;

        // ------------------------------------------------ the square, ready
        let outline = stage.arena_mut().add(
            Polygon::new(vec![at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)])
                .style(line_style(ink(), 3.0))
                .build(),
        );
        let one = kit.tex(stage, "1", 90.0, dim(), &[], at(0.5, 0.5))?;
        let mut pcs: Vec<Pc> = Vec::new();
        for k in 1..=n {
            let mob = piece(stage, k, hero(), corner(), 1.0);
            let lab = label(&kit, stage, k, &read.pieces[k as usize - 1])?;
            pcs.push(Pc { mob, label: Some(lab) });
        }
        let first_label = pcs[0].label.ok_or_else(|| fail("piece 1 has no label".into()))?;

        // ---------------------------------------------------------- 10_1
        let line = say(&kit, stage, LINES[0])?;
        line.wait_sentence(stage, 1)?;
        play_with(stage, 0.0, |s| Ok(vec![arrive(s, outline, 0.6)?, arrive(s, one, 0.6)?]))?;
        line.wait_sentence(stage, 2)?;
        let first = pcs[0].mob;
        play_with(stage, 0.0, |s| {
            Ok(vec![
                leave(s, one, 0.4)?,
                arrive(s, first, 0.6)?,
                arrive(s, first_label, 0.6)?,
            ])
        })?;
        line.wait_sentence(stage, 3)?;
        step(stage, Some(&pcs[0]), &pcs[1], 0.6)?;
        line.wait_sentence(stage, 4)?;
        step(stage, Some(&pcs[1]), &pcs[2], 0.6)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 10_2
        let line = say(&kit, stage, LINES[1])?;
        step(stage, Some(&pcs[2]), &pcs[3], 0.5)?;
        step(stage, Some(&pcs[3]), &pcs[4], 0.5)?;
        let readout_x = -5.8;
        let filled_label = kit.text_left(stage, READOUT_FILLED, 34.0, dim(), v(readout_x, FILLED_Y))?;
        let filled_value = kit.tex_left(
            stage,
            &tex_fraction(&read.filled),
            60.0,
            ink(),
            &[],
            v(readout_x + 1.9, FILLED_Y),
        )?;
        let gap_label = kit.text_left(stage, READOUT_GAP, 34.0, dim(), v(readout_x, 0.9))?;
        let gap_value = kit.tex_left(
            stage,
            &tex_fraction(&read.gap),
            60.0,
            ink(),
            &[],
            v(readout_x + 1.9, 0.9),
        )?;
        line.wait_for(stage, "thirty-one out of thirty-two")?;
        play_with(stage, 0.0, |s| {
            Ok(vec![arrive(s, filled_label, 0.5)?, arrive(s, filled_value, 0.5)?])
        })?;
        line.wait_sentence(stage, 1)?;
        let gap_ring = gap_outline(stage, n, corner(), 1.0);
        play_with(stage, 0.0, |s| {
            Ok(vec![
                arrive(s, gap_ring, 0.5)?,
                arrive(s, gap_label, 0.5)?,
                arrive(s, gap_value, 0.5)?,
            ])
        })?;
        line.wait_for(stage, "exactly the size of the last piece")?;
        flash(stage, &[gap_ring, pcs[4].mob], hero(), 1.2, 0.9)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 10_3
        let line = say(&kit, stage, LINES[2])?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("10_3 has no sentences".into()))?;
        let question = question_card(&kit, stage, &words, v(-3.4, QUESTION_Y))?;
        // the labels and readouts have said their piece
        let labels: Vec<Mob> = pcs.iter().filter_map(|p| p.label).collect();
        play_with(stage, 0.0, |s| {
            let mut away: Moves = Vec::new();
            for &l in &labels {
                away.push(leave(s, l, 0.4)?);
            }
            for m in [filled_label, filled_value, gap_label, gap_value, gap_ring] {
                away.push(leave(s, m, 0.4)?);
            }
            away.push(arrive(s, question, 0.5)?);
            Ok(away)
        })?;
        line.wait_end(stage)?;
        // a window opens at the corner and the corner's picture is magnified
        // inside it, so that none of it leaves the frame: the window, with
        // what is in it, is drawn at the corner's own size and grows
        let c = corner();
        let patch = window_at(1.0);
        let mut window: Vec<Mob> = vec![rectangle(
            stage,
            v(patch.0, patch.1),
            patch.2 - patch.0,
            patch.3 - patch.1,
            0.0,
            ground(),
            1.0,
        )];
        let colours = [size(4), size(4), size(4), size(4), hero()];
        let mut win_pieces: Vec<Mob> = Vec::new();
        for k in 1..=n {
            if let Some(m) = windowed_piece(stage, piece_rect(k), 1.0, colours[k as usize - 1], 1.0) {
                win_pieces.push(m);
            }
        }
        // the piece that is gold (the last) is the one the next replaces
        let win_last = *win_pieces.last().ok_or_else(|| fail("no piece in the window".into()))?;
        window.extend(win_pieces.iter().copied());
        window.push(window_outline(stage, 1.0));
        window.push(window_frame(stage, 1.0));
        for &m in &window {
            stage.add_to_scene(m)?;
        }
        let mut grow = Vec::new();
        for &m in &window {
            grow.push(zoom_about(m, CORNER_ZOOM, c, 1.6)?);
        }
        grow.push(repaint(pcs[4].mob, size(4), 1.0, 0.1)?);
        together(stage, grow, 0.0)?;
        // pieces 6 to 8 in the magnified corner, each gold as it comes
        let mut more: Vec<Pc> = Vec::new();
        for k in (n + 1)..=DRAWN_PIECES {
            let m = windowed_piece(stage, piece_rect(k), CORNER_ZOOM, hero(), 1.0)
                .ok_or_else(|| fail(format!("piece {k} is outside the window")))?;
            more.push(Pc { mob: m, label: None });
        }
        let new_gap = gap_outline(stage, DRAWN_PIECES, c, CORNER_ZOOM);
        let seen = Pc {
            mob: win_last,
            label: None,
        };
        let mut before: Option<&Pc> = Some(&seen);
        for p in &more {
            step(stage, before, p, 0.45)?;
            before = Some(p);
        }
        play_with(stage, 0.0, |s| Ok(vec![arrive(s, new_gap, 0.5)?]))?;
        // the prediction's pause carries the halving heard as a rhythm: a
        // soft note on the downbeat and one at each half of what is left,
        // crowding toward the end of the bar and never reaching it (the
        // question is answered in sound before it is answered in words)
        kit.hold(stage)?;
        let wav = sound::halving(QUESTION_HOLD_S)?;
        kit.sonify(stage, QUESTION_HOLD_S, &format!("{}:10_3", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        wait_until(stage, start + QUESTION_HOLD_S)?;

        // ---------------------------------------------------------- 10_4
        let line = say(&kit, stage, LINES[3])?;
        // the window closes back to the corner and gives way to the square,
        // which has the new pieces in the same place at the same size
        let last_piece = more.last().map(|p| p.mob);
        play_with(stage, 0.0, |s| {
            let mut back: Moves = vec![leave(s, question, 0.4)?];
            for &m in window.iter().chain(more.iter().map(|p| &p.mob)).chain([&new_gap]) {
                back.extend(prepared(s, vec![zoom_about(m, 1.0 / CORNER_ZOOM, c, 1.2)?])?);
            }
            // the last piece is blue now that no more are coming
            if let Some(m) = last_piece {
                back.extend(prepared(s, vec![repaint(m, size(4), 1.0, 0.4)?])?);
            }
            Ok(back)
        })?;
        for &m in window.iter().chain(more.iter().map(|p| &p.mob)).chain([&new_gap]) {
            stage.remove_from_scene(m);
        }
        let mut picture: Vec<Mob> = vec![outline];
        picture.extend(pcs.iter().map(|p| p.mob));
        for k in (n + 1)..=DRAWN_PIECES {
            let small = piece(stage, k, size(4), c, 1.0);
            stage.add_to_scene(small)?;
            picture.push(small);
        }
        let small_gap = gap_outline(stage, DRAWN_PIECES, c, 1.0);
        stage.add_to_scene(small_gap)?;
        picture.push(small_gap);

        // the number line: its ends, and a tick for every piece up to the one
        // that passes the first guess
        let to_first = read.passes[0].0;
        let to_second = read.passes[1].0;
        let number_line = stage.arena_mut().add(
            polyline(vec![v(SQUARE_X, LINE_Y), line_end()])
                .style(line_style(ink(), 3.0))
                .build(),
        );
        let zero_tick = tick(stage, SQUARE_X, 0.2, ink());
        let one_tick = tick(stage, right(), 0.2, ink());
        let zero_label = kit.tex(stage, "0", 34.0, ink(), &[], v(SQUARE_X, LINE_Y - NUMERAL_DROP))?;
        let one_label = kit.tex(stage, "1", 34.0, ink(), &[], v(right(), LINE_Y - NUMERAL_DROP))?;
        let mut ticks: Vec<Mark> = Vec::new();
        for k in 1..=to_first {
            let x = line_x(filled(k), 1.0);
            ticks.push(Mark {
                mob: tick(stage, x, TICK, size(4)),
                home: v(x, LINE_Y),
            });
        }
        let tick_mobs: Vec<Mob> = ticks.iter().map(|t| t.mob).collect();
        play_with(stage, 0.0, |s| {
            let mut intro: Moves = vec![
                arrive(s, number_line, 0.6)?,
                arrive(s, zero_tick, 0.6)?,
                arrive(s, one_tick, 0.6)?,
                arrive(s, zero_label, 0.6)?,
                arrive(s, one_label, 0.6)?,
            ];
            for &t in &tick_mobs {
                intro.push(arrive(s, t, 0.6)?);
            }
            Ok(intro)
        })?;
        let zero_mark = Mark {
            mob: zero_label,
            home: v(SQUARE_X, LINE_Y - NUMERAL_DROP),
        };
        let zero_tick_mark = Mark {
            mob: zero_tick,
            home: v(SQUARE_X, LINE_Y),
        };

        // the first guess drops onto the line, then the line is zoomed so the
        // piece that passes it can be told from it
        let zoom = f64::from(1u32 << to_first);
        line.wait_sentence(stage, 1)?;
        let first_x = line_x(GUESSES[0].1, 1.0);
        let (first_card, first_half) = guess_card(&kit, stage, GUESSES[0].0, first_x)?;
        play_with(stage, 0.0, |s| {
            Ok(vec![arrive_sliding(s, first_card, v(0.0, -0.5), 0.5)?])
        })?;
        let first_home = v(first_x, LINE_Y + 0.55);
        line.wait_for(stage, "and ten pieces pass it")?;
        let mut zoom_moves = vec![zoom_about(number_line, zoom, line_end(), 2.4)?];
        for m in ticks.iter().chain([&zero_mark, &zero_tick_mark]) {
            zoom_moves.push(slide(m.mob, slide_in_zoom(m.home, zoom), 2.4)?);
        }
        zoom_moves.push(slide(first_card, slide_in_zoom(first_home, zoom), 2.4)?);
        together(stage, zoom_moves, 0.0)?;
        // what the zoom has carried off the frame leaves the scene (it stays
        // exactly where the zoom put it and comes back with the zoom out),
        // and the stretched line is replaced by the part of it in the frame
        let carried_off: Vec<Mob> = ticks
            .iter()
            .chain([&zero_mark, &zero_tick_mark])
            .filter(|m| zoomed(m.home, line_end(), zoom)[0] < -FRAME_X_RADIUS - 0.5)
            .map(|m| m.mob)
            .collect();
        for &m in &carried_off {
            stage.remove_from_scene(m);
        }
        let visible_line = stage.arena_mut().add(
            polyline(vec![v(-FRAME_X_RADIUS + 0.2, LINE_Y), line_end()])
                .style(line_style(ink(), 3.0))
                .build(),
        );
        stage.remove_from_scene(number_line);
        stage.add_to_scene(visible_line)?;
        // piece 10 is the one that passes it: gold, numbered, and the guess is
        // crossed
        let passing = *ticks.last().ok_or_else(|| fail("no ticks".into()))?;
        let before_it = ticks[ticks.len() - 2];
        let passing_at = zoomed(passing.home, line_end(), zoom);
        let before_at = zoomed(before_it.home, line_end(), zoom);
        let n_passing = kit.text(
            stage,
            &to_first.to_string(),
            24.0,
            hero(),
            v(passing_at[0], LINE_Y - NUMERAL_DROP),
        )?;
        let n_before = kit.text(
            stage,
            &(to_first - 1).to_string(),
            24.0,
            dim(),
            v(before_at[0], LINE_Y - NUMERAL_DROP),
        )?;
        let first_cross = cross(
            stage,
            v(line_x(GUESSES[0].1, zoom) - first_half - CROSS_BESIDE, LINE_Y + 0.55),
            0.4,
        );
        let grown = fmn::animation::grow_from_center(stage.arena_mut(), first_cross, None)?;
        let passing_mob = passing.mob;
        play_with(stage, 0.0, |s| {
            Ok(vec![
                s.prepare(repaint(passing_mob, hero(), 1.0, 0.4)?)?,
                arrive(s, n_passing, 0.4)?,
                arrive(s, n_before, 0.4)?,
                s.prepare(grown.rt(0.4))?,
            ])
        })?;
        line.wait_sentence(stage, 2)?;
        // the second guess is a millionth short: on this scale it is a speck
        // from the edge, so its card hangs from the edge by its leader
        let second_x = line_x(GUESSES[1].1, zoom);
        let (second_card, second_half) = guess_card(&kit, stage, GUESSES[1].0, second_x)?;
        let number_20 = kit.text(
            stage,
            &format!("{PIECE} {to_second}"),
            24.0,
            hero(),
            v(second_x - 1.0, LINE_Y - NUMERAL_DROP),
        )?;
        // the programmers' tag comes with the second guess, which is the
        // sentence's start, and stays until the zoom-out is over: it came
        // after the line's last word and stood a second and a half, for
        // twenty-two words
        let tag_words = format!(
            "in 64-bit floats this sum reaches exactly 1.0 after {FLOAT_PIECES} pieces; the machine cannot see the gap"
        );
        let tag = aside(&kit, stage, TAG, &tag_words, v(ASIDE_AT[0], ASIDE_AT[1] + 0.35))?;
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "10_4", "tag", stage.scene().time().to_f64());
        play_with(stage, 0.0, |s| {
            Ok(vec![
                arrive_sliding(s, second_card, v(0.0, -0.5), 0.5)?,
                arrive(s, number_20, 0.5)?,
                arrive(s, tag, 0.5)?,
            ])
        })?;
        line.wait_for(stage, "twenty pieces pass it")?;
        let second_cross = cross(stage, v(second_x - second_half - CROSS_BESIDE, LINE_Y + 0.55), 0.4);
        let grown = fmn::animation::grow_from_center(stage.arena_mut(), second_cross, None)?;
        play_with(stage, 0.0, |s| Ok(vec![s.prepare(grown.rt(0.4))?]))?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 10_5
        let line = say(&kit, stage, LINES[4])?;
        // the cards go; each guess's cross comes down to sit on the line; the
        // picture zooms back out
        let values = [GUESSES[0].1, GUESSES[1].1];
        let crosses = [first_cross, second_cross];
        play_with(stage, 0.0, |s| {
            let mut settle: Moves = vec![
                leave(s, first_card, 0.4)?,
                leave(s, second_card, 0.4)?,
                leave(s, n_passing, 0.4)?,
                leave(s, n_before, 0.4)?,
                leave(s, number_20, 0.4)?,
            ];
            for (&c, &value) in crosses.iter().zip(&values) {
                let to = v(line_x(value, zoom), LINE_Y + 0.3);
                settle.extend(prepared(s, vec![fly(c, to, 0.44, 0.5)?])?);
            }
            Ok(settle)
        })?;
        stage.remove_from_scene(visible_line);
        stage.add_to_scene(number_line)?;
        for &m in &carried_off {
            stage.add_to_scene(m)?;
        }
        let mut out = vec![zoom_about(number_line, 1.0 / zoom, line_end(), 1.4)?];
        for m in ticks.iter().chain([&zero_mark, &zero_tick_mark]) {
            out.push(slide(m.mob, slide_out_zoom(m.home, zoom), 1.4)?);
        }
        for (&c, &value) in crosses.iter().zip(&values) {
            out.push(slide(c, v(line_x(value, 1.0) - line_x(value, zoom), 0.0), 1.4)?);
        }
        together(stage, out, 0.0)?;
        // the band: each piece's share of the line, half of what was left,
        // laid down with its cross at the right end; what is not yet laid
        // down is the gap
        let band_y = LINE_Y - 0.08;
        let mut band: Vec<(Mob, Mob)> = Vec::new();
        for j in 1..=BAND_PIECES {
            let (from, to) = (filled(j - 1), filled(j));
            let x0 = line_x(from, 1.0);
            let share = rectangle(stage, v(x0, band_y), (to - from) * SIDE, 0.16, 0.0, warn(), 0.4);
            let c = cross(stage, v(line_x(to, 1.0), LINE_Y + 0.3), 0.22);
            band.push((share, c));
        }
        let mut bar = gap_bar(stage, 0);
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "10_4", "tag_out", stage.scene().time().to_f64());
        play_with(stage, 0.0, |s| Ok(vec![arrive(s, bar, 0.3)?, leave(s, tag, 0.4)?]))?;
        line.wait_sentence(stage, 0)?;
        // the halving is laid down across the sentence that says it, each
        // piece at its own pace, the last as the sentence ends (all eight
        // came in the first two seconds and the picture stood still under
        // ten seconds of voice)
        let from = stage.scene().time().to_f64();
        let until = line.sentence(1)?;
        let slot = ((until - from - 0.4) / band.len() as f64).clamp(0.3, 1.2);
        for (j, &(share, c)) in band.iter().enumerate() {
            wait_until(stage, from + slot * j as f64)?;
            #[cfg(test)]
            if j + 1 == band.len() {
                super::draw::quiz_log::record(&kit.chapter, "10_5", "last_piece", stage.scene().time().to_f64());
                super::draw::quiz_log::record(&kit.chapter, "10_5", "sentence_1", until);
            }
            let next_bar = gap_bar(stage, j as u32 + 1);
            let old = bar;
            play_with(stage, 0.0, |s| {
                Ok(vec![
                    arrive(s, share, 0.3)?,
                    arrive(s, c, 0.3)?,
                    leave(s, old, 0.3)?,
                    arrive(s, next_bar, 0.3)?,
                ])
            })?;
            bar = next_bar;
        }
        line.wait_sentence(stage, 1)?;
        let mut flashing = vec![bar];
        flashing.extend(band.iter().map(|&(share, _)| share));
        flash(stage, &flashing, warn(), 1.05, 0.8)?;
        line.wait_sentence(stage, 2)?;
        flash(stage, &[one_tick], hero(), 1.5, 0.8)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 10_6
        let line = say(&kit, stage, LINES[5])?;
        let (one_card, _) = guess_card(&kit, stage, "1", right())?;
        play_with(stage, 0.0, |s| {
            Ok(vec![arrive_sliding(s, one_card, v(0.0, -0.5), 0.5)?])
        })?;
        // it turns gold: the card is replaced by its gold twin
        let gold_text = kit.text(stage, "1", 26.0, hero(), v(right(), LINE_Y + 0.55))?;
        let gold_back = Rectangle::new()
            .width(0.8)
            .height(0.46)
            .corner_radius(0.1)
            .style(Style::default().fill(panel(), 0.96).stroke(hero(), 2.5, 1.0))
            .build()?
            .moved_to(v(right(), LINE_Y + 0.55));
        let gold_back = stage.arena_mut().add(gold_back);
        let gold = group(stage, &[gold_back, gold_text])?;
        swap(stage, &[one_card], &[gold], 0.5)?;
        // everything of the picture on the screen, to go to the shelf
        picture.extend([number_line, zero_tick, one_tick, zero_label, one_label, gold, bar]);
        picture.extend(ticks.iter().map(|t| t.mob));
        picture.extend(crosses);
        for &(share, c) in &band {
            picture.extend([share, c]);
        }
        // the picture makes room: it shrinks and goes to the left, under the
        // shelf, so the card's close-up has the ring's place on the right
        let square_mid = at(0.5, 0.5);
        line.wait_sentence(stage, 1)?;
        relocate(stage, &picture, square_mid, PICTURE_AT, PICTURE_K, 1.0)?;
        let sum = formula(
            &kit,
            stage,
            "\\frac{1}{2} + \\frac{1}{4} + \\frac{1}{8} + \\cdots = 1",
            52.0,
            ink(),
            v(3.4, 2.55),
        )?;
        write_formula(stage, &sum, 1.4)?;
        line.wait_end(stage)?;
        let precisely = lower_third(&kit, stage, LOWER_LABEL, LOWER_TEXT)?;
        play_with(stage, 0.0, |s| Ok(vec![arrive(s, precisely, 0.5)?]))?;
        // the guess nothing passes holds the tonic: the finished picture is a
        // finished chord, in the window where the statement is read
        let wav = sound::tonic_hold(PRECISELY_S)?;
        kit.sonify(stage, PRECISELY_S, &format!("{}:10_6", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        wait_until(stage, start + PRECISELY_S)?;
        play_with(stage, 0.0, |s| Ok(vec![leave(s, precisely, 0.4)?]))?;
        let mut card = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Text(CARD_SHOWN)),
                CardRow::Assumed(Words::Text(CARD_ASSUMED)),
            ],
        )?;
        card.show(stage, 0.6)?;
        pause(stage, 0.8)?;
        // the close-up of the fit the Assumed line names: a piece against half
        // of what was left (what was left outlined, its left half the piece)
        let ring = lens(stage);
        let (lx, ly) = (LENS_AT[0], LENS_AT[1]);
        let left_over = stage.arena_mut().add(
            Polygon::new(vec![
                v(lx - 0.8, ly - 0.45),
                v(lx + 0.8, ly - 0.45),
                v(lx + 0.8, ly + 0.45),
                v(lx - 0.8, ly + 0.45),
            ])
            .style(line_style(dim(), 2.5))
            .build(),
        );
        let the_piece = rectangle(stage, v(lx - 0.8, ly - 0.45), 0.8, 0.9, 0.04, size(4), 1.0);
        play_with(stage, 0.0, |s| {
            Ok(vec![
                arrive(s, ring, 0.5)?,
                arrive(s, left_over, 0.5)?,
                arrive(s, the_piece, 0.5)?,
            ])
        })?;
        card.tick(&kit, stage, 1, CARD_REASON)?;
        let tick = lens_mark(stage, LENS_AT, 0.7, true);
        play_with(stage, 0.0, |s| Ok(vec![arrive(s, tick, 0.3)?]))?;
        pause(stage, 1.2)?;
        // the card has passed: its picture goes to the shelf
        let mut leaving = vec![ring, left_over, the_piece, tick, sum.mob];
        leaving.extend(card.mobs());
        play_with(stage, 0.0, |s| {
            leaving
                .iter()
                .map(|&m| leave(s, m, 0.4))
                .collect::<fmn::Result<Moves>>()
        })?;
        file_on_shelf(stage, &picture, PICTURE_AT, SIDE * PICTURE_K, SLOT, 1.0)?;
        pause(stage, 0.6)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// The dim bar from the `j`-th partial sum to the edge of the line: what is
/// left of the line, which is the gap. Detached.
fn gap_bar(stage: &mut Stage<'_>, j: u32) -> Mob {
    stage.arena_mut().add(
        polyline(vec![v(line_x(filled(j), 1.0), LINE_Y), line_end()])
            .style(line_style(dim(), 9.0))
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::{Q, q};

    /// The chapter speaks exactly the script's 10_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("10_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// What the chapter reads is the model's, and the guesses are what the
    /// script says: ten pieces pass a thousandth short of one (and nine do
    /// not), twenty a millionth short (and nineteen do not).
    #[test]
    fn the_facts_are_the_models_and_the_guesses_are_passed_when_the_script_says() {
        let r = reads().unwrap();
        assert_eq!(r.pieces, ["1/2", "1/4", "1/8", "1/16", "1/32"]);
        assert_eq!((r.filled.as_str(), r.gap.as_str()), ("31/32", "1/32"));
        let bounds = [q(1, 1000), q(1, 1_000_000)];
        let left = |k: u32| series::area(series::left_after_halves(k));
        for (i, bound) in bounds.into_iter().enumerate() {
            let (k, gap) = (r.passes[i].0, &r.passes[i].1);
            assert!(left(k) < bound, "{k} pieces pass the guess");
            assert!(left(k - 1) >= bound, "{} pieces do not", k - 1);
            assert_eq!(*gap, left(k).to_string());
            // the guess on the line is that far short of one
            assert!((1.0 - GUESSES[i].1 - bound.to_f64()).abs() < 1e-12);
        }
        assert_eq!((r.passes[0].0, r.passes[1].0), (10, 20));
    }

    /// The pieces are the halved square: each lies inside what was left
    /// before it, and is half of it; pieces and gap together are the square.
    #[test]
    fn the_pieces_are_halves_of_what_was_left_and_close_the_square() {
        let eps = 1e-12;
        for k in 1..=DRAWN_PIECES + 4 {
            let (x, y, w, h) = piece_rect(k);
            let before = if k == 1 { (0.0, 0.0, 1.0, 1.0) } else { gap_rect(k - 1) };
            assert!(x >= before.0 - eps && y >= before.1 - eps);
            assert!(x + w <= before.0 + before.2 + eps && y + h <= before.1 + before.3 + eps);
            assert!((w * h - before.2 * before.3 / 2.0).abs() < eps, "piece {k} is half");
            // what is left is the rest of what was left
            let left = gap_rect(k);
            assert!((left.2 * left.3 - before.2 * before.3 / 2.0).abs() < eps);
            let total: f64 = (1..=k).map(|i| piece_rect(i).2 * piece_rect(i).3).sum();
            assert!((total + left.2 * left.3 - 1.0).abs() < eps);
        }
        // exact: the pieces and the gap are the square, to the last place
        let mut sum = Q::ZERO;
        for k in 1..=40 {
            sum = sum + series::area(series::half(k));
        }
        assert_eq!(sum + series::area(series::left_after_halves(40)), Q::ONE);
    }

    /// Where things are drawn: the square and the line fit the frame with the
    /// room the card needs, and the band's pieces are end to end up to the
    /// last partial sum.
    #[test]
    fn the_square_the_line_and_the_band_fit_the_frame() {
        let (lo, hi) = (at(0.0, 0.0), at(1.0, 1.0));
        assert!(lo[0] > -FRAME_X_RADIUS && hi[0] < FRAME_X_RADIUS - 0.3);
        assert!(hi[1] < FRAME_Y_RADIUS - 0.2);
        // the line is as long as the square's side
        assert!((line_x(1.0, 1.0) - line_x(0.0, 1.0) - SIDE).abs() < 1e-12);
        // the band's segments are end to end, from 0 to the last partial sum
        let mut edge = 0.0;
        for j in 1..=BAND_PIECES {
            assert!((filled(j - 1) - edge).abs() < 1e-15);
            edge = filled(j);
        }
        assert!((edge - (1.0 - 0.5f64.powi(BAND_PIECES as i32))).abs() < 1e-15);
        // the guess cards sit between the square's foot and the line
    }

    /// The zoom of 10_4 separates what it must: with the line zoomed by
    /// 2^10, the piece that passes the thousandth-short guess lies nearer one
    /// than the guess does, and the piece before it does not; all three are
    /// on the frame.
    #[test]
    fn the_zoom_tells_the_guess_from_the_piece_that_passes_it() {
        let zoom = f64::from(1u32 << 10);
        let guess = line_x(GUESSES[0].1, zoom);
        let tenth = line_x(filled(10), zoom);
        let ninth = line_x(filled(9), zoom);
        assert!(tenth > guess, "piece 10 is past the guess");
        assert!(ninth < guess, "piece 9 is not");
        assert!(tenth - guess > 0.05, "they are {} apart on the stage", tenth - guess);
        for x in [guess, tenth, ninth] {
            assert!(x > -FRAME_X_RADIUS + 0.3 && x < FRAME_X_RADIUS, "{x} is off the frame");
        }
        // a mark slid by the zoom lands where the zoom puts it, and the way
        // back is exact
        let home = v(line_x(filled(10), 1.0), LINE_Y);
        let by = slide_in_zoom(home, zoom);
        assert!((home[0] + by[0] - tenth).abs() < 1e-9);
        let there = zoomed(home, line_end(), zoom);
        assert!((there[0] + slide_out_zoom(home, zoom)[0] - home[0]).abs() < 1e-9);
        // the second guess is the same speck from the edge, whatever the
        // zoom, and still inside the frame at the zoom of 10_4
        let second = line_x(GUESSES[1].1, zoom);
        assert!(right() - second < 0.01 && right() - second > 0.0);
    }

    /// 54 pieces take a 64-bit float to exactly 1.0 and 53 do not.
    #[test]
    fn a_float_sum_reaches_one_after_the_tagged_number_of_pieces() {
        let mut sum = 0.0f64;
        let mut first = None;
        for k in 1..=80u32 {
            sum += 0.5f64.powi(k as i32);
            if sum == 1.0 && first.is_none() {
                first = Some(k);
            }
        }
        assert_eq!(first, Some(FLOAT_PIECES));
    }

    /// Fractions are written as the facts print them, and a long
    /// denominator is grouped.
    #[test]
    fn fractions_are_set_as_mathematics() {
        assert_eq!(tex_fraction("31/32"), "\\frac{31}{32}");
        assert_eq!(tex_fraction("1/1024"), "\\frac{1}{1{,}024}");
        assert_eq!(tex_fraction("1"), "1");
    }

    /// 10_4: the programmers' tag (twenty-two words) comes in with the
    /// second guess, while the line is still being said, and stands at
    /// least four seconds; it came in after the line's last word and stood
    /// a second and a half (P5, the dense read).
    #[test]
    fn the_programmers_tag_stands_long_enough_to_read() {
        let voiced = testing::run_voiced("10_halves").unwrap();
        let events = super::super::draw::quiz_log::take(&voiced.tag, "10_4");
        let at = |k: &str| {
            events
                .iter()
                .find(|e| e.0 == k)
                .unwrap_or_else(|| panic!("no {k}: {events:?}"))
                .1
        };
        let (_, start, len) = voiced.lines.iter().find(|l| l.0 == "10_4").cloned().unwrap();
        let (shown, left) = (at("tag"), at("tag_out"));
        assert!(
            shown >= start && shown < start + len,
            "it comes in at {shown:.2}, the line {start:.2}.."
        );
        assert!(left - shown >= 4.0, "it stands {:.2} s", left - shown);
    }

    /// 10_5: the halving is laid down across the sentence that says it:
    /// the last piece goes down in the sentence's last quarter (all the
    /// pieces came in the first two seconds, and the picture then stood
    /// still under ten seconds of voice).
    #[test]
    fn the_halving_is_laid_down_across_the_sentence_that_says_it() {
        let voiced = testing::run_voiced("10_halves").unwrap();
        let events = super::super::draw::quiz_log::take(&voiced.tag, "10_5");
        let at = |k: &str| {
            events
                .iter()
                .find(|e| e.0 == k)
                .unwrap_or_else(|| panic!("no {k}: {events:?}"))
                .1
        };
        let (_, start, _) = voiced.lines.iter().find(|l| l.0 == "10_5").cloned().unwrap();
        let (last, s1) = (at("last_piece"), at("sentence_1"));
        assert!(
            last >= start + 0.75 * (s1 - start),
            "the last piece goes down at {last:.2}, the sentence ends {s1:.2}"
        );
        assert!(last < s1, "and before the next sentence ({s1:.2})");
    }

    /// The scene constructs end to end (every phrase it waits for is in its
    /// line, every fact it reads is there, every string typesets), the layout
    /// lint of the kit finds nothing, and the chapter is no longer than the
    /// storyboard's budget (85 s).
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("10_halves").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
