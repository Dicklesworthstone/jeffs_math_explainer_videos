//! The thumbnail (one still, 1280×720): the video's first whole proof, the
//! odd numbers as layers (03): five L-shaped layers in the size colours and
//! the next one in gold making the six by six square, beside the sum it
//! proves, under the title.
//!
//! Every tile is `model::layers::layer(k)` and every number is a fact: the
//! layers' counts `F-O.layers`, the next layer `F-O.next`; the square's side
//! is checked against the tiles before anything is drawn.

use explainer_kit::{Kit, v};
use fmn::prelude::*;

use super::draw::{Board, dim, fact_int, fact_ints, fail, hero, ink, size};
use super::glyphs::TITLE;
use crate::model::layers::layer;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The title's size and height, and the widest it may be.
const TITLE_PT: f64 = 120.0;
const TITLE_Y: f64 = 2.75;
const TITLE_MAX_W: f64 = 12.6;
/// The square of layers: its lower-left corner and a tile's side.
const BOARD_AT: Vec3 = [-6.3, -3.6, 0.0];
const CELL: f64 = 0.74;
/// The sum on the right: its two rows' heights, the room they share, its size.
const SUM_X: (f64, f64) = (-0.9, 6.5);
const SUM_ROWS: (f64, f64) = (-0.35, -2.25);
const SUM_PT: f64 = 84.0;
/// The space between two terms of the sum, as a share of the text's height.
const GAP: f64 = 0.35;

/// The terms the picture adds, in its colours: each layer's count, and the
/// next layer in gold; and the side of the square they make, which the
/// tiles must agree with.
pub fn terms() -> fmn::Result<(Vec<(i64, Srgb)>, i64)> {
    let mut terms: Vec<(i64, Srgb)> = fact_ints("F-O.layers")?
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n, size(i + 1)))
        .collect();
    terms.push((fact_int("F-O.next")?, hero()));
    let side = terms.len() as i64;
    let tiles: i64 = (1..=side).map(|k| layer(k).len() as i64).sum();
    let total: i64 = terms.iter().map(|t| t.0).sum();
    if tiles != total || total != side * side {
        return Err(fail(format!(
            "the layers' facts add to {total}, the tiles to {tiles}, the square to {}",
            side * side
        )));
    }
    for (k, &(n, _)) in terms.iter().enumerate() {
        if layer(k as i64 + 1).len() as i64 != n {
            return Err(fail(format!(
                "layer {} of the model is not its fact's count {n}",
                k + 1
            )));
        }
    }
    Ok((terms, side))
}

/// Lay `mobs` out in a row from `left` at height `y`, `gap` apart; returns
/// the row's right end.
fn row(stage: &mut Stage<'_>, mobs: &[Mob], left: f64, y: f64, gap: f64) -> f64 {
    let mut x = left;
    for &m in mobs {
        stage.move_to(m, v(x, y), LEFT);
        x += stage.get_width(m) + gap;
    }
    x - gap
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (terms, side) = terms()?;

        // the square of layers, the last one in gold
        let board = Board::new(BOARD_AT, CELL);
        for (k, &(_, color)) in terms.iter().enumerate() {
            let piece = board.tiles(stage, &layer(k as i64 + 1), color)?;
            stage.add_to_scene(piece)?;
        }

        // the sum it proves: the counts in their layers' colours, then the
        // square's side squared
        let mut top = Vec::new();
        for (i, &(n, color)) in terms.iter().enumerate() {
            if i > 0 {
                top.push(kit.text(stage, "+", SUM_PT, dim(), ORIGIN)?);
            }
            top.push(kit.text(stage, &n.to_string(), SUM_PT, color, ORIGIN)?);
        }
        let bottom = vec![
            kit.text(stage, "=", SUM_PT, dim(), ORIGIN)?,
            kit.tex(stage, &format!("{side}^{{2}}"), SUM_PT * 1.2, ink(), &[], ORIGIN)?,
        ];
        let gap = GAP * stage.get_height(top[0]);
        let room = SUM_X.1 - SUM_X.0;
        let width = row(stage, &top, 0.0, SUM_ROWS.0, gap);
        let scale = (room / width).min(1.0);
        let mut all = top.clone();
        all.extend(&bottom);
        for &m in &all {
            stage.scale(m, scale);
        }
        let gap = gap * scale;
        let width = row(stage, &top, 0.0, SUM_ROWS.0, gap);
        let left = SUM_X.0 + (room - width) / 2.0;
        row(stage, &top, left, SUM_ROWS.0, gap);
        row(stage, &bottom, left, SUM_ROWS.1, gap);
        for m in all {
            stage.add_to_scene(m)?;
        }

        // the title, as wide as the frame allows
        let title = kit.text_styled(stage, TITLE, TITLE_PT, ink(), v(0.0, TITLE_Y), true, false)?;
        let w = stage.get_width(title);
        if w > TITLE_MAX_W {
            stage.scale(title, TITLE_MAX_W / w);
        }
        stage.add_to_scene(title)?;

        // the layout lint looks once (a no-op without one), then one second
        // at the thumbnail's 1 fps: the one captured frame
        kit.lint_here(stage, "thumbnail");
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::Linter;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

    /// The sum on the thumbnail is the picture's: 1 + 3 + 5 + 7 + 9 and the
    /// gold 11 make the six by six square the tiles fill.
    #[test]
    fn the_sum_is_the_square_the_layers_fill() {
        let (terms, side) = terms().unwrap();
        let counts: Vec<i64> = terms.iter().map(|t| t.0).collect();
        assert_eq!(counts, vec![1, 3, 5, 7, 9, 11]);
        assert_eq!(side, 6);
        assert_eq!(counts.iter().sum::<i64>(), side * side);
        assert_eq!(terms.last().unwrap().1, hero(), "the next layer is the gold one");
    }

    /// The square stands inside the frame, under the title, and clear of
    /// the sum's room on the right.
    #[test]
    fn the_square_keeps_to_its_room() {
        let (_, side) = terms().unwrap();
        let (left, bottom) = (BOARD_AT[0], BOARD_AT[1]);
        let (right, top) = (left + side as f64 * CELL, bottom + side as f64 * CELL);
        assert!(left > -FRAME_X_RADIUS + 0.25 && bottom > -FRAME_Y_RADIUS + 0.25);
        assert!(top < TITLE_Y - 0.9, "the square reaches {top:.2}, under the title");
        assert!(right < SUM_X.0 - 0.4, "the square ends at {right:.2}");
    }

    /// The scene runs, and the layout lint (overlaps, edges, strokes through
    /// text) finds nothing on the still.
    #[test]
    fn the_thumbnail_runs_and_its_layout_is_clean() {
        struct Run(Box<dyn SceneConstruct>);
        impl SceneConstruct for Run {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                self.0.construct(stage)
            }
        }
        let lint = Rc::new(Linter::new());
        let kit = Kit::new().unwrap().with_lint(Rc::clone(&lint));
        fmn::run_scene(&mut Run(make(kit)), RuntimeConfig::default(), 1, &mut NullSceneSink)
            .expect("the thumbnail runs");
        assert_eq!(lint.count(), 0, "{}", lint.report());
    }
}
