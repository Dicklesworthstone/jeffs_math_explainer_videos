//! The thumbnail (one still, 1280×720): a bakery ticket beside a slate
//! caught mid-stroke, its nine half rubbed out and its ten half written
//! (01_1), under the title. The ticket's number is the one Blue takes among
//! the slates 0, 3, 0, 0, 2 (03_4, `facts::take_ticket`); the slate's two
//! numbers are Lamport's (`facts::cited`).

use explainer_kit::{Kit, Palette, play, v};
use fmn::prelude::*;

use super::draw::{board, chalk, chalk_number, fail, group, hardware, order_gold};
use super::glyphs as g;
use crate::facts::{cited, take_ticket};
use crate::model::SLATES;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The ticket's width and height, and where it sits.
const TICKET: (f64, f64) = (4.0, 2.8);
const TICKET_AT: (f64, f64) = (-3.7, -1.5);
/// The slate's board, and where it sits.
const SLATE: (f64, f64) = (5.0, 3.5);
const SLATE_AT: (f64, f64) = (3.2, -1.5);
/// The numerals' size, as `chalk_number` takes it (a slate that tall).
const NUMERAL: f64 = 4.8;
/// How far the ten's writing has got: outlined, its chalk part filled in.
const WRITTEN: f64 = 0.78;
/// The rubbed-out nine, fainter than 01_1's so the ten leads in a small picture.
const RUBBED: f64 = 0.22;

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // the title on two lines, broken before "Is"
        let (top, rest) = g::TITLE
            .split_once(" Is ")
            .ok_or_else(|| fail("the title has no \" Is \" to break at".into()))?;
        let bottom = format!("Is {rest}");
        for (words, y) in [(top, 3.05), (bottom.as_str(), 1.85)] {
            let line = kit.text_styled(stage, words, 100.0, Palette::hero(), v(0.0, y), true, false)?;
            stage.add_to_scene(line)?;
        }

        // the ticket Blue takes: a gold stub with its tear line, tilted
        let (w, h) = TICKET;
        let at = v(TICKET_AT.0, TICKET_AT.1);
        let stub = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.32)
            .style(Style::default().fill(board(), 1.0).stroke(order_gold(), 7.0, 1.0))
            .build()?
            .moved_to(at);
        let stub = stage.arena_mut().add(stub);
        let tear_x = at[0] - 0.5 * w + 0.75;
        let tear = DashedLine::new(v(tear_x, at[1] + 0.5 * h - 0.2), v(tear_x, at[1] - 0.5 * h + 0.2))
            .dash_length(0.16)
            .style(Style::default().stroke(order_gold(), 4.0, 0.8))
            .build()?;
        let tear = stage.arena_mut().add(tear);
        let number = take_ticket(&SLATES, 1);
        let num = kit.text_styled(
            stage,
            &number.to_string(),
            90.0 * h,
            order_gold(),
            v(at[0] + 0.375, at[1]),
            true,
            false,
        )?;
        let ticket = group(stage, &[stub, tear, num])?;
        stage.rotate(ticket, 0.12, OUT, None, None);
        stage.add_to_scene(ticket)?;

        // the slate, caught mid-stroke: the nine rubbed down, the ten part
        // written over it (01_1), its strokes thickened for a small picture
        let slate_at = v(SLATE_AT.0, SLATE_AT.1);
        let slab = Rectangle::new()
            .width(SLATE.0)
            .height(SLATE.1)
            .corner_radius(0.12 * SLATE.1)
            .style(Style::default().fill(board(), 1.0).stroke(hardware(), 6.0, 1.0))
            .build()?
            .moved_to(slate_at);
        stage.add(slab)?;
        let nine = chalk_number(&kit, stage, cited::TORN_FROM, slate_at, NUMERAL)?;
        stage.add_to_scene(nine)?;
        stage.set_opacity(nine, RUBBED, true);
        let ten = chalk_number(&kit, stage, cited::TORN_TO, slate_at, NUMERAL)?;
        let mut half = write(stage.arena(), ten);
        half.update_rate_info(Some(1.0), Some(RateFunc::smooth().not_quite_there(WRITTEN)), None);
        play!(stage; half);
        stage.set_stroke(ten, Some(chalk()), Some(6.0), Some(1.0), None, true);

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}
