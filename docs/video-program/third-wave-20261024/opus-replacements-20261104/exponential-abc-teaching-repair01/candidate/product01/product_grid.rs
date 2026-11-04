//! Native staged product-grid proof. Type-checked at the pin; never rendered.
use crate::product_model::{
    LADDER_HIGH, LADDER_LOW, SIDE, cell_tex, diagonal, e_label, ladder_fractions, ladder_negatives, ladder_whole,
    total_tex,
};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, dot_cloud, rect, stroke_style, v};
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
    "the inputs we have earned",
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
    r"E(2)=e\cdot e,\quad E(3)=e^3",
    r"E\left(\tfrac12\right)^2=e\ \text{and}\ E>0,\ \text{so}\ E\left(\tfrac12\right)=e^{1/2}",
    r"E(-x)=\frac{1}{E(x)}",
    r"E(x)=e^x",
];
/// The diagonal totals stay in one column, so the terms of E(x+y) build up in view.
const TOTALS_X: f64 = 1.9;
const TOTAL_Y: [f64; 4] = [1.7, 0.9, -0.05, -1.05];
/// The working formulas, in a second column clear of the totals.
const WORK_X: f64 = 4.7;
/// The 03_7 line, from LADDER_LOW to LADDER_HIGH.
const LADDER_Y: f64 = -1.35;
/// Word for word from 03_2 and 03_7: these two sentences carry more than one reveal.
const ROWS_SENTENCE: &str = "Instead of collecting the products by rows, collect them by total degree.";
const LADDER_SENTENCE: &str = "Roots give fractions, reciprocals give negatives, and continuity fills in the rest.";
fn ladder_x(input: f64) -> f64 {
    -4.0 + 1.6 * (input - f64::from(LADDER_LOW))
}
/// Wait for `clause` of sentence `j`: the sentence's real window, shared out by its characters. An estimate inside
/// a sentence, not a heard boundary. Nothing if that moment has passed, or without a narrator.
fn until_clause(k: &Kit, s: &mut Stage<'_>, id: &str, j: usize, sentence: &str, clause: &str) -> fmn::Result<()> {
    let Some(narrator) = k.narrator.as_ref() else {
        return Ok(());
    };
    let Some(from) = narrator.sentence_at(id, j) else {
        return Ok(());
    };
    let end = narrator
        .sentence_at(id, 0)
        .zip(narrator.length(id))
        .map(|(began, length)| began + length);
    let Some(to) = narrator.sentence_at(id, j + 1).or(end) else {
        return Ok(());
    };
    let byte = sentence.find(clause).expect("the clause is in its sentence");
    let at = from + (to - from) * byte as f64 / sentence.len() as f64;
    let (now, fps) = (s.scene().time().to_f64(), s.scene().time().fps());
    if at > now {
        wait_frames(s, frames_covering(at - now, fps))?;
    }
    Ok(())
}
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
fn paint(s: &mut Stage<'_>, m: Mob, color: Srgb) -> fmn::Result<Box<dyn Animation>> {
    let a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.3),
            ..AnimateArgs::default()
        })?
        .set_color_with_opacity(color, 1.0)?;
    s.prepare(a)
}
/// Each complete diagonal takes one colour and its total takes the same one, so the terms of E(x+y) can be
/// read back to the squares they came from. Squares past degree three stay dim: this corner cuts their diagonals.
fn link_diagonals(s: &mut Stage<'_>, cells: &[(usize, usize, Mob)], totals: &[Mob]) -> fmn::Result<()> {
    let colors = [Palette::teal(), Palette::purple(), Palette::hero(), Palette::ink()];
    let mut animations = Vec::new();
    for &(i, j, m) in cells {
        let color = colors.get(i + j).copied().unwrap_or_else(Palette::dim);
        animations.push(paint(s, m, color)?);
    }
    for (&m, &color) in totals.iter().zip(colors.iter()) {
        animations.push(paint(s, m, color)?);
    }
    s.play_prepared(animations)?;
    Ok(())
}
/// A cloud of input marks on the 03_7 line.
fn ladder_marks(s: &mut Stage<'_>, inputs: &[f64], color: Srgb, radius: f64) -> Mob {
    let points: Vec<Vec3> = inputs.iter().map(|&x| v(ladder_x(x), LADDER_Y)).collect();
    let colors = vec![color; points.len()];
    s.arena_mut().add(dot_cloud(&points, &colors, radius))
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
        // A row first, then a line along one diagonal: the change of reading is seen before degree zero lands.
        let row = rect(v(-4.85, 0.9), 5.4, 1.0, stroke_style(Palette::yellow(), 3.0))?;
        let row = s.arena_mut().add(row);
        reveal(s, row)?;
        until_clause(&k, s, "03_2", 0, ROWS_SENTENCE, "collect them")?;
        let along = Line::new(v(-4.55, -1.86), v(0.2, 1.66))
            .style(stroke_style(Palette::yellow(), 4.0))
            .build()?;
        let along = s.arena_mut().add(along);
        k.allow_crossing(along);
        let row_out = fade_out(s.arena_mut(), row, ORIGIN, 1.0)?;
        let along_in = fade_in(s.arena_mut(), along, ORIGIN, 1.0)?;
        let swap = vec![s.prepare(row_out.rt(0.35))?, s.prepare(along_in.rt(0.35))?];
        s.play_prepared(swap)?;
        k.until_sentence(s, "03_2", 1)?;
        hide(s, along)?;
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
        link_diagonals(s, &cells, &totals)?;
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
        k.hold(s)?;

        k.say(s, "03_7")?;
        let mut fades = Vec::new();
        for m in [nonzero, half, increasing] {
            let a = fade_out(s.arena_mut(), m, ORIGIN, 1.0)?;
            fades.push(s.prepare(a.rt(0.3))?);
        }
        s.play_prepared(fades)?;
        // The input line starts empty. Each thing the law earns marks more of it, until none of it is left out.
        let axis = Line::new(
            v(ladder_x(f64::from(LADDER_LOW)), LADDER_Y),
            v(ladder_x(f64::from(LADDER_HIGH)), LADDER_Y),
        )
        .style(stroke_style(Palette::dim(), 2.0))
        .build()?;
        let axis = s.arena_mut().add(axis);
        let mut line_parts = vec![axis, k.text(s, TEXT[5], 23.0, Palette::dim(), v(0.0, -2.35))?];
        for input in LADDER_LOW..=LADDER_HIGH {
            line_parts.push(k.tex(
                s,
                &input.to_string(),
                22.0,
                Palette::dim(),
                &[],
                v(ladder_x(f64::from(input)), LADDER_Y - 0.4),
            )?);
        }
        reveal_all(s, &line_parts, 0.35)?;

        k.until_sentence(s, "03_7", 1)?;
        let named = format!(r"e=E(1)=1+1+\frac12+\frac16+\cdots\approx {}", e_label());
        let name = k.tex(s, &named, 29.0, Palette::hero(), &[], v(0.0, 2.2))?;
        reveal(s, name)?;

        k.until_sentence(s, "03_7", 2)?;
        let integers = k.tex(s, TEX[11], 28.0, Palette::blue(), &[], v(0.0, 1.35))?;
        let whole_marks = ladder_marks(s, &ladder_whole(), Palette::blue(), 0.09);
        reveal_all(s, &[integers, whole_marks], 0.4)?;

        k.until_sentence(s, "03_7", 3)?;
        let roots = k.tex(s, TEX[12], 28.0, Palette::green(), &[], v(0.0, 0.55))?;
        let fraction_marks = ladder_marks(s, &ladder_fractions(), Palette::green(), 0.07);
        reveal_all(s, &[roots, fraction_marks], 0.4)?;
        until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "reciprocals")?;
        let reciprocal = k.tex(s, TEX[13], 28.0, Palette::gold(), &[], v(0.0, -0.3))?;
        let negative_marks = ladder_marks(s, &ladder_negatives(), Palette::gold(), 0.07);
        reveal_all(s, &[reciprocal, negative_marks], 0.4)?;
        until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "continuity")?;
        let solid = Line::new(
            v(ladder_x(f64::from(LADDER_LOW)), LADDER_Y),
            v(ladder_x(f64::from(LADDER_HIGH)), LADDER_Y),
        )
        .style(stroke_style(Palette::hero(), 5.0))
        .build()?;
        let solid = s.arena_mut().add(solid);
        s.play(show_creation(solid).rt(0.9))?;

        k.until_sentence(s, "03_7", 4)?;
        let earned = k.tex(s, TEX[14], 42.0, Palette::hero(), &[], v(0.0, -3.05))?;
        reveal(s, earned)?;
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
    #[test]
    fn the_change_of_reading_is_seen_before_degree_zero_lands() {
        assert_in_order(&[
            r#"k.say(s, "03_2")?;"#,
            "let row = rect(",
            r#"until_clause(&k, s, "03_2", 0, ROWS_SENTENCE, "collect them")?;"#,
            "let along = Line::new(",
            r#"k.until_sentence(s, "03_2", 1)?;"#,
            "hide(s, along)?;",
            "color_diagonal(s, &cells, degree)?;",
        ]);
    }
    #[test]
    fn each_diagonal_and_its_total_share_a_colour_on_the_key_sentence() {
        assert_in_order(&[
            r#"until_sentence(s, "03_5", 2)"#,
            "link_diagonals(s, &cells, &totals)?;",
            "TEX[8]",
            r#"until_sentence(s, "03_5", 3)"#,
        ]);
    }
    #[test]
    fn the_name_is_earned_in_the_order_the_voice_earns_it() {
        assert_in_order(&[
            r#"k.say(s, "03_7")?;"#,
            r#"until_sentence(s, "03_7", 1)"#,
            "e_label()",
            r#"until_sentence(s, "03_7", 2)"#,
            "TEX[11]",
            r#"until_sentence(s, "03_7", 3)"#,
            "TEX[12]",
            r#"until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "reciprocals")?;"#,
            "TEX[13]",
            r#"until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "continuity")?;"#,
            "show_creation(solid)",
            r#"until_sentence(s, "03_7", 4)"#,
            "TEX[14]",
            "k.finish(s)",
        ]);
    }
}
