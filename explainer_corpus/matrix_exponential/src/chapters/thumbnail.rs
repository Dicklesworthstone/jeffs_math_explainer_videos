//! The thumbnail (one still, 1280×720): the video's opening question and
//! its answer, under the title. On the left, `e` raised to the quarter turn
//! `J = [0, −1; 1, 0]` (01_1, the fact `F-J.power.k1`), equal to a gold
//! question mark; on the right, what it is (16_4): the plane's dots inside the
//! gold circle, an arc as long as the radius in the rule's teal, the point it
//! carries 1 to and the spoke to it, a rotation by one radian.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{MatrixGrid, entries, faded_line, fail, flat, lattice, result, rule};
use super::glyphs::TITLE;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The title's two lines, broken before "Raise".
pub fn title_lines() -> fmn::Result<(&'static str, &'static str)> {
    let at = TITLE
        .find(" Raise ")
        .ok_or_else(|| fail("the title has no \" Raise \" to break at".into()))?;
    Ok((&TITLE[..at], &TITLE[at + 1..]))
}

/// Where the two title lines stand, and their size.
const TITLE_Y: [f64; 2] = [3.05, 1.85];
const TITLE_PT: f64 = 96.0;
/// The question's middle (e, its exponent, "=", "?"), and the gaps of its row.
const QUESTION_AT: Vec3 = [-3.2, -1.55, 0.0];
const EXPONENT_GAP: f64 = 0.08;
const EXPONENT_LIFT: f64 = 1.0;
const ROW_GAP: f64 = 0.35;
/// The answer's circle: centre and radius, and the dots' spacing inside it.
const CIRCLE_AT: Vec3 = [4.1, -1.45, 0.0];
const RADIUS: f64 = 1.85;
const DOT_SPACING: f64 = 0.46;

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub title: [Mob; 2],
    pub question: Vec<Mob>,
    pub answer: Vec<Mob>,
}

/// Build and lay out the thumbnail's pieces, detached.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let (top, bottom) = title_lines()?;
    let line = |stage: &mut Stage<'_>, words: &str, y: f64| {
        kit.text_styled(stage, words, TITLE_PT, Palette::hero(), v(0.0, y), true, false)
    };
    let title = [line(stage, top, TITLE_Y[0])?, line(stage, bottom, TITLE_Y[1])?];

    // e^J = ?, laid by measure about QUESTION_AT
    let base_e = kit.tex(stage, "e", 210.0, Palette::ink(), &[], QUESTION_AT)?;
    let power = MatrixGrid::build(kit, stage, &entries("F-J.power.k1", None)?, 70.0, rule(), QUESTION_AT)?;
    let equals = kit.tex(stage, "=", 130.0, Palette::ink(), &[], QUESTION_AT)?;
    let ask = kit.text_styled(stage, "?", 170.0, result(), QUESTION_AT, true, false)?;
    let widths = [
        stage.get_width(base_e),
        stage.get_width(power.group),
        stage.get_width(equals),
        stage.get_width(ask),
    ];
    let total = widths.iter().sum::<f64>() + EXPONENT_GAP + 2.0 * ROW_GAP;
    let y = QUESTION_AT[1];
    let mut x = QUESTION_AT[0] - total / 2.0;
    stage.move_to(base_e, v(x, y), LEFT);
    x += widths[0] + EXPONENT_GAP;
    stage.move_to(power.group, v(x, y + EXPONENT_LIFT), LEFT);
    x += widths[1] + ROW_GAP;
    stage.move_to(equals, v(x, y - 0.15), LEFT);
    x += widths[2] + ROW_GAP;
    stage.move_to(ask, v(x, y), LEFT);
    let question = vec![base_e, power.group, equals, ask];

    // the rotation by one radian
    let mut answer = Vec::new();
    for p in lattice(DOT_SPACING, RADIUS - 0.3) {
        let d = Dot::new()
            .point(v(CIRCLE_AT[0] + p[0], CIRCLE_AT[1] + p[1]))
            .radius(0.045)
            .style(flat(Palette::dim(), 0.9))
            .build();
        answer.push(stage.arena_mut().add(d));
    }
    let circle = Circle::new()
        .radius(RADIUS)
        .arc_center(CIRCLE_AT)
        .style(faded_line(result(), 9.0, 1.0))
        .build();
    answer.push(stage.arena_mut().add(circle));
    let on = |a: f64| v(CIRCLE_AT[0] + RADIUS * a.cos(), CIRCLE_AT[1] + RADIUS * a.sin());
    let arc: Vec<Vec3> = (0..=40).map(|k| on(f64::from(k) / 40.0)).collect();
    answer.push(
        stage
            .arena_mut()
            .add(polyline(arc).style(faded_line(rule(), 16.0, 1.0)).build()),
    );
    for a in [0.0, 1.0] {
        let spoke = polyline(vec![CIRCLE_AT, on(a)])
            .style(faded_line(Palette::ink(), 5.0, 0.85))
            .build();
        answer.push(stage.arena_mut().add(spoke));
    }
    let tip = Dot::new().point(on(1.0)).radius(0.2).style(flat(result(), 1.0)).build();
    answer.push(stage.arena_mut().add(tip));
    Ok(Parts {
        title,
        question,
        answer,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let parts = lay(&kit, stage)?;
        for &m in parts.title.iter().chain(&parts.question).chain(&parts.answer) {
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

    /// The thumbnail's pieces stand inside the frame with room, the title
    /// over the pictures, the question left of the circle, nothing on
    /// anything else; and the exponent is the quarter turn.
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
        assert_eq!(title_lines().unwrap(), ("What Does It Mean to", "Raise e to a Matrix?"));
        assert_eq!(
            super::super::draw::matrix("F-J.power.k1").unwrap(),
            [[0.0, -1.0], [1.0, 0.0]]
        );
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
            for ms in [&p.title[..], &p.question[..], &p.answer[..]] {
                let (l, r, b, t) = span(stage, ms);
                assert!(l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3, "{l:.2}..{r:.2}");
                assert!(
                    b > -FRAME_Y_RADIUS + 0.25 && t < FRAME_Y_RADIUS - 0.25,
                    "{b:.2}..{t:.2}"
                );
            }
            let (_, _, title_foot, _) = span(stage, &p.title[..]);
            let (_, q_right, _, q_top) = span(stage, &p.question[..]);
            let (a_left, _, _, a_top) = span(stage, &p.answer[..]);
            assert!(
                title_foot > q_top.max(a_top) + 0.2,
                "title foot {title_foot:.2}, pictures' top {:.2}",
                q_top.max(a_top)
            );
            assert!(
                a_left > q_right + 0.4,
                "the question ends {q_right:.2}, the circle begins {a_left:.2}"
            );
            // the two title lines do not touch
            assert!(stage.get_bottom(p.title[0])[1] > stage.get_top(p.title[1])[1]);
            Ok(())
        });
    }
}
