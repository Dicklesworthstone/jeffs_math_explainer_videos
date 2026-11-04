//! Staged isomorphism scene, not registered, built, rendered, or reviewed.
use crate::d4::{card, show_move};
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Isomorphism {
    pub kit: Kit,
}
fn fade(stage: &mut Stage<'_>, items: &[Mob], on: bool) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in items {
        let a = if on {
            fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?
        };
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}
impl SceneConstruct for Isomorphism {
    fn name(&self) -> &str {
        "12_isomorphism"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "12_1")?;
        let title = k.text(
            stage,
            "Same structure, different objects",
            36.0,
            Palette::ink(),
            v(0.0, 3.0),
        )?;
        let left = k.text(stage, "Clock moves modulo 4", 28.0, Palette::blue(), v(-3.2, 2.0))?;
        let right = k.text(stage, "Square rotations", 28.0, Palette::hero(), v(3.2, 2.0))?;
        fade(stage, &[title, left, right], true)?;
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(1.05)
                .style(stroke_style(Palette::blue(), 3.0))
                .build()
                .moved_to(v(-3.2, 0.1)),
        );
        let positions = [v(-3.2, 0.9), v(-2.4, 0.1), v(-3.2, -0.7), v(-4.0, 0.1)];
        let mut opening = vec![ring];
        for (i, &at) in positions.iter().enumerate() {
            opening.push(k.text(stage, &i.to_string(), 25.0, Palette::ink(), at)?);
        }
        fade(stage, &opening, true)?;
        let square = card(stage, &k, (3.2, 0.1), "r")?;
        k.until_sentence(stage, "12_1", 1)?;
        fade(stage, &square.objects, true)?;
        k.until_sentence(stage, "12_1", 2)?;
        let marker = stage.arena_mut().add(
            Circle::new()
                .radius(0.11)
                .style(ink_style(Palette::gold()))
                .build()
                .moved_to(v(-3.2, 0.63)),
        );
        fade(stage, &[marker], true)?;
        let movement = marker
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .move_to(v(-2.4, -0.17), ORIGIN)?;
        stage.play(movement)?;
        show_move(stage, &square.markers, (3.2, 0.1), 1, false)?;
        opening.push(marker);
        opening.extend(square.objects);
        k.say(stage, "12_2")?;
        fade(stage, &opening, false)?;
        let mut displayed = vec![title, left, right];
        for (i, (x, name)) in [(-4.8, "e"), (-1.6, "r"), (1.6, "r²"), (4.8, "r³")]
            .into_iter()
            .enumerate()
        {
            let a = card(stage, &k, (x, 0.2), name)?;
            let clock = k.text(stage, &i.to_string(), 28.0, Palette::blue(), v(x, 1.45))?;
            let link = k.tex(stage, r"\downarrow", 28.0, Palette::green(), &[], v(x, 1.05))?;
            fade(stage, &a.objects, true)?;
            fade(stage, &[clock, link], true)?;
            if i > 0 {
                show_move(stage, &a.markers, (x, 0.2), i as u8, false)?;
            }
            displayed.extend(a.objects);
            displayed.extend([clock, link]);
        }
        k.until_sentence(stage, "12_2", 2)?;
        let q = k.text(stage, "2 + 3 on the four-clock?", 30.0, Palette::gold(), v(0.0, -2.3))?;
        fade(stage, &[q], true)?;
        displayed.push(q);
        k.hold(stage)?;
        stage.wait(3.0)?;
        k.say(stage, "12_3")?;
        let ex = k.tex(
            stage,
            r"2+3\equiv1\pmod4\quad\longleftrightarrow\quad r^2r^3=r",
            33.0,
            Palette::green(),
            &[],
            v(0.0, -3.0),
        )?;
        fade(stage, &[ex], true)?;
        displayed.push(ex);
        k.say(stage, "12_4")?;
        fade(stage, &displayed, false)?;
        let map = k.tex(stage, r"\phi(k)=r^k", 40.0, Palette::blue(), &[], v(0.0, 1.6))?;
        let preserve = k.tex(
            stage,
            r"\phi(a+b)=r^{a+b}=r^ar^b=\phi(a)\phi(b)",
            35.0,
            Palette::green(),
            &[],
            v(0.0, 0.4),
        )?;
        let four = k.tex(stage, r"r^4=e", 36.0, Palette::hero(), &[], v(0.0, -0.8))?;
        fade(stage, &[map], true)?;
        k.until_sentence(stage, "12_4", 1)?;
        fade(stage, &[preserve], true)?;
        k.until_sentence(stage, "12_4", 3)?;
        fade(stage, &[four], true)?;
        k.say(stage, "12_5")?;
        let words = k.text(
            stage,
            "Bijection + operation preservation",
            30.0,
            Palette::ink(),
            v(0.0, -2.0),
        )?;
        fade(stage, &[words], true)?;
        k.say(stage, "12_6")?;
        fade(stage, &[map, preserve, four, words], false)?;
        let same = k.tex(
            stage,
            r"g^m=e\quad\Longleftrightarrow\quad\phi(g)^m=e",
            36.0,
            Palette::blue(),
            &[],
            v(0.0, 0.8),
        )?;
        let orders = k.text(
            stage,
            "The first positive return is preserved",
            31.0,
            Palette::green(),
            v(0.0, -0.5),
        )?;
        fade(stage, &[same, orders], true)?;
        k.say(stage, "12_7")?;
        fade(stage, &[same, orders], false)?;
        let rotation = k.text(
            stage,
            "Square rotation: an element of order 4",
            30.0,
            Palette::blue(),
            v(0.0, 0.8),
        )?;
        let units = k.text(
            stage,
            "U(12): no element has order 4",
            30.0,
            Palette::hero(),
            v(0.0, -0.4),
        )?;
        fade(stage, &[rotation, units], true)?;
        k.say(stage, "12_8")?;
        fade(stage, &[rotation, units], false)?;
        let action = k.tex(
            stage,
            r"L_g(x)=gx,\qquad L_gL_h=L_{gh}",
            36.0,
            Palette::ink(),
            &[],
            v(0.0, 2.1),
        )?;
        let undo = k.tex(
            stage,
            r"L_{g^{-1}}L_g=\operatorname{id}",
            36.0,
            Palette::green(),
            &[],
            v(0.0, -1.2),
        )?;
        fade(stage, &[action], true)?;
        let cycle = k.tex(
            stage,
            r"L_1:\quad 0\rightarrow1\rightarrow2\rightarrow3\rightarrow0",
            34.0,
            Palette::blue(),
            &[],
            v(0.0, 0.8),
        )?;
        k.until_sentence(stage, "12_8", 2)?;
        fade(stage, &[cycle], true)?;
        let example = k.tex(stage, r"L_1(L_2(0))=3", 34.0, Palette::hero(), &[], v(0.0, -0.2))?;
        k.until_sentence(stage, "12_8", 3)?;
        fade(stage, &[undo, example], true)?;
        k.say(stage, "12_9")?;
        let identity = k.tex(
            stage,
            r"L_g(e)=g\quad\Longrightarrow\quad L_g=L_h\Rightarrow g=h",
            33.0,
            Palette::hero(),
            &[],
            v(0.0, -2.2),
        )?;
        fade(stage, &[identity], true)?;
        k.say(stage, "12_10")?;
        fade(stage, &[action, undo, cycle, example, identity], false)?;
        let auto = k.tex(stage, r"k\longmapsto-k\pmod 4", 40.0, Palette::blue(), &[], v(0.0, 0.6))?;
        let relabel = k.text(
            stage,
            "An automorphism: structure-preserving relabelling",
            29.0,
            Palette::ink(),
            v(0.0, -0.7),
        )?;
        fade(stage, &[auto, relabel], true)?;
        k.finish(stage)
    }
}
