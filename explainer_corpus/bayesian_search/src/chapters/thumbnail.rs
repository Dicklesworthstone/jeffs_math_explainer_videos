//! The thumbnail (one still, 1280×720): the Navy's map of 1968, as chapter
//! 04 draws it (Richardson and Stone's Figure 2, `y1968::POINTS`: the grid,
//! the blue heat of every cell that holds runs, the dots themselves), with
//! E5, the cell the map liked best, outlined in white, and the ring of the
//! true place in the corner of F6 that touches E5, where the wreck lay
//! (04_6). Beside it, the title in three lines. No number is shown.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{Heat, Lattice, faded_line, line_style, map, true_place};
use crate::model::y1968;

/// The title's three lines, the last in the map's blue.
const TITLE_LINES: [&str; 3] = ["How to Find", "a Lost", "Submarine"];
const TITLE_SIZE: f64 = 120.0;
/// The title's column: its centre's x, its lines' heights, and the widest a
/// line may be.
const TITLE_X: f64 = 3.95;
const TITLE_Y: [f64; 3] = [1.55, 0.05, -1.45];
const TITLE_MAX_W: f64 = 5.6;
/// The map: its centre and a cell's height (a cell is 0.84 of that wide).
const MAP_AT: [f64; 2] = [-3.25, -0.25];
const CELL_H: f64 = 0.42;
/// E5's white outline, and the ring of the true place.
const E5_STROKE: f64 = 5.0;
const RING_R: f64 = 0.12;
const RING_STROKE: f64 = 4.0;
// the title's column ends inside the frame's safe area (x 6.96)
const _: () = assert!(TITLE_X + TITLE_MAX_W / 2.0 < 6.96);

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// A cell of the 1968 map by the paper's name.
fn cell(name: &str) -> fmn::Result<(usize, usize)> {
    y1968::at(name).ok_or_else(|| super::draw::fail(format!("no cell {name} on the 1968 map")))
}

/// Where the ring of the true place goes: in F6, at the corner it shares
/// with E5, pulled into F6 by a ring and a half so it is inside the cell.
fn ring_at(map_1968: &Lattice) -> fmn::Result<Vec3> {
    let (er, ec) = cell("E5")?;
    let (fr, fc) = cell("F6")?;
    // F6 lies east (a column on) and south (a row on) of E5: their shared
    // corner is F6's north-west one
    if fc != ec + 1 || fr != er + 1 {
        return Err(super::draw::fail("F6 is not E5's south-east neighbour".into()));
    }
    let (west, _, _, north) = map_1968.cell_rect(fr, fc);
    let pull = 1.5 * RING_R;
    Ok(v(west + pull, north - pull))
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // the map of 1968: lines, heat, dots
        let map_1968 = Lattice::build(stage, v(MAP_AT[0], MAP_AT[1]), CELL_H, &[], Heat::LATTICE)?;
        for m in map_1968.mobs() {
            stage.add_to_scene(m)?;
        }

        // E5 outlined, the true place's ring in F6
        let (er, ec) = cell("E5")?;
        let (x0, y0, x1, y1) = map_1968.cell_rect(er, ec);
        let e5 = Rectangle::new()
            .width(x1 - x0)
            .height(y1 - y0)
            .style(faded_line(Palette::ink(), E5_STROKE, 1.0))
            .build()?
            .moved_to(v((x0 + x1) / 2.0, (y0 + y1) / 2.0));
        stage.add(e5)?;
        let ring = Circle::new()
            .radius(RING_R)
            .style(line_style(true_place(), RING_STROKE))
            .build()
            .moved_to(ring_at(&map_1968)?);
        stage.add(ring)?;

        // the title, its lines the same size
        let mut lines = Vec::new();
        for (i, (s, y)) in TITLE_LINES.into_iter().zip(TITLE_Y).enumerate() {
            let color = if i == TITLE_LINES.len() - 1 {
                map()
            } else {
                Palette::ink()
            };
            lines.push(kit.text_styled(stage, s, TITLE_SIZE, color, v(TITLE_X, y), true, false)?);
        }
        let widest = lines.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
        for m in lines {
            if widest > TITLE_MAX_W {
                stage.scale(m, TITLE_MAX_W / widest);
            }
            stage.add_to_scene(m)?;
        }

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{LAT_COLS, LAT_ROWS};
    use super::super::glyphs::TITLE;
    use super::*;

    /// The title's three lines are the title, word for word.
    #[test]
    fn the_three_lines_are_the_title() {
        assert_eq!(TITLE_LINES.join(" "), TITLE);
    }

    /// The picture's story holds on the printed map: E5 is the cell with
    /// the most runs, and F6, where the wreck was found, holds runs (the
    /// map did not rule it out) but far fewer.
    #[test]
    fn e5_is_the_map_s_favourite_and_f6_its_neighbour_with_fewer() {
        let most = y1968::cells().max_by_key(|&(_, _, n)| n).expect("cells");
        assert_eq!(y1968::name(most.0, most.1), "E5");
        let f6 = y1968::points("F6").expect("F6");
        assert!(f6 > 0 && f6 < most.2, "F6 holds {f6}");
    }

    /// The map stands inside the frame's safe area, left of the title's
    /// column, and the ring sits inside F6.
    #[test]
    fn the_map_fits_beside_the_title_and_the_ring_is_in_f6() {
        // the frame less its 0.15 margin (the layout lint's safe area)
        const SAFE: [f64; 2] = [6.96, 3.85];
        let cell_w = CELL_H * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64();
        let (w, h) = (LAT_COLS as f64 * cell_w, LAT_ROWS as f64 * CELL_H);
        let (left, right) = (MAP_AT[0] - w / 2.0, MAP_AT[0] + w / 2.0);
        let (bottom, top) = (MAP_AT[1] - h / 2.0, MAP_AT[1] + h / 2.0);
        assert!(
            left > -SAFE[0] && bottom > -SAFE[1] && top < SAFE[1],
            "{left} {bottom} {top}"
        );
        let title_left = TITLE_X - TITLE_MAX_W / 2.0;
        assert!(
            right < title_left - 0.2,
            "the map ends at {right:.2}, the title begins at {title_left:.2}"
        );
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let mut inside = None;
        fmn::run_scene(
            &mut Probe(|stage: &mut Stage<'_>| {
                let map_1968 = Lattice::build(stage, v(MAP_AT[0], MAP_AT[1]), CELL_H, &[], Heat::LATTICE)?;
                let at = ring_at(&map_1968)?;
                let (fr, fc) = cell("F6")?;
                let (x0, y0, x1, y1) = map_1968.cell_rect(fr, fc);
                inside = Some(at[0] - RING_R > x0 && at[0] + RING_R < x1 && at[1] - RING_R > y0 && at[1] + RING_R < y1);
                Ok(())
            }),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the probe scene runs");
        assert_eq!(inside, Some(true), "the ring leaves F6");
    }
}
