//! Closing callback and a computed four-of-seven extension on the same dial.
use super::board::{Board, Grid};
use super::draw::{
    Rail, Vault, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until,
};
use crate::model::{
    self, Dial, FOUR_OF_SEVEN, FOUR_OF_SEVEN_SEEN, MARKS, OPENING_TRIO, PAIR, Recipe, SECRET, TRUSTEES,
};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct ThreeKeysTwoNothing {
    pub kit: Kit,
}
const NOTE: &str = "Honest dealer; genuine shares at distinct nonzero positions; secret, fair, independent spins including zero: three recover the secret; two leave prior knowledge unchanged.";
struct Working {
    pair: Vec<Recipe>,
    cubics: Vec<Recipe>,
    example: Recipe,
    xs: Vec<u32>,
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let pair_points = PAIR
        .iter()
        .map(|&x| (x, Recipe::example().share(d, x)))
        .collect::<Vec<_>>();
    let make = |points: &[(u32, u32)], count: usize| -> Result<Vec<Recipe>, String> {
        d.all()
            .map(|s| {
                let fits = model::fitting_spins(d, points, s, count);
                match fits.as_slice() {
                    [spins] => Ok(Recipe::new(s, spins)),
                    _ => Err("expected one recipe per secret".into()),
                }
            })
            .collect()
    };
    let pair = make(&pair_points, OPENING_TRIO.len() - 1)?;
    let cubics = make(&FOUR_OF_SEVEN_SEEN, FOUR_OF_SEVEN.0 - 1)?;
    let example = cubics
        .iter()
        .find(|r| r.secret == SECRET)
        .ok_or("missing example")?
        .clone();
    let xs = (1..=FOUR_OF_SEVEN.1 as u32).collect::<Vec<_>>();
    let f = facts::facts()?;
    for (key, value) in [
        (
            "S-F1.opening_shares",
            model::csv(&Recipe::example().deal(d, &OPENING_TRIO)),
        ),
        ("S-F13.curves_left", pair.len().to_string()),
        ("S-F13.four_of_seven.recipe", model::csv(&example.coefficients())),
        ("S-F13.four_of_seven.cards", model::csv(&example.deal(d, &xs))),
        (
            "S-F13.four_of_seven.fourth_by_secret",
            model::csv(
                &cubics
                    .iter()
                    .map(|r| r.share(d, FOUR_OF_SEVEN.0 as u32))
                    .collect::<Vec<_>>(),
            ),
        ),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture differs from {key}"));
        }
    }
    Ok(Working {
        pair,
        cubics,
        example,
        xs,
    })
}
fn txt(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
impl SceneConstruct for ThreeKeysTwoNothing {
    fn name(&self) -> &str {
        "23_three_keys_two_nothing"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::vault();
        let spoken = say(&k, stage, "23_1")?;
        let vault = Vault::build(&k, stage, d)?;
        vault.show(stage, 0.5)?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.5)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&OPENING_TRIO, palette::trio()), 0.3)?;
        let cards = TRUSTEES
            .iter()
            .map(|&x| {
                card(
                    &k,
                    stage,
                    x,
                    &Recipe::example().share(d, x).to_string(),
                    if PAIR.contains(&x) {
                        palette::pair()
                    } else {
                        palette::trio()
                    },
                )
            })
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards[..OPENING_TRIO.len()], 0.4)?;
        let board = Board::build(
            &k,
            stage,
            Grid {
                x0: 0.4,
                y0: 0.05,
                dx: 1.05,
                dy: 0.24,
                marks: MARKS,
                last: 5,
            },
            "Dial recipes · dots only",
            true,
        )?;
        board.show(stage, 0.5)?;
        let dots = (0..=5)
            .map(|x| {
                board.point(
                    stage,
                    x,
                    Recipe::example().share(d, x),
                    if x == 0 { palette::secret() } else { palette::trio() },
                )
            })
            .collect::<Vec<_>>();
        fade_in_all(stage, &dots[1..4], 0.5)?;
        spoken.wait_for(stage, "five")?;
        fade_in_all(stage, &dots[..1], 0.4)?;
        vault.turn(stage, SECRET, 0.7)?;
        recolor_all(
            stage,
            &[(vault.door, palette::open()), (vault.lamp, palette::open())],
            0.5,
        )?;
        let opening = txt(
            &k,
            stage,
            &format!("{} genuine cards → secret {}", OPENING_TRIO.len(), SECRET),
            27.,
            3.3,
            -0.85,
            palette::trio(),
        )?;
        fade_in_all(stage, &[opening], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "23_2")?;
        recolor_all(
            stage,
            &[(vault.door, palette::ghost()), (vault.lamp, palette::ghost())],
            0.4,
        )?;
        vault.turn(stage, 0, 0.4)?;
        fade_out_all(stage, &[opening, dots[0], dots[1], dots[2], dots[3]], 0.4)?;
        recolor_all(stage, &rail.tint(&OPENING_TRIO, palette::ghost()), 0.3)?;
        recolor_all(stage, &rail.tint(&PAIR, palette::pair()), 0.3)?;
        fade_in_all(stage, &cards[OPENING_TRIO.len()..], 0.4)?;
        let pairdots = PAIR
            .iter()
            .map(|&x| board.point(stage, x, Recipe::example().share(d, x), palette::pair()))
            .collect::<Vec<_>>();
        fade_in_all(stage, &pairdots, 0.4)?;
        let mut family = Vec::new();
        for r in &w.pair {
            let row = (0..=5)
                .filter(|x| !PAIR.contains(x))
                .map(|x| {
                    board.point(
                        stage,
                        x,
                        r.share(d, x),
                        if x == 0 { palette::secret() } else { Palette::dim() },
                    )
                })
                .collect::<Vec<_>>();
            fade_in_all(stage, &row, 0.12)?;
            family.extend(row);
        }
        let count = txt(
            &k,
            stage,
            &format!("{} recipes · one for each secret", w.pair.len()),
            26.,
            3.3,
            -0.85,
            palette::pair(),
        )?;
        fade_in_all(stage, &[count], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "23_3")?;
        let equal = txt(
            &k,
            stage,
            &format!("Each secret: one of {} spin pairs", MARKS.pow(2)),
            25.,
            3.3,
            -1.55,
            Palette::ink(),
        )?;
        let prior = txt(
            &k,
            stage,
            "Equal likelihood → prior unchanged",
            24.,
            3.3,
            -2.2,
            palette::pair(),
        )?;
        fade_in_all(stage, &[equal, prior], 0.5)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "23_4")?;
        let mut gone = family;
        gone.extend(pairdots);
        gone.extend_from_slice(board.parts());
        gone.extend([count, equal, prior]);
        fade_out_all(stage, &gone, 0.5)?;
        fade_out_all(stage, &cards, 0.4)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::share()), 0.3)?;
        let extra = Rail::build(&k, stage, &w.xs[TRUSTEES.len()..])?;
        extra.reveal(stage, 0.5)?;
        let rule = txt(
            &k,
            stage,
            &format!("Another rule: any {} of {}", FOUR_OF_SEVEN.0, FOUR_OF_SEVEN.1),
            31.,
            3.1,
            2.4,
            Palette::ink(),
        )?;
        let question = txt(&k, stage, "How many secret spins?", 29., 3.1, 1.3, palette::spin())?;
        fade_in_all(stage, &[rule, question], 0.5)?;
        let think = think(&k, stage, v(3.2, -0.2))?;

        say(&k, stage, "23_5")?;
        fade_out_all(stage, &[question, think], 0.3)?;
        let answer = txt(
            &k,
            stage,
            &format!("{} spins · one fewer than {}", FOUR_OF_SEVEN.0 - 1, FOUR_OF_SEVEN.0),
            28.,
            3.1,
            1.35,
            palette::spin(),
        )?;
        let formula = txt(&k, stage, "secret + ax + bx² + cx³", 28., 3.1, 0.45, Palette::ink())?;
        let degree = txt(
            &k,
            stage,
            &format!("Degree at most {} · a, b, c may be zero", FOUR_OF_SEVEN.0 - 1),
            23.,
            3.1,
            -0.35,
            Palette::dim(),
        )?;
        let spin_count = txt(
            &k,
            stage,
            &format!(
                "Fair independent spins: {} possibilities",
                MARKS.pow((FOUR_OF_SEVEN.0 - 1) as u32)
            ),
            23.,
            3.1,
            -1.15,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[answer, formula, degree, spin_count], 0.6)?;
        k.hold(stage)?;
        pause(stage, 2.5)?;

        say(&k, stage, "23_6")?;
        fade_out_all(stage, &[answer, formula, degree, spin_count], 0.4)?;
        let extension_cards =
            w.xs.iter()
                .map(|&x| card(&k, stage, x, &w.example.share(d, x).to_string(), palette::share()))
                .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &extension_cards[..FOUR_OF_SEVEN_SEEN.len()], 0.5)?;
        let mut table = Vec::new();
        let mut fourths = Vec::new();
        for (label, y) in [("secret", 1.65), ("a", 1.05), ("b", 0.45), ("c", -0.15)] {
            table.push(txt(&k, stage, label, 19., 0.15, y, Palette::dim())?);
        }
        for (i, r) in w.cubics.iter().enumerate() {
            let x = 0.85 + i as f64 * 0.53;
            for (j, n) in r.coefficients().iter().enumerate() {
                table.push(txt(
                    &k,
                    stage,
                    &n.to_string(),
                    21.,
                    x,
                    1.65 - j as f64 * 0.6,
                    if j == 0 { palette::secret() } else { Palette::dim() },
                )?);
            }
            fourths.push(txt(
                &k,
                stage,
                &r.share(d, FOUR_OF_SEVEN.0 as u32).to_string(),
                21.,
                x,
                -0.95,
                Palette::ink(),
            )?);
        }
        fade_in_all(stage, &table, 0.7)?;
        let all = txt(
            &k,
            stage,
            &format!("{} recipes still fit · one per secret", w.cubics.len()),
            24.,
            3.3,
            -1.8,
            palette::pair(),
        )?;
        fade_in_all(stage, &[all], 0.4)?;
        k.hold(stage)?;
        pause(stage, 4.)?;

        let spoken = say(&k, stage, "23_7")?;
        fade_in_all(stage, &extension_cards[FOUR_OF_SEVEN_SEEN.len()..FOUR_OF_SEVEN.0], 0.4)?;
        let prediction = txt(
            &k,
            stage,
            &format!("x={}", FOUR_OF_SEVEN.0),
            19.,
            0.15,
            -0.95,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[prediction], 0.4)?;
        fade_in_all(stage, &fourths, 0.6)?;
        spoken.wait_for(stage, "secret five")?;
        let winner = w.example.secret as usize;
        let halo = super::board::ring(stage, v(0.85 + winner as f64 * 0.53, -0.95), 0.19, palette::trio());
        fade_in_all(stage, &[halo], 0.4)?;
        fade_out_all(stage, &[all], 0.3)?;
        let selected = txt(
            &k,
            stage,
            &format!("Matching column: secret {}", w.example.secret),
            25.,
            3.3,
            -1.8,
            palette::trio(),
        )?;
        fade_in_all(stage, &[selected], 0.4)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "23_8")?;
        table.extend(fourths);
        table.extend([prediction, halo, selected]);
        fade_out_all(stage, &table, 0.5)?;
        fade_in_all(stage, &extension_cards[FOUR_OF_SEVEN.0..], 0.4)?;
        let proof = [
            format!("Any {} genuine cards fix the recipe", FOUR_OF_SEVEN.0),
            format!("A nonzero degree≤{} difference", FOUR_OF_SEVEN.0 - 1),
            format!("cannot vanish at {} distinct positions", FOUR_OF_SEVEN.0),
            format!("Any {} cards: one recipe per secret", FOUR_OF_SEVEN.0 - 1),
            format!("Same likelihood: 1/{}", MARKS.pow((FOUR_OF_SEVEN.0 - 1) as u32)),
        ]
        .iter()
        .enumerate()
        .map(|(i, s)| {
            txt(
                &k,
                stage,
                s,
                24.,
                3.2,
                1.65 - i as f64 * 0.75,
                if i >= 3 { palette::pair() } else { Palette::ink() },
            )
        })
        .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &proof, 0.7)?;
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "23_9")?;
        fade_out_all(stage, &proof, 0.5)?;
        fade_out_all(stage, &[rule], 0.3)?;
        let attribution_lines = [
            ("D = secret · pieces = shares · k = threshold".to_owned(), 2.7, 23.),
            ("“even complete knowledge of k − 1 pieces".to_owned(), 1.65, 25.),
            ("reveals absolutely no information about D.”".to_owned(), 1.05, 25.),
            (
                format!("Shamir · How to Share a Secret · {}", facts::SHAMIR_YEAR),
                0.1,
                23.,
            ),
            ("Communications of the ACM · abstract".to_owned(), -0.45, 21.),
            (
                "Search: Shamir secret sharing · Lagrange interpolation".to_owned(),
                -1.3,
                20.,
            ),
            (
                format!("Blakley {}: different scheme cited by Shamir", facts::SHAMIR_YEAR),
                -1.95,
                20.,
            ),
        ];
        let attribution_reading = attribution_lines
            .iter()
            .map(|(s, ..)| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let attribution = attribution_lines
            .iter()
            .map(|(s, y, size)| txt(&k, stage, s, *size, 3.1, *y, Palette::ink()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &attribution, 0.7)?;
        let quote_deadline = read_by(stage, &attribution_reading) + 2.;
        k.hold(stage)?;
        wait_until(stage, quote_deadline)?;
        pause(stage, 1.)?;

        say(&k, stage, "23_10")?;
        fade_out_all(stage, &attribution, 0.4)?;
        fade_out_all(stage, &extension_cards, 0.4)?;
        fade_out_all(stage, extra.parts(), 0.4)?;
        recolor_all(stage, &rail.tint(&OPENING_TRIO, palette::trio()), 0.3)?;
        recolor_all(stage, &rail.tint(&PAIR, palette::pair()), 0.3)?;
        fade_in_all(stage, &cards, 0.5)?;
        vault.turn(stage, SECRET, 0.7)?;
        recolor_all(
            stage,
            &[(vault.door, palette::open()), (vault.lamp, palette::open())],
            0.5,
        )?;
        let title = [
            ("Three Keys Open the Secret.", 2.1),
            ("Two Reveal Nothing.", 1.3),
            ("Genuine cards · proper secret spins", 0.25),
            ("No added information — even about parity", -0.5),
        ]
        .iter()
        .enumerate()
        .map(|(i, (s, y))| {
            txt(
                &k,
                stage,
                s,
                if i < 2 { 29. } else { 22. },
                3.1,
                *y,
                if i == 0 { palette::trio() } else { palette::pair() },
            )
        })
        .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &title, 0.6)?;
        let note = precisely(&k, stage, NOTE)?;
        let anim = fade_in(stage.arena_mut(), note, ORIGIN, 1.)?.rt(0.5);
        stage.play(anim)?;
        let deadline = read_by(stage, NOTE);
        k.hold(stage)?;
        wait_until(stage, deadline)?;
        pause(stage, 1.)?;
        k.finish(stage)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_example_has_one_cubic_per_secret_and_unique_fourth_values() {
        let w = working().unwrap();
        let d = Dial::vault();
        assert_eq!(w.pair.len(), MARKS as usize);
        assert_eq!(w.cubics.len(), MARKS as usize);
        assert_eq!(w.example.coefficients(), vec![5, 4, 7, 10]);
        assert_eq!(w.example.deal(d, &w.xs), vec![4, 0, 9, 3, 9, 10, 0]);
        let values = w
            .cubics
            .iter()
            .map(|r| r.share(d, 4))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(values.len(), MARKS as usize);
        for xs in model::combinations(&w.xs, FOUR_OF_SEVEN.0) {
            let points = xs.iter().map(|&x| (x, w.example.share(d, x))).collect::<Vec<_>>();
            assert_eq!(model::rebuild(d, &points), Some(SECRET));
        }
    }
}
