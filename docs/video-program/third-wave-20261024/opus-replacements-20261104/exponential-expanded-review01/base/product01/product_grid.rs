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
];
fn reveal(s: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
    let a = fade_in(s.arena_mut(), m, ORIGIN, 1.0)?;
    s.play(a.rt(0.35))?;
    Ok(())
}
fn hide(s: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
    let a = fade_out(s.arena_mut(), m, ORIGIN, 1.0)?;
    s.play(a.rt(0.25))?;
    Ok(())
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
                reveal(s, m)?;
                cells.push((i, j, m));
                objects.push(m);
            }
        }
        for j in 0..=SIDE {
            let m = s.arena_mut().add(
                Line::new(v(-4.85, 1.9 - j as f64), v(0.5, 1.9 - j as f64))
                    .style(stroke_style(Palette::dim(), 1.0))
                    .build()?,
            );
            reveal(s, m)?;
            objects.push(m);
        }
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
                let m = k.tex(
                    s,
                    &total_tex(degree),
                    28.0,
                    Palette::green(),
                    &[],
                    v(3.0, 1.7 - degree as f64),
                )?;
                reveal(s, m)?;
                totals.push(m);
            }
        }
        let question = k.text(s, TEXT[2], 23.0, Palette::hero(), v(2.9, -1.1))?;
        reveal(s, question)?;
        k.hold(s)?;
        s.wait(4.0)?;
        hide(s, question)?;
        k.say(s, "03_3")?;
        for m in totals {
            hide(s, m)?;
        }
        let expression = k.tex(s, TEX[0], 27.0, Palette::blue(), &[], v(3.0, 0.5))?;
        reveal(s, expression)?;
        for (sentence, index) in [(1, 1), (3, 2)] {
            k.until_sentence(s, "03_3", sentence)?;
            let to = k.tex(s, TEX[index], 27.0, Palette::blue(), &[], v(3.0, 0.5))?;
            s.play(Transform::new(expression, to).rt(0.6))?;
        }
        k.hold(s)?;
        k.say(s, "03_4")?;
        color_diagonal(s, &cells, 3)?;
        let third = k.tex(s, &total_tex(3), 28.0, Palette::green(), &[], v(3.0, -0.5))?;
        reveal(s, third)?;
        k.until_sentence(s, "03_4", 1)?;
        let general = k.tex(s, TEX[3], 27.0, Palette::ink(), &[], v(2.9, 1.6))?;
        reveal(s, general)?;
        k.hold(s)?;
        k.say(s, "03_5")?;
        let permission = k.text(s, TEXT[4], 22.0, Palette::gold(), v(0.0, -3.1))?;
        reveal(s, permission)?;
        k.until_sentence(s, "03_5", 3)?;
        let law = k.tex(s, TEX[4], 33.0, Palette::green(), &[], v(2.8, -1.55))?;
        reveal(s, law)?;
        k.hold(s)?;
        let mut fades = Vec::new();
        objects.extend([expression, third, general, permission, law]);
        for m in objects {
            let a = fade_out(s.arena_mut(), m, ORIGIN, 1.0)?;
            fades.push(s.prepare(a.rt(0.5))?);
        }
        s.play_prepared(fades)?;
        k.say(s, "03_6")?;
        let nonzero = k.tex(s, TEX[5], 32.0, Palette::blue(), &[], v(0.0, 1.2))?;
        reveal(s, nonzero)?;
        k.until_sentence(s, "03_6", 2)?;
        let positive = k.tex(s, TEX[6], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        reveal(s, positive)?;
        k.finish(s)
    }
}
