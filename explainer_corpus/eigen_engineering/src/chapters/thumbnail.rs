//! The thumbnail (one still, 1280×720): the three systems of 01_2, each in
//! its own way of moving, under the title. A Pratt truss bent into its
//! softest own pattern (the model's, `Truss::pratt().patterns()`, oriented
//! to sag) over its faint unbent outline; a box of air whose slices stand
//! where its lowest mode puts them (`lowest_air_mode`); and an economy's two
//! goods in the mix that reproduces itself, round after round at the largest
//! eigenvalue (`K-E2.mix`, `K-E2.largest`). Their names are 01_2's.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{faded_line, fail, flat, numbers, own};
use super::glyphs::TITLE;
use super::the_bridge_that_swayed::lowest_air_mode;
use super::truss::{Layout, member_width, outline};
use super::twenty_one_patterns::{deformed, sag_oriented, scaled_to};
use crate::model::truss::Truss;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The title's two lines, broken after "Behind".
pub fn title_lines() -> fmn::Result<(&'static str, &'static str)> {
    TITLE
        .split_once(" Behind ")
        .map(|(head, tail)| (&TITLE[..head.len() + " Behind".len()], tail))
        .ok_or_else(|| fail("the title has no \" Behind \" to break at".into()))
}

/// The title's lines: where they stand, their sizes, and the widest the
/// second may be.
const TITLE_Y: [f64; 2] = [3.0, 1.85];
const TITLE_PT: [f64; 2] = [100.0, 72.0];
const TITLE_ROOM: f64 = 12.8;
/// The three panels' middles across, the pictures' middle height, and the
/// names' height under them.
const PANELS_X: [f64; 3] = [-4.5, 0.0, 4.5];
const PICTURE_Y: f64 = -0.75;
const NAMES_Y: f64 = -2.75;
/// The truss: metres to scene units, and its sag on screen at the softest
/// pattern's largest move.
const TRUSS_SCALE: f64 = 0.17;
const TRUSS_SAG: f64 = 0.35;
/// The box of air and its slices, and how far the middle slice stands from
/// its place at rest, in gaps between slices: far enough that a still shows
/// the air crowded to one side.
const BOX: (f64, f64) = (3.5, 2.3);
const SLICES: usize = 12;
const SLOSH: f64 = 1.6;

/// What the thumbnail is made of, laid out (the scene and the test share it).
pub struct Parts {
    pub title: [Mob; 2],
    /// One group of pieces per panel, and the panels' names.
    pub panels: [Vec<Mob>; 3],
    pub names: [Mob; 3],
}

/// Build and lay out the thumbnail's pieces, detached.
pub fn lay(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Parts> {
    let (ink, dim, gold) = (Palette::ink(), Palette::dim(), own());
    let (top, bottom) = title_lines()?;
    // 01_3's title card: the first line in ink, the second in gold
    let first = kit.text_styled(stage, top, TITLE_PT[0], ink, v(0.0, TITLE_Y[0]), true, false)?;
    let second = kit.text_styled(stage, bottom, TITLE_PT[1], gold, v(0.0, TITLE_Y[1]), true, false)?;
    if stage.get_width(second) > TITLE_ROOM {
        stage.set_width(second, TITLE_ROOM, false);
        stage.move_to(second, v(0.0, TITLE_Y[1]), ORIGIN);
    }

    // a bridge: the Pratt truss in its softest pattern, sagging
    let t = Truss::pratt();
    let span = t.joints.iter().fold(0.0_f64, |m, j| m.max(j.0));
    let height = t.joints.iter().fold(0.0_f64, |m, j| m.max(j.1));
    let layout = Layout {
        origin: v(
            PANELS_X[0] - TRUSS_SCALE * span / 2.0,
            PICTURE_Y - TRUSS_SCALE * height / 2.0 + 0.2,
        ),
        scale: TRUSS_SCALE,
    };
    let softest = sag_oriented(&t, &t.patterns().vectors[0]);
    let field = scaled_to(&softest, 1.0);
    let mut bridge = outline(stage, &t, layout, Palette::faint(), 0.8);
    for (m, (a, b)) in t
        .members
        .iter()
        .zip(deformed(&t, layout, &field, TRUSS_SAG / TRUSS_SCALE))
    {
        let line = polyline(vec![a, b])
            .style(faded_line(gold, member_width(m.kind) + 1.0, 1.0))
            .build();
        bridge.push(stage.arena_mut().add(line));
    }

    // the air in a room: slices of air where its lowest mode puts them
    let mode = lowest_air_mode(SLICES);
    let at = v(PANELS_X[1], PICTURE_Y);
    let frame = Rectangle::new()
        .width(BOX.0)
        .height(BOX.1)
        .corner_radius(0.08)
        .style(faded_line(ink, 4.0, 1.0))
        .build()?
        .moved_to(at);
    let mut air = vec![stage.arena_mut().add(frame)];
    let step = BOX.0 / (SLICES as f64 + 1.0);
    let (y0, y1) = (at[1] - BOX.1 / 2.0 + 0.12, at[1] + BOX.1 / 2.0 - 0.12);
    for (k, &m) in mode.iter().enumerate() {
        let x = at[0] - BOX.0 / 2.0 + step * (k as f64 + 1.0) + SLOSH * step * m;
        air.push(
            stage.arena_mut().add(
                polyline(vec![v(x, y0), v(x, y1)])
                    .style(faded_line(gold, 6.0, 1.0))
                    .build(),
            ),
        );
    }

    // an economy: its two goods in the mix that reproduces itself, three rounds
    let mix = numbers("K-E2.mix")?;
    let largest = numbers("K-E2.largest")?[0];
    let tallest = mix.iter().fold(0.0_f64, |m, x| m.max(*x));
    let floor = PICTURE_Y - BOX.1 / 2.0;
    let mut economy = Vec::new();
    for round in 0..3 {
        for (k, &share) in mix.iter().enumerate() {
            let h = BOX.1 * share / tallest * largest.powi(round);
            let x = PANELS_X[2] - 1.2 + 1.2 * f64::from(round) + 0.42 * k as f64 - 0.21;
            let bar = Rectangle::new()
                .width(0.34)
                .height(h)
                .style(flat(gold, 1.0))
                .build()?
                .moved_to(v(x, floor + h / 2.0));
            economy.push(stage.arena_mut().add(bar));
        }
    }

    let names = [
        kit.text(stage, "a bridge", 36.0, dim, v(PANELS_X[0], NAMES_Y))?,
        kit.text(stage, "the air in a room", 36.0, dim, v(PANELS_X[1], NAMES_Y))?,
        kit.text(stage, "an economy", 36.0, dim, v(PANELS_X[2], NAMES_Y))?,
    ];
    Ok(Parts {
        title: [first, second],
        panels: [bridge, air, economy],
        names,
    })
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = lay(&kit, stage)?;
        let all = p.title.iter().chain(p.panels.iter().flatten()).chain(&p.names);
        for &m in all {
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

    use super::*;

    /// The title breaks after "Behind"; the three pictures stand inside the
    /// frame with room, apart from one another, under the title and over
    /// their names; the slices of air keep their order inside their box; the
    /// truss sags (its middle bottom joint moves down).
    #[test]
    fn the_thumbnail_is_laid_out_inside_the_frame_without_overlap() {
        assert_eq!(
            title_lines().unwrap(),
            ("The One Idea Behind", "Bridges, Concert Halls and Economies")
        );
        let mut checked = false;
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let p = lay(&kit, stage)?;
            let span = |stage: &Stage<'_>, ms: &[Mob]| {
                let l = ms.iter().map(|&m| stage.get_left(m)[0]).fold(f64::MAX, f64::min);
                let r = ms.iter().map(|&m| stage.get_right(m)[0]).fold(f64::MIN, f64::max);
                let b = ms.iter().map(|&m| stage.get_bottom(m)[1]).fold(f64::MAX, f64::min);
                let t = ms.iter().map(|&m| stage.get_top(m)[1]).fold(f64::MIN, f64::max);
                (l, r, b, t)
            };
            let (tl, tr, title_foot, title_top) = span(stage, &p.title);
            assert!(title_top < FRAME_Y_RADIUS - 0.25, "title top {title_top:.2}");
            assert!(
                tl > -FRAME_X_RADIUS + 0.25 && tr < FRAME_X_RADIUS - 0.25,
                "title {tl:.2}..{tr:.2}"
            );
            let mut right_of_last = f64::MIN;
            for (k, panel) in p.panels.iter().enumerate() {
                let (l, r, b, t) = span(stage, panel);
                assert!(
                    l > right_of_last + 0.4,
                    "panel {k} begins {l:.2}, the one before ends {right_of_last:.2}"
                );
                assert!(
                    l > -FRAME_X_RADIUS + 0.3 && r < FRAME_X_RADIUS - 0.3,
                    "panel {k}: {l:.2}..{r:.2}"
                );
                assert!(
                    t < title_foot - 0.25,
                    "panel {k}'s top {t:.2}, the title's foot {title_foot:.2}"
                );
                let name_top = stage.get_top(p.names[k])[1];
                assert!(
                    b > name_top + 0.2,
                    "panel {k}'s foot {b:.2}, its name's top {name_top:.2}"
                );
                assert!(stage.get_bottom(p.names[k])[1] > -FRAME_Y_RADIUS + 0.25);
                right_of_last = r;
            }
            // the slices of air keep their order inside the box, a third of a
            // gap apart at the closest
            let (frame, slices) = p.panels[1].split_first().expect("the box and its slices");
            let (inside, wall) = (stage.get_left(*frame)[0], stage.get_right(*frame)[0]);
            let xs: Vec<f64> = slices.iter().map(|&m| stage.get_center(m)[0]).collect();
            let gap = BOX.0 / (SLICES as f64 + 1.0);
            assert!(
                xs[0] > inside + gap / 3.0 && xs[SLICES - 1] < wall - gap / 3.0,
                "{xs:.2?}"
            );
            assert!(xs.windows(2).all(|w| w[1] - w[0] > gap / 3.0), "{xs:.2?}");
            checked = true;
            Ok(())
        });
        assert!(checked);
        // the softest pattern, oriented to sag, moves the middle down
        let t = Truss::pratt();
        let field = sag_oriented(&t, &t.patterns().vectors[0]);
        let all = t.all_moves(&field);
        let mid = crate::model::truss::PANELS / 2;
        assert!(
            all[2 * mid + 1] < 0.0,
            "the middle bottom joint moves down: {}",
            all[2 * mid + 1]
        );
    }
}
