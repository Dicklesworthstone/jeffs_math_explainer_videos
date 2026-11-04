//! 01 One more seat: three towns, fifteen seats split 2, 3, 10; a sixteenth
//! seat; Ash drops to one. Every count comes from `model::hamilton`.

use explainer_kit::{Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    self, ROW_Y, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, grid, move_all, name, pause, refill,
    say, think_here, token, token_at, token_spot, town_color, wipe,
};
use crate::model::{self, ASH, BIRCH, CEDAR, POP, grouped};

pub struct OneMoreSeat {
    pub kit: Kit,
}

/// Population bars: 26,000 people span this many scene units.
const POP_SCALE: f64 = 5.6 / 26_000.0;

impl SceneConstruct for OneMoreSeat {
    fn name(&self) -> &str {
        "01_one_more_seat"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let before = model::hamilton(&POP, 15).map_err(fail)?;
        let after = model::hamilton(&POP, 16).map_err(fail)?;

        // 01_1 three towns and their people
        let s = say(&kit, stage, "01_1")?;
        let header = kit.text(stage, "15 seats", 34.0, Palette::ink(), v(-1.2, 3.3))?;
        let mut names = Vec::new();
        for (i, &y) in ROW_Y.iter().enumerate() {
            names.extend(name(&kit, stage, i, y, "")?);
        }
        fade_in_all(stage, &[header], 0.6)?;
        s.wait_for(stage, "Ash has")?;
        let mut bars = Vec::new();
        let mut counts = Vec::new();
        for i in 0..3 {
            let w = POP[i] as f64 * POP_SCALE;
            let bar = rect(v(draw::GRID_X, ROW_Y[i] - 0.18), w, 0.36, flat(town_color(i), 0.75))?;
            bars.push(stage.arena_mut().add(bar));
            let label = grouped(u128::from(POP[i]));
            counts.push(kit.text_left(stage, &label, 24.0, town_color(i), v(draw::GRID_X + w + 0.2, ROW_Y[i]))?);
        }
        let mut row_mobs = Vec::new();
        for i in 0..3 {
            row_mobs.extend([names[i], bars[i], counts[i]]);
        }
        fade_in_lagged(stage, &row_mobs, 2.4, 0.3)?;

        // 01_2 the careful rule's answer: tokens drop in, town by town
        let s = say(&kit, stage, "01_2")?;
        let mut grids = Vec::new();
        for y in ROW_Y {
            grids.extend(grid(stage, y, 12)?);
        }
        let dim: Vec<(Mob, Srgb, f64)> = (0..3).map(|i| (bars[i], town_color(i), 0.12)).collect();
        refill(stage, &dim, 0.6)?;
        fade_out_all(stage, &counts, 0.4)?;
        fade_in_all(stage, &grids, 0.5)?;
        s.wait_for(stage, "Ash gets two")?;
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for (i, phrase) in [
            (ASH, "Ash gets two"),
            (BIRCH, "Birch gets three"),
            (CEDAR, "Cedar gets ten"),
        ] {
            s.wait_for(stage, phrase)?;
            let row = draw::tokens(stage, ROW_Y[i], 0, before[i] as usize, town_color(i))?;
            fade_in_lagged(stage, &row, 0.25 * row.len() as f64 + 0.3, 0.5)?;
            toks.push(row);
        }

        // 01_3 one more seat
        let s = say(&kit, stage, "01_3")?;
        let extra = token_at(stage, v(3.2, 2.7), Palette::hero())?;
        let plus = kit.text(stage, "+1", 30.0, Palette::hero(), v(3.85, 2.7))?;
        fade_in_all(stage, &[extra, plus], 0.5)?;
        let header16 = kit.text(stage, "16 seats", 34.0, Palette::ink(), v(-1.2, 3.3))?;
        draw::morph_all(stage, &[(header, header16)], 0.6)?;
        s.wait_for(stage, "What happens to Ash?")?;
        flash_all(stage, &[names[ASH]], 0.8)?;
        let q = think_here(&kit, stage)?;

        // 01_4 Ash drops to one, Birch and Cedar gain
        say(&kit, stage, "01_4")?;
        fade_out_all(stage, &[q, plus], 0.3)?;
        let lost = toks[ASH][1];
        refill(stage, &[(lost, Palette::red(), 1.0)], 0.4)?;
        move_all(stage, &[(lost, v(draw::seat_x(1.5), ROW_Y[ASH] + 1.0))], 0.6)?;
        fade_out_all(stage, &[lost], 0.3)?;
        let to_birch = token_spot(ROW_Y[BIRCH], before[BIRCH] as usize);
        move_all(stage, &[(extra, to_birch)], 0.7)?;
        refill(stage, &[(extra, town_color(BIRCH), 0.95)], 0.3)?;
        let cedar_new = token(stage, ROW_Y[CEDAR], before[CEDAR] as usize, town_color(CEDAR))?;
        fade_in_all(stage, &[cedar_new], 0.4)?;
        // the choreography above (Ash loses its second token, Birch and Cedar
        // gain one each) must be what the model says
        let gains: Vec<i64> = (0..3).map(|i| after[i] as i64 - before[i] as i64).collect();
        if gains != [-1, 1, 1] {
            return Err(fail(format!(
                "01_4 choreography expects gains -1, +1, +1; the model gives {gains:?}"
            )));
        }
        let note = kit.text_left(
            stage,
            &format!("{} → {}", before[ASH], after[ASH]),
            30.0,
            Palette::red(),
            v(draw::READ_X, ROW_Y[ASH]),
        )?;
        fade_in_all(stage, &[note], 0.4)?;
        flash_all(stage, &[names[ASH], note], 0.9)?;
        pause(stage, 0.8)?;

        // 01_5 the paradox has a name; the real question
        wipe(&kit, stage)?;
        say(&kit, stage, "01_5")?;
        let title = kit.text_styled(stage, "Adding a Seat", 54.0, Palette::ink(), v(0.0, 2.4), true, false)?;
        let line15 = format!(
            "15 seats:   Ash {}   Birch {}   Cedar {}",
            before[ASH], before[BIRCH], before[CEDAR]
        );
        let line16 = format!(
            "16 seats:   Ash {}   Birch {}   Cedar {}",
            after[ASH], after[BIRCH], after[CEDAR]
        );
        let a = kit.text_t2c(
            stage,
            &line15,
            28.0,
            Palette::ink(),
            &[
                ("Ash", town_color(ASH)),
                ("Birch", town_color(BIRCH)),
                ("Cedar", town_color(CEDAR)),
            ],
            v(0.0, 1.0),
        )?;
        let b = kit.text_t2c(
            stage,
            &line16,
            28.0,
            Palette::ink(),
            &[
                ("Ash", town_color(ASH)),
                ("Birch", town_color(BIRCH)),
                ("Cedar", town_color(CEDAR)),
            ],
            v(0.0, 0.35),
        )?;
        fade_in_lagged(stage, &[title, a, b], 1.6, 0.4)?;

        // 01_6 the road ahead: a dial, a list of signposts, a scorecard
        let s = say(&kit, stage, "01_6")?;
        let mut icons = Vec::new();
        s.wait_for(stage, "see exactly why")?;
        let at = v(-3.5, -1.6);
        let ring = Circle::new()
            .radius(0.55)
            .style(draw::line_style(Palette::blue(), 3.0, 0.9))
            .build()
            .moved_to(at);
        let ring = stage.arena_mut().add(ring);
        let hand = draw::stroke(
            stage,
            vec![at, draw::dial_point(at, 0.62, 0.5)],
            draw::line_style(Palette::blue(), 5.0, 1.0),
        );
        icons.extend([ring, hand]);
        fade_in_all(stage, &[ring, hand], 0.5)?;
        s.wait_for(stage, "meet the rules")?;
        let mut list = Vec::new();
        for (j, i) in [CEDAR, CEDAR, BIRCH, CEDAR].into_iter().enumerate() {
            let r = Rectangle::new()
                .width(1.5)
                .height(0.22)
                .corner_radius(0.04)
                .style(flat(town_color(i), 0.45))
                .build()?
                .moved_to(v(0.0, -1.15 - 0.3 * j as f64));
            list.push(stage.arena_mut().add(r));
        }
        fade_in_lagged(stage, &list, 0.8, 0.3)?;
        icons.extend(list);
        s.wait_for(stage, "a theorem")?;
        let mut cells = Vec::new();
        for r in 0..3 {
            for c in 0..3 {
                let ok = (r + c) % 3 != 0;
                let sq = Rectangle::new()
                    .width(0.36)
                    .height(0.36)
                    .corner_radius(0.04)
                    .style(flat(if ok { Palette::green() } else { Palette::red() }, 0.6))
                    .build()?
                    .moved_to(v(3.1 + 0.44 * c as f64, -1.15 - 0.44 * r as f64));
                cells.push(stage.arena_mut().add(sq));
            }
        }
        fade_in_lagged(stage, &cells, 0.9, 0.15)?;
        icons.extend(cells);
        pause(stage, 0.5)?;
        kit.finish(stage)
    }
}
