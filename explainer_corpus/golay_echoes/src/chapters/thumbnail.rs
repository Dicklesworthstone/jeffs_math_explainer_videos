//! The thumbnail (one still, 1280×720): the video's one picture under the
//! title. Pattern A's score row and pattern B's, each with its ghosts, stacked
//! as A + B, and beside them, after "=", the two rows added: one clean echo
//! and nothing at any other shift (01_9). Every row is the model's
//! (`facts::score_a`, `score_b`, `sum`), drawn as chapter 01's blocks: an
//! agreement a green block above the line, a disagreement a red one below.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{Place, STEP, fail, hline};
use super::glyphs::TITLE;
use crate::facts;
use crate::model::Scores;
use crate::palette;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

// the thumbnail's fixed words (the glyph probe renders chapters only, so
// `glyphs::TEXT` lists them)
pub const NAME_A: &str = "A";
pub const NAME_B: &str = "B";
pub const PLUS: &str = "+";
pub const EQUALS: &str = "=";

/// The title's two lines, broken before "One".
pub fn title_lines() -> fmn::Result<(&'static str, &'static str)> {
    let at = TITLE
        .find(" One ")
        .ok_or_else(|| fail("the title has no \" One \" to break at".into()))?;
    Ok((&TITLE[..at], &TITLE[at + 1..]))
}

/// Where the two title lines stand, and their size.
const TITLE_Y: [f64; 2] = [3.05, 1.9];
const TITLE_PT: f64 = 92.0;
/// The charts' size, both ways (chapter 01's blocks are 1.0).
const SCALE: f64 = 1.05;
/// Shift 0 of each chart: A over B on the left, the sum on the right, its
/// middle level with the middle of the two rows.
const LEFT_X: f64 = -3.05;
const RIGHT_X: f64 = 3.7;
const A_Y: f64 = -0.15;
const B_Y: f64 = -3.0;
const SUM_Y: f64 = -2.3;
/// The names left of the two rows, the plus between them, the equals sign
/// between the stack and the sum, both signs in the gap between A's lowest
/// block and B's highest; their sizes.
const NAME_X: f64 = -6.35;
const EQUALS_X: f64 = 0.33;
const SIGN_Y: f64 = -1.13;
const NAME_PT: f64 = 64.0;
const SIGN_PT: f64 = 80.0;
/// The charts' line: thicker than the chapters', for a small screen.
const LINE_W: f64 = 4.0;

/// One chart: its line and its blocks.
pub struct Row {
    pub line: Mob,
    pub blocks: Vec<Mob>,
}

impl Row {
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = vec![self.line];
        all.extend(&self.blocks);
        all
    }
}

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub title: [Mob; 2],
    /// A's row, B's row, then the two added.
    pub rows: [Row; 3],
    pub names: [Mob; 2],
    pub plus: Mob,
    pub equals: Mob,
}

/// The three rows the thumbnail draws: A's, B's, and the two added.
pub fn scores() -> [Scores; 3] {
    [facts::score_a(), facts::score_b(), facts::sum()]
}

/// A chart with shift 0 at (`x`, `y`) over `shifts`.
fn place(x: f64, y: f64, shifts: (i64, i64)) -> Place {
    Place {
        shifts,
        ..Place::blocks(v(x, y), SCALE)
    }
}

/// `scores` as a line and its blocks at `place`, detached: each column is
/// the score's count of blocks, on the side of its sign (the line runs as
/// far past the end columns as a chapter chart's).
fn row(stage: &mut Stage<'_>, place: Place, scores: &Scores) -> fmn::Result<Row> {
    let (first, last) = place.shifts;
    let half = (last - first) as f64 * STEP * place.scale / 2.0 + 0.45 * place.scale;
    let c = place.line_centre();
    let line = stage
        .arena_mut()
        .add(hline(c[0] - half, c[0] + half, c[1], Palette::dim(), LINE_W));
    let mut blocks = Vec::new();
    for k in first..=last {
        let s = scores.at(k);
        for level in 1..=s.unsigned_abs() as usize {
            blocks.push(place.unit(stage, s.signum(), place.slot(k, level, s.signum()))?);
        }
    }
    Ok(Row { line, blocks })
}

/// Build and lay out the thumbnail's pieces, detached.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let (top, bottom) = title_lines()?;
    let line = |stage: &mut Stage<'_>, words: &str, y: f64| {
        kit.text_styled(stage, words, TITLE_PT, Palette::hero(), v(0.0, y), true, false)
    };
    let title = [line(stage, top, TITLE_Y[0])?, line(stage, bottom, TITLE_Y[1])?];

    // every chart spans the shifts A's row scores at, and the others fit
    let [a, b, sum] = scores();
    let shifts = (a.first, a.last());
    for (what, s) in [("B's row", &b), ("the sum", &sum)] {
        if s.first < shifts.0 || s.last() > shifts.1 {
            return Err(fail(format!("{what} runs past A's shifts {shifts:?}")));
        }
    }
    let rows = [
        row(stage, place(LEFT_X, A_Y, shifts), &a)?,
        row(stage, place(LEFT_X, B_Y, shifts), &b)?,
        row(stage, place(RIGHT_X, SUM_Y, shifts), &sum)?,
    ];
    let names = [
        kit.text_styled(
            stage,
            NAME_A,
            NAME_PT,
            palette::pattern_a(),
            v(NAME_X, A_Y),
            true,
            false,
        )?,
        kit.text_styled(
            stage,
            NAME_B,
            NAME_PT,
            palette::pattern_b(),
            v(NAME_X, B_Y),
            true,
            false,
        )?,
    ];
    let plus = kit.text(stage, PLUS, SIGN_PT, Palette::ink(), v(NAME_X, SIGN_Y))?;
    let equals = kit.text(stage, EQUALS, SIGN_PT, Palette::ink(), v(EQUALS_X, SIGN_Y))?;
    Ok(Parts {
        title,
        rows,
        names,
        plus,
        equals,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = lay(&kit, stage)?;
        let rows = p.rows.iter().flat_map(Row::mobs);
        for m in p.title.into_iter().chain(rows).chain(p.names).chain([p.plus, p.equals]) {
            stage.add_to_scene(m)?;
        }
        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

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

    /// The two rows are imperfect, each with ghosts, and add to one clean
    /// echo: the sum the thumbnail draws is A's row plus B's at every shift.
    #[test]
    fn the_rows_are_two_imperfect_ones_adding_to_one_clean_echo() {
        let [a, b, sum] = scores();
        let echo = facts::echo_step(&crate::model::STILL).unwrap();
        for s in [&a, &b] {
            assert!(s.largest_ghost(&[echo]) > 0, "{:?}", s.values);
        }
        for k in a.first - 1..=a.last() + 1 {
            assert_eq!(sum.at(k), a.at(k) + b.at(k), "at {k}");
        }
        assert_eq!(sum.largest_ghost(&[echo]), 0);
        assert!(sum.at(echo) > a.at(echo).max(b.at(echo)));
    }

    /// The title breaks before "One"; every piece stands inside the frame
    /// with room, under the title; A's row over B's, the names and the plus
    /// left of them, the equals sign between the stack and the sum; each
    /// chart holds one block per unit of its scores.
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
        assert_eq!(
            title_lines().unwrap(),
            ("Two Bad Echoes Can Make", "One Perfect Measurement")
        );
        let mut checked = false;
        run(|stage| {
            let kit = Kit::new()?;
            let p = lay(&kit, stage)?;
            let span = |stage: &Stage<'_>, ms: &[Mob]| {
                let l = ms.iter().map(|&m| stage.get_left(m)[0]).fold(f64::MAX, f64::min);
                let r = ms.iter().map(|&m| stage.get_right(m)[0]).fold(f64::MIN, f64::max);
                let b = ms.iter().map(|&m| stage.get_bottom(m)[1]).fold(f64::MAX, f64::min);
                let t = ms.iter().map(|&m| stage.get_top(m)[1]).fold(f64::MIN, f64::max);
                (l, r, b, t)
            };
            for (r, s) in p.rows.iter().zip(scores()) {
                let units: u64 = s.values.iter().map(|v| v.unsigned_abs()).sum();
                assert_eq!(r.blocks.len() as u64, units, "{:?}", s.values);
            }
            let [a, b, sum] = [p.rows[0].mobs(), p.rows[1].mobs(), p.rows[2].mobs()];
            let signs = [p.plus, p.equals];
            for ms in [&p.title[..], &a[..], &b[..], &sum[..], &p.names[..], &signs[..]] {
                let (l, r, bottom, top) = span(stage, ms);
                assert!(l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3, "{l:.2}..{r:.2}");
                assert!(
                    bottom > -FRAME_Y_RADIUS + 0.25 && top < FRAME_Y_RADIUS - 0.25,
                    "{bottom:.2}..{top:.2}"
                );
            }
            let (_, _, title_foot, _) = span(stage, &p.title[..]);
            let (a_left, a_right, a_foot, a_top) = span(stage, &a[..]);
            let (b_left, _, _, b_top) = span(stage, &b[..]);
            let (sum_left, _, _, sum_top) = span(stage, &sum[..]);
            assert!(
                title_foot > a_top.max(sum_top) + 0.2,
                "title foot {title_foot:.2}, the pictures' top {:.2}",
                a_top.max(sum_top)
            );
            // A's row over B's, the plus between them and between the names,
            // the equals sign level with it and inside the sum's height
            assert!(a_foot > b_top + 0.3, "A's foot {a_foot:.2}, B's top {b_top:.2}");
            let (_, _, plus_foot, plus_top) = span(stage, &[p.plus]);
            assert!(
                a_foot > plus_top + 0.1 && plus_foot > b_top + 0.1,
                "{plus_foot:.2}..{plus_top:.2}"
            );
            let (_, _, a_name_foot, _) = span(stage, &p.names[..1]);
            let (_, _, _, b_name_top) = span(stage, &p.names[1..]);
            assert!(a_name_foot > plus_top + 0.1 && plus_foot > b_name_top + 0.1);
            let (_, _, eq_foot, eq_top) = span(stage, &[p.equals]);
            let (_, _, sum_foot, _) = span(stage, &sum[..]);
            assert!(eq_foot > sum_foot && eq_top < sum_top, "{eq_foot:.2}..{eq_top:.2}");
            // the names and the plus left of the rows
            let (_, names_right, _, _) = span(stage, &[p.names[0], p.names[1], p.plus]);
            assert!(names_right + 0.15 < a_left.min(b_left), "{names_right:.2}");
            // the equals sign between the stack and the sum
            let (eq_left, eq_right, _, _) = span(stage, &[p.equals]);
            assert!(eq_left > a_right + 0.15 && eq_right + 0.15 < sum_left);
            // the two title lines do not touch
            assert!(stage.get_bottom(p.title[0])[1] > stage.get_top(p.title[1])[1]);
            checked = true;
            Ok(())
        });
        assert!(checked);
    }
}
