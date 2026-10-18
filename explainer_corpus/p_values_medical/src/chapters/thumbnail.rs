//! The thumbnail (one still, 1280×720): the title over chapter 01's block of
//! positives, clinic A's 585 people who tested positive in rows of 45, the
//! top two rows gold (the 90 with the disease), the rest blue (the 495 false
//! alarms); beside it the test's false-alarm rate, "5%", and the video's
//! question, "of what?". Every count is the model's (`clinic_a`,
//! `positives_block_of`); no claim about doctors and no p-value is drawn.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{disease, fail, flat, healthy};
use super::glyphs::TITLE;
use crate::facts::BLOCK_COLS;
use crate::model::{PositivesBlock, clinic_a, positives_block_of};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

// the thumbnail's fixed words (the glyph probe renders chapters only, so
// `glyphs::TEXT` lists them)
pub const OF_WHAT: &str = "of what?";

/// The title's two lines, broken before "Even".
pub fn title_lines() -> fmn::Result<(&'static str, &'static str)> {
    let at = TITLE
        .find(" Even ")
        .ok_or_else(|| fail("the title has no \" Even \" to break at".into()))?;
    Ok((&TITLE[..at], &TITLE[at + 1..]))
}

/// Where the two title lines stand, and their size.
const TITLE_Y: [f64; 2] = [3.05, 1.95];
const TITLE_PT: f64 = 92.0;
/// The block: the pitch of its cells, the share of the pitch a cell fills,
/// and the centre of its top-left cell (the block sits under the title, in
/// the left part of the frame).
const CELL: f64 = 0.16;
const CELL_FILL: f64 = 0.8;
const BLOCK_X0: f64 = -6.45;
const BLOCK_Y0: f64 = -0.3;
/// The words right of the block: the rate, big, over the question.
const WORDS_X: f64 = 4.1;
const RATE_Y: f64 = -0.75;
const RATE_PT: f64 = 150.0;
const OF_WHAT_Y: f64 = -2.3;
const OF_WHAT_PT: f64 = 84.0;

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub title: [Mob; 2],
    /// One cell per person who tested positive, row by row from the top.
    pub cells: Vec<Mob>,
    pub rate: Mob,
    pub of_what: Mob,
}

/// The block the thumbnail draws: clinic A's positives in rows of 45 (the
/// block of 01_9).
pub fn block() -> fmn::Result<PositivesBlock> {
    let c = clinic_a().counts().map_err(fail)?;
    positives_block_of(&c, BLOCK_COLS).map_err(fail)
}

/// The words beside it: clinic A's false-alarm rate, "5%".
pub fn rate_words() -> String {
    format!("{}%", clinic_a().false_alarm.percent(0))
}

/// Build and lay out the thumbnail's pieces, detached.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let (top, bottom) = title_lines()?;
    let line = |stage: &mut Stage<'_>, words: &str, y: f64| {
        kit.text_styled(stage, words, TITLE_PT, Palette::hero(), v(0.0, y), true, false)
    };
    let title = [line(stage, top, TITLE_Y[0])?, line(stage, bottom, TITLE_Y[1])?];
    let b = block()?;
    let side = CELL * CELL_FILL;
    let mut cells = Vec::new();
    for row in 0..b.rows {
        let color = if row < b.disease_rows { disease() } else { healthy() };
        for col in 0..b.cols {
            let at = v(BLOCK_X0 + CELL * col as f64, BLOCK_Y0 - CELL * row as f64);
            let cell = Rectangle::new()
                .width(side)
                .height(side)
                .style(flat(color, 1.0))
                .build()?
                .moved_to(at);
            cells.push(stage.arena_mut().add(cell));
        }
    }
    let rate = kit.text_styled(
        stage,
        &rate_words(),
        RATE_PT,
        healthy(),
        v(WORDS_X, RATE_Y),
        true,
        false,
    )?;
    let of_what = kit.text_styled(
        stage,
        OF_WHAT,
        OF_WHAT_PT,
        Palette::ink(),
        v(WORDS_X, OF_WHAT_Y),
        true,
        false,
    )?;
    Ok(Parts {
        title,
        cells,
        rate,
        of_what,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = lay(&kit, stage)?;
        for m in p.title.into_iter().chain(p.cells).chain([p.rate, p.of_what]) {
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
    use crate::facts::facts;

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

    /// The picture is the clinic's facts: 13 rows of 45, the top 2 the 90
    /// people with the disease, the other 11 the 495 false alarms, and the
    /// rate beside it is the test's false-alarm rate, a share of the 9,900
    /// without the disease, not of the positives drawn (whose share with the
    /// disease is 15%).
    #[test]
    fn the_block_and_the_rate_are_the_clinics_facts() {
        let f = facts().unwrap();
        let b = block().unwrap();
        let fact = |key: &str| f.get(key).unwrap().parse::<u64>().unwrap();
        assert_eq!(
            (b.cols, b.rows, b.disease_rows),
            (
                fact("M-F1.block.cols"),
                fact("M-F1.block.rows"),
                fact("M-F1.block.disease_rows")
            )
        );
        assert_eq!(b.disease_rows * b.cols, fact("M-F1.true_pos"));
        assert_eq!((b.rows - b.disease_rows) * b.cols, fact("M-F1.false_pos"));
        assert_eq!(rate_words(), format!("{}%", f.get("M-F1.false_alarm_percent").unwrap()));
        assert_ne!(f.get("M-F1.false_alarm_percent"), f.get("M-F1.ppv_percent"));
        assert_eq!(OF_WHAT, "of what?");
        assert!(!TITLE.contains("Most") && !OF_WHAT.contains("doctor"));
    }

    /// The title breaks before "Even"; every piece stands inside the frame
    /// with room; the block and the words sit under the title, side by side;
    /// the rate stands over the question; one cell per positive.
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
        assert_eq!(title_lines().unwrap(), ("Why P-Values Confuse", "Even Doctors"));
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
            let b = block()?;
            assert_eq!(p.cells.len() as u64, b.rows * b.cols);
            for ms in [&p.title[..], &p.cells[..], &[p.rate][..], &[p.of_what][..]] {
                let (l, r, bottom, top) = span(stage, ms);
                assert!(l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3, "{l:.2}..{r:.2}");
                assert!(
                    bottom > -FRAME_Y_RADIUS + 0.25 && top < FRAME_Y_RADIUS - 0.25,
                    "{bottom:.2}..{top:.2}"
                );
            }
            let (_, _, title_foot, _) = span(stage, &p.title[..]);
            let (_, block_right, _, block_top) = span(stage, &p.cells[..]);
            let (rate_left, _, rate_foot, rate_top) = span(stage, &[p.rate]);
            let (words_left, _, _, question_top) = span(stage, &[p.of_what]);
            assert!(title_foot > block_top.max(rate_top) + 0.2, "{title_foot:.2}");
            assert!(rate_left.min(words_left) > block_right + 0.3, "{block_right:.2}");
            assert!(rate_foot > question_top + 0.1, "{rate_foot:.2} over {question_top:.2}");
            assert!(stage.get_bottom(p.title[0])[1] > stage.get_top(p.title[1])[1]);
            checked = true;
            Ok(())
        });
        assert!(checked);
    }
}
