//! Sequential continuity on a real domain, then a nonlinear error budget.
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, say, think_pause, wait_until};
use crate::model::Frac;

pub struct Sequences {
    pub kit: Kit,
}
pub const STAGES: [i64; 4] = [1, 2, 4, 8];

/// An exact countersequence for the opening limit at its endpoint.
pub fn approaching(n: i64) -> Result<Frac, String> {
    if n < 1 {
        return Err("sequence index must be positive".to_owned());
    }
    Frac::whole(1).minus(Frac::whole(1).over(Frac::whole(n))?)
}

/// x=1/n², sqrt(x)=1/n and their ratio; computed as exact fractions.
pub fn root_sample(n: i64) -> Result<(Frac, Frac, Frac), String> {
    if n < 1 {
        return Err("sequence index must be positive".to_owned());
    }
    let y = Frac::whole(1).over(Frac::whole(n))?;
    let x = y.times(y)?;
    Ok((x, y, y.over(x)?))
}

fn tex(kit: &Kit, s: &mut Stage<'_>, text: &str, y: f64, size: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.tex(s, text, size, c, &[], v(0.0, y))
}
fn label(kit: &Kit, s: &mut Stage<'_>, text: &str, y: f64, size: f64) -> fmn::Result<Mob> {
    kit.text(s, text, size, Palette::ink(), v(0.0, y))
}

impl SceneConstruct for Sequences {
    fn name(&self) -> &str {
        "10_approaching_along_sequences"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let _line = say(&kit, stage, "10_1")?;
        let title = label(&kit, stage, "Approach along sequences", 3.45, 34.0)?;
        let rule = tex(
            &kit,
            stage,
            r"x_n\in D,\quad x_n\to a\quad\Longrightarrow\quad f(x_n)\to f(a)",
            2.15,
            34.0,
            Palette::teal(),
        )?;
        let every = label(
            &kit,
            stage,
            "continuity requires this for every approaching sequence",
            1.3,
            25.0,
        )?;
        fade_in_lagged(stage, &[title, rule, every], 0.7, 0.1)?;

        let _line = say(&kit, stage, "10_2")?;
        let seq = tex(&kit, stage, r"x_n=1-1/n\to1,\qquad n\ge1", 0.3, 34.0, Palette::blue())?;
        fade_in_all(stage, &[seq], 0.4)?;
        let axis = stage.arena_mut().add(
            Line::new(v(-5.0, -1.0), v(5.0, -1.0))
                .style(line_style(Palette::dim(), 3.0))
                .build()?,
        );
        let zero = kit.text(stage, "0", 25.0, Palette::ink(), v(-5.0, -1.6))?;
        let one = kit.text(stage, "1", 25.0, Palette::ink(), v(5.0, -1.6))?;
        fade_in_all(stage, &[axis, zero, one], 0.4)?;
        let mut old = None;
        for n in STAGES {
            let x = approaching(n).map_err(fail)?;
            let dot = stage.arena_mut().add(
                Dot::new()
                    .point(v(-5.0 + 10.0 * x.f64(), -1.0))
                    .radius(0.09)
                    .style(flat(Palette::blue(), 1.0))
                    .build(),
            );
            let tag = tex(
                &kit,
                stage,
                &format!(r"n={n}:\quad x_n={x}"),
                -2.3,
                29.0,
                Palette::blue(),
            )?;
            if let Some(o) = old {
                fade_out_all(stage, &[o], 0.15)?;
            }
            fade_in_all(stage, &[dot, tag], 0.35)?;
            stage.wait(0.65)?;
            old = Some(tag);
        }
        let _line = say(&kit, stage, "10_3")?;
        let fail_line = tex(&kit, stage, r"f(x_n)=0\ne1=f(1)", -3.15, 34.0, Palette::gold())?;
        fade_in_all(stage, &[fail_line], 0.5)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _question = say(&kit, stage, "10_4")?;
        let question = label(&kit, stage, "Can checking several good sequences settle it?", 2.7, 30.0)?;
        let mark = label(&kit, stage, "?", 0.0, 46.0)?;
        fade_in_all(stage, &[question, mark], 0.5)?;
        think_pause(&kit, stage, v(0.0, 0.0), 0.5)?;
        fade_out_all(stage, &[mark], 0.2)?;
        let _line = say(&kit, stage, "10_5")?;
        let answer = label(&kit, stage, "No. Every approaching sequence must work.", 1.75, 29.0)?;
        let failure = tex(
            &kit,
            stage,
            r"\exists\varepsilon_0>0\quad\forall\delta>0\quad\exists x\in D",
            0.6,
            32.0,
            Palette::ink(),
        )?;
        let bad = tex(
            &kit,
            stage,
            r"|x-a|<\delta,\qquad |f(x)-f(a)|\ge\varepsilon_0",
            -0.15,
            32.0,
            Palette::gold(),
        )?;
        fade_in_lagged(stage, &[answer, failure, bad], 0.6, 0.1)?;
        let _line = say(&kit, stage, "10_6")?;
        let choice = tex(
            &kit,
            stage,
            r"\delta=1/n:\quad |x_n-a|<1/n\to0",
            -1.3,
            32.0,
            Palette::blue(),
        )?;
        let miss = tex(
            &kit,
            stage,
            r"|f(x_n)-f(a)|\ge\varepsilon_0\quad\text{for every }n",
            -2.15,
            31.0,
            Palette::gold(),
        )?;
        let conclusion = label(
            &kit,
            stage,
            "failure of continuity produces a failing sequence",
            -3.0,
            26.0,
        )?;
        fade_in_lagged(stage, &[choice, miss, conclusion], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 10.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "10_7")?;
        let title2 = label(&kit, stage, "Continuity need not give a linear budget", 3.45, 33.0)?;
        let sqrt = tex(
            &kit,
            stage,
            r"g(x)=\sqrt{x},\quad x\ge0,\quad a=0",
            2.4,
            36.0,
            Palette::teal(),
        )?;
        let delta = tex(
            &kit,
            stage,
            r"\delta=\varepsilon^2:\quad 0\le x<\delta\ \Longrightarrow\ \sqrt{x}<\varepsilon",
            1.4,
            33.0,
            Palette::ink(),
        )?;
        fade_in_lagged(stage, &[title2, sqrt, delta], 0.7, 0.1)?;
        let _line = say(&kit, stage, "10_8")?;
        let columns = [-3.1, -1.15, 1.2, 3.5];
        let headers = [r"n", r"x=1/n^2", r"\sqrt{x}=1/n", r"\sqrt{x}/x"];
        let mut heading = Vec::new();
        for (x, value) in columns.iter().zip(headers) {
            heading.push(kit.tex(stage, value, 28.0, Palette::ink(), &[], v(*x, 0.2))?);
        }
        fade_in_all(stage, &heading, 0.4)?;
        for (i, n) in [2, 4, 8].into_iter().enumerate() {
            let (x, y, r) = root_sample(n).map_err(fail)?;
            let values = [n.to_string(), x.to_string(), y.to_string(), r.to_string()];
            let mut row = Vec::new();
            for (at, value) in columns.iter().zip(&values) {
                row.push(kit.tex(stage, value, 30.0, Palette::blue(), &[], v(*at, -0.7 - i as f64 * 0.65))?);
            }
            fade_in_all(stage, &row, 0.4)?;
            stage.wait(0.65)?;
        }
        let _line = say(&kit, stage, "10_9")?;
        let ratio = tex(
            &kit,
            stage,
            r"\frac{|g(1/n^2)-g(0)|}{|1/n^2-0|}=n\to\infty",
            -2.8,
            32.0,
            Palette::gold(),
        )?;
        fade_in_all(stage, &[ratio], 0.5)?;
        let _line = say(&kit, stage, "10_10")?;
        let next = label(&kit, stage, "next: continuity through open sets", -3.5, 24.0)?;
        fade_in_all(stage, &[next], 0.4)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approaching_stays_in_domain_and_closes_exact_distance() {
        for n in 1..=256 {
            let x = approaching(n).unwrap();
            assert!(x >= Frac::whole(0) && x < Frac::whole(1));
            assert_eq!(
                Frac::whole(1).minus(x).unwrap().times(Frac::whole(n)).unwrap(),
                Frac::whole(1)
            );
            assert_eq!(crate::model::limit(x.f64()).unwrap(), 0.0);
        }
        assert!(approaching(0).is_err());
    }
    #[test]
    fn root_ratio_diverges_while_both_errors_shrink() {
        let mut last = Frac::whole(2);
        for n in 1..=256 {
            let (x, y, r) = root_sample(n).unwrap();
            assert_eq!(y.times(y).unwrap(), x);
            assert_eq!(r, Frac::whole(n));
            assert!(y < last);
            last = y;
        }
        assert!(root_sample(0).is_err());
    }
}
