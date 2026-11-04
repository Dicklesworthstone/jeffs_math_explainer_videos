//! Native staged product-grid proof, not yet compiled or reviewed as pictures.
use crate::product_model::{SIDE, cell_tex, diagonal, total_tex};
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;
pub struct ProductGrid {
    pub kit: Kit,
}
pub const TEXT: &[&str] = &[
    "Why addition becomes multiplication",
    "A corner of an infinite grid",
    "What is the degree-two diagonal?",
    "Complete diagonals: degrees 0 through 3",
    "Absolute convergence permits regrouping",
];
pub const TEX: &[&str] = &[
    r"\frac{x^2}{2}+xy+\frac{y^2}{2}",
    r"\frac12(x^2+2xy+y^2)",
    r"\frac{(x+y)^2}{2}",
    r"\frac{m!}{k!(m-k)!}=\binom{m}{k}",
    r"E(x)E(y)=E(x+y)",
    r"E(x)E(-x)=E(0)=1",
    r"E(x)=E(x/2)^2>0\quad(x\in\mathbb{R})",
    r"\sum_{k=0}^{m}\frac{x^k y^{m-k}}{k!(m-k)!}=\frac{(x+y)^m}{m!}",
    r"\sum_{m=0}^{\infty}\frac{(x+y)^m}{m!}=E(x+y)",
    r"E'=E>0",
    r"E(x)=E(x/2)^2",
];
/// The diagonal totals stay in one column, so the terms of E(x+y) build up in view.
const TOTALS_X: f64 = 1.9;
const TOTAL_Y: [f64; 4] = [1.7, 0.9, -0.05, -1.05];
/// The working formulas, in a second column clear of the totals.
const WORK_X: f64 = 4.7;
fn reveal(s: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
    let a = fade_in(s.arena_mut(), m, ORIGIN, 1.0)?;
    s.play(a.rt(0.35))?;
    Ok(())
}
fn reveal_all(s: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(s.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(s.prepare(a.rt(seconds))?);
    }
    s.play_prepared(animations)?;
    Ok(())
}
fn hide(s: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
    let a = fade_out(s.arena_mut(), m, ORIGIN, 1.0)?;
    s.play(a.rt(0.25))?;
    Ok(())
}
fn rule(s: &mut Stage<'_>, a: Vec3, b: Vec3) -> fmn::Result<Mob> {
    Ok(s.arena_mut()
        .add(Line::new(a, b).style(stroke_style(Palette::dim(), 1.0)).build()?))
}
fn color_diagonal(s: &mut Stage<'_>, cells: &[(usize, usize, Mob)], degree: usize) -> fmn::Result<()> {
    let selected = diagonal(degree);
    let mut animations = Vec::new();
    for &(i, j, m) in cells {
        let color = if selected.contains(&(i, j)) {
            Palette::hero()
        } else {
            Palette::dim()
        };
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.3),
                ..AnimateArgs::default()
            })?
            .set_color_with_opacity(color, 1.0)?;
        animations.push(s.prepare(a)?);
    }
    s.play_prepared(animations)?;
    Ok(())
}
impl SceneConstruct for ProductGrid {
    fn name(&self) -> &str {
        "03_product_grid"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(s, "03_1")?;
        let title = k.text(s, TEXT[0], 34.0, Palette::ink(), v(0.0, 3.15))?;
        reveal(s, title)?;
        let mut cells = Vec::new();
        let mut objects = Vec::new();
        // The two inputs are named, and the dots say both edges go on.
        k.until_sentence(s, "03_1", 2)?;
        let edges = [
            k.tex(s, "E(x)", 26.0, Palette::blue(), &[], v(-2.175, 2.72))?,
            k.tex(s, "E(y)", 26.0, Palette::gold(), &[], v(-6.25, -0.1))?,
            k.tex(s, r"\cdots", 24.0, Palette::blue(), &[], v(1.05, 2.2))?,
            k.tex(s, r"\vdots", 24.0, Palette::gold(), &[], v(-5.3, -2.45))?,
        ];
        reveal_all(s, &edges, 0.4)?;
        objects.extend(edges);
        for i in 0..SIDE {
            let a = k.tex(
                s,
                &cell_tex(i, 0),
                24.0,
                Palette::blue(),
                &[],
                v(-4.2 + 1.35 * i as f64, 2.2),
            )?;
            reveal(s, a)?;
            objects.push(a);
            let a = k.tex(
                s,
                &cell_tex(0, i),
                24.0,
                Palette::gold(),
                &[],
                v(-5.3, 1.4 - 1.0 * i as f64),
            )?;
            reveal(s, a)?;
            objects.push(a);
        }
        k.until_sentence(s, "03_1", 3)?;
        // Rules both ways: the voice says "each square".
        let mut rules = Vec::new();
        for j in 0..=SIDE {
            let y = 1.9 - j as f64;
            rules.push(rule(s, v(-4.85, y), v(0.55, y))?);
        }
        for i in 0..=SIDE {
            let x = -4.85 + 1.35 * i as f64;
            rules.push(rule(s, v(x, 1.9), v(x, -2.1))?);
        }
        reveal_all(s, &rules, 0.4)?;
        objects.extend(rules);
        for i in 0..SIDE {
            for j in 0..SIDE {
                let m = k.tex(
                    s,
                    &cell_tex(i, j),
                    23.0,
                    Palette::dim(),
                    &[],
                    v(-4.2 + 1.35 * i as f64, 1.4 - j as f64),
                )?;
                // Quicker than `reveal`: at 0.35 s each the sixteen run past the next two sentences.
                let a = fade_in(s.arena_mut(), m, ORIGIN, 1.0)?;
                s.play(a.rt(0.2))?;
                cells.push((i, j, m));
                objects.push(m);
            }
        }
        k.until_sentence(s, "03_1", 5)?;
        let corner = k.text(s, TEXT[1], 22.0, Palette::dim(), v(-2.1, -2.65))?;
        reveal(s, corner)?;
        objects.push(corner);
        k.hold(s)?;
        k.say(s, "03_2")?;
        let mut totals = Vec::new();
        for (degree, sentence) in [(0, 1), (1, 2), (2, 4)] {
            k.until_sentence(s, "03_2", sentence)?;
            color_diagonal(s, &cells, degree)?;
            if degree < 2 {
                if degree == 1 {
                    k.until_sentence(s, "03_2", 3)?;
                }
                let m = k.tex(
                    s,
                    &total_tex(degree),
                    28.0,
                    Palette::green(),
                    &[],
                    v(TOTALS_X, TOTAL_Y[degree]),
                )?;
                reveal(s, m)?;
                totals.push(m);
            }
        }
        k.until_sentence(s, "03_2", 5)?;
        let question = k.text(s, TEXT[2], 23.0, Palette::hero(), v(4.2, 0.0))?;
        reveal(s, question)?;
        k.hold(s)?;
        k.mark(s, "predict");
        k.music_breath(s, 4.0)?;
        hide(s, question)?;
        k.say(s, "03_3")?;
        let expression = k.tex(s, TEX[0], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
        reveal(s, expression)?;
        for (sentence, index) in [(1, 1), (3, 2)] {
            k.until_sentence(s, "03_3", sentence)?;
            let to = k.tex(s, TEX[index], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
            s.play(Transform::new(expression, to).rt(0.6))?;
        }
        let second = k.tex(s, &total_tex(2), 28.0, Palette::green(), &[], v(TOTALS_X, TOTAL_Y[2]))?;
        reveal(s, second)?;
        totals.push(second);
        k.hold(s)?;
        k.say(s, "03_4")?;
        color_diagonal(s, &cells, 3)?;
        let third = k.tex(s, &total_tex(3), 28.0, Palette::green(), &[], v(TOTALS_X, TOTAL_Y[3]))?;
        reveal(s, third)?;
        totals.push(third);
        k.until_sentence(s, "03_4", 1)?;
        let general = k.tex(s, TEX[3], 27.0, Palette::ink(), &[], v(WORK_X, 1.95))?;
        reveal(s, general)?;
        k.until_sentence(s, "03_4", 2)?;
        let identity = k.tex(s, TEX[7], 27.0, Palette::ink(), &[], v(WORK_X, 1.0))?;
        reveal(s, identity)?;
        k.until_sentence(s, "03_4", 3)?;
        let complete = k.text(s, TEXT[3], 22.0, Palette::dim(), v(WORK_X, -1.05))?;
        reveal(s, complete)?;
        k.hold(s)?;
        k.say(s, "03_5")?;
        hide(s, complete)?;
        k.until_sentence(s, "03_5", 1)?;
        let permission = k.text(s, TEXT[4], 22.0, Palette::gold(), v(0.0, -3.1))?;
        reveal(s, permission)?;
        k.until_sentence(s, "03_5", 2)?;
        let series = k.tex(s, TEX[8], 27.0, Palette::green(), &[], v(WORK_X, -1.05))?;
        reveal(s, series)?;
        k.until_sentence(s, "03_5", 3)?;
        let law = k.tex(s, TEX[4], 33.0, Palette::green(), &[], v(WORK_X, -2.2))?;
        reveal(s, law)?;
        k.hold(s)?;
        let mut fades = Vec::new();
        objects.extend(totals);
        objects.extend([expression, general, identity, permission, series, law]);
        for m in objects {
            let a = fade_out(s.arena_mut(), m, ORIGIN, 1.0)?;
            fades.push(s.prepare(a.rt(0.5))?);
        }
        s.play_prepared(fades)?;
        k.say(s, "03_6")?;
        k.until_sentence(s, "03_6", 1)?;
        let nonzero = k.tex(s, TEX[5], 32.0, Palette::blue(), &[], v(0.0, 1.2))?;
        reveal(s, nonzero)?;
        k.until_sentence(s, "03_6", 2)?;
        let half = k.tex(s, TEX[10], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        reveal(s, half)?;
        // The law first; "positive" waits for the sentence that concludes it.
        k.until_sentence(s, "03_6", 3)?;
        let positive = k.tex(s, TEX[6], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        s.play(Transform::new(half, positive).rt(0.5))?;
        k.until_sentence(s, "03_6", 4)?;
        let increasing = k.tex(s, TEX[9], 31.0, Palette::green(), &[], v(0.0, -1.9))?;
        reveal(s, increasing)?;
        k.finish(s)
    }
}

#[cfg(test)]
mod tests {
    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("product_grid.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("split yields a first part")
    }
    fn assert_in_order(needles: &[&str]) {
        let mut from = 0;
        for &needle in needles {
            let at = scene()[from..]
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} missing or out of order"));
            from += at + needle.len();
        }
    }
    #[test]
    fn the_think_gap_is_authored_and_the_answer_waits() {
        assert_in_order(&[
            "reveal(s, question)?;",
            "k.hold(s)?;",
            r#"k.mark(s, "predict");"#,
            "k.music_breath(s, 4.0)?;",
            "hide(s, question)?;",
            r#"k.say(s, "03_3")?;"#,
            "TEX[0]",
        ]);
    }
    #[test]
    fn the_proof_steps_land_on_their_sentences() {
        assert_in_order(&[
            r#"until_sentence(s, "03_4", 2)"#,
            "TEX[7]",
            r#"until_sentence(s, "03_4", 3)"#,
            "TEXT[3]",
            r#"until_sentence(s, "03_5", 2)"#,
            "TEX[8]",
            r#"until_sentence(s, "03_5", 3)"#,
            "TEX[4]",
            r#"until_sentence(s, "03_6", 4)"#,
            "TEX[9]",
        ]);
    }
}
