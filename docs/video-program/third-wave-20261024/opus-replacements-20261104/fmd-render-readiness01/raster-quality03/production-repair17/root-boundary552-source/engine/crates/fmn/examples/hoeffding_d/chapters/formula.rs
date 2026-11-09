//! Chapter 6 — three sums, one statistic: D1, D2, D3 on the worked
//! example (counted up live), Hoeffding's normalization, and the answer.

use fmn::animation::{indicate, replacement_transform};
use fmn::prelude::*;

use crate::kit::{Kit, Palette, Timed, fade_all, tween, v};
use crate::play;
use crate::stats::{self, HEIGHTS, WEIGHTS};

pub struct Formula {
    pub kit: Kit,
}

fn colors() -> Vec<(&'static str, Srgb)> {
    vec![
        ("Q", Palette::q()),
        ("R", Palette::x_rank()),
        ("S", Palette::y_rank()),
        ("D_1", Palette::d1()),
        ("D_2", Palette::d2()),
        ("D_3", Palette::d3()),
    ]
}

impl SceneConstruct for Formula {
    fn name(&self) -> &str {
        "06_formula"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let h = stats::hoeffding(&HEIGHTS, &WEIGHTS);
        let c = colors();
        let title = kit.text(
            stage,
            "Step 3:  three sums, one statistic",
            40.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        kit.say(stage, "f1")?;
        play!(stage; write(stage.arena(), title).rt(1.2));

        let sums = [
            (
                r"D_1 = \sum_i (Q_i-1)(Q_i-2)",
                h.d1,
                2usize,
                Palette::d1(),
                "how tightly points pile up below-left of each other",
            ),
            (
                r"D_2 = \sum_i (R_i-1)(R_i-2)(S_i-1)(S_i-2)",
                h.d2,
                0,
                Palette::d2(),
                "what the ranks alone (the marginals) would give",
            ),
            (
                r"D_3 = \sum_i (R_i-2)(S_i-2)(Q_i-1)",
                h.d3,
                1,
                Palette::d3(),
                "the cross term tying the two together",
            ),
        ];
        let mut ys = Vec::new();
        for (k, &(src, value, places, color, gloss)) in sums.iter().enumerate() {
            let y = 2.35 - 1.3 * k as f64;
            ys.push(y);
            let line = kit.tex_left(stage, src, 38.0, Palette::ink(), &c, v(-6.6, y))?;
            if k > 0 {
                kit.say(stage, ["f2", "f3"][k - 1])?;
            }
            play!(stage; write(stage.arena(), line).rt(1.4));
            let eq = kit.tex(stage, "=", 38.0, Palette::ink(), &[], v(3.55, y))?;
            let t = stage.add_value_tracker(0.0);
            play!(stage; fade_in(stage.arena_mut(), eq, ORIGIN, 1.0)?.rt(0.3));
            kit.live_number(stage, t, places, false, 40.0, color, v(3.95, y), LEFT)?;
            let note = kit.text_left(stage, gloss, 22.0, Palette::dim(), v(-6.0, y - 0.62))?;
            play!(stage;
                tween(t, value, 1.4)?,
                fade_in(stage.arena_mut(), note, v(0.0, 0.1), 1.0)?.rt(1.4),
            );
            stage.wait(0.6)?;
        }

        // Hoeffding's normalization.
        let (big, big_spans) = kit.tex_spans(
            stage,
            r"D = 30\,\frac{(N-2)(N-3)\,D_1 + D_2 - 2(N-2)\,D_3}{N(N-1)(N-2)(N-3)(N-4)}",
            44.0,
            Palette::ink(),
            &c,
            v(0.0, -2.15),
        )?;
        kit.say(stage, "f4")?;
        play!(stage; write(stage.arena(), big).rt(2.0));
        stage.wait(1.4)?;
        let n_is = kit.tex(stage, r"N = 10", 36.0, Palette::dim(), &[], v(5.6, -3.45))?;
        kit.say(stage, "f5")?;
        play!(stage; fade_in(stage.arena_mut(), n_is, ORIGIN, 1.0)?.rt(0.6));
        let (numeric, numeric_spans) = kit.tex_spans(
            stage,
            r"D = 30\cdot\frac{8\cdot 7\cdot 196.25 + 10696 - 2\cdot 8\cdot 1329.5}{10\cdot 9\cdot 8\cdot 7\cdot 6}",
            44.0,
            Palette::ink(),
            &[("196.25", Palette::d1()), ("10696", Palette::d2()), ("1329.5", Palette::d3())],
            v(0.0, -2.15),
        )?;
        let morph = fmn::library::TransformMatchingTex::new(
            stage.arena_mut(),
            big,
            numeric,
            &big_spans,
            &numeric_spans,
        )?;
        play!(stage; morph.rt(1.8));
        stage.wait(1.4)?;
        let (simplified, simplified_spans) = kit.tex_spans(
            stage,
            r"D = 30\cdot\frac{414}{30240}",
            48.0,
            Palette::ink(),
            &[],
            v(0.0, -2.15),
        )?;
        let morph = fmn::library::TransformMatchingTex::new(
            stage.arena_mut(),
            numeric,
            simplified,
            &numeric_spans,
            &simplified_spans,
        )?;
        kit.say(stage, "f6")?;
        play!(stage; morph.rt(1.4));
        stage.wait(0.8)?;
        let answer = kit.tex(
            stage,
            &format!(r"D = {:.4}", h.d),
            64.0,
            Palette::hoeffding(),
            &[],
            v(0.0, -2.15),
        )?;
        kit.chime(stage, 880.0)?;
        play!(stage; replacement_transform(simplified, answer).rt(1.2));
        let boxed = SurroundingRectangle::from_extent(Some((v(-1.9, -2.75), v(1.9, -1.55))))
            .color(Palette::hoeffding())
            .build();
        let boxed = stage.arena_mut().add(boxed);
        play!(stage;
            show_creation(boxed).rt(0.8),
            indicate(stage.arena_mut(), answer, 1.15, None)?.rt(1.2),
        );
        stage.wait(1.5)?;

        // Put it on its scale.
        let others = kit.tex(
            stage,
            &format!(
                r"\text{{Pearson }} r = {:.2} \qquad \text{{Kendall }} \tau = {:.2}",
                stats::pearson(&HEIGHTS, &WEIGHTS),
                stats::kendall(&HEIGHTS, &WEIGHTS)
            ),
            30.0,
            Palette::dim(),
            &[],
            v(0.0, -3.35),
        )?;
        kit.say(stage, "f7")?;
        play!(stage; fade_out(stage.arena_mut(), n_is, ORIGIN, 1.0)?, fade_in(stage.arena_mut(), others, v(0.0, 0.15), 1.0)?);
        stage.wait(1.4)?;
        let scale = kit.text(
            stage,
            "D has its own scale: ≈ 0 under independence, 1 for perfectly monotone data, never below −0.5.",
            26.0,
            Palette::ink(),
            v(0.0, -3.35),
        )?;
        kit.say(stage, "f8")?;
        play!(stage; replacement_transform(others, scale).rt(1.0));
        stage.wait(1.5)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 5)?;
        Ok(())
    }
}
