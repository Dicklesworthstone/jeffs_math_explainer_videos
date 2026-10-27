//! A false card: inconsistency is easier to detect than a culprit is to identify.
use super::draw::{Rail, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until};
use crate::model::{self, Dial, FOURTH, LIAR, Recipe, THRESHOLD, TRUSTEES};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct ALiar {
    pub kit: Kit,
}
const NOTE: &str = "Assume an honest dealer, known distinct trustee positions and at most one false share. Five submitted shares identify it here; basic secret sharing alone does not authenticate cards.";
struct Working {
    handed: Vec<(u32, u32)>,
    candidates: Vec<(u32, Vec<u32>)>,
    accepted: (u32, Vec<u32>),
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let handed = TRUSTEES
        .iter()
        .map(|&x| {
            (
                x,
                if x == LIAR.0 {
                    LIAR.1
                } else {
                    Recipe::example().share(d, x)
                },
            )
        })
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for suspect in &handed[..FOURTH as usize] {
        let rest = handed[..FOURTH as usize]
            .iter()
            .copied()
            .filter(|p| p != suspect)
            .collect::<Vec<_>>();
        let secret = model::rebuild(d, &rest).ok_or("candidate does not reconstruct")?;
        let along = TRUSTEES
            .iter()
            .map(|&x| model::interpolate(d, &rest, i64::from(x)))
            .collect::<Option<Vec<_>>>()
            .ok_or("candidate cannot predict")?;
        candidates.push((secret, along));
    }
    let near = model::curves_through(d, &handed, TRUSTEES.len() - 1);
    let [accepted] = near.as_slice() else {
        return Err("expected one curve through four".into());
    };
    let f = facts::facts()?;
    for (key, value) in [
        (
            "S-F12.four.secret_if_liar",
            model::csv(&candidates.iter().map(|r| r.0).collect::<Vec<_>>()),
        ),
        (
            "S-F12.four.candidates",
            candidates
                .iter()
                .map(|r| format!("{}:{}", r.0, model::csv(&r.1)))
                .collect::<Vec<_>>()
                .join(";"),
        ),
        ("S-F12.liar.predictions", model::csv(&candidates[FOURTH as usize - 1].1)),
        ("S-F12.five.curve_secret", accepted.0.to_string()),
        (
            "S-F12.five.min_common",
            (2 * (TRUSTEES.len() - 1) - TRUSTEES.len()).to_string(),
        ),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}"));
        }
    }
    Ok(Working {
        handed,
        candidates,
        accepted: accepted.clone(),
    })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
impl SceneConstruct for ALiar {
    fn name(&self) -> &str {
        "22_a_liar"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::vault();
        let spoken = say(&k, stage, "22_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.5)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&TRUSTEES[..THRESHOLD], palette::pair()), 0.3)?;
        let cards = TRUSTEES
            .iter()
            .map(|&x| {
                card(
                    &k,
                    stage,
                    x,
                    &Recipe::example().share(d, x).to_string(),
                    palette::pair(),
                )
            })
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards[..THRESHOLD], 0.4)?;
        let title = text(&k, stage, "A false card", 32., 0., 3.2, Palette::ink())?;
        let setup = text(
            &k,
            stage,
            "Honest dealer · at most one false share",
            27.,
            0.,
            2.25,
            Palette::ink(),
        )?;
        let question = text(
            &k,
            stage,
            "Does a wrong answer reveal the liar?",
            27.,
            0.,
            1.3,
            palette::pair(),
        )?;
        fade_in_all(stage, &[title, setup, question], 0.5)?;
        spoken.wait_for(stage, "hands in two instead of one")?;
        fade_out_all(stage, &[cards[LIAR.0 as usize - 1]], 0.3)?;
        let false_card = card(&k, stage, LIAR.0, &LIAR.1.to_string(), palette::pair())?;
        fade_in_all(stage, &[false_card], 0.3)?;
        let changed = text(
            &k,
            stage,
            &format!(
                "Trustee {}: genuine {} → submitted {}",
                LIAR.0,
                Recipe::example().share(d, LIAR.0),
                LIAR.1
            ),
            27.,
            0.,
            0.3,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[changed], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "22_2")?;
        fade_out_all(stage, &[setup, question, changed], 0.4)?;
        let points = &w.handed[..THRESHOLD];
        let terms = model::paper_terms(&points.iter().map(|&(x, y)| (x, i64::from(y))).collect::<Vec<_>>());
        let sum = model::ordinary_sum(points);
        let result = model::rebuild(d, points).ok_or_else(|| fail("no reconstruction".into()))?;
        let weights = text(&k, stage, "Same weights: 3, −3, 1", 28., 0., 2.2, Palette::ink())?;
        let calculation = text(
            &k,
            stage,
            &format!("{} + ({}) + {} = {} → {}", terms[0], terms[1], terms[2], sum, result),
            34.,
            0.,
            1.2,
            palette::secret(),
        )?;
        let fits = text(
            &k,
            stage,
            "Any three submitted values fit a quadratic on this dial",
            25.,
            0.,
            0.2,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[weights, calculation, fits], 0.6)?;
        k.hold(stage)?;
        pause(stage, 2.5)?;

        let spoken = say(&k, stage, "22_3")?;
        let external = text(&k, stage, "Toy vault's external check", 27., 2.4, -1.1, Palette::ink())?;
        fade_in_all(stage, &[external], 0.4)?;
        spoken.wait_for(stage, "rejects six")?;
        let alarm = text(
            &k,
            stage,
            &format!("{} ≠ {} · ALARM", result, Recipe::example().secret),
            30.,
            2.4,
            -2.0,
            palette::leak(),
        )?;
        fade_in_all(stage, &[alarm], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        let spoken = say(&k, stage, "22_4")?;
        fade_out_all(stage, &[weights, calculation, fits, external, alarm], 0.5)?;
        recolor_all(stage, &rail.tint(&[FOURTH], palette::pair()), 0.3)?;
        fade_in_all(stage, &[cards[FOURTH as usize - 1]], 0.4)?;
        let heading = text(
            &k,
            stage,
            "Compare a prediction with the next card",
            28.,
            0.,
            2.25,
            Palette::ink(),
        )?;
        let predicted = text(
            &k,
            stage,
            &format!(
                "First three submitted cards predict: x={} → {}",
                FOURTH,
                w.candidates[FOURTH as usize - 1].1[FOURTH as usize - 1]
            ),
            27.,
            0.,
            1.3,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[heading, predicted], 0.5)?;
        spoken.wait_for(stage, "actually holds two")?;
        let actual = text(
            &k,
            stage,
            &format!("Actual card: x={} → {}", FOURTH, w.handed[FOURTH as usize - 1].1),
            29.,
            0.,
            0.4,
            palette::pair(),
        )?;
        let inconsistency = text(
            &k,
            stage,
            "No quadratic fits all four · culprit still unknown",
            26.,
            0.,
            -0.45,
            palette::leak(),
        )?;
        fade_in_all(stage, &[actual, inconsistency], 0.5)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "22_5")?;
        fade_out_all(stage, &[heading, predicted, actual, inconsistency], 0.5)?;
        let mut table = Vec::new();
        for (label, x) in [("Suppose liar", -5.3), ("Secret", -3.5)] {
            table.push(text(&k, stage, label, 23., x, 2.3, Palette::dim())?);
        }
        let column_x = |i: usize| -1.8 + i as f64 * 1.65;
        for (i, &x) in TRUSTEES.iter().enumerate().take(FOURTH as usize) {
            table.push(text(
                &k,
                stage,
                &format!("x={x}"),
                23.,
                column_x(i),
                2.3,
                Palette::dim(),
            )?);
        }
        fade_in_all(stage, &table, 0.4)?;
        let mut row_mobs = Vec::new();
        for (i, (secret, along)) in w.candidates.iter().enumerate() {
            let y = 1.5 - i as f64 * 0.7;
            let mut row = vec![
                text(&k, stage, &(i + 1).to_string(), 26., -5.3, y, Palette::ink())?,
                text(&k, stage, &secret.to_string(), 26., -3.5, y, palette::secret())?,
            ];
            for (j, value) in along.iter().enumerate().take(FOURTH as usize) {
                row.push(text(
                    &k,
                    stage,
                    &value.to_string(),
                    26.,
                    column_x(j),
                    y,
                    if i == j { Palette::dim() } else { palette::pair() },
                )?);
            }
            fade_in_all(stage, &row, 0.45)?;
            pause(stage, 0.5)?;
            table.extend(&row);
            row_mobs.push(row);
        }
        let meaning = text(
            &k,
            stage,
            "Each row agrees with the other three cards",
            24.,
            2.2,
            -1.65,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[meaning], 0.4)?;
        k.hold(stage)?;
        pause(stage, 4.)?;

        say(&k, stage, "22_6")?;
        recolor_all(stage, &rail.tint(&[TRUSTEES[4]], palette::pair()), 0.3)?;
        fade_in_all(stage, &[cards[4]], 0.4)?;
        fade_out_all(stage, &[meaning], 0.3)?;
        let question = think(&k, stage, v(2.5, -1.7))?;

        let spoken = say(&k, stage, "22_7")?;
        fade_out_all(stage, &[question], 0.2)?;
        let fifth_header = text(
            &k,
            stage,
            &format!("x={}", TRUSTEES[4]),
            23.,
            column_x(4),
            2.3,
            Palette::dim(),
        )?;
        table.push(fifth_header);
        fade_in_all(stage, &[fifth_header], 0.3)?;
        let mut fifth = Vec::new();
        for (i, (_, along)) in w.candidates.iter().enumerate() {
            let m = text(
                &k,
                stage,
                &along[4].to_string(),
                26.,
                column_x(4),
                1.5 - i as f64 * 0.7,
                Palette::ink(),
            )?;
            fifth.push(m);
            table.push(m);
            row_mobs[i].push(m);
        }
        fade_in_all(stage, &fifth, 0.6)?;
        spoken.wait_for(stage, "Only the dealer's curve")?;
        let winner = LIAR.0 as usize - 1;
        for (i, row) in row_mobs.iter().enumerate() {
            recolor_all(
                stage,
                &row.iter()
                    .map(|&m| (m, if i == winner { palette::pair() } else { Palette::dim() }))
                    .collect::<Vec<_>>(),
                0.3,
            )?;
        }
        let recovered = text(
            &k,
            stage,
            &format!(
                "Four matches · secret {} · false card at trustee {}",
                w.accepted.0, LIAR.0
            ),
            24.,
            2.2,
            -1.7,
            palette::pair(),
        )?;
        fade_in_all(stage, &[recovered], 0.4)?;
        recolor_all(stage, &[(false_card, palette::leak())], 0.4)?;
        k.hold(stage)?;
        pause(stage, 4.)?;

        let spoken = say(&k, stage, "22_8")?;
        fade_out_all(stage, &table, 0.5)?;
        fade_out_all(stage, &[recovered], 0.3)?;
        let why = text(
            &k,
            stage,
            "Could two different curves each fit four cards?",
            27.,
            0.,
            2.35,
            Palette::ink(),
        )?;
        let mut proof = vec![why];
        fade_in_all(stage, &proof, 0.4)?;
        let groups = [
            TRUSTEES[1..].to_vec(),
            vec![TRUSTEES[0], TRUSTEES[2], TRUSTEES[3], TRUSTEES[4]],
        ];
        for (i, group) in groups.iter().enumerate() {
            let y = 1.5 - i as f64 * 0.85;
            let label = text(
                &k,
                stage,
                if i == 0 {
                    "Hypothetical A fits"
                } else {
                    "Hypothetical B fits"
                },
                23.,
                -3.9,
                y,
                Palette::dim(),
            )?;
            proof.push(label);
            fade_in_all(stage, &[label], 0.3)?;
            for &x in &TRUSTEES {
                let m = text(
                    &k,
                    stage,
                    &x.to_string(),
                    27.,
                    -0.4 + f64::from(x - 1) * 1.15,
                    y,
                    if group.contains(&x) {
                        palette::pair()
                    } else {
                        palette::ghost()
                    },
                )?;
                proof.push(m);
                fade_in_all(stage, &[m], 0.15)?;
            }
        }
        spoken.wait_for(stage, "at least three positions")?;
        let common = groups[0]
            .iter()
            .filter(|x| groups[1].contains(x))
            .copied()
            .collect::<Vec<_>>();
        let overlap = text(
            &k,
            stage,
            &format!(
                "Shared here: {} · minimum 4 + 4 − 5 = {}",
                model::csv(&common),
                common.len()
            ),
            25.,
            0.,
            -0.3,
            Palette::ink(),
        )?;
        proof.push(overlap);
        fade_in_all(stage, &[overlap], 0.5)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "22_9")?;
        fade_out_all(stage, &proof, 0.5)?;
        let same = text(
            &k,
            stage,
            "Three common points → the same quadratic",
            28.,
            0.,
            2.1,
            palette::pair(),
        )?;
        let unique = text(
            &k,
            stage,
            "At most one false card → at least four honest cards",
            26.,
            0.,
            1.1,
            Palette::ink(),
        )?;
        let returned = text(
            &k,
            stage,
            &format!(
                "Received: {} · genuine: {}",
                model::csv(&w.handed.iter().map(|p| p.1).collect::<Vec<_>>()),
                model::csv(&w.accepted.1)
            ),
            26.,
            0.,
            0.1,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[same, unique, returned], 0.6)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "22_10")?;
        fade_out_all(stage, &[same, unique, returned], 0.4)?;
        let limit = text(&k, stage, "Privacy is not authentication", 29., 0., 2.1, Palette::ink())?;
        let recovered = text(
            &k,
            stage,
            &format!(
                "This example: secret {} · trustee {} supplied the false card",
                w.accepted.0, LIAR.0
            ),
            26.,
            0.,
            1.1,
            palette::pair(),
        )?;
        fade_in_all(stage, &[limit, recovered], 0.5)?;
        let note = precisely(&k, stage, NOTE)?;
        let a = fade_in(stage.arena_mut(), note, ORIGIN, 1.)?.rt(0.5);
        stage.play(a)?;
        let deadline = read_by(stage, NOTE);
        k.hold(stage)?;
        wait_until(stage, deadline)?;
        pause(stage, 0.6)?;
        k.finish(stage)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fourth_card_detects_but_fifth_card_identifies_the_single_false_card() {
        let w = working().unwrap();
        assert_eq!(w.candidates.iter().map(|r| r.0).collect::<Vec<_>>(), vec![8, 3, 5, 6]);
        for (i, (_, along)) in w.candidates.iter().enumerate() {
            assert_eq!(
                along[..4]
                    .iter()
                    .zip(&w.handed[..4])
                    .filter(|(a, p)| **a == p.1)
                    .count(),
                3
            );
            assert_ne!(along[i], w.handed[i].1);
        }
        assert_eq!(w.accepted, (5, vec![8, 3, 1, 2, 6]));
        assert_eq!(w.candidates.iter().filter(|r| r.1[4] == w.handed[4].1).count(), 1);
    }
}
