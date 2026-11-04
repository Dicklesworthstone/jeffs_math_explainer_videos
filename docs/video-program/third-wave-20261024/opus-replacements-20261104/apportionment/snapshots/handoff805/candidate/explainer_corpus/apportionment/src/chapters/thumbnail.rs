//! The thumbnail: one more seat, one less for Ash. Ash's seats at 15 and 16
//! come from `model::hamilton`; the lost seat is an empty red outline.
//! Candidate: must be built, rendered and actually looked at before use.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{flat, line_style, town_color};
use crate::model::{self, ASH, POP};

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

struct Thumbnail {
    kit: Kit,
}

/// A big seat token at `at`: filled, or an empty outline.
fn seat(stage: &mut Stage<'_>, at: Vec3, color: Srgb, filled: bool) -> fmn::Result<Mob> {
    let style = if filled {
        flat(color, 0.95)
    } else {
        line_style(color, 6.0, 1.0)
    };
    let r = Rectangle::new()
        .width(0.62)
        .height(0.62)
        .corner_radius(0.1)
        .style(style)
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let fail = |e: String| super::draw::fail(format!("thumbnail: {e}"));
        let before = model::hamilton(&POP, 15).map_err(fail)?;
        let after = model::hamilton(&POP, 16).map_err(fail)?;
        if after[ASH] >= before[ASH] {
            return Err(super::draw::fail(
                "thumbnail: Ash must lose a seat from 15 to 16".into(),
            ));
        }
        for (text, size, color, at) in [
            ("ONE MORE SEAT", 62.0, Palette::ink(), v(-3.0, 1.75)),
            ("ONE LESS?", 88.0, Palette::gold(), v(-3.2, 0.05)),
            ("THE ALABAMA PARADOX", 36.0, Palette::blue(), v(-3.2, -2.0)),
            ("15 SEATS", 36.0, Palette::ink(), v(3.4, 2.3)),
            ("16 SEATS", 36.0, Palette::ink(), v(3.4, -0.25)),
        ] {
            let m = kit.text_styled(stage, text, size, color, at, true, false)?;
            stage.add_to_scene(m)?;
        }
        let ash = town_color(ASH);
        for (row, y) in [(&before, 1.25), (&after, -1.25)] {
            let name = kit.text_styled(stage, "Ash", 34.0, ash, v(1.6, y), true, false)?;
            stage.add_to_scene(name)?;
            for k in 0..before[ASH] {
                let at = v(2.7 + k as f64 * 0.85, y);
                let m = if k < row[ASH] {
                    seat(stage, at, ash, true)?
                } else {
                    seat(stage, at, Palette::red(), false)?
                };
                stage.add_to_scene(m)?;
            }
        }
        stage.wait(1.0)?;
        Ok(())
    }
}
