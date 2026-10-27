//! Retain one set of cards through a failed ordinary sum and its integer repair.
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Rail, card, fade_in_all, fade_out_all, fail, panel, pause, read_by, recolor_all, say, think, wait_until,
};
use super::envelope::outline;
use crate::model::{self, Dial, OTHER_SPINS, Ratio, Recipe, SECOND_TRIO, SECRET, SPINS, TRUSTEES};
use crate::{facts, palette};

pub struct ThirdsThatDoNotCancel {
    pub kit: Kit,
}
const NOTE: &str = "We recovered the secret by undoing times three on the eleven-mark dial. The ordinary sum of four thirds was never a dial mark. Integer weights made wrapping safe again.";

struct Working {
    heights: Vec<i64>,
    shares: Vec<u32>,
    weights: Vec<Ratio>,
    fraction: Ratio,
    clear: i64,
    cleared: Vec<i64>,
    terms: Vec<i64>,
    total: i64,
    grid: Vec<(u32, u32, bool)>,
    row: Vec<u32>,
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let r = Recipe::new(SECRET, &OTHER_SPINS);
    let heights = SECOND_TRIO.iter().map(|&x| r.height(x)).collect::<Vec<_>>();
    let shares = SECOND_TRIO.iter().map(|&x| r.share(d, x)).collect::<Vec<_>>();
    let points = SECOND_TRIO
        .iter()
        .copied()
        .zip(shares.iter().copied())
        .collect::<Vec<_>>();
    let weights = model::ordinary_weights(&SECOND_TRIO);
    let fraction = model::ordinary_sum(&points);
    let (clear, cleared) = model::cleared_weights(&SECOND_TRIO);
    let terms = cleared
        .iter()
        .zip(&shares)
        .map(|(&w, &s)| w * i64::from(s))
        .collect::<Vec<_>>();
    let total = terms.iter().sum::<i64>();
    let grid = model::tuples(d, SPINS.len())
        .map(|spins| {
            let r = Recipe::new(SECRET, &spins);
            let points = SECOND_TRIO.iter().map(|&x| (x, r.share(d, x))).collect::<Vec<_>>();
            (spins[0], spins[1], model::ordinary_sum(&points).whole().is_none())
        })
        .collect::<Vec<_>>();
    let row = d.times_row(u32::try_from(clear).map_err(|_| "negative multiplier")?);
    let f = facts::facts()?;
    for (key, value) in [
        ("S-F10.other.heights", model::csv(&heights)),
        ("S-F10.other.shares", model::csv(&shares)),
        ("S-F10.weights", model::csv(&weights)),
        ("S-F10.other.fraction_sum", fraction.to_string()),
        ("S-F10.clear", clear.to_string()),
        ("S-F10.cleared_weights", model::csv(&cleared)),
        ("S-F10.other.cleared_terms", model::csv(&terms)),
        ("S-F10.other.cleared_sum", total.to_string()),
        (
            "S-F10.fractional_spin_pairs",
            grid.iter().filter(|p| p.2).count().to_string(),
        ),
        ("S-F6.spin_pairs", grid.len().to_string()),
        ("S-F10.times3", model::csv(&row)),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}: {value}"));
        }
    }
    if fraction.whole().is_some() || d.wrap(total) != d.wrap(clear * i64::from(SECRET)) {
        return Err("failed-fraction example or integer repair changed".into());
    }
    Ok(Working {
        heights,
        shares,
        weights,
        fraction,
        clear,
        cleared,
        terms,
        total,
        grid,
        row,
    })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
fn expression(weights: &[impl std::fmt::Display], values: &[u32]) -> String {
    weights
        .iter()
        .zip(values)
        .map(|(w, s)| format!("({w}) × {s}"))
        .collect::<Vec<_>>()
        .join(" + ")
}
impl SceneConstruct for ThirdsThatDoNotCancel {
    fn name(&self) -> &str {
        "17_thirds_that_do_not_cancel"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let spoken = say(&k, stage, "17_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&SECOND_TRIO, palette::trio()), 0.3)?;
        let cards = SECOND_TRIO
            .iter()
            .zip(&w.shares)
            .map(|(&x, &s)| card(&k, stage, x, &s.to_string(), palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        let rail_note = text(
            &k,
            stage,
            "Same trustees; a new dealing",
            20.0,
            -3.9,
            -2.75,
            Palette::dim(),
        )?;
        let heading = text(
            &k,
            stage,
            &format!("Same secret; spins {} and {}", OTHER_SPINS[0], OTHER_SPINS[1]),
            30.0,
            0.0,
            3.0,
            palette::spin(),
        )?;
        fade_in_all(stage, &[heading], 0.5)?;
        spoken.wait_for(stage, "On paper")?;
        let heights = text(
            &k,
            stage,
            &format!("Paper heights: {}", model::csv(&w.heights).replace(',', "     ")),
            34.0,
            0.0,
            1.9,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[heights], 0.5)?;
        spoken.wait_for(stage, "Their wrapped cards")?;
        fade_in_all(stage, &cards, 0.5)?;
        fade_in_all(stage, &[rail_note], 0.3)?;
        k.hold(stage)?;

        let spoken = say(&k, stage, "17_2")?;
        fade_out_all(stage, &[heights], 0.3)?;
        let expression = text(
            &k,
            stage,
            &expression(&w.weights, &w.shares),
            33.0,
            0.0,
            1.8,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[expression], 0.5)?;
        spoken.wait_for(stage, "gives four thirds")?;
        let failed = text(
            &k,
            stage,
            &format!("= {}     no dial mark", w.fraction),
            36.0,
            0.0,
            0.75,
            palette::leak(),
        )?;
        fade_in_all(stage, &[failed], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.0)?;

        let spoken = say(&k, stage, "17_3")?;
        fade_out_all(stage, &[heading, expression, failed], 0.4)?;
        let mut grid_parts = vec![text(
            &k,
            stage,
            "Every spin pair, with this secret",
            28.0,
            0.0,
            3.25,
            Palette::ink(),
        )?];
        let mut cells = Vec::new();
        for &(a, b, fractional) in &w.grid {
            let at = v(-5.25 + f64::from(a) * 0.32, 0.05 + f64::from(b) * 0.26);
            let cell = panel(stage, Palette::dim(), 0.75, at, 0.27, 0.21)?;
            cells.push((a, cell, fractional));
            grid_parts.push(cell);
        }
        for n in Dial::vault().all() {
            grid_parts.push(text(
                &k,
                stage,
                &n.to_string(),
                15.0,
                -5.25 + f64::from(n) * 0.32,
                -0.28,
                Palette::dim(),
            )?);
            grid_parts.push(text(
                &k,
                stage,
                &n.to_string(),
                15.0,
                -5.65,
                0.05 + f64::from(n) * 0.26,
                Palette::dim(),
            )?);
        }
        grid_parts.push(text(
            &k,
            stage,
            "spin on position →",
            19.0,
            -3.65,
            -0.65,
            Palette::dim(),
        )?);
        grid_parts.push(text(
            &k,
            stage,
            "spin on position squared ↑",
            19.0,
            -3.65,
            2.95,
            Palette::dim(),
        )?);
        let key = text(
            &k,
            stage,
            "red: ordinary sum is fractional",
            23.0,
            2.25,
            1.75,
            palette::leak(),
        )?;
        let other_key = text(
            &k,
            stage,
            "teal: ordinary sum is whole",
            23.0,
            2.25,
            1.20,
            palette::trio(),
        )?;
        grid_parts.extend([key, other_key]);
        fade_in_all(stage, &grid_parts, 0.7)?;
        for a in Dial::vault().all() {
            let column = cells
                .iter()
                .filter(|p| p.0 == a)
                .map(|&(_, cell, fractional)| (cell, if fractional { palette::leak() } else { palette::trio() }))
                .collect::<Vec<_>>();
            recolor_all(stage, &column, 0.18)?;
        }
        spoken.wait_for(stage, "eighty")?;
        let count = text(
            &k,
            stage,
            &format!("{} of {}", w.grid.iter().filter(|p| p.2).count(), w.grid.len()),
            43.0,
            2.25,
            2.5,
            palette::leak(),
        )?;
        let example = outline(stage, v(-5.25, 0.31), 0.31, 0.25, palette::secret())?;
        let note = text(
            &k,
            stage,
            "A third of an eleven is not a whole eleven",
            21.0,
            2.25,
            0.2,
            Palette::ink(),
        )?;
        grid_parts.extend([count, example, note]);
        fade_in_all(stage, &[count, example, note], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;

        let spoken = say(&k, stage, "17_4")?;
        fade_out_all(stage, &grid_parts, 0.4)?;
        let old = text(
            &k,
            stage,
            &format!("Weights: {}", model::csv(&w.weights).replace(',', "     ")),
            32.0,
            0.0,
            2.6,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[old], 0.4)?;
        spoken.wait_for(stage, "Three times each weight")?;
        let new = text(
            &k,
            stage,
            &format!("× {}  →  {}", w.clear, model::csv(&w.cleared).replace(',', "     ")),
            38.0,
            0.0,
            1.6,
            palette::secret(),
        )?;
        fade_in_all(stage, &[new], 0.6)?;
        spoken.wait_for(stage, "On paper")?;
        let identity = text(&k, stage, "On paper: sum = 3 × secret", 30.0, 0.0, 0.55, Palette::ink())?;
        let wrapping = text(
            &k,
            stage,
            "Whole weights keep removed elevens whole",
            23.0,
            0.0,
            -0.2,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[identity, wrapping], 0.5)?;
        k.hold(stage)?;

        let spoken = say(&k, stage, "17_5")?;
        fade_out_all(stage, &[old, new, identity, wrapping], 0.4)?;
        let cards_expression = text(
            &k,
            stage,
            &self::expression(&w.cleared, &w.shares),
            34.0,
            0.0,
            2.5,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[cards_expression], 0.5)?;
        let mut terms = Vec::new();
        for (i, phrase) in ["ninety", "minus a hundred and fifty", "plus sixty-four"]
            .iter()
            .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            let term = text(
                &k,
                stage,
                &format!("{}", w.terms[i]),
                40.0,
                -2.7 + i as f64 * 2.7,
                1.45,
                Palette::ink(),
            )?;
            fade_in_all(stage, &[term], 0.4)?;
            terms.push(term);
        }
        spoken.wait_for(stage, "which is four")?;
        let total = text(
            &k,
            stage,
            &format!("sum = {}", w.total),
            38.0,
            0.0,
            0.55,
            palette::secret(),
        )?;
        fade_in_all(stage, &[total], 0.5)?;
        k.hold(stage)?;

        say(&k, stage, "17_6")?;
        terms.extend([cards_expression, total]);
        fade_out_all(stage, &terms, 0.4)?;
        let title = text(
            &k,
            stage,
            "Undo times three on the dial",
            30.0,
            0.0,
            3.1,
            Palette::ink(),
        )?;
        let input_label = text(&k, stage, "mark", 20.0, -6.2, 2.05, Palette::dim())?;
        let output_label = text(&k, stage, "× 3", 20.0, -6.2, 0.9, Palette::dim())?;
        let target = text(
            &k,
            stage,
            &format!("3 × secret lands on {}", Dial::vault().wrap(w.total)),
            31.0,
            1.0,
            -0.2,
            palette::secret(),
        )?;
        let mut table = vec![title, input_label, output_label, target];
        let mut outputs = Vec::new();
        for (i, &value) in w.row.iter().enumerate() {
            let x = -5.0 + i as f64;
            table.push(text(&k, stage, &i.to_string(), 29.0, x, 2.05, Palette::ink())?);
            table.push(outline(stage, v(x, 0.9), 0.73, 0.65, Palette::dim())?);
            outputs.push(text(&k, stage, &value.to_string(), 30.0, x, 0.9, palette::trio())?);
        }
        fade_in_all(stage, &table, 0.6)?;
        let question = think(&k, stage, v(2.3, -1.2))?;

        let spoken = say(&k, stage, "17_7")?;
        fade_out_all(stage, &[question], 0.25)?;
        let chosen = outline(stage, v(-5.0 + f64::from(SECRET), 1.5), 0.86, 1.9, palette::secret())?;
        fade_in_all(stage, &[outputs[SECRET as usize], chosen], 0.5)?;
        let result = text(
            &k,
            stage,
            &format!(
                "{} × {} = {} → {}",
                w.clear,
                SECRET,
                w.clear * i64::from(SECRET),
                Dial::vault().wrap(w.total)
            ),
            30.0,
            2.3,
            -1.1,
            palette::secret(),
        )?;
        fade_in_all(stage, &[result], 0.5)?;
        spoken.wait_for(stage, "row of threes")?;
        let other_outputs = outputs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != SECRET as usize)
            .map(|(_, &m)| m)
            .collect::<Vec<_>>();
        fade_in_all(stage, &other_outputs, 0.8)?;
        k.hold(stage)?;
        fade_out_all(stage, &[rail_note], 0.3)?;
        let note = precisely(&k, stage, NOTE)?;
        fade_in_all(stage, &[note], 0.6)?;
        let until = read_by(stage, NOTE);
        wait_until(stage, until)?;
        pause(stage, 0.6)?;
        k.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_and_integer_repair_have_their_claimed_scope() {
        let w = working().unwrap();
        assert_eq!(w.grid.len(), 121);
        assert_eq!(w.grid.iter().filter(|p| p.2).count(), 80);
        let mut permutation = w.row.clone();
        permutation.sort_unstable();
        assert_eq!(permutation, Dial::vault().all().collect::<Vec<_>>());
        for secret in Dial::vault().all() {
            for spins in model::tuples(Dial::vault(), SPINS.len()) {
                let r = Recipe::new(secret, &spins);
                let cards: i64 = w
                    .cleared
                    .iter()
                    .zip(SECOND_TRIO)
                    .map(|(weight, x)| weight * i64::from(r.share(Dial::vault(), x)))
                    .sum();
                assert_eq!(
                    Dial::vault().wrap(cards),
                    Dial::vault().wrap(w.clear * i64::from(secret))
                );
            }
        }
    }
}
