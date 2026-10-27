//! Fair independent spins preserve a prior; excluding straight lines leaks.
use super::draw::{Rail, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until};
use crate::model::{self, Dial, MARKS, PAIR, Recipe, TRUSTEES};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct WhatItPromises {
    pub kit: Kit,
}
const NOTE: &str = "With an honest dealer and fair, independent, secret spins over every mark, two shares preserve the prior chances. A large dial alone does not make a secret hard to guess.";
struct Working {
    rows: Vec<Recipe>,
    straight: Recipe,
    counts: Vec<usize>,
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let points = PAIR
        .iter()
        .map(|&x| (x, Recipe::example().share(d, x)))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for s in d.all() {
        let fits = model::fitting_spins(d, &points, s, 2);
        let [spins] = fits.as_slice() else {
            return Err("pair does not have exactly one recipe per secret".into());
        };
        rows.push(Recipe::new(s, spins));
    }
    let lines = rows.iter().filter(|r| r.spins[1] == 0).collect::<Vec<_>>();
    let [straight] = lines.as_slice() else {
        return Err("expected unique straight line".into());
    };
    let straight = (*straight).clone();
    let counts = rows.iter().map(|r| usize::from(r.spins[1] != 0)).collect::<Vec<_>>();
    let f = facts::facts()?;
    for (key, value) in [
        (
            "S-F7.spins_by_secret",
            rows.iter()
                .map(|r| format!("{}:{}", r.spins[0], r.spins[1]))
                .collect::<Vec<_>>()
                .join(","),
        ),
        ("S-F12.straight_recipe", model::csv(&straight.coefficients())),
        (
            "S-F12.straight_heights",
            model::csv(&PAIR.iter().map(|&x| straight.height(x)).collect::<Vec<_>>()),
        ),
        ("S-F12.nonzero_last.excluded", straight.secret.to_string()),
        ("S-F12.nonzero_last.left", counts.iter().sum::<usize>().to_string()),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}"));
        }
    }
    Ok(Working { rows, straight, counts })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
impl SceneConstruct for WhatItPromises {
    fn name(&self) -> &str {
        "21_what_it_promises"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::vault();
        say(&k, stage, "21_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.5)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&PAIR, palette::pair()), 0.3)?;
        let cards = PAIR
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
        fade_in_all(stage, &cards, 0.4)?;
        let title = text(
            &k,
            stage,
            &format!("The promise · {} marks", MARKS),
            32.,
            0.,
            3.2,
            Palette::ink(),
        )?;
        let recover = text(
            &k,
            stage,
            &format!("Any {} genuine shares: recover", model::THRESHOLD),
            28.,
            0.,
            2.2,
            palette::trio(),
        )?;
        let privacy = text(
            &k,
            stage,
            &format!("Any {} shares: no added information", PAIR.len()),
            28.,
            0.,
            1.3,
            palette::pair(),
        )?;
        fade_in_all(stage, &[title, recover, privacy], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;

        say(&k, stage, "21_2")?;
        fade_out_all(stage, &[recover, privacy], 0.4)?;
        let conditions = [
            "Honest dealer",
            "Fair spins · independent of each other and the secret",
            "Spins kept secret · every mark allowed, including 0",
        ]
        .iter()
        .enumerate()
        .map(|(i, s)| text(&k, stage, s, 26., 0., 2.3 - i as f64 * 0.9, Palette::ink()))
        .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &conditions, 0.7)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "21_3")?;
        fade_out_all(stage, &conditions, 0.4)?;
        let labels = [("secret", 2.15), ("ways", 1.45), ("spin a", 0.75), ("spin b", 0.05)]
            .iter()
            .map(|&(s, y)| text(&k, stage, s, 23., -5.6, y, Palette::dim()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &labels[..2], 0.4)?;
        let mut secrets = Vec::new();
        let mut counts = Vec::new();
        let mut spins = Vec::new();
        for (i, r) in w.rows.iter().enumerate() {
            let x = -3.9 + i as f64 * 0.88;
            secrets.push(text(&k, stage, &r.secret.to_string(), 28., x, 2.15, palette::secret())?);
            counts.push(text(&k, stage, "1", 26., x, 1.45, palette::pair())?);
            spins.push(text(&k, stage, &r.spins[0].to_string(), 25., x, 0.75, Palette::dim())?);
            spins.push(text(&k, stage, &r.spins[1].to_string(), 25., x, 0.05, Palette::dim())?);
        }
        fade_in_all(stage, &secrets, 0.5)?;
        fade_in_all(stage, &counts, 0.5)?;
        let likelihood = text(
            &k,
            stage,
            &format!("Chance of these cards, given any secret: 1/{}", MARKS * MARKS),
            26.,
            0.,
            0.45,
            Palette::ink(),
        )?;
        let prior = text(
            &k,
            stage,
            "Same likelihood → prior chances unchanged",
            26.,
            0.,
            -0.35,
            palette::pair(),
        )?;
        fade_in_all(stage, &[likelihood, prior], 0.5)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "21_4")?;
        fade_out_all(stage, &[likelihood, prior], 0.4)?;
        let rule = text(
            &k,
            stage,
            "Changed rule, known to trustees: second spin ≠ 0",
            26.,
            0.,
            -0.65,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[rule], 0.4)?;
        let question = think(&k, stage, v(3.2, -1.55))?;

        let spoken = say(&k, stage, "21_5")?;
        fade_out_all(stage, &[question], 0.2)?;
        fade_in_all(stage, &labels[2..], 0.4)?;
        fade_in_all(stage, &spins, 0.6)?;
        let excluded = w.straight.secret as usize;
        recolor_all(
            stage,
            &[
                (secrets[excluded], palette::trio()),
                (spins[excluded * 2], palette::trio()),
                (spins[excluded * 2 + 1], palette::trio()),
            ],
            0.4,
        )?;
        let equation = text(
            &k,
            stage,
            &format!("Line: {} + {}x", w.straight.secret, w.straight.spins[0]),
            27.,
            2.5,
            -1.25,
            palette::trio(),
        )?;
        fade_in_all(stage, &[equation], 0.4)?;
        spoken.wait_for(stage, "twenty-four")?;
        let evaluations = PAIR
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                text(
                    &k,
                    stage,
                    &format!("x={x}: {} → {}", w.straight.height(x), w.straight.share(d, x)),
                    26.,
                    1.0 + i as f64 * 3.,
                    -2.05,
                    palette::pair(),
                )
            })
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &evaluations[..1], 0.4)?;
        spoken.wait_for(stage, "twenty-eight")?;
        fade_in_all(stage, &evaluations[1..], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.5)?;

        let spoken = say(&k, stage, "21_6")?;
        spoken.wait_for(stage, "eight is impossible")?;
        fade_out_all(stage, &[counts[excluded]], 0.3)?;
        let zero = text(
            &k,
            stage,
            "0",
            26.,
            -3.9 + excluded as f64 * 0.88,
            1.45,
            palette::ghost(),
        )?;
        fade_in_all(stage, &[zero], 0.3)?;
        recolor_all(stage, &[(secrets[excluded], palette::ghost())], 0.3)?;
        fade_out_all(stage, &evaluations, 0.3)?;
        let verdict = text(
            &k,
            stage,
            &format!(
                "Secret {} excluded · {} still possible",
                w.straight.secret,
                w.counts.iter().sum::<usize>()
            ),
            25.,
            2.5,
            -2.05,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[verdict], 0.4)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "21_7")?;
        fade_out_all(stage, &[zero, rule, equation, verdict], 0.4)?;
        fade_in_all(stage, &[counts[excluded]], 0.3)?;
        recolor_all(stage, &[(secrets[excluded], palette::secret())], 0.3)?;
        pause(stage, 1.)?;
        let mut table = labels;
        table.extend(secrets);
        table.extend(counts);
        table.extend(spins);
        fade_out_all(stage, &table, 0.6)?;
        let uniform = text(&k, stage, "If the toy secret is uniform:", 28., 0., 2.3, Palette::ink())?;
        let guess = text(
            &k,
            stage,
            &format!("Best guess before = after = 1/{}", MARKS),
            29.,
            0.,
            1.4,
            palette::pair(),
        )?;
        let biased = text(&k, stage, "A biased prior stays biased", 27., 0., 0.5, Palette::ink())?;
        fade_in_all(stage, &[uniform, guess, biased], 0.6)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "21_8")?;
        fade_out_all(stage, &[uniform, guess, biased], 0.4)?;
        let real = text(
            &k,
            stage,
            "Large space + unpredictable secret + proper spins",
            27.,
            0.,
            2.1,
            Palette::ink(),
        )?;
        let limit = text(
            &k,
            stage,
            "Privacy preserves what was already guessable",
            27.,
            0.,
            1.1,
            palette::pair(),
        )?;
        fade_in_all(stage, &[real, limit], 0.5)?;
        let note = precisely(&k, stage, NOTE)?;
        let anim = fade_in(stage.arena_mut(), note, ORIGIN, 1.)?.rt(0.5);
        stage.play(anim)?;
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
    fn excluding_zero_second_spin_removes_only_the_straight_line_secret() {
        let w = working().unwrap();
        assert_eq!(w.straight.coefficients(), vec![8, 4, 0]);
        assert_eq!(w.counts.iter().sum::<usize>(), 10);
        assert_eq!(w.counts[8], 0);
        for (r, &n) in w.rows.iter().zip(&w.counts) {
            assert_eq!(n, usize::from(r.spins[1] != 0));
            assert_eq!(
                r.deal(Dial::vault(), &PAIR),
                Recipe::example().deal(Dial::vault(), &PAIR)
            );
        }
    }
}
