//! The thumbnail (one still, 1280×720): the video's first trick and the
//! picture it rests on. On the left, the square of 85 after the one move
//! (01_7, 02): the strip of 5 off its top stands against its side, which
//! leaves a rectangle 80 tall by 90 wide, ruled into its 72 hundreds, and,
//! where the end of the strip stands above it, a small square of 5 by 5 in
//! the corner's orange. On the right, the answer of the quiz the
//! video opens with (01_1), `85² = 7225`, its hundreds in blue and its 25 in
//! orange, and the two products that make it (01_2): `8 × 9 = 72` and
//! `5 × 5 = 25`. Over both, the title's words before its parenthesis. Every
//! number comes from `model.rs` (`ends_in_five`, `one_move`) on the hook's
//! 85 (`facts::shown::HOOK`).

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{Board, MoveFigure, corner, fail, fit_width, hundreds, num, strip};
use super::glyphs::TITLE;
use crate::facts::shown;
use crate::model::{ends_in_five, one_move};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The words of the title the thumbnail carries: the title up to its
/// parenthesis (the upload page shows the whole title beside it).
pub fn words() -> fmn::Result<&'static str> {
    TITLE
        .split_once(" (")
        .map(|(head, _)| head)
        .ok_or_else(|| fail("the title has no parenthesis to stop at".into()))
}

/// The room the picture is fitted into (scene units, wide by tall) and its
/// lower-left corner.
const PICTURE_ROOM: (f64, f64) = (5.4, 4.5);
const PICTURE_AT: (f64, f64) = (-5.95, -2.95);
/// The right column's centre line, and the width its answer may take.
const COLUMN_X: f64 = 3.15;
const COLUMN_W: f64 = 6.6;
/// The title's line, and the width it may take.
const TITLE_Y: f64 = 2.95;
const TITLE_W: f64 = 13.2;

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let hook = shown::HOOK;
        let five = ends_in_five(hook).ok_or_else(|| fail(format!("{hook} does not end in five")))?;
        let m = one_move(hook, hook % 10).ok_or_else(|| fail(format!("no strip comes off a square of {hook}")))?;
        let (blue, orange, ink) = (hundreds(), corner(), Palette::ink());

        // the title's words, across the top
        let title = kit.text_styled(stage, words()?, 76.0, Palette::hero(), v(0.0, TITLE_Y), true, false)?;
        fit_width(stage, title, TITLE_W);
        stage.add_to_scene(title)?;

        // the picture: the square of 85 after the move (01_7, 02): the
        // rectangle it leaves, one piece in the hundreds' blue and ruled into
        // its hundreds (8 rows of 9: the 72), and the end of the strip above
        // it in the corner's orange (the 25)
        let (wide, tall) = (m.wide() as f64, m.side as f64);
        let board = Board::fit(
            v(PICTURE_AT.0, PICTURE_AT.1),
            wide,
            tall,
            (PICTURE_ROOM.0, PICTURE_ROOM.1),
        );
        let figure = MoveFigure::build(stage, board, m, blue, strip(), 1.0)?;
        let grid = board.hundreds(&kit, stage, figure.rectangle_rect(), false)?;
        for piece in [figure.body, figure.merged, grid.lines, figure.corner, figure.corner_cut] {
            stage.add_to_scene(piece)?;
        }
        // its sides, tall by wide, and the corner's area
        let rect = figure.rectangle_rect();
        let side_pt = 54.0;
        let tall_label = kit.tex(stage, &num(m.tall()), side_pt, ink, &[], board.p(0.0, rect[3] / 2.0))?;
        let w = stage.get_width(tall_label);
        stage.shift(tall_label, v(-(w / 2.0 + 0.2), 0.0));
        let wide_label = kit.tex(stage, &num(m.wide()), side_pt, ink, &[], board.p(rect[2] / 2.0, 0.0))?;
        let h = stage.get_height(wide_label);
        stage.shift(wide_label, v(0.0, -(h / 2.0 + 0.18)));
        let c = figure.corner_rect();
        let corner_label = kit.tex(
            stage,
            &num(m.corner()),
            side_pt,
            orange,
            &[],
            board.p(c[0] + c[2] / 2.0, c[1] + c[3]),
        )?;
        let h = stage.get_height(corner_label);
        stage.shift(corner_label, v(0.0, h / 2.0 + 0.15));
        for l in [tall_label, wide_label, corner_label] {
            stage.add_to_scene(l)?;
        }

        // the answer, and the two products that make it
        let square = num(five.square);
        let (lead, tail) = (num(five.hundreds), num(m.corner()));
        if square != format!("{lead}{tail}") {
            return Err(fail(format!("{square} is not {lead} then {tail}")));
        }
        let answer = kit.tex(
            stage,
            &format!(r"{hook}^{{2}} = {square}"),
            150.0,
            ink,
            &[(&lead, blue), (&tail, orange)],
            v(COLUMN_X, 0.75),
        )?;
        fit_width(stage, answer, COLUMN_W);
        stage.add_to_scene(answer)?;
        let products = [
            (
                format!(r"{} \times {} = {lead}", five.n, five.n + 1),
                lead.clone(),
                blue,
                -1.15,
            ),
            (
                format!(r"{s} \times {s} = {tail}", s = m.strip),
                tail.clone(),
                orange,
                -2.55,
            ),
        ];
        for (src, part, color, y) in products {
            let line = kit.tex(stage, &src, 96.0, ink, &[(&part, color)], v(COLUMN_X, y))?;
            stage.add_to_scene(line)?;
        }
        // the layout lint looks here (a thumbnail has no line to look at)
        kit.lint_here(stage, "thumbnail");
        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Kit, Linter};

    use super::super::draw::testing;
    use super::*;

    /// The thumbnail draws, the layout lint finds nothing on it, and what it
    /// says is the model's: the title's words, 85 squared, its two products.
    #[test]
    fn the_thumbnail_says_the_models_numbers_and_the_lint_is_clean() {
        let lint = Rc::new(Linter::new());
        let kit = Kit::new().unwrap().with_lint(Rc::clone(&lint));
        let mut scene = make(kit.clone());
        let (_, frames) = testing::render(scene.as_mut(), "thumbnail").unwrap();
        assert!(frames >= 1, "the thumbnail drew {frames} frames");
        assert_eq!(lint.count(), 0, "{}", lint.report());
        let said: Vec<String> = kit.registered().into_iter().map(|r| r.source).collect();
        let five = ends_in_five(shown::HOOK).unwrap();
        let m = one_move(shown::HOOK, 5).unwrap();
        for want in [
            words().unwrap().to_owned(),
            format!(r"{}^{{2}} = {}", shown::HOOK, five.square),
            format!(r"{} \times {} = {}", five.n, five.n + 1, five.hundreds),
            format!(r"5 \times 5 = {}", m.corner()),
            num(m.tall()),
            num(m.wide()),
        ] {
            assert!(said.contains(&want), "the thumbnail never says {want:?}: {said:?}");
        }
        assert_eq!(words().unwrap(), "Mental Math Tricks That Actually Work");
        assert_eq!((five.square, m.tall(), m.wide()), (7225, 80, 90));
    }
}
