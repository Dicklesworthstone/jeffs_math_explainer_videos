//! 06 It comes and goes: at seventeen seats Ash gets its seat back; across
//! every one-seat addition from 1 to 100 seats, five cost Ash a seat; and
//! the 1880 census calculation that named the paradox, with Seaton's words
//! verbatim as Peyton Young quotes them. Counts from `model`, history from
//! the pinned facts.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::dials::{CENTRES, house_trackers, ranking, three_dials};
use super::draw::{
    fade_in_all, fade_in_lagged, fail, flash_all, flat, morph_all, pause, refill, run_trackers, say, step_arc,
    think_here, wait_until,
};
use crate::model::{self, KEYS, POP, share};

pub struct NotEveryTime {
    pub kit: Kit,
}

/// The sentence of Seaton's letter the narration speaks, verbatim as Young
/// prints it (census PDF p. 13, printed p. 12), in two screen lines. The
/// letter's first sentence (8 of 299, 7 of 300) is what 06_4 already shows;
/// the whole letter is quoted in the description.
const QUOTE: [&str; 2] = [
    "\"Such a result as this is to me conclusive proof",
    "that the process employed in obtaining it is defective.\"",
];
/// A short source line; the full citation is in docs/description.md.
const SOURCE: &str = "C. W. Seaton, letter to Congress, quoted by H. P. Young";
/// Reading speed for words on screen, characters a second.
const READ_CPS: f64 = 15.0;
/// The council the 06_1 prediction starts from and the one it asks about: the
/// dials, the ranking, the seats, the steps and the question all read these.
const HOUSE_NOW: u64 = 16;
const HOUSE_NEXT: u64 = HOUSE_NOW + 1;

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Town `i`'s step for one more seat: its exact share of the population as a
/// fraction of a turn, over the towns' smallest common denominator, the form
/// 05_2 says and shows.
fn step_fraction(i: usize) -> (u64, u64) {
    let total = model::total(&POP);
    let common = POP.iter().fold(total, |d, &p| gcd(d, p));
    (POP[i] / common, total / common)
}

/// The label under a dial during the 06_1 prediction.
fn step_label(i: usize) -> String {
    let (num, den) = step_fraction(i);
    format!("+{num}/{den} turn")
}

/// The line on screen during the 06_1 prediction.
fn question() -> String {
    format!(
        "{HOUSE_NOW} seats → {HOUSE_NEXT}: does {} get its seat back?",
        model::NAMES[model::ASH]
    )
}

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
        let t = house_trackers(stage, HOUSE_NOW as f64);

        // 06_1 seventeen seats: does Ash get its seat back?
        say(&kit, stage, "06_1")?;
        let dials = three_dials(&kit, stage, t, HOUSE_NOW as f64)?;
        let (rank16, _) = ranking(HOUSE_NOW);
        let rank_t = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.75))?;
        // What the prediction needs, as in 05_3: each hand's step for one more
        // seat, and the question. They join the dials' fade and leave with the
        // "?", so neither adds any time.
        let mut inputs = Vec::new();
        for (i, centre) in CENTRES.iter().enumerate() {
            let at = v(centre[0], centre[1]);
            let from = share(&POP, i, HOUSE_NOW).tail().to_f64();
            let (num, den) = step_fraction(i);
            inputs.push(step_arc(stage, at, from, num as f64 / den as f64, Palette::ink()));
            inputs.push(kit.text(stage, &step_label(i), 22.0, Palette::ink(), v(at[0], at[1] - 1.75))?);
        }
        inputs.push(kit.text(stage, &question(), 30.0, Palette::ink(), v(0.0, 3.3))?);
        let mut first = dials.clone();
        first.push(rank_t);
        first.extend_from_slice(&inputs);
        fade_in_all(stage, &first, 0.6)?;
        let q = think_here(&kit, stage)?;

        // 06_2 it does: two, four, eleven
        let s = say(&kit, stage, "06_2")?;
        let mut cleared = inputs;
        cleared.push(q);
        super::draw::fade_out_all(stage, &cleared, 0.2)?;
        s.wait_for(stage, "The whole parts become")?;
        run_trackers(stage, &t.map(|m| (m, HOUSE_NEXT as f64)), 3.0, true)?;
        let (rank17, order17) = ranking(HOUSE_NEXT);
        let rank_t17 = kit.text(stage, &rank17, 24.0, Palette::ink(), v(0.0, -2.75))?;
        morph_all(stage, &[(rank_t, rank_t17)], 0.7)?;
        let seats = model::hamilton(&POP, HOUSE_NEXT).map_err(fail)?;
        s.wait_for(stage, "Two, four, eleven")?;
        let named: Vec<String> = model::NAMES
            .iter()
            .zip(&seats)
            .map(|(name, seat)| format!("{name} {seat}"))
            .collect();
        let result = kit.text(stage, &named.join(", "), 36.0, Palette::hero(), v(0.0, 3.3))?;
        fade_in_all(stage, &[result], 0.5)?;
        let leaders: Vec<Mob> = order17.iter().take(2).map(|&i| dials[i * 6]).collect();
        flash_all(stage, &leaders, 0.9)?;

        // 06_3 Keep the seventeen-seat result visible during the introduction.
        kit.hold(stage)?;
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
        // Fade current roots without waiting for the new cue to finish.
        explainer_kit::fade_all(stage, 0.6)?;
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
        // Says what the tags mean. It arrives last in the tags' own group, whose
        // total run time is fixed; the guard below covers "always from Ash".
        let summary = kit.text(
            stage,
            &format!(
                "{} of {} additions take a seat away, always from {}",
                fact("alabama.count")?,
                fact("alabama.additions")?,
                model::NAMES[model::ASH]
            ),
            24.0,
            Palette::red(),
            v(0.0, y - 1.15),
        )?;
        refill(stage, &red, 0.8)?;
        let mut arrivals = tags.clone();
        arrivals.push(summary);
        fade_in_lagged(stage, &arrivals, 1.5, 0.3)?;
        s.wait_for(stage, "every time it is Ash")?;
        flash_all(stage, &tags, 1.0)?;
        if fact("alabama.loser")? != KEYS[model::ASH] {
            return Err(fail("06_3 says every loser is Ash".into()));
        }

        // 06_4 Retain the loss strip while introducing Seaton and his tables.
        kit.hold(stage)?;
        let s = say(&kit, stage, "06_4")?;
        let range = format!(
            "{} census, C. W. Seaton: Hamilton apportionments for every House size from {} to {}",
            crate::facts::seaton_census_year().map_err(fail)?,
            fact("history.seaton.range_low")?,
            fact("history.seaton.range_high")?
        );
        let range_t = kit.text(stage, &range, 24.0, Palette::dim(), v(0.0, 2.6))?;
        s.wait_for(stage, "worked out Hamilton")?;
        explainer_kit::fade_all(stage, 0.6)?;
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

        // 06_5 the sentence the narration speaks, verbatim, with a short source
        let s = say(&kit, stage, "06_5")?;
        let mut lines = Vec::new();
        for (j, line) in QUOTE.iter().enumerate() {
            lines.push(kit.text_styled(
                stage,
                line,
                28.0,
                Palette::ink(),
                v(0.0, -0.45 - 0.62 * j as f64),
                false,
                true,
            )?);
        }
        let src = kit.text(stage, SOURCE, 20.0, Palette::dim(), v(0.0, -2.2))?;
        fade_in_lagged(stage, &lines, 1.0, 0.2)?;
        fade_in_all(stage, &[src], 0.4)?;
        let shown = stage.scene().time().to_f64();
        s.wait_for(stage, "Such a result")?;
        let spoken: Vec<(Mob, Srgb, f64)> = lines.iter().map(|&m| (m, Palette::hero(), 1.0)).collect();
        refill(stage, &spoken, 0.6)?;

        // 06_6 alternative calculations, not an enacted change
        let s = say(&kit, stage, "06_6")?;
        s.wait_for(stage, "alternative calculations")?;
        // the quote and its source stay at full strength until they can be
        // read at READ_CPS, however fast the line runs
        let chars: usize = QUOTE.iter().chain([SOURCE].iter()).map(|l| l.chars().count()).sum();
        wait_until(stage, shown + chars as f64 / READ_CPS)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The steps printed in 06_1 are each town's exact share, over the towns'
    /// smallest common denominator, with the oracle's numerators.
    #[test]
    fn the_printed_steps_are_the_exact_shares() {
        let labels: Vec<String> = (0..POP.len()).map(step_label).collect();
        assert_eq!(labels, ["+4/39 turn", "+9/39 turn", "+26/39 turn"]);
        let facts = crate::facts::facts().expect("facts build");
        let total = model::total(&POP);
        let mut common = 0;
        for (i, &pop) in POP.iter().enumerate() {
            let (num, den) = step_fraction(i);
            assert_eq!(num * total, pop * den, "{i}");
            assert_eq!(
                facts.get(&format!("speed39.{}", KEYS[i])),
                Some(num.to_string().as_str())
            );
            common = gcd(common, num);
        }
        assert_eq!(gcd(common, step_fraction(0).1), 1);
    }

    /// The question names the councils the ranking and the seats are read at.
    #[test]
    fn the_question_reads_the_house_constants() {
        assert_eq!(question(), "16 seats → 17: does Ash get its seat back?");
        assert_eq!(ranking(HOUSE_NOW).1.last(), Some(&model::ASH));
        assert_eq!(model::hamilton(&POP, HOUSE_NEXT).expect("hamilton"), vec![2, 4, 11]);
    }
}
