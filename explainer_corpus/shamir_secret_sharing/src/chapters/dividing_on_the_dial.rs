//! Undo multiplication with its partner, keeping the original shares in view.
use super::draw::{Rail, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until};
use super::envelope::outline;
use crate::model::{self, Dial, Recipe, SECOND_TRIO, SECRET, TEN, TRUSTEES};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct DividingOnTheDial {
    pub kit: Kit,
}
const NOTE: &str = "Division on a dial undoes multiplication using a partner whose product lands on one. Every nonzero mark has a partner on eleven marks; not every dial allows that.";
struct Working {
    shares: Vec<u32>,
    clear: u32,
    cleared: Vec<i64>,
    total: i64,
    lands: u32,
    partner: u32,
    products: Vec<i64>,
    weights: Vec<u32>,
    terms: Vec<u32>,
    sum: u32,
    row: Vec<u32>,
    partners: Vec<(u32, u32)>,
    missing: Vec<u32>,
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let r = Recipe::example();
    let shares = SECOND_TRIO.iter().map(|&x| r.share(d, x)).collect::<Vec<_>>();
    let (clear, cleared) = model::cleared_weights(&SECOND_TRIO);
    let clear = u32::try_from(clear).map_err(|_| "negative multiplier")?;
    let total = cleared.iter().zip(&shares).map(|(w, &s)| w * i64::from(s)).sum();
    let lands = d.wrap(total);
    let partner = d.inverse(clear).ok_or("no partner")?;
    let ordinary = model::ordinary_weights(&SECOND_TRIO);
    let products = ordinary
        .iter()
        .map(|w| {
            // Use each fraction's own denominator; -5 is already whole.
            match w.whole() {
                Some(n) => Ok(n),
                None => Ok(w.times(i64::from(clear)).whole().ok_or("uncleared fraction")? * i64::from(partner)),
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    let weights = model::weights_at(d, &SECOND_TRIO, 0).ok_or("no weights")?;
    let terms = weights.iter().zip(&shares).map(|(&w, &s)| w * s).collect::<Vec<_>>();
    let sum = terms.iter().sum::<u32>();
    let row = d.times_row(clear);
    let partners = (1..d.marks())
        .map(|x| d.inverse(x).map(|p| (x, p)).ok_or("missing partner"))
        .collect::<Result<Vec<_>, _>>()?;
    let ten = Dial::new(TEN);
    let missing = (1..TEN).filter(|&x| ten.inverse(x).is_none()).collect::<Vec<_>>();
    let f = facts::facts()?;
    for (key, value) in [
        ("S-F10.trio_shares", model::csv(&shares)),
        ("S-F10.clear", clear.to_string()),
        ("S-F10.cleared_weights", model::csv(&cleared)),
        ("S-F10.cleared_sum", total.to_string()),
        ("S-F10.cleared_lands", lands.to_string()),
        ("S-F10.third", partner.to_string()),
        ("S-F10.weight_products", model::csv(&products)),
        ("S-F10.weights_on_dial", model::csv(&weights)),
        ("S-F10.dial_terms", model::csv(&terms)),
        ("S-F10.sum_unwrapped", sum.to_string()),
        ("S-F10.times3", model::csv(&row)),
        ("S-F10.ten.noninvertible", model::csv(&missing)),
        ("S-F10.partner_product", (clear * partner).to_string()),
        ("S-F10.undo_product", (lands * partner).to_string()),
        ("S-F10.whole_turns", (sum / d.marks()).to_string()),
        (
            "S-F10.inverses",
            partners
                .iter()
                .map(|(a, b)| format!("{a}:{b}"))
                .collect::<Vec<_>>()
                .join(","),
        ),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}: {value}"));
        }
    }
    if products.iter().map(|&x| d.wrap(x)).collect::<Vec<_>>() != weights
        || d.mul(lands, partner) != SECRET
        || d.wrap(i64::from(sum)) != SECRET
    {
        return Err("inverse reconstruction changed".into());
    }
    Ok(Working {
        shares,
        clear,
        cleared,
        total,
        lands,
        partner,
        products,
        weights,
        terms,
        sum,
        row,
        partners,
        missing,
    })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
impl SceneConstruct for DividingOnTheDial {
    fn name(&self) -> &str {
        "18_dividing_on_the_dial"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::vault();
        let spoken = say(&k, stage, "18_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&SECOND_TRIO, palette::trio()), 0.3)?;
        let cards = SECOND_TRIO
            .iter()
            .zip(&w.shares)
            .map(|(&x, &s)| card(&k, stage, x, &s.to_string(), palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards, 0.5)?;
        let rail_note = text(
            &k,
            stage,
            "Same trustees; original shares",
            20.0,
            -3.9,
            -2.75,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[rail_note], 0.3)?;
        let title = text(&k, stage, "Undo multiplication", 32.0, 0.0, 3.1, Palette::ink())?;
        fade_in_all(stage, &[title], 0.4)?;
        spoken.wait_for(stage, "Ten times three")?;
        let expr = w
            .cleared
            .iter()
            .zip(&w.shares)
            .map(|(a, b)| format!("({a}) × {b}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let sum = text(&k, stage, &expr, 32.0, 0.0, 1.9, Palette::ink())?;
        fade_in_all(stage, &[sum], 0.5)?;
        spoken.wait_for(stage, "forty-eight")?;
        let answer = text(
            &k,
            stage,
            &format!("= {} → {}     undo × {} → {}", w.total, w.lands, w.clear, SECRET),
            33.0,
            0.0,
            0.9,
            palette::secret(),
        )?;
        fade_in_all(stage, &[answer], 0.5)?;
        k.hold(stage)?;

        say(&k, stage, "18_2")?;
        fade_out_all(stage, &[sum, answer], 0.3)?;
        let il = text(&k, stage, "mark", 20.0, -6.2, 2.05, Palette::dim())?;
        let ol = text(&k, stage, &format!("× {}", w.clear), 20.0, -6.2, 0.9, Palette::dim())?;
        let target = text(
            &k,
            stage,
            &format!("{} × ? lands on {}", w.clear, d.mul(w.clear, w.partner)),
            31.0,
            1.7,
            -0.25,
            palette::secret(),
        )?;
        let mut table = vec![il, ol, target];
        let mut outputs = Vec::new();
        for (i, &n) in w.row.iter().enumerate() {
            let x = -5.0 + i as f64;
            table.push(text(&k, stage, &i.to_string(), 29.0, x, 2.05, Palette::ink())?);
            table.push(outline(stage, v(x, 0.9), 0.73, 0.65, Palette::dim())?);
            outputs.push(text(&k, stage, &n.to_string(), 30.0, x, 0.9, palette::trio())?);
        }
        fade_in_all(stage, &table, 0.6)?;
        let question = think(&k, stage, v(2.3, -1.2))?;
        let spoken = say(&k, stage, "18_3")?;
        fade_out_all(stage, &[question, target], 0.25)?;
        let chosen = outline(stage, v(-5.0 + f64::from(w.partner), 1.5), 0.86, 1.9, palette::secret())?;
        fade_in_all(stage, &[chosen, outputs[w.partner as usize]], 0.5)?;
        let partner_eq = text(
            &k,
            stage,
            &format!(
                "{} × {} = {} → {}",
                w.clear,
                w.partner,
                w.clear * w.partner,
                d.mul(w.clear, w.partner)
            ),
            31.0,
            2.0,
            -0.3,
            palette::secret(),
        )?;
        fade_in_all(stage, &[partner_eq], 0.5)?;
        spoken.wait_for(stage, "back where you started")?;
        let identity = text(
            &k,
            stage,
            &format!(
                "× {} then × {}  =  × {} on this dial",
                w.clear,
                w.partner,
                d.mul(w.clear, w.partner)
            ),
            25.0,
            2.1,
            -1.0,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[identity], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.2)?;

        let spoken = say(&k, stage, "18_4")?;
        let others = outputs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != w.partner as usize)
            .map(|(_, &m)| m)
            .collect::<Vec<_>>();
        fade_in_all(stage, &others, 0.6)?;
        fade_out_all(stage, &[chosen, partner_eq, identity], 0.3)?;
        let solution = outline(stage, v(-5.0 + f64::from(SECRET), 1.5), 0.86, 1.9, palette::secret())?;
        fade_in_all(stage, &[solution], 0.4)?;
        let undo = text(
            &k,
            stage,
            &format!(
                "target {} × {} = {} → {}",
                w.lands,
                w.partner,
                w.lands * w.partner,
                SECRET
            ),
            30.0,
            2.1,
            -0.3,
            palette::secret(),
        )?;
        fade_in_all(stage, &[undo], 0.5)?;
        spoken.wait_for(stage, "dividing by three")?;
        let rule = text(
            &k,
            stage,
            &format!("÷ {} = × {}   on {} marks", w.clear, w.partner, d.marks()),
            27.0,
            2.1,
            -1.0,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.0)?;

        let spoken = say(&k, stage, "18_5")?;
        table.extend(outputs);
        table.extend([solution, undo, rule]);
        fade_out_all(stage, &table, 0.4)?;
        let ordinary = model::ordinary_weights(&SECOND_TRIO);
        let phrases = ["Ten thirds", "Minus five", "Eight thirds"];
        let mut conversions = Vec::new();
        for (i, phrase) in phrases.iter().enumerate() {
            spoken.wait_for(stage, phrase)?;
            let conversion = if ordinary[i].whole().is_some() {
                format!("{}  →  {}", ordinary[i], w.weights[i])
            } else {
                format!(
                    "{}  →  {} × {} = {}  →  {}",
                    ordinary[i],
                    w.products[i] / i64::from(w.partner),
                    w.partner,
                    w.products[i],
                    w.weights[i]
                )
            };
            let row = text(
                &k,
                stage,
                &conversion,
                30.0,
                0.0,
                2.05 - i as f64 * 0.9,
                palette::trio(),
            )?;
            fade_in_all(stage, &[row], 0.5)?;
            conversions.push(row);
        }
        k.hold(stage)?;
        pause(stage, 2.0)?;

        let spoken = say(&k, stage, "18_6")?;
        fade_out_all(stage, &conversions, 0.4)?;
        let expression = w
            .weights
            .iter()
            .zip(&w.shares)
            .map(|(a, b)| format!("{a} × {b}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let weighted = text(&k, stage, &expression, 34.0, 0.0, 2.0, Palette::ink())?;
        fade_in_all(stage, &[weighted], 0.5)?;
        let terms = text(
            &k,
            stage,
            &format!("{} = {}", model::csv(&w.terms).replace(',', " + "), w.sum),
            34.0,
            0.0,
            1.1,
            palette::trio(),
        )?;
        spoken.wait_for(stage, "ninety-three")?;
        fade_in_all(stage, &[terms], 0.5)?;
        let secret = text(
            &k,
            stage,
            &format!(
                "{} = {} × {} + {}  →  {}",
                w.sum,
                w.sum / d.marks(),
                d.marks(),
                SECRET,
                SECRET
            ),
            34.0,
            0.0,
            0.2,
            palette::secret(),
        )?;
        spoken.wait_for(stage, "eight elevens")?;
        fade_in_all(stage, &[secret], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;

        say(&k, stage, "18_7")?;
        fade_out_all(stage, &[weighted, terms, secret, title], 0.4)?;
        let heading = text(
            &k,
            stage,
            &format!("Partners on {} marks", d.marks()),
            30.0,
            -2.9,
            3.1,
            Palette::ink(),
        )?;
        let mut partners = vec![heading];
        for (i, (a, b)) in w.partners.iter().enumerate() {
            let col = i / 5;
            let row = i % 5;
            partners.push(text(
                &k,
                stage,
                &format!("{a} × {b} → {}", d.mul(*a, *b)),
                25.0,
                -5.0 + col as f64 * 3.0,
                2.3 - row as f64 * 0.6,
                palette::trio(),
            )?);
        }
        fade_in_all(stage, &partners, 0.8)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;
        say(&k, stage, "18_8")?;
        let ten = Dial::new(TEN);
        let title = text(&k, stage, &format!("On {} marks", TEN), 30.0, 3.1, 3.1, Palette::ink())?;
        let yes = (1..TEN).filter(|&a| ten.inverse(a).is_some()).collect::<Vec<_>>();
        let good = text(
            &k,
            stage,
            &format!("partners: {}", model::csv(&yes).replace(',', ", ")),
            24.0,
            3.1,
            2.1,
            palette::trio(),
        )?;
        let bad = text(
            &k,
            stage,
            &format!("none: {}", model::csv(&w.missing).replace(',', ", ")),
            24.0,
            3.1,
            1.35,
            palette::leak(),
        )?;
        let zero = text(
            &k,
            stage,
            "Zero has no partner on either dial",
            21.0,
            2.5,
            0.3,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[title, good, bad, zero], 0.7)?;
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
    fn partners_undo_every_mark_and_weights_recover_every_dealing() {
        let w = working().unwrap();
        let d = Dial::vault();
        for (a, b) in &w.partners {
            for x in d.all() {
                assert_eq!(d.mul(*b, d.mul(*a, x)), x);
            }
        }
        for secret in d.all() {
            for spins in model::tuples(d, 2) {
                let r = Recipe::new(secret, &spins);
                let total = w
                    .weights
                    .iter()
                    .zip(SECOND_TRIO)
                    .map(|(&a, x)| i64::from(a) * i64::from(r.share(d, x)))
                    .sum();
                assert_eq!(d.wrap(total), secret);
            }
        }
        for a in w.missing {
            assert!(Dial::new(TEN).all().all(|x| Dial::new(TEN).mul(a, x) != 1));
        }
    }
}
