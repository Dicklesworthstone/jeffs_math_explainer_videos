//! Chapter 09: the epsilon/3 proof, as a detour between four values.
//! The path is schematic; the equal boxes allocate error budgets, not
//! measured distances. Uniform convergence chooses N before x, then
//! continuity of that fixed function supplies delta at a.

use explainer_kit::{Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    fact, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, say, think_pause, wait_until,
};
use super::limit_that_jumps::read_s;
use crate::facts;

pub struct ErrorBridge {
    pub kit: Kit,
}

pub const TERMS: [&str; 3] = [r"|f(x)-f_N(x)|", r"|f_N(x)-f_N(a)|", r"|f_N(a)-f(a)|"];
const VALUES: [&str; 4] = [r"f(x)", r"f_N(x)", r"f_N(a)", r"f(a)"];
const POS: [Vec3; 4] = [[-5.4, 1.55, 0.0], [-1.8, 2.0, 0.0], [1.8, 2.0, 0.0], [5.4, 1.55, 0.0]];
const CELL_X: [f64; 3] = [-3.6, 0.0, 3.6];
const TITLE: &str = "Carry continuity through the limit";
const ASSUMPTION: &str = "continuous functions, uniform convergence";
const BUDGETS: &str = "three error budgets";
const ORDER: &str = "tolerance → one fixed curve → input window → every nearby input";
const SCOPE: &str = "uniform convergence is sufficient";
const CAVEAT: &str = "a continuous limit need not arise uniformly";
const NEXT: &str = "next: approaching along sequences";
const READ_S: f64 = 10.0;

fn ink() -> Srgb {
    Palette::ink()
}

fn tex(kit: &Kit, stage: &mut Stage<'_>, s: &str, at: Vec3, size: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, s, size, color, &[], at)
}

fn budget(stage: &mut Stage<'_>, x: f64, c: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(rect(v(x - 1.5, -0.65), 3.0, 0.4, flat(c, 0.25))?))
}

impl SceneConstruct for ErrorBridge {
    fn name(&self) -> &str {
        "09_the_error_bridge"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let k = fact(&f, "C9.pieces")?;
        let share = format!(r"\varepsilon/{k}");
        let colors = [Palette::blue(), Palette::teal(), Palette::blue()];
        let _line = say(&kit, stage, "09_1")?;
        let title = kit.text(stage, TITLE, 34.0, ink(), v(0.0, 3.5))?;
        let hypothesis = kit.text(stage, ASSUMPTION, 24.0, Palette::dim(), v(0.0, 2.7))?;
        let point = tex(&kit, stage, r"a\in I,\qquad\varepsilon>0", v(0.0, 1.7), 36.0, ink())?;
        fade_in_lagged(stage, &[title, hypothesis, point], 0.7, 0.1)?;

        let line = say(&kit, stage, "09_2")?;
        fade_out_all(stage, &[hypothesis, point], 0.4)?;
        let mut labels = Vec::new();
        for (i, at) in POS.iter().enumerate() {
            let d = Dot::new()
                .point(*at)
                .radius(0.07)
                .style(flat(Palette::gold(), 1.0))
                .build();
            labels.push(stage.arena_mut().add(d));
            labels.push(tex(&kit, stage, VALUES[i], v(at[0], at[1] + 0.45), 34.0, ink())?);
        }
        fade_in_lagged(stage, &labels, 0.6, 0.08)?;
        line.wait_for(stage, "three gaps")?;
        let mut paths = Vec::new();
        for i in 0..TERMS.len() {
            paths.push(
                stage.arena_mut().add(
                    Line::new(POS[i], POS[i + 1])
                        .style(line_style(colors[i], 5.0))
                        .build()?,
                ),
            );
            paths.push(tex(&kit, stage, TERMS[i], v(CELL_X[i], 0.8), 29.0, colors[i])?);
        }
        fade_in_lagged(stage, &paths, 0.7, 0.08)?;
        let caption = kit.text(stage, BUDGETS, 23.0, Palette::dim(), v(0.0, 0.0))?;
        fade_in_all(stage, &[caption], 0.3)?;

        let _line = say(&kit, stage, "09_3")?;
        let mut boxes = Vec::new();
        for i in [0, 2] {
            boxes.push(budget(stage, CELL_X[i], colors[i])?);
            boxes.push(tex(
                &kit,
                stage,
                &format!(r"< {share}"),
                v(CELL_X[i], -0.45),
                29.0,
                colors[i],
            )?);
        }
        fade_in_lagged(stage, &boxes, 0.7, 0.1)?;
        let outer = tex(
            &kit,
            stage,
            &format!(r"\text{{choose }}N:\quad |f_N(y)-f(y)|<{share}\quad\forall y\in I"),
            v(0.0, -1.3),
            29.0,
            colors[0],
        )?;
        fade_in_all(stage, &[outer], 0.4)?;

        let _line = say(&kit, stage, "09_4")?;
        let middle_box = budget(stage, CELL_X[1], colors[1])?;
        let middle_share = tex(&kit, stage, &format!(r"<{share}"), v(0.0, -0.45), 29.0, colors[1])?;
        let middle = tex(
            &kit,
            stage,
            &format!(r"x\in I,\ |x-a|<\delta\ \Longrightarrow\ |f_N(x)-f_N(a)|<{share}"),
            v(0.0, -2.0),
            29.0,
            colors[1],
        )?;
        fade_in_lagged(stage, &[middle_box, middle_share, middle], 0.7, 0.1)?;
        let read = stage.scene().time().to_f64() + READ_S;

        let _question = say(&kit, stage, "09_5")?;
        let ask = kit.text(stage, "?", 42.0, Palette::hero(), v(0.0, -2.9))?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, v(0.0, -2.9), 0.35)?;
        wait_until(stage, read)?;
        fade_out_all(stage, &[ask], 0.2)?;

        let _line = say(&kit, stage, "09_6")?;
        let chain = format!(r"|f(x)-f(a)|\le {}+{}+{}<\varepsilon", TERMS[0], TERMS[1], TERMS[2]);
        let sum = tex(&kit, stage, &chain, v(0.0, -2.9), 26.0, ink())?;
        fade_in_all(stage, &[sum], 0.5)?;
        let proof_read = stage.scene().time().to_f64() + READ_S;
        let _line = say(&kit, stage, "09_7")?;
        let result = kit.text(
            stage,
            "a was arbitrary: f is continuous on I",
            25.0,
            Palette::gold(),
            v(0.0, -3.5),
        )?;
        fade_in_all(stage, &[result], 0.4)?;
        wait_until(stage, proof_read)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "09_8")?;
        let order_title = kit.text(stage, "The order makes the proof work", 34.0, ink(), v(0.0, 2.85))?;
        let order = kit.text(stage, ORDER, 27.0, ink(), v(0.0, 1.65))?;
        let before_x = tex(
            &kit,
            stage,
            r"\varepsilon\ \longrightarrow\ N\ \longrightarrow\ \delta\ \longrightarrow\ x\in I",
            v(0.0, 0.5),
            36.0,
            Palette::teal(),
        )?;
        fade_in_lagged(stage, &[order_title, order, before_x], 0.7, 0.1)?;
        let _line = say(&kit, stage, "09_9")?;
        let scope = kit.text(stage, SCOPE, 30.0, Palette::gold(), v(0.0, -0.7))?;
        let caveat = kit.text(stage, CAVEAT, 26.0, ink(), v(0.0, -1.45))?;
        let next = kit.text(stage, NEXT, 25.0, Palette::dim(), v(0.0, -2.6))?;
        fade_in_lagged(stage, &[scope, caveat, next], 0.7, 0.1)?;
        // they stay until read, counted once from their completed fade; with
        // 09_8 as short as it is, this reading, not the voice, can end the
        // chapter
        let tail_up = stage.scene().time().to_f64();
        wait_until(stage, tail_up + read_s(SCOPE) + read_s(CAVEAT) + read_s(NEXT))?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}
