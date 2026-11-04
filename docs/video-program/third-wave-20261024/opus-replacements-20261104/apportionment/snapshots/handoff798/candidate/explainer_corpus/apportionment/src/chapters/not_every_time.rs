//! 06 It comes and goes: at seventeen seats Ash gets its seat back; across
//! every one-seat addition from 1 to 100 seats, five cost Ash a seat; and
//! the 1880 census calculation that named the paradox, with Seaton's words
//! verbatim as Peyton Young quotes them. Counts from `model`, history from
//! the pinned facts.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::dials::{ranking, three_dials};
use super::draw::{
    fade_in_all, fade_in_lagged, fail, flash_all, flat, morph_all, pause, refill, run_tracker, say, think_here, wipe,
};
use crate::model::{self, KEYS, POP};

pub struct NotEveryTime {
    pub kit: Kit,
}

/// Seaton's letter as Young prints it (census PDF p. 13, printed p. 12),
/// broken into screen lines; the words are unchanged.
const QUOTE: [&str; 5] = [
    "\"While making these calculations I met with the so-called",
    "'Alabama' paradox where Alabama was allotted 8 Representatives",
    "out of a total of 299, receiving but 7 when the total became 300.",
    "Such a result as this is to me conclusive proof that the",
    "process employed in obtaining it is defective.\"",
];
const SOURCE: &str =
    "C. W. Seaton, letter to Congress, as quoted in H. P. Young, Fairness in Apportionment, U.S. Census Bureau";

impl SceneConstruct for NotEveryTime {
    fn name(&self) -> &str {
        "06_not_every_time"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let fact = |key: &str| {
            facts
                .get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let house = stage.add_value_tracker(16.0);

        // 06_1 seventeen seats: does Ash get its seat back?
        say(&kit, stage, "06_1")?;
        let dials = three_dials(&kit, stage, house, 16.0)?;
        let (rank16, _) = ranking(16);
        let rank_t = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.75))?;
        let mut first = dials.clone();
        first.push(rank_t);
        fade_in_all(stage, &first, 0.6)?;
        let q = think_here(&kit, stage)?;

        // 06_2 it does: two, four, eleven
        let s = say(&kit, stage, "06_2")?;
        super::draw::fade_out_all(stage, &[q], 0.2)?;
        s.wait_for(stage, "The whole parts become")?;
        run_tracker(stage, house, 17.0, 3.0)?;
        let (rank17, order17) = ranking(17);
        let rank_t17 = kit.text(stage, &rank17, 24.0, Palette::ink(), v(0.0, -2.75))?;
        morph_all(stage, &[(rank_t, rank_t17)], 0.7)?;
        let seats = model::hamilton(&POP, 17).map_err(fail)?;
        s.wait_for(stage, "Two, four, eleven")?;
        let result = kit.text(
            stage,
            &format!("{}, {}, {}", seats[0], seats[1], seats[2]),
            36.0,
            Palette::hero(),
            v(0.0, 3.3),
        )?;
        fade_in_all(stage, &[result], 0.5)?;
        let leaders: Vec<Mob> = order17.iter().take(2).map(|&i| dials[i * 6]).collect();
        flash_all(stage, &leaders, 0.9)?;

        // 06_3 every addition from 1 seat to 100
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "06_3")?;
        let events = model::alabama_events(&POP).map_err(fail)?;
        let (x0, w, y) = (-5.75, 0.116, 0.2);
        let mut cells = Vec::new();
        for h in 1..100_u64 {
            let c = Rectangle::new()
                .width(w * 0.82)
                .height(0.6)
                .style(flat(Palette::faint(), 0.55))
                .build()?
                .moved_to(v(x0 + (h as f64 - 0.5) * w, y));
            cells.push(stage.arena_mut().add(c));
        }
        let ends = [
            kit.text(stage, "adding seat 2", 18.0, Palette::dim(), v(x0 + 0.6, y - 0.6))?,
            kit.text(
                stage,
                "adding seat 100",
                18.0,
                Palette::dim(),
                v(x0 + 99.0 * w - 0.7, y - 0.6),
            )?,
        ];
        s.wait_for(stage, "Run Hamilton's rule")?;
        fade_in_lagged(stage, &cells, 3.0, 0.05)?;
        fade_in_all(stage, &ends, 0.4)?;
        s.wait_for(stage, "Five of them")?;
        let mut tags = Vec::new();
        let mut red = Vec::new();
        for (k, (h, losers)) in events.iter().enumerate() {
            let cell = cells[*h as usize - 1];
            red.push((cell, Palette::red(), 1.0));
            let who: Vec<&str> = losers.iter().map(|&i| model::NAMES[i]).collect();
            let label = format!("{}→{}  {}", h, h + 1, who.join(", "));
            let ty = if k % 2 == 0 { y + 0.7 } else { y + 1.1 };
            tags.push(kit.text(stage, &label, 18.0, Palette::red(), v(x0 + (*h as f64 - 0.5) * w, ty))?);
        }
        refill(stage, &red, 0.8)?;
        fade_in_lagged(stage, &tags, 1.5, 0.3)?;
        s.wait_for(stage, "every time it is Ash")?;
        flash_all(stage, &tags, 1.0)?;
        if fact("alabama.loser")? != KEYS[model::ASH] {
            return Err(fail("06_3 says every loser is Ash".into()));
        }

        // 06_4 1880: Seaton's tables
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "06_4")?;
        let range = format!(
            "Hamilton apportionments for every House size from {} to {}",
            fact("history.seaton.range_low")?,
            fact("history.seaton.range_high")?
        );
        let range_t = kit.text(stage, &range, 24.0, Palette::dim(), v(0.0, 2.6))?;
        s.wait_for(stage, "worked out Hamilton")?;
        fade_in_all(stage, &[range_t], 0.6)?;
        s.wait_for(stage, "Alabama received")?;
        let left = kit.text(
            stage,
            &format!(
                "House of {}:  Alabama {}",
                fact("history.seaton.house_before")?,
                fact("history.seaton.alabama_before")?
            ),
            34.0,
            Palette::ink(),
            v(-3.2, 1.2),
        )?;
        let right = kit.text(
            stage,
            &format!(
                "House of {}:  Alabama {}",
                fact("history.seaton.house_after")?,
                fact("history.seaton.alabama_after")?
            ),
            34.0,
            Palette::red(),
            v(3.2, 1.2),
        )?;
        fade_in_lagged(stage, &[left, right], 1.6, 0.6)?;

        // 06_5 the letter, verbatim
        let s = say(&kit, stage, "06_5")?;
        let mut lines = Vec::new();
        for (j, line) in QUOTE.iter().enumerate() {
            lines.push(kit.text_styled(
                stage,
                line,
                24.0,
                Palette::ink(),
                v(0.0, -0.1 - 0.48 * j as f64),
                false,
                true,
            )?);
        }
        let src = kit.text(stage, SOURCE, 16.0, Palette::dim(), v(0.0, -2.75))?;
        fade_in_lagged(stage, &lines, 1.6, 0.2)?;
        fade_in_all(stage, &[src], 0.4)?;
        s.wait_for(stage, "Such a result")?;
        let spoken: Vec<(Mob, Srgb, f64)> = lines[3..].iter().map(|&m| (m, Palette::hero(), 1.0)).collect();
        refill(stage, &spoken, 0.6)?;

        // 06_6 alternative calculations, not an enacted change
        let s = say(&kit, stage, "06_6")?;
        s.wait_for(stage, "alternative calculations")?;
        let mut dim: Vec<(Mob, Srgb, f64)> = lines.iter().map(|&m| (m, Palette::dim(), 0.5)).collect();
        dim.push((src, Palette::dim(), 0.5));
        refill(stage, &dim, 0.6)?;
        let note = kit.text(
            stage,
            "alternative calculations, not an enacted change",
            24.0,
            Palette::hero(),
            v(0.0, 3.3),
        )?;
        fade_in_all(stage, &[note], 0.5)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
