//! A real two-share trial motivates the root bound behind three-share uniqueness.
use super::draw::{Rail, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until};
use super::envelope::outline;
use crate::model::{self, Dial, Recipe, SECOND_TRIO, SECRET, TRUSTEES};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct AnyThree {
    pub kit: Kit,
}
const NOTE: &str = "On a prime dial, three distinct positions determine a polynomial of degree at most two. The difference of two candidates has three roots, so it must be zero.";
struct Working {
    trios: Vec<Vec<u32>>,
    answers: Vec<u32>,
    trial: Recipe,
    difference: Recipe,
    trial_shares: Vec<u32>,
    differences: Vec<u32>,
    left: Vec<u32>,
    right: Vec<u32>,
}
fn working() -> Result<Working, String> {
    let d = Dial::vault();
    let original = Recipe::example();
    let [a, b, _] = SECOND_TRIO;
    let difference = Recipe::new(d.mul(a, b), &[d.wrap(-i64::from(a) - i64::from(b)), 1]);
    let trial = original.plus(d, &difference);
    let trios = model::combinations(&TRUSTEES, 3);
    let answers = trios
        .iter()
        .map(|xs| {
            let points = xs.iter().map(|&x| (x, original.share(d, x))).collect::<Vec<_>>();
            model::rebuild(d, &points).ok_or_else(|| "trio failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let trial_shares = trial.deal(d, &SECOND_TRIO);
    let differences = d.all().map(|x| difference.share(d, x)).collect::<Vec<_>>();
    let left = d.all().map(|x| d.wrap(i64::from(x) - i64::from(a))).collect::<Vec<_>>();
    let right = d.all().map(|x| d.wrap(i64::from(x) - i64::from(b))).collect::<Vec<_>>();
    let roots = d.all().filter(|&x| difference.share(d, x) == 0).collect::<Vec<_>>();
    let f = facts::facts()?;
    for (key, value) in [
        ("S-F11.trios", trios.len().to_string()),
        ("S-F11.trial", model::csv(&trial.coefficients())),
        ("S-F11.difference", model::csv(&difference.coefficients())),
        ("S-F11.trial_shares", model::csv(&trial_shares)),
        ("S-F11.difference_row", model::csv(&differences)),
        ("S-F11.left_row", model::csv(&left)),
        ("S-F11.right_row", model::csv(&right)),
        ("S-F11.roots", model::csv(&roots)),
        (
            "S-F11.third_difference",
            differences[SECOND_TRIO[2] as usize].to_string(),
        ),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}: {value}"));
        }
    }
    if answers.iter().any(|&x| x != SECRET)
        || roots != vec![a, b]
        || left.iter().zip(&right).map(|(&a, &b)| d.mul(a, b)).collect::<Vec<_>>() != differences
    {
        return Err("trial or reconstruction changed".into());
    }
    Ok(Working {
        trios,
        answers,
        trial,
        difference,
        trial_shares,
        differences,
        left,
        right,
    })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
fn formula(r: &Recipe) -> String {
    format!("{} + {}x + {}x²", r.secret, r.spins[0], r.spins[1])
}
impl SceneConstruct for AnyThree {
    fn name(&self) -> &str {
        "19_any_three"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::vault();
        let original = Recipe::example();
        let spoken = say(&k, stage, "19_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.5)?;
        let cards = TRUSTEES
            .iter()
            .map(|&x| card(&k, stage, x, &original.share(d, x).to_string(), palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards, 0.5)?;
        let title = text(
            &k,
            stage,
            &format!("Any three · {}-mark dial", d.marks()),
            32.0,
            0.0,
            3.15,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[title], 0.3)?;
        let mut grid = Vec::new();
        for (i, (xs, answer)) in w.trios.iter().zip(&w.answers).enumerate() {
            let words = format!(
                "{} → {}",
                xs.iter().map(u32::to_string).collect::<Vec<_>>().join(" · "),
                answer
            );
            grid.push(text(
                &k,
                stage,
                &words,
                27.0,
                -4.8 + (i % 5) as f64 * 2.4,
                2.1 - (i / 5) as f64 * 1.0,
                palette::trio(),
            )?);
        }
        spoken.wait_for(stage, "ten ways")?;
        fade_in_all(stage, &grid, 0.8)?;
        k.hold(stage)?;
        pause(stage, 2.0)?;

        let spoken = say(&k, stage, "19_2")?;
        fade_out_all(stage, &grid, 0.4)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&SECOND_TRIO, palette::trio()), 0.3)?;
        let mut table = Vec::new();
        table.push(text(
            &k,
            stage,
            "Trial: matches two shares",
            27.0,
            0.0,
            2.65,
            Palette::dim(),
        )?);
        let mut unknown = Vec::new();
        for (i, &x) in SECOND_TRIO.iter().enumerate() {
            let at = -0.3 + i as f64 * 2.5;
            table.push(text(
                &k,
                stage,
                &format!("trustee {x}"),
                22.0,
                at,
                1.85,
                Palette::dim(),
            )?);
            table.push(text(
                &k,
                stage,
                &original.share(d, x).to_string(),
                30.0,
                at,
                1.1,
                palette::trio(),
            )?);
            let b = if i == 2 {
                "?".into()
            } else {
                w.trial_shares[i].to_string()
            };
            let m = text(&k, stage, &b, 30.0, at, 0.35, palette::secret())?;
            if i == 2 {
                unknown.push(m);
            } else {
                table.push(m);
            }
        }
        table.push(text(
            &k,
            stage,
            &format!("A: {}", formula(&original)),
            23.0,
            -3.9,
            1.1,
            palette::trio(),
        )?);
        table.push(text(
            &k,
            stage,
            &format!("B: {}", formula(&w.trial)),
            23.0,
            -3.9,
            0.35,
            palette::secret(),
        )?);
        spoken.wait_for(stage, "two and four")?;
        fade_in_all(stage, &table, 0.5)?;
        fade_in_all(stage, &unknown, 0.3)?;
        k.hold(stage)?;

        let spoken = say(&k, stage, "19_3")?;
        let difference_label = text(
            &k,
            stage,
            &format!("B − A: {}", formula(&w.difference)),
            23.0,
            -3.9,
            -0.4,
            Palette::ink(),
        )?;
        table.push(difference_label);
        spoken.wait_for(stage, "Subtract")?;
        fade_in_all(stage, &[difference_label], 0.4)?;
        for (i, &x) in SECOND_TRIO.iter().enumerate() {
            let t = if i == 2 {
                "?".into()
            } else {
                w.differences[x as usize].to_string()
            };
            let m = text(&k, stage, &t, 30.0, -0.3 + i as f64 * 2.5, -0.4, Palette::ink())?;
            fade_in_all(stage, &[m], 0.3)?;
            if i == 2 {
                unknown.push(m);
            } else {
                table.push(m);
            }
        }
        k.hold(stage)?;
        say(&k, stage, "19_4")?;
        let question = think(&k, stage, v(3.0, -1.0))?;

        let spoken = say(&k, stage, "19_5")?;
        fade_out_all(stage, &[question], 0.2)?;
        let requirement = text(
            &k,
            stage,
            "To match all three: three zeros required",
            27.0,
            0.0,
            -1.0,
            palette::secret(),
        )?;
        fade_in_all(stage, &[requirement], 0.4)?;
        spoken.wait_for(stage, "this trial fails")?;
        fade_out_all(stage, &unknown, 0.3)?;
        let b = text(
            &k,
            stage,
            &w.trial_shares[2].to_string(),
            30.0,
            4.7,
            0.35,
            palette::secret(),
        )?;
        let delta = text(
            &k,
            stage,
            &w.differences[SECOND_TRIO[2] as usize].to_string(),
            30.0,
            4.7,
            -0.4,
            palette::secret(),
        )?;
        table.extend([b, delta]);
        fade_in_all(stage, &[b, delta], 0.4)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;

        let spoken = say(&k, stage, "19_6")?;
        fade_out_all(stage, &table, 0.4)?;
        fade_out_all(stage, &[requirement], 0.3)?;
        let [a, b, _] = SECOND_TRIO;
        let factoring = text(
            &k,
            stage,
            &format!("g(x) = {} = (x − {a})(x − {b})", formula(&w.difference)),
            29.0,
            0.0,
            2.6,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[factoring], 0.5)?;
        let mut factor_table = Vec::new();
        for (i, row) in [None, Some(&w.left), Some(&w.right), Some(&w.differences)]
            .iter()
            .enumerate()
        {
            let y = 1.7 - i as f64 * 0.65;
            let label = match i {
                0 => "position".into(),
                1 => format!("x − {a}"),
                2 => format!("x − {b}"),
                _ => "product".into(),
            };
            factor_table.push(text(&k, stage, &label, 21.0, -6.0, y, Palette::dim())?);
            for x in d.all() {
                let n = row.map_or(x, |r| r[x as usize]);
                let color = if i > 0 && n == 0 {
                    palette::secret()
                } else {
                    Palette::ink()
                };
                factor_table.push(text(
                    &k,
                    stage,
                    &n.to_string(),
                    25.0,
                    -4.7 + f64::from(x) * 1.0,
                    y,
                    color,
                )?);
            }
        }
        spoken.wait_for(stage, "two factors")?;
        fade_in_all(stage, &factor_table, 0.7)?;
        k.hold(stage)?;
        pause(stage, 2.0)?;

        let spoken = say(&k, stage, "19_7")?;
        fade_out_all(stage, &factor_table, 0.3)?;
        fade_out_all(stage, &[factoring], 0.3)?;
        let mut proof = Vec::new();
        for (words, y) in [
            ("u · v = 0", 2.25),
            ("u nonzero → multiply by its partner", 1.35),
            ("(partner · u) · v = v = 0", 0.45),
        ] {
            proof.push(text(&k, stage, words, 30.0, 0.0, y, Palette::ink())?);
        }
        fade_in_all(stage, &proof[..1], 0.4)?;
        spoken.wait_for(stage, "nonzero factor")?;
        fade_in_all(stage, &proof[1..2], 0.4)?;
        spoken.wait_for(stage, "other factor must be zero")?;
        fade_in_all(stage, &proof[2..], 0.4)?;
        k.hold(stage)?;
        pause(stage, 1.3)?;

        let spoken = say(&k, stage, "19_8")?;
        fade_out_all(stage, &proof, 0.3)?;
        let generic = text(&k, stage, "g(x) = (x − r) · (c + d·x)", 33.0, 0.0, 2.55, Palette::ink())?;
        let left_box = outline(stage, v(-3.4, 0.9), 5.3, 1.55, palette::trio())?;
        let right_box = outline(stage, v(3.1, 0.9), 5.6, 1.55, palette::secret())?;
        let left = text(&k, stage, "x − r = 0   only at r", 26.0, -3.4, 0.9, palette::trio())?;
        let right = text(
            &k,
            stage,
            "Nonzero line: at most one zero",
            23.0,
            3.1,
            0.9,
            palette::secret(),
        )?;
        let constant = text(
            &k,
            stage,
            "If d = 0: a nonzero constant has no zeros",
            23.0,
            0.0,
            -0.4,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[generic, left_box, right_box], 0.5)?;
        spoken.wait_for(stage, "first factor")?;
        fade_in_all(stage, &[left], 0.4)?;
        spoken.wait_for(stage, "line that is not zero everywhere")?;
        fade_in_all(stage, &[right, constant], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.0)?;

        let spoken = say(&k, stage, "19_9")?;
        let bound = text(
            &k,
            stage,
            "At most two roots, unless g is zero everywhere",
            27.0,
            0.0,
            -1.0,
            palette::secret(),
        )?;
        spoken.wait_for(stage, "at most two")?;
        fade_in_all(stage, &[bound], 0.4)?;
        k.hold(stage)?;
        pause(stage, 1.3)?;
        say(&k, stage, "19_10")?;
        fade_out_all(
            stage,
            &[generic, left_box, right_box, left, right, constant, bound],
            0.4,
        )?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::trio()), 0.3)?;
        fade_in_all(stage, &grid, 0.5)?;
        let same = text(
            &k,
            stage,
            "Three matching shares → same recipe → same secret",
            26.0,
            0.0,
            -0.1,
            palette::secret(),
        )?;
        fade_in_all(stage, &[same], 0.4)?;
        let note = precisely(&k, stage, NOTE)?;
        let anim = fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5);
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
    fn real_trial_matches_only_the_selected_pair() {
        let w = working().unwrap();
        let d = Dial::vault();
        let r = Recipe::example();
        for &x in &SECOND_TRIO[..2] {
            assert_eq!(w.trial.share(d, x), r.share(d, x));
        }
        let third = SECOND_TRIO[2];
        assert_ne!(w.trial.share(d, third), r.share(d, third));
        for x in d.all() {
            assert_eq!(
                d.wrap(i64::from(w.trial.share(d, x)) - i64::from(r.share(d, x))),
                w.difference.share(d, x)
            );
        }
    }
}
