//! The thumbnail (one still, 1280×720): "ONE TURN" over "SMALLER BOX", and
//! under them, at one scale and on one floor, the five straight tiles in the
//! box of three and the five where they fit in the box 2 + 1/√2, the turned
//! tile gold, with the outline of the box of three around them, so the
//! smaller box shows at a glance. Both packings are the model's
//! (`five_straight`, `five_turned`), each checked to fit before it is drawn;
//! the picture shows a fit and claims no minimum (that is chapters 04 to 07).

use explainer_kit::{Kit, v};
use fmn::prelude::*;

use super::draw::{Board, corner, dim, fail, flat, ink, line_style, path, polygon, turned};
use crate::model::{self, Packing, Surd, Tile};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

// the thumbnail's fixed words (the glyph probe renders chapters only, so
// `glyphs::TEXT` and `glyphs::TEX` list them)
pub const HOOK: [&str; 2] = ["ONE TURN", "SMALLER BOX"];
pub const THREE: &str = "3";
pub const SIDE: &str = r"2 + \frac{1}{\sqrt{2}}";

/// The hook's two lines: where they stand, and their size.
const HOOK_Y: [f64; 2] = [3.2, 2.15];
const HOOK_PT: f64 = 100.0;
/// One scale for both packings: a tile is K scene units wide.
pub const K: f64 = 1.35;
/// Both boxes stand on one floor; the straight box's centre across, and the
/// turned box's.
const FLOOR: f64 = -2.85;
const LEFT_X: f64 = -3.55;
const RIGHT_X: f64 = 3.35;
/// The labels under the boxes: where, and their size.
const LABEL_Y: f64 = -3.4;
const LABEL_PT: f64 = 64.0;
/// The outlines' widths: the boxes', and the box of three around the turned
/// five.
const BOX_W: f64 = 5.0;
const GHOST_W: f64 = 4.0;

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub hook: [Mob; 2],
    /// The straight five and their box; the turned five and theirs (tiles
    /// first, the outline last).
    pub straight: Vec<Mob>,
    pub turned: Vec<Mob>,
    /// The box of three drawn around the turned five, sharing its lower
    /// left corner.
    pub ghost: Mob,
    /// "3" under the straight box, 2 + 1/√2 under the turned one.
    pub labels: [Mob; 2],
}

/// The board that draws a box `side` wide with its centre across at `x`
/// and its floor at [`FLOOR`], a tile [`K`] wide.
pub fn board(x: f64, side: f64) -> Board {
    Board {
        at: [x, FLOOR + K * side / 2.0, 0.0],
        k: K,
    }
}

/// An exact tile as drawn.
fn as_f64(t: &Tile<Surd>) -> Tile<f64> {
    Tile {
        centre: (t.centre.0.to_f64(), t.centre.1.to_f64()),
        cos: t.cos.to_f64(),
        sin: t.sin.to_f64(),
    }
}

/// A packing on `b`: its tiles (straight blue, turned gold), then its box.
/// Detached.
fn packing(stage: &mut Stage<'_>, b: Board, p: &Packing<Surd>) -> Vec<Mob> {
    let side = p.side.to_f64();
    let mut out: Vec<Mob> = p
        .tiles
        .iter()
        .map(|t| {
            let c = if t.sin == Surd::ZERO { corner() } else { turned() };
            polygon(stage, b.tile(side, &as_f64(t)), flat(c, 0.9))
        })
        .collect();
    out.push(path(stage, b.outline(side), line_style(ink(), BOX_W)));
    out
}

/// Build and lay out the thumbnail's pieces, detached, after checking that
/// both packings fit.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let straight = model::five_straight();
    let five = model::five_turned();
    for (what, p) in [("the straight five", &straight), ("the turned five", &five)] {
        let v = p.check_exact();
        if !v.fits() {
            return Err(fail(format!("{what} do not fit: {}", v.summary())));
        }
    }
    let hook = [
        kit.text_styled(stage, HOOK[0], HOOK_PT, turned(), v(0.0, HOOK_Y[0]), true, false)?,
        kit.text_styled(stage, HOOK[1], HOOK_PT, ink(), v(0.0, HOOK_Y[1]), true, false)?,
    ];
    let (three, b) = (straight.side.to_f64(), five.side.to_f64());
    let right = board(RIGHT_X, b);
    let straight_mobs = packing(stage, board(LEFT_X, three), &straight);
    let turned_mobs = packing(stage, right, &five);
    let ghost = path(
        stage,
        [(0.0, 0.0), (three, 0.0), (three, three), (0.0, three), (0.0, 0.0)]
            .iter()
            .map(|&q| right.p(b, q))
            .collect(),
        line_style(dim(), GHOST_W),
    );
    let labels = [
        kit.text_styled(stage, THREE, LABEL_PT, dim(), v(LEFT_X, LABEL_Y), true, false)?,
        kit.tex(stage, SIDE, LABEL_PT * 0.75, turned(), &[], v(RIGHT_X, LABEL_Y))?,
    ];
    Ok(Parts {
        hook,
        straight: straight_mobs,
        turned: turned_mobs,
        ghost,
        labels,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = lay(&kit, stage)?;
        // the ghost box first, so the packing draws over it
        let all = std::iter::once(p.ghost)
            .chain(p.straight)
            .chain(p.turned)
            .chain(p.hook)
            .chain(p.labels);
        for m in all {
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

    /// One scale: a tile is K wide in both packings on screen, both boxes
    /// stand on one floor, and the box of three around the turned five is as
    /// wide as the straight box.
    #[test]
    fn both_packings_share_one_scale_and_one_floor() {
        let (three, b) = (model::five_straight().side.to_f64(), model::five_turned().side.to_f64());
        let (left, right) = (board(LEFT_X, three), board(RIGHT_X, b));
        for (bd, side, p) in [(left, three, model::five_straight()), (right, b, model::five_turned())] {
            for t in &p.tiles {
                let c = bd.tile(side, &as_f64(t));
                let edge = ((c[1][0] - c[0][0]).powi(2) + (c[1][1] - c[0][1]).powi(2)).sqrt();
                let pulled = K * (1.0 - 2.0 * super::super::draw::SEAM);
                assert!((edge - pulled).abs() < 1e-9, "{edge}");
            }
        }
        assert!((left.p(three, (0.0, 0.0))[1] - FLOOR).abs() < 1e-12);
        assert!((right.p(b, (0.0, 0.0))[1] - FLOOR).abs() < 1e-12);
        let ghost_width = right.p(b, (three, 0.0))[0] - right.p(b, (0.0, 0.0))[0];
        let straight_width = left.p(three, (three, 0.0))[0] - left.p(three, (0.0, 0.0))[0];
        assert!((ghost_width - straight_width).abs() < 1e-12);
        assert!(b < three);
    }

    /// Every piece stands inside the frame with room; the hook's two lines
    /// do not touch and stand over the packings; the two packings (with the
    /// box of three) do not touch; the labels sit under their boxes.
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
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
            let mut right_side = p.turned.clone();
            right_side.push(p.ghost);
            for ms in [&p.hook[..], &p.straight[..], &right_side[..], &p.labels[..]] {
                let (l, r, bottom, top) = span(stage, ms);
                assert!(l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3, "{l:.2}..{r:.2}");
                assert!(
                    bottom > -FRAME_Y_RADIUS + 0.2 && top < FRAME_Y_RADIUS - 0.2,
                    "{bottom:.2}..{top:.2}"
                );
            }
            assert!(stage.get_bottom(p.hook[0])[1] > stage.get_top(p.hook[1])[1]);
            let (_, _, hook_foot, _) = span(stage, &p.hook[..]);
            let (left_l, left_r, left_foot, left_top) = span(stage, &p.straight[..]);
            let (right_l, _, right_foot, right_top) = span(stage, &right_side[..]);
            assert!(hook_foot > left_top.max(right_top) + 0.15, "{hook_foot:.2}");
            assert!(right_l > left_r + 0.3, "{left_r:.2} vs {right_l:.2}");
            for (k, (foot, l)) in [(left_foot, left_l), (right_foot, right_l)].into_iter().enumerate() {
                let (ll, _, _, label_top) = span(stage, &p.labels[k..=k]);
                assert!(label_top < foot - 0.05, "label {k} under its box");
                assert!(ll > l - 0.5, "label {k} under its box, not beside it");
            }
            checked = true;
            Ok(())
        });
        assert!(checked);
    }
}
