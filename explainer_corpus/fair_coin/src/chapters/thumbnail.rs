//! The thumbnail (one still, 1280×720): the video's one picture under the
//! title. The unit square of two tosses of the video's coin (02_4 to 02_7,
//! and 11_3's "opening square at 80% with its gold mirror line"): the two
//! matching blocks thrown away in grey, heads then tails in Alice's green and
//! tails then heads in Bob's violet, mirror images of each other in the gold
//! diagonal; on the right the two pairs and their chances, the same for both
//! (02_6: "Four each. Exactly equal."), from the model's coin (`bent()`).

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{Block, SquarePose, alice, bob, faded_line, fail, flat, gold, line_style, pair_tile, thrown};
use super::glyphs::TITLE;
use crate::model::{Face, PairLaw, bent};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The title's two lines, broken before "Out" as 01_2's title card breaks it.
pub fn title_lines() -> fmn::Result<(&'static str, &'static str)> {
    let at = TITLE
        .find(" Out ")
        .ok_or_else(|| fail("the title has no \" Out \" to break at".into()))?;
    Ok((&TITLE[..at], &TITLE[at + 1..]))
}

/// Where the two title lines stand, and their size.
const TITLE_Y: [f64; 2] = [3.05, 1.9];
const TITLE_PT: f64 = 92.0;
/// The square: its lower-left corner and side.
const SQUARE_AT: Vec3 = [-6.3, -3.5, 0.0];
const SQUARE_SIDE: f64 = 4.6;
/// The two pairs on the right: the tiles' middle across, the chances' middle
/// across, each row's height, the tiles' scale and the chances' size; the
/// words between the rows.
const TILE_X: f64 = 0.9;
const CHANCE_X: f64 = 4.25;
const ROWS_Y: [f64; 2] = [-0.25, -2.65];
const TILE_SCALE: f64 = 1.5;
const CHANCE_PT: f64 = 110.0;
const EQUAL_WORDS: &str = "exactly equal";
const EQUAL_PT: f64 = 40.0;

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub title: [Mob; 2],
    /// HH, HT, TH, TT, then the cuts, the frame and the diagonal.
    pub square: Vec<Mob>,
    /// Each row: the pair's tile (card, then its two discs and letters), and
    /// its chance.
    pub rows: [(Vec<Mob>, Mob); 2],
    pub equal: Mob,
}

/// The square of one coin with chance `p` of heads, at the thumbnail's place.
pub fn pose(p: f64) -> SquarePose {
    SquarePose {
        x: p,
        y_h: p,
        y_t: p,
        left: SQUARE_AT[0],
        bottom: SQUARE_AT[1],
        side: SQUARE_SIDE,
    }
}

/// Build and lay out the thumbnail's pieces, detached.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let (top, bottom) = title_lines()?;
    let line = |stage: &mut Stage<'_>, words: &str, y: f64| {
        kit.text_styled(stage, words, TITLE_PT, Palette::hero(), v(0.0, y), true, false)
    };
    let title = [line(stage, top, TITLE_Y[0])?, line(stage, bottom, TITLE_Y[1])?];

    // the square: four blocks, the matching ones thrown away
    let p = bent();
    let s = pose(p.f64());
    let mut square = Vec::new();
    for (b, color) in [
        (Block::HH, thrown()),
        (Block::HT, alice()),
        (Block::TH, bob()),
        (Block::TT, thrown()),
    ] {
        let (u0, w0, u1, w1) = s.unit_rect(b);
        let corners = [s.point(u0, w0), s.point(u1, w0), s.point(u1, w1), s.point(u0, w1)];
        square.push(
            stage
                .arena_mut()
                .add(Polygon::new(corners).style(flat(color, 1.0)).build()),
        );
    }
    let cut = line_style(explainer_kit::hex("#0E1116"), 5.0);
    for l in [
        [s.point(s.x, 0.0), s.point(s.x, 1.0)],
        [s.point(0.0, s.y_h), s.point(s.x, s.y_h)],
        [s.point(s.x, s.y_t), s.point(1.0, s.y_t)],
    ] {
        square.push(stage.arena_mut().add(polyline(l.to_vec()).style(cut).build()));
    }
    let corners = [
        s.point(0.0, 0.0),
        s.point(1.0, 0.0),
        s.point(1.0, 1.0),
        s.point(0.0, 1.0),
    ];
    square.push(
        stage.arena_mut().add(
            Polygon::new(corners)
                .style(faded_line(Palette::dim(), 2.5, 0.9))
                .build(),
        ),
    );
    let diagonal = DashedLine::new(s.point(0.0, 0.0), s.point(1.0, 1.0))
        .dash_length(0.2)
        .style(line_style(gold(), 5.0))
        .build()
        .map_err(|e| fail(format!("the thumbnail's diagonal: {e}")))?;
    square.push(stage.arena_mut().add(diagonal));

    // the two pairs that decide, and their chances
    let law = PairLaw::same_coin(p);
    let mut rows = Vec::new();
    for ((a, c), color, y) in [
        ((Face::H, Face::T), alice(), ROWS_Y[0]),
        ((Face::T, Face::H), bob(), ROWS_Y[1]),
    ] {
        let tile = pair_tile(kit, stage, (a, c), v(TILE_X, y), TILE_SCALE)?;
        let words = format!("{}%", law.prob(a, c).percent(0));
        let chance = kit.text_styled(stage, &words, CHANCE_PT, color, v(CHANCE_X, y), true, false)?;
        rows.push((tile, chance));
    }
    let middle = (ROWS_Y[0] + ROWS_Y[1]) / 2.0;
    let equal = kit.text(
        stage,
        EQUAL_WORDS,
        EQUAL_PT,
        gold(),
        v((TILE_X + CHANCE_X) / 2.0, middle),
    )?;
    let rows: [(Vec<Mob>, Mob); 2] = rows.try_into().map_err(|_| fail("the thumbnail has two rows".into()))?;
    Ok(Parts {
        title,
        square,
        rows,
        equal,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = lay(&kit, stage)?;
        let rows = p.rows.iter().flat_map(|(tile, chance)| tile.iter().chain([chance]));
        for &m in p.title.iter().chain(&p.square).chain(rows).chain([&p.equal]) {
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

    /// The title breaks before "Out"; the square, the two rows and the
    /// words between them stand inside the frame with room, under the title,
    /// the square left of the rows and the rows apart; the two deciding
    /// blocks are the same size and the two chances read the same.
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
        assert_eq!(
            title_lines().unwrap(),
            ("How to Make a Fair Coin", "Out of an Unfair One")
        );
        let s = pose(bent().f64());
        assert!((s.share(Block::HT) - s.share(Block::TH)).abs() < 1e-12);
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
            let row = |k: usize| {
                let mut ms = p.rows[k].0.clone();
                ms.push(p.rows[k].1);
                ms
            };
            let (row0, row1) = (row(0), row(1));
            for ms in [&p.title[..], &p.square[..], &row0[..], &row1[..]] {
                let (l, r, b, t) = span(stage, ms);
                assert!(l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3, "{l:.2}..{r:.2}");
                assert!(
                    b > -FRAME_Y_RADIUS + 0.25 && t < FRAME_Y_RADIUS - 0.25,
                    "{b:.2}..{t:.2}"
                );
            }
            let (_, _, title_foot, _) = span(stage, &p.title[..]);
            let (_, square_right, _, square_top) = span(stage, &p.square[..]);
            let (row_left, _, row0_foot, row0_top) = span(stage, &row0[..]);
            let (row1_left, _, _, row1_top) = span(stage, &row1[..]);
            assert!(
                title_foot > square_top.max(row0_top) + 0.2,
                "title foot {title_foot:.2}, the pictures' top {:.2}",
                square_top.max(row0_top)
            );
            assert!(
                row_left.min(row1_left) > square_right + 0.4,
                "the square ends {square_right:.2}, the rows begin {:.2}",
                row_left.min(row1_left)
            );
            // the words sit between the rows, clear of both
            let (_, _, eq_foot, eq_top) = span(stage, &[p.equal]);
            assert!(
                row0_foot > eq_top + 0.1 && eq_foot > row1_top + 0.1,
                "{eq_foot:.2}..{eq_top:.2}"
            );
            // each chance stands clear of its tile
            for (tile, chance) in &p.rows {
                let (_, tile_right, _, _) = span(stage, tile);
                assert!(stage.get_left(*chance)[0] > tile_right + 0.4);
            }
            // the two title lines do not touch
            assert!(stage.get_bottom(p.title[0])[1] > stage.get_top(p.title[1])[1]);
            checked = true;
            Ok(())
        });
        assert!(checked);
    }
}
