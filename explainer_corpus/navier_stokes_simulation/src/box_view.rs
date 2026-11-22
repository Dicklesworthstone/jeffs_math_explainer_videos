//! The four-cell box as chapters 08 and 09 draw it: one geometry and one colour contract.
//!
//! Cells follow `four_cell::CELL_AT` (A lower left, B lower right, C upper left, D upper right) and
//! faces follow `four_cell::FACES` (A→B, C→D, A→C, B→D), each pointing from its first cell to its second.
//! A face's line runs from the box centre to a wall; its flow arrow crosses that line along the face's
//! positive direction, one unit of speed spanning `FLOW_SCALE`.
//!
//! Chapters 08 and 09 both draw the box through this module: chapter 08 builds its last picture from
//! it, and chapter 09 rebuilds the same picture from it at t = 0, so the two frames on either side of
//! the cut place every object the same way. Text is made by the caller, through the kit.

use explainer_kit::palette::{Palette, hex};
use fmn::prelude::*;

use crate::four_cell::{CELL_AT, FACES, Q, UNITS};

/// The box centre (the 07_6 placement).
pub const BOX: [f64; 2] = [-3.5, -0.3];
/// One cell's side in scene units.
pub const SIDE: f64 = 2.0;
/// Scene units per unit of face speed: every face arrow uses this one scale.
pub const FLOW_SCALE: f64 = 1.6;
/// How far a face's value label sits from the face's midpoint along its line, toward the box centre
/// (and how far chapter 09's push arrows sit the other way).
pub const LANE: f64 = 0.36;
/// How far a face's value label sits off the face's line, along the face direction (into its "to" cell).
/// Root's estimate-timed preview of chapter 09 (pressure02) showed the gold face highlight, drawn along
/// the line, crossing labels placed on it; root's pressure03 moves only these labels by this amount.
pub const FLOW_LABEL_OFFSET: f64 = 0.32;
pub const FLOW_THICKNESS: f64 = 4.0;
pub const LETTER_SIZE: f64 = 28.0;
pub const WALL_SIZE: f64 = 24.0;
pub const FLOW_LABEL_SIZE: f64 = 26.0;
pub const BADGE_SIZE: f64 = 24.0;
pub const TAG_SIZE: f64 = 22.0;
pub const CARD_WORD_SIZE: f64 = 24.0;
/// The four step cards, left to right (script 08_1; "push" is kept for pressure alone).
pub const CARDS: [&str; 4] = ["carry", "smooth", "forces", "correct"];
/// The card lit at the end of chapter 08 and the start of chapter 09.
pub const CORRECT: usize = 3;
pub const CARD_Y: f64 = 3.45;

pub fn p(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

pub fn cell_center(i: usize) -> [f64; 2] {
    let (c, r) = CELL_AT[i];
    [BOX[0] + (c as f64 - 0.5) * SIDE, BOX[1] + (r as f64 - 0.5) * SIDE]
}

/// (−1 or +1, −1 or +1): which way is out of the box from this cell.
fn outward(i: usize) -> [f64; 2] {
    let (c, r) = CELL_AT[i];
    [2.0 * c as f64 - 1.0, 2.0 * r as f64 - 1.0]
}

/// A point offset from cell `i`'s centre, `dx` and `dy` toward the outside of the box.
pub fn at_cell(i: usize, dx: f64, dy: f64) -> Vec3 {
    let [x, y] = cell_center(i);
    let [ox, oy] = outward(i);
    p(x + dx * ox, y + dy * oy)
}

pub fn face_mid(f: usize) -> [f64; 2] {
    let (a, b) = FACES[f];
    let (ca, cb) = (cell_center(a), cell_center(b));
    [(ca[0] + cb[0]) / 2.0, (ca[1] + cb[1]) / 2.0]
}

/// The face's grey positive direction, a unit vector from its "from" cell to its "to" cell.
pub fn face_dir(f: usize) -> [f64; 2] {
    let (a, b) = FACES[f];
    let (ca, cb) = (cell_center(a), cell_center(b));
    [(cb[0] - ca[0]) / SIDE, (cb[1] - ca[1]) / SIDE]
}

/// The unit vector from the face's midpoint toward the box centre (along the face's line).
fn face_inward(f: usize) -> [f64; 2] {
    let m = face_mid(f);
    [(BOX[0] - m[0]) / (SIDE / 2.0), (BOX[1] - m[1]) / (SIDE / 2.0)]
}

/// A point `lane` toward the box centre from the face's midpoint and `t` along its direction.
pub fn on_face(f: usize, lane: f64, t: f64) -> Vec3 {
    let (m, d, n) = (face_mid(f), face_dir(f), face_inward(f));
    p(m[0] + lane * n[0] + t * d[0], m[1] + lane * n[1] + t * d[1])
}

pub fn push_color() -> Srgb {
    hex("#FF7A47")
}

pub fn flow_color() -> Srgb {
    Palette::blue()
}

/// Badge colour: green for a balanced cell, red otherwise.
pub fn badge_color(value: Q) -> Srgb {
    if value.is_zero() {
        Palette::green()
    } else {
        Palette::red()
    }
}

pub fn letter_at(i: usize) -> Vec3 {
    at_cell(i, 0.7, 0.7)
}

pub fn badge_at(i: usize) -> Vec3 {
    at_cell(i, 0.0, 0.7)
}

/// Face `f`'s value label: beside the face's line, never on it.
pub fn flow_label_at(f: usize) -> Vec3 {
    on_face(f, LANE, FLOW_LABEL_OFFSET)
}

/// The wall flux labels, in the order left, right, bottom, top.
pub fn wall_label_at(k: usize) -> Vec3 {
    let half = SIDE + 0.25;
    let [dx, dy] = [[-half, 0.0], [half, 0.0], [0.0, -half], [0.0, half]][k];
    p(BOX[0] + dx, BOX[1] + dy)
}

/// The fine-print units tag, left edge at the box's upper left (script 08_5; stays until 09_9).
pub fn tag_at() -> Vec3 {
    p(BOX[0] - SIDE, BOX[1] + SIDE + 0.72)
}

pub fn tag_tex() -> String {
    format!(
        "\\text{{box units: cell side }} h = {},\\ \\text{{density }} \\rho = {},\\ \\text{{time step }} \\Delta t = {}",
        UNITS.h, UNITS.rho, UNITS.dt
    )
}

pub fn card_x(k: usize) -> f64 {
    BOX[0] - 2.55 + 1.7 * k as f64
}

pub fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, thickness: f64) -> fmn::Result<Mob> {
    let arrow = Arrow::new(a, b)
        .buff(0.0)
        .thickness(thickness)
        .max_tip_length_to_length_ratio(0.45)
        .color(color)
        .build()?;
    Ok(stage.arena_mut().add(arrow))
}

pub fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(a, b)
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

/// An open rounded frame (badges, matrix outlines).
pub fn frame(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.07)
        .style(Style::default().fill(color, 0.0).stroke(color, width, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

pub fn badge_frame(stage: &mut Stage<'_>, i: usize, color: Srgb) -> fmn::Result<Mob> {
    frame(stage, badge_at(i), 0.66, 0.42, color, 2.2)
}

/// The solid outer wall.
pub fn outline(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(2.0 * SIDE)
            .height(2.0 * SIDE)
            .style(
                Style::default()
                    .fill(Palette::ink(), 0.0)
                    .stroke(Palette::ink(), 5.0, 1.0),
            )
            .build()?
            .moved_to(p(BOX[0], BOX[1])),
    ))
}

/// Face `f`'s imaginary line, from the box centre to the wall.
pub fn dashed_face(stage: &mut Stage<'_>, f: usize) -> fmn::Result<Mob> {
    let dashed = DashedLine::new(on_face(f, SIDE / 2.0, 0.0), on_face(f, -SIDE / 2.0, 0.0))
        .dash_length(0.14)
        .style(Style::default().stroke(Palette::dim(), 1.6, 1.0))
        .build()?;
    Ok(stage.arena_mut().add(dashed))
}

/// The small grey positive-direction arrow near the wall end of face `f`.
pub fn direction_arrow(stage: &mut Stage<'_>, f: usize) -> fmn::Result<Mob> {
    let at = SIDE / 2.0 - 0.22;
    arrow(
        stage,
        on_face(f, -at, -0.17),
        on_face(f, -at, 0.17),
        Palette::dim(),
        1.6,
    )
}

/// The orange pressure point at cell `i`'s centre.
pub fn dot(stage: &mut Stage<'_>, i: usize) -> Mob {
    let [x, y] = cell_center(i);
    stage.arena_mut().add(
        Dot::new()
            .point(p(x, y))
            .radius(0.07)
            .style(Style::default().fill(push_color(), 1.0).stroke(push_color(), 0.0, 1.0))
            .build(),
    )
}

/// A flow arrow for `value` on face `f`'s line, from `start` along the face direction; `None` for zero.
pub fn flow_arrow(stage: &mut Stage<'_>, f: usize, value: Q, start: f64) -> fmn::Result<Option<Mob>> {
    if value.is_zero() {
        return Ok(None);
    }
    let end = start + value.to_f64() * FLOW_SCALE;
    Ok(Some(arrow(
        stage,
        on_face(f, 0.0, start),
        on_face(f, 0.0, end),
        flow_color(),
        FLOW_THICKNESS,
    )?))
}

/// The flow arrow for `value`, centred on face `f`'s midpoint.
pub fn centred_flow(stage: &mut Stage<'_>, f: usize, value: Q) -> fmn::Result<Option<Mob>> {
    flow_arrow(stage, f, value, -value.to_f64() * FLOW_SCALE / 2.0)
}

/// Step card `k`'s rounded rectangle, lit or dim; returns it with the colour its word takes.
pub fn card(stage: &mut Stage<'_>, k: usize, lit: bool) -> fmn::Result<(Mob, Srgb)> {
    let (rim, width) = if lit {
        (Palette::hero(), 2.6)
    } else {
        (Palette::dim(), 1.6)
    };
    let card = stage.arena_mut().add(
        Rectangle::new()
            .width(1.5)
            .height(0.62)
            .corner_radius(0.12)
            .style(
                Style::default()
                    .fill(rim, if lit { 0.12 } else { 0.0 })
                    .stroke(rim, width, 1.0),
            )
            .build()?
            .moved_to(p(card_x(k), CARD_Y)),
    );
    Ok((card, rim))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::four_cell::CELLS;

    fn close(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12)
    }

    #[test]
    fn faces_join_their_cells_and_point_right_or_up() {
        for (f, &(a, b)) in FACES.iter().enumerate() {
            let (ca, cb) = (cell_center(a), cell_center(b));
            let d = face_dir(f);
            assert!(close(&[ca[0] + SIDE * d[0], ca[1] + SIDE * d[1]], &cb), "face {f}");
            assert!(d[0] >= 0.0 && d[1] >= 0.0 && (d[0] + d[1] - 1.0).abs() < 1e-12);
            // The face line runs from the wall (lane −SIDE/2) to the box centre (lane +SIDE/2).
            assert!(close(&on_face(f, SIDE / 2.0, 0.0), &p(BOX[0], BOX[1])));
        }
        assert_eq!(CELLS.len(), CARDS.len());
    }

    #[test]
    fn value_labels_sit_beside_the_face_line_inside_the_to_cell() {
        for (f, &(_, to)) in FACES.iter().enumerate() {
            let (m, d, at) = (face_mid(f), face_dir(f), flow_label_at(f));
            // Distance off the face's line, measured along the face direction (the line's normal).
            let off = (at[0] - m[0]) * d[0] + (at[1] - m[1]) * d[1];
            assert!((off - FLOW_LABEL_OFFSET).abs() < 1e-12, "face {f}");
            // The widest label (-1/4 at FLOW_LABEL_SIZE) reaches about 0.18 toward the line by TeX
            // metrics; even 0.25 would clear a 4-point line (half-width 0.02).
            assert!(off - 0.25 > 0.02, "face {f}");
            let [cx, cy] = cell_center(to);
            assert!(
                (at[0] - cx).abs() < SIDE / 2.0 && (at[1] - cy).abs() < SIDE / 2.0,
                "face {f}"
            );
        }
    }

    #[test]
    fn labels_and_cards_stay_inside_the_frame() {
        // Camera frame 16 × 9 (frame_height 9 at 16:9).
        let inside =
            |q: Vec3, w: f64, h: f64| q[0] - w / 2.0 > -8.0 && q[0] + w / 2.0 < 8.0 && q[1].abs() + h / 2.0 < 4.5;
        for k in 0..CARDS.len() {
            assert!(inside(p(card_x(k), CARD_Y), 1.5, 0.62), "card {k}");
        }
        for i in 0..4 {
            assert!(inside(letter_at(i), 0.3, 0.3) && inside(badge_at(i), 0.66, 0.42));
            assert!(inside(wall_label_at(i), 0.2, 0.3));
        }
    }
}
