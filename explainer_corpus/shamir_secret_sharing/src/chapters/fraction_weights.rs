//! Ordinary helper weights for a new trio. The picture changes heights
//! into cards while retaining the trustee rail and the same weights.
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Rail, Vault, card, disc, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, stroke, think,
    wait_until,
};
use super::envelope::outline;
use super::two_keys::glow;
use crate::model::{self, Dial, Ratio, Recipe, SECOND_TRIO, SECRET, TRUSTEES};
use crate::{facts, palette};

pub struct FractionWeights {
    pub kit: Kit,
}
const NOTE: &str = "These ordinary fractions happen to give whole terms for these cards. That is not a general rule for wrapped shares. We still need a way to divide on the dial.";
const ROW_Y: [f64; 3] = [2.55, 1.25, -0.05];

struct Working {
    heights: Vec<i64>,
    shares: Vec<u32>,
    weights: Vec<Ratio>,
    own: Vec<i64>,
    at_zero: Vec<i64>,
    terms: Vec<Ratio>,
    total: i64,
}
fn working() -> Result<Working, String> {
    let recipe = Recipe::example();
    let dial = Dial::vault();
    let heights = SECOND_TRIO.iter().map(|&x| recipe.height(x)).collect::<Vec<_>>();
    let shares = SECOND_TRIO.iter().map(|&x| recipe.share(dial, x)).collect::<Vec<_>>();
    let weights = model::ordinary_weights(&SECOND_TRIO);
    let terms = weights
        .iter()
        .zip(&shares)
        .map(|(&w, &s)| w.times(i64::from(s)))
        .collect::<Vec<_>>();
    let total = terms
        .iter()
        .fold(Ratio::int(0), |a, &b| a.plus(b))
        .whole()
        .ok_or("cards did not give a whole sum")?;
    let own = (0..SECOND_TRIO.len())
        .map(|i| model::helper_at_own(&SECOND_TRIO, i))
        .collect::<Vec<_>>();
    let at_zero = (0..SECOND_TRIO.len())
        .map(|i| model::helper_at(&SECOND_TRIO, i, 0))
        .collect::<Vec<_>>();
    let f = facts::facts()?;
    for (key, value) in [
        ("S-F10.trio", model::csv(&SECOND_TRIO)),
        ("S-F10.paper.heights", model::csv(&heights)),
        ("S-F10.trio_shares", model::csv(&shares)),
        ("S-F10.weights", model::csv(&weights)),
        ("S-F10.helper_at_own", model::csv(&own)),
        ("S-F10.helper_at_zero", model::csv(&at_zero)),
        ("S-F10.fraction_terms", model::csv(&terms)),
        ("S-F10.fraction_sum", total.to_string()),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}: {value}"));
        }
    }
    let paper = weights
        .iter()
        .zip(&heights)
        .fold(Ratio::int(0), |a, (&w, &h)| a.plus(w.times(h)));
    if paper != Ratio::int(i64::from(SECRET)) || dial.wrap(total) != SECRET || terms.iter().any(|t| t.whole().is_none())
    {
        return Err("fraction example lost its intended limited cancellation".into());
    }
    Ok(Working {
        heights,
        shares,
        weights,
        own,
        at_zero,
        terms,
        total,
    })
}
fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, size, c, v(x, y))
}
fn point(x: f64, h: f64, top: f64) -> Vec3 {
    v(-5.8 + x * 0.88, -0.35 + h / top * 2.95)
}
fn plot(kit: &Kit, stage: &mut Stage<'_>, w: &Working) -> fmn::Result<Vec<Mob>> {
    let top = *w.heights.last().ok_or_else(|| fail("no heights".into()))? as f64;
    let last = *SECOND_TRIO.last().ok_or_else(|| fail("no positions".into()))?;
    let mut parts = vec![
        text(
            kit,
            stage,
            "On paper: heights before wrapping",
            24.0,
            -3.5,
            3.35,
            Palette::ink(),
        )?,
        stroke(
            stage,
            vec![
                point(0.0, top * 1.06, top),
                point(0.0, 0.0, top),
                point(f64::from(last) + 0.3, 0.0, top),
            ],
            Palette::dim(),
            2.0,
        ),
    ];
    for x in 0..=last {
        parts.push(text(
            kit,
            stage,
            &x.to_string(),
            18.0,
            point(f64::from(x), 0.0, top)[0],
            -0.65,
            Palette::dim(),
        )?);
    }
    let recipe = Recipe::example();
    let curve = (0..=100)
        .map(|i| {
            let x = f64::from(last) * f64::from(i) / 100.0;
            // The same polynomial as Recipe::height, evaluated continuously only for geometry.
            let h = f64::from(SECRET)
                + model::SPINS
                    .iter()
                    .enumerate()
                    .map(|(k, &a)| f64::from(a) * x.powi(k as i32 + 1))
                    .sum::<f64>();
            point(x, h, top)
        })
        .collect();
    parts.push(stroke(stage, curve, Palette::dim(), 2.5));
    for (&x, &h) in SECOND_TRIO.iter().zip(&w.heights) {
        if recipe.height(x) != h {
            return Err(fail("plot height mismatch".into()));
        }
        let at = point(f64::from(x), h as f64, top);
        parts.push(disc(stage, palette::trio(), at, 0.07));
        parts.push(text(
            kit,
            stage,
            &h.to_string(),
            24.0,
            at[0] - 0.30,
            at[1] + 0.26,
            palette::trio(),
        )?);
    }
    Ok(parts)
}
fn helper_row(kit: &Kit, stage: &mut Stage<'_>, w: &Working, i: usize) -> fmn::Result<(Vec<Mob>, Mob)> {
    let x = SECOND_TRIO[i];
    let other = SECOND_TRIO.iter().copied().filter(|&n| n != x).collect::<Vec<_>>();
    let y = ROW_Y[i];
    let product = text(
        kit,
        stage,
        &format!("Trustee {x}: (x − {}) × (x − {})", other[0], other[1]),
        23.0,
        2.35,
        y,
        Palette::ink(),
    )?;
    let evaluations = text(
        kit,
        stage,
        &format!("at {x}: {}       at 0: {}", w.own[i], w.at_zero[i]),
        23.0,
        2.35,
        y - 0.39,
        Palette::ink(),
    )?;
    let weight = text(
        kit,
        stage,
        &format!("weight = {} ÷ ({}) = {}", w.at_zero[i], w.own[i], w.weights[i]),
        24.0,
        2.35,
        y - 0.80,
        palette::secret(),
    )?;
    Ok((vec![product, evaluations], weight))
}
fn weighted(w: &Working, values: &[i64]) -> String {
    w.weights
        .iter()
        .zip(values)
        .map(|(a, b)| format!("({a}) × {b}"))
        .collect::<Vec<_>>()
        .join(" + ")
}
impl SceneConstruct for FractionWeights {
    fn name(&self) -> &str {
        "16_fraction_weights"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let w = working().map_err(fail)?;
        let spoken = say(&kit, stage, "16_1")?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&SECOND_TRIO, palette::trio()), 0.3)?;
        let cards = SECOND_TRIO
            .iter()
            .zip(&w.shares)
            .map(|(&x, &s)| card(&kit, stage, x, &s.to_string(), palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards, 0.5)?;
        let rail_note = text(
            &kit,
            stage,
            "The trustees hold only these cards",
            20.0,
            -3.9,
            -2.75,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[rail_note], 0.3)?;
        spoken.wait_for(stage, "On paper")?;
        let paper = plot(&kit, stage, &w)?;
        fade_in_all(stage, &paper, 0.7)?;
        kit.hold(stage)?;

        let helper_label = text(
            &kit,
            stage,
            "Helpers: x means position",
            24.0,
            2.35,
            3.35,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[helper_label], 0.3)?;
        let mut working_rows = vec![helper_label];
        let mut weights = Vec::new();
        for (i, phrase) in ["ten thirds", "weight, minus five", "eight thirds"].iter().enumerate() {
            let spoken = say(&kit, stage, &format!("16_{}", i + 2))?;
            let (row, weight) = helper_row(&kit, stage, &w, i)?;
            fade_in_all(stage, &row[..1], 0.5)?;
            spoken.wait_for(stage, "At its own position")?;
            fade_in_all(stage, &row[1..], 0.5)?;
            spoken.wait_for(stage, phrase)?;
            fade_in_all(stage, &[weight], 0.5)?;
            working_rows.extend(row);
            weights.push(weight);
            kit.hold(stage)?;
        }
        say(&kit, stage, "16_5")?;
        fade_out_all(stage, &paper, 0.4)?;
        fade_out_all(stage, &working_rows, 0.4)?;
        fade_out_all(stage, &weights, 0.4)?;
        let heading = text(
            &kit,
            stage,
            "On paper: the same helper construction",
            28.0,
            0.0,
            2.9,
            Palette::ink(),
        )?;
        let equation = text(&kit, stage, &weighted(&w, &w.heights), 30.0, 0.0, 1.95, Palette::ink())?;
        let answer = text(&kit, stage, &format!("= {SECRET}"), 42.0, 0.0, 1.05, palette::secret())?;
        fade_in_all(stage, &[heading, equation, answer], 0.6)?;
        kit.hold(stage)?;
        pause(stage, 1.2)?;

        say(&kit, stage, "16_6")?;
        fade_out_all(stage, &[heading, equation, answer], 0.4)?;
        let heading = text(&kit, stage, "Now use the wrapped cards", 28.0, 0.0, 3.0, Palette::ink())?;
        fade_in_all(stage, &[heading], 0.4)?;
        let cards_i = w.shares.iter().map(|&s| i64::from(s)).collect::<Vec<_>>();
        let equation = text(&kit, stage, &weighted(&w, &cards_i), 32.0, 0.0, 2.0, Palette::ink())?;
        fade_in_all(stage, &[equation], 0.5)?;
        let question = think(&kit, stage, v(1.2, 0.25))?;

        let spoken = say(&kit, stage, "16_7")?;
        fade_out_all(stage, &[question, equation], 0.3)?;
        let mut terms = Vec::new();
        for (i, phrase) in ["ten thirds of three", "minus five times two", "eight thirds of six"]
            .iter()
            .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            let term = text(
                &kit,
                stage,
                &format!("({}) × {} = {}", w.weights[i], w.shares[i], w.terms[i]),
                31.0,
                2.25,
                2.0 - i as f64 * 0.75,
                Palette::ink(),
            )?;
            fade_in_all(stage, &[term], 0.5)?;
            terms.push(term);
        }
        spoken.wait_for(stage, "lands on five")?;
        let vault = Vault::build(&kit, stage, Dial::vault())?;
        vault.show(stage, 0.6)?;
        vault.turn(stage, w.total as u32, 1.4)?;
        let total = text(
            &kit,
            stage,
            &format!(
                "{} + ({}) + {} = {} → {}",
                w.terms[0],
                w.terms[1],
                w.terms[2],
                w.total,
                Dial::vault().wrap(w.total)
            ),
            28.0,
            2.3,
            -0.35,
            palette::secret(),
        )?;
        fade_in_all(stage, &[total], 0.5)?;
        kit.hold(stage)?;

        say(&kit, stage, "16_8")?;
        let halos = SECOND_TRIO
            .iter()
            .zip(&w.weights)
            .filter(|(_, weight)| weight.whole().is_none())
            .map(|(&x, _)| outline(stage, super::draw::frame_at(x), 0.82, 0.64, palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        glow(stage, &halos, 1.0)?;
        fade_out_all(stage, &[rail_note], 0.3)?;
        let note = precisely(&kit, stage, NOTE)?;
        fade_in_all(stage, &[note], 0.6)?;
        let until = read_by(stage, NOTE);
        kit.hold(stage)?;
        wait_until(stage, until)?;
        pause(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractions_are_safe_here_but_not_a_general_card_recipe() {
        let w = working().unwrap();
        let other = Recipe::new(SECRET, &model::OTHER_SPINS);
        let points = SECOND_TRIO
            .iter()
            .map(|&x| (x, other.share(Dial::vault(), x)))
            .collect::<Vec<_>>();
        assert!(model::ordinary_sum(&points).whole().is_none());
        assert_eq!(model::rebuild(Dial::vault(), &points), Some(SECRET));
        assert!(w.weights.iter().any(|w| w.whole().is_none()));
        for i in 0..SECOND_TRIO.len() {
            assert_eq!(Ratio::new(w.at_zero[i], w.own[i]), w.weights[i]);
            for &x in &SECOND_TRIO {
                assert_eq!(
                    Ratio::new(model::helper_at(&SECOND_TRIO, i, i64::from(x)), w.own[i]),
                    Ratio::int(i64::from(x == SECOND_TRIO[i]))
                );
            }
        }
    }
}
