//! A nonzero line can hide from a trio on the ten-mark dial.
use super::draw::{Rail, card, fade_in_all, fade_out_all, fail, pause, read_by, recolor_all, say, think, wait_until};
use crate::model::{self, Dial, Recipe, TEN, TRUSTEES, View};
use crate::{facts, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct TenMarksAgain {
    pub kit: Kit,
}
const NOTE: &str = "For this ten-mark dealing, nine trios determine the secret; trustees 1, 3, 5 allow both 0 and 5. A composite dial does not guarantee that every trio can reconstruct.";
struct Working {
    trios: Vec<Vec<u32>>,
    possible: Vec<Vec<u32>>,
    trio: Vec<u32>,
    ghost: Recipe,
    moved: Recipe,
}
fn working() -> Result<Working, String> {
    let d = Dial::new(TEN);
    let original = Recipe::example();
    let trios = model::combinations(&TRUSTEES, model::THRESHOLD);
    let possible = trios
        .iter()
        .map(|xs| {
            View::Shares {
                points: xs.iter().map(|&x| (x, original.share(d, x))).collect(),
                spins: 2,
            }
            .possible(d)
        })
        .collect::<Vec<_>>();
    let loose = trios
        .iter()
        .zip(&possible)
        .filter(|(_, p)| p.len() > 1)
        .collect::<Vec<_>>();
    let [(trio, _)] = loose.as_slice() else {
        return Err("expected one ambiguous trio".into());
    };
    let trio = (*trio).clone();
    let ghost = model::ghost(d, &trio).ok_or("missing invisible line")?;
    let moved = original.plus(d, &ghost);
    let table = trios
        .iter()
        .zip(&possible)
        .map(|(xs, p)| format!("{}:{}", model::csv(xs), model::csv(p)))
        .collect::<Vec<_>>()
        .join(";");
    let f = facts::facts()?;
    for (key, value) in [
        ("S-F11.ten.trio", model::csv(&trio)),
        ("S-F11.ten.trio_shares", model::csv(&original.deal(d, &trio))),
        ("S-F11.ten.ghost", model::csv(&ghost.coefficients())),
        (
            "S-F11.ten.ghost.heights",
            model::csv(&trio.iter().map(|&x| ghost.height(x)).collect::<Vec<_>>()),
        ),
        ("S-F11.ten.moved_recipe", model::csv(&moved.coefficients())),
        ("S-F11.ten.trio_table", table),
        (
            "S-F11.ten.trios_that_pin",
            possible.iter().filter(|p| p.len() == 1).count().to_string(),
        ),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture disagrees with {key}: {value}"));
        }
    }
    if ghost.spins[1] != 0 || original.deal(d, &trio) != moved.deal(d, &trio) || original.secret == moved.secret {
        return Err("counterexample changed".into());
    }
    Ok(Working {
        trios,
        possible,
        trio,
        ghost,
        moved,
    })
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, c, v(x, y))
}
fn formula(r: &Recipe) -> String {
    format!("{} + {}x + {}x²", r.secret, r.spins[0], r.spins[1])
}
impl SceneConstruct for TenMarksAgain {
    fn name(&self) -> &str {
        "20_ten_marks_again"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let w = working().map_err(fail)?;
        let d = Dial::new(TEN);
        let original = Recipe::example();
        let spoken = say(&k, stage, "20_1")?;
        let rail = Rail::build(&k, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.5)?;
        recolor_all(stage, &rail.tint(&TRUSTEES, palette::ghost()), 0.2)?;
        recolor_all(stage, &rail.tint(&w.trio, palette::trio()), 0.3)?;
        let cards = w
            .trio
            .iter()
            .map(|&x| card(&k, stage, x, &original.share(d, x).to_string(), palette::trio()))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_all(stage, &cards, 0.4)?;
        let title = text(
            &k,
            stage,
            &format!("Back to {} marks", d.marks()),
            32.0,
            0.0,
            3.2,
            Palette::ink(),
        )?;
        let a = text(
            &k,
            stage,
            &format!("Original: {}", formula(&original)),
            27.0,
            0.0,
            2.6,
            palette::trio(),
        )?;
        fade_in_all(stage, &[title, a], 0.4)?;
        let positions = std::iter::once(0).chain(w.trio.iter().copied()).collect::<Vec<_>>();
        let column_x = [-3.6, 0.2, 2.5, 4.8];
        let mut comparison = vec![a];
        for (&x, &at) in positions.iter().zip(&column_x) {
            let label = if x == 0 {
                format!("secret · x={x}")
            } else {
                format!("trustee {x}")
            };
            comparison.push(text(&k, stage, &label, 23.0, at, 1.95, Palette::dim())?);
            comparison.push(text(
                &k,
                stage,
                &original.share(d, x).to_string(),
                32.0,
                at,
                1.25,
                palette::trio(),
            )?);
        }
        spoken.wait_for(stage, "trustees one, three and five")?;
        fade_in_all(stage, &comparison[1..], 0.5)?;
        k.hold(stage)?;
        pause(stage, 1.0)?;

        let spoken = say(&k, stage, "20_2")?;
        let add = text(
            &k,
            stage,
            &format!("Add {} + {}x", w.ghost.secret, w.ghost.spins[0]),
            26.0,
            0.0,
            0.6,
            palette::secret(),
        )?;
        fade_in_all(stage, &[add], 0.4)?;
        spoken.wait_for(stage, "ten, twenty and thirty")?;
        fade_out_all(stage, &[add], 0.3)?;
        let mut additions = Vec::new();
        for (&x, &at) in positions.iter().zip(&column_x) {
            additions.push(text(
                &k,
                stage,
                &format!("+{}", w.ghost.height(x)),
                28.0,
                at,
                0.6,
                palette::secret(),
            )?);
        }
        fade_in_all(stage, &additions, 0.5)?;
        spoken.wait_for(stage, "full turns")?;
        for (&x, &at) in positions.iter().zip(&column_x) {
            comparison.push(text(
                &k,
                stage,
                &format!("lands {}", w.ghost.share(d, x)),
                25.0,
                at,
                -0.05,
                palette::secret(),
            )?);
        }
        fade_in_all(stage, &comparison[comparison.len() - positions.len()..], 0.5)?;
        comparison.extend(additions);
        k.hold(stage)?;
        pause(stage, 2.0)?;

        say(&k, stage, "20_3")?;
        let question = think(&k, stage, v(3.1, -1.35))?;

        let spoken = say(&k, stage, "20_4")?;
        fade_out_all(stage, &[question], 0.2)?;
        let mut result = Vec::new();
        for (&x, &at) in positions.iter().zip(&column_x) {
            result.push(text(
                &k,
                stage,
                &format!("→ {}", w.moved.share(d, x)),
                32.0,
                at,
                -0.75,
                if x == 0 { palette::secret() } else { palette::trio() },
            )?);
        }
        // First reveal the unchanged shares, then the different secret.
        fade_in_all(stage, &result[1..], 0.4)?;
        spoken.wait_for(stage, "secret becomes zero")?;
        fade_in_all(stage, &result[..1], 0.4)?;
        let b = text(
            &k,
            stage,
            &format!("New recipe: {}", formula(&w.moved)),
            26.0,
            2.35,
            -1.6,
            palette::secret(),
        )?;
        fade_in_all(stage, &[b], 0.4)?;
        comparison.extend(result);
        comparison.push(b);
        k.hold(stage)?;
        pause(stage, 1.5)?;

        say(&k, stage, "20_5")?;
        let same = text(
            &k,
            stage,
            "Same cards. Different secrets.",
            26.0,
            2.35,
            -2.3,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[same], 0.4)?;
        comparison.push(same);
        k.hold(stage)?;
        pause(stage, 2.0)?;

        let spoken = say(&k, stage, "20_6")?;
        fade_out_all(stage, &comparison, 0.5)?;
        let caption = text(
            &k,
            stage,
            "Possible secrets · original ten-mark dealing",
            26.0,
            0.0,
            2.6,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[caption], 0.4)?;
        let mut grid = Vec::new();
        for (i, (xs, p)) in w.trios.iter().zip(&w.possible).enumerate() {
            let words = format!(
                "{} → {}",
                xs.iter().map(u32::to_string).collect::<Vec<_>>().join("·"),
                p.iter().map(u32::to_string).collect::<Vec<_>>().join(" or ")
            );
            grid.push(text(
                &k,
                stage,
                &words,
                27.0,
                -4.8 + (i % 5) as f64 * 2.4,
                1.75 - (i / 5) as f64 * 1.0,
                if p.len() > 1 {
                    palette::secret()
                } else {
                    palette::trio()
                },
            )?);
        }
        spoken.wait_for(stage, "every trio")?;
        fade_in_all(stage, &grid, 0.9)?;
        k.hold(stage)?;
        pause(stage, 3.0)?;

        say(&k, stage, "20_7")?;
        let prime = text(
            &k,
            stage,
            &format!(
                "{} marks: a nonzero line has at most {} zero",
                model::MARKS,
                model::most_zeros(Dial::vault(), 1)
            ),
            26.0,
            0.0,
            -0.3,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[prime], 0.4)?;
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
    fn invisible_line_changes_secret_and_exactly_one_trio_is_ambiguous() {
        let w = working().unwrap();
        let d = Dial::new(TEN);
        let r = Recipe::example();
        assert_eq!(w.possible.iter().filter(|p| p.len() > 1).count(), 1);
        for &x in &w.trio {
            assert_eq!(w.ghost.share(d, x), 0);
            assert_eq!(r.share(d, x), w.moved.share(d, x));
        }
        assert_ne!(r.secret, w.moved.secret);
    }
}
