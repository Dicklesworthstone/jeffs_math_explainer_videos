//! First chapter — pose the counting question (`01_1`), morph the symbolic
//! formula into numbers, and count up live to the exact answer (`01_2`).

use explainer_kit::{Kit, Timed, fade_all, play, thousands, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use crate::{facts, palette};

pub struct Intro {
    pub kit: Kit,
}

/// "How many ways to choose 4 of 5,000 points?"
pub fn question_text() -> String {
    format!(
        "How many ways to choose {} of {} points?",
        facts::PICK,
        thousands(facts::POINTS)
    )
}

/// `C(N, 4) = \frac{N(N-1)(N-2)(N-3)}{24}`: the falling product over k!.
pub fn formula_tex() -> String {
    let k = facts::PICK;
    let product: String = (0..k)
        .map(|i| match i {
            0 => "N".to_owned(),
            _ => format!("(N-{i})"),
        })
        .collect();
    format!(r"C(N, {k}) = \frac{{{product}}}{{{}}}", (1..=k).product::<u64>())
}

/// `C(5000, 4) = \frac{5000\cdot 4999\cdot 4998\cdot 4997}{24}`: the formula at N.
pub fn numbers_tex() -> String {
    let (n, k) = (facts::POINTS, facts::PICK);
    let product: Vec<String> = (0..k).map(|i| (n - i).to_string()).collect();
    format!(
        r"C({n}, {k}) = \frac{{{}}}{{{}}}",
        product.join(r"\cdot "),
        (1..=k).product::<u64>()
    )
}

impl SceneConstruct for Intro {
    fn name(&self) -> &str {
        "01_intro"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 01_1 — the question in words and in symbols.
        let question = kit.text(stage, &question_text(), 44.0, palette::ink(), v(0.0, 2.6))?;
        let (sym, sym_spans) = kit.tex_spans(
            stage,
            &formula_tex(),
            52.0,
            palette::ink(),
            &[("N", palette::question())],
            v(0.0, 0.6),
        )?;
        kit.say(stage, "01_1")?;
        play!(stage; write(stage.arena(), question).rt(1.4));
        play!(stage; write(stage.arena(), sym).rt(1.8));
        stage.wait(0.8)?;

        // 01_2 — morph N -> 5000 and count up live to the exact value.
        let n = facts::POINTS.to_string();
        let (num, num_spans) = kit.tex_spans(
            stage,
            &numbers_tex(),
            52.0,
            palette::ink(),
            &[(n.as_str(), palette::question())],
            v(0.0, 0.6),
        )?;
        kit.say(stage, "01_2")?;
        let morph = fmn::library::TransformMatchingTex::new(stage.arena_mut(), sym, num, &sym_spans, &num_spans)?;
        play!(stage; morph.rt(1.6));

        let total = facts::quadruples() as f64;
        let t = stage.add_value_tracker(0.0);
        kit.live_count(stage, t, 72.0, palette::hero(), v(0.0, -1.6))?;
        stage.play(
            t.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.4),
                    rate_func: Some(fmn::core::rate::rush_into),
                    ..AnimateArgs::default()
                })?
                .set_value(total)?,
        )?;
        let boxed = SurroundingRectangle::from_extent(Some((v(-3.3, -2.25), v(3.3, -0.95))))
            .color(palette::hero())
            .build();
        let boxed = stage.arena_mut().add(boxed);
        kit.mark(stage, "count_reveal");
        play!(stage; show_creation(boxed).rt(0.8));
        play!(stage; indicate(stage.arena_mut(), question, 1.06, None)?.rt(0.9));
        kit.listen(stage, 1.5, "count_reveal")?;

        fade_all(stage, 1.0)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::glyphs;

    #[test]
    fn glyph_probe_covers_intro_strings() {
        let text = question_text();
        assert!(glyphs::TEXT.contains(&text.as_str()), "not probed: {text}");
        for tex in [formula_tex(), numbers_tex()] {
            assert!(glyphs::TEX.contains(&tex.as_str()), "not probed: {tex}");
        }
    }
}
