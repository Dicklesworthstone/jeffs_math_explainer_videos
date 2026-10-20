//! Chapter 08: move the point-choice across the stage-choice. The same
//! checked powers that opened the video now distinguish pointwise from
//! uniform convergence. Taking n = N is enough to defeat every proposed
//! uniform stage; it says nothing against convergence at a fixed point.

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, PLOT, curve_color, curve_label, fact, fade_in_all, fade_in_lagged, fade_out_all, fail, fixed, gap_color,
    moving, say, think_pause, wait_until,
};
use super::limit_that_jumps::{half_line, witness_at};
use crate::{facts, model};

pub struct FixedOrMoving {
    pub kit: Kit,
}

const TITLE: &str = "Who chooses the point, and when?";
const POINTWISE: &str = "Pointwise convergence";
const UNIFORM: &str = "Uniform convergence";
const POINT_ORDER: &str = "point → tolerance → stage";
const UNIFORM_ORDER: &str = "tolerance → stage → every point";
const LATER: &str = "and every later curve";
const WAITS: [(&str, &str); 3] = [
    ("0.9", "C1.wait.x0.9"),
    ("0.99", "C1.wait.x0.99"),
    ("0.999", "C1.wait.x0.999"),
];
const POINT_RULE: &str = r"\forall x\in[0,1]\ \forall\varepsilon>0\ \exists N\ \forall n\ge N";
const UNIFORM_RULE: &str = r"\forall\varepsilon>0\ \exists N\ \forall n\ge N\ \forall x\in[0,1]";
const ERROR_RULE: &str = r"|f_n(x)-f(x)|<\varepsilon";
const X: f64 = 3.1;
const READ_S: f64 = 8.0;

fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, size: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, size, color, v(X, y))
}

fn math(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, size: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, s, size, color, &[], v(X, y))
}

/// Rebuild only this chapter's plot, keeping the earlier chapters untouched.
/// Text swaps are sequential; the curve and witness morph simultaneously.
fn next_stage(
    kit: &Kit,
    stage: &mut Stage<'_>,
    curve: &mut Mob,
    witness: &mut [Mob; 2],
    label: &mut Mob,
    n: u32,
) -> fmn::Result<()> {
    let next_curve = PLOT.curve(stage, n, curve_color(), BRIGHT_W)?;
    let next_witness = witness_at(stage, n)?;
    let next_label = math(kit, stage, &format!(r"n=N={n}"), -0.75, 30.0, ink())?;
    fade_out_all(stage, &[*label], 0.2)?;
    let anims = vec![
        stage.prepare(fmn::animation::replacement_transform(*curve, next_curve).rt(0.8))?,
        stage.prepare(fmn::animation::replacement_transform(witness[0], next_witness[0]).rt(0.8))?,
        stage.prepare(fmn::animation::replacement_transform(witness[1], next_witness[1]).rt(0.8))?,
    ];
    stage.play_prepared(anims)?;
    fade_in_all(stage, &[next_label], 0.2)?;
    *curve = next_curve;
    *witness = next_witness;
    *label = next_label;
    wait_until(stage, stage.scene().time().to_f64() + 1.5)
}

fn ink() -> Srgb {
    Palette::ink()
}

impl SceneConstruct for FixedOrMoving {
    fn name(&self) -> &str {
        "08_fixed_or_moving"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let initial = model::POWERS[model::POWERS.len() - 1];
        let _line = say(&kit, stage, "08_1")?;
        let title = kit.text(stage, TITLE, 32.0, ink(), v(0.0, 3.55))?;
        let mut setup = PLOT.axes(&kit, stage)?;
        setup.extend(PLOT.limit(stage)?);
        let mut curve = PLOT.curve(stage, initial, curve_color(), BRIGHT_W)?;
        let first_label = curve_label(&kit, stage, initial)?;
        setup.extend([title, curve, first_label]);
        let example = math(&kit, stage, r"f_n(x)=x^n,\quad x\in[0,1]", 3.0, 29.0, ink())?;
        let limit = math(&kit, stage, r"f(x)=0\ (x<1),\quad f(1)=1", 2.35, 26.0, Palette::gold())?;
        setup.extend([example, limit]);
        fade_in_lagged(stage, &setup, 0.7, 0.04)?;

        let line = say(&kit, stage, "08_2")?;
        let head = text(&kit, stage, POINTWISE, 1.55, 29.0, fixed())?;
        let order = text(&kit, stage, POINT_ORDER, 0.95, 24.0, ink())?;
        fade_in_all(stage, &[head, order], 0.5)?;
        line.wait_for(stage, "Beyond some stage")?;
        let later = text(&kit, stage, LATER, 0.4, 22.0, ink())?;
        let dep = math(&kit, stage, r"N=N(x,\varepsilon)", -0.15, 28.0, fixed())?;
        fade_in_all(stage, &[later, dep], 0.4)?;
        let point_rule = kit.tex(stage, POINT_RULE, 27.0, ink(), &[], v(0.0, -2.75))?;
        let error = kit.tex(stage, ERROR_RULE, 29.0, ink(), &[], v(0.0, -3.35))?;
        fade_in_lagged(stage, &[point_rule, error], 0.6, 0.1)?;
        let point_read = stage.scene().time().to_f64() + READ_S;

        let _line = say(&kit, stage, "08_3")?;
        let tolerance = math(&kit, stage, r"\varepsilon=\tfrac12", -0.85, 26.0, moving())?;
        let mut table = vec![tolerance];
        for (i, (x, key)) in WAITS.iter().enumerate() {
            let row = math(
                &kit,
                stage,
                &format!("x={x}:\\quad N={}", fact(&f, key)?),
                -1.35 - 0.4 * i as f64,
                25.0,
                fixed(),
            )?;
            table.push(row);
        }
        fade_in_lagged(stage, &table, 0.7, 0.1)?;
        wait_until(stage, point_read)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[head, order, later, dep, point_rule], 0.3)?;
        fade_out_all(stage, &table, 0.3)?;

        let line = say(&kit, stage, "08_4")?;
        let head = text(&kit, stage, UNIFORM, 1.55, 29.0, moving())?;
        let order = text(&kit, stage, UNIFORM_ORDER, 0.95, 24.0, ink())?;
        let later = text(&kit, stage, LATER, 0.4, 22.0, ink())?;
        fade_in_lagged(stage, &[head, order, later], 0.6, 0.1)?;
        line.wait_for(stage, "Choose it before")?;
        let dep = math(
            &kit,
            stage,
            r"N=N(\varepsilon)\quad\text{independent of }x",
            -0.15,
            26.0,
            moving(),
        )?;
        let uniform_rule = kit.tex(stage, UNIFORM_RULE, 27.0, ink(), &[], v(0.0, -2.75))?;
        fade_in_all(stage, &[dep, uniform_rule], 0.4)?;
        let uniform_read = stage.scene().time().to_f64() + READ_S;

        let _question = say(&kit, stage, "08_5")?;
        let ask = text(&kit, stage, "?", -1.25, 52.0, Palette::hero())?;
        fade_in_all(stage, &[ask], 0.25)?;
        think_pause(&kit, stage, v(X, -1.25), 0.5)?;
        wait_until(stage, uniform_read)?;
        fade_out_all(stage, &[ask], 0.2)?;

        let line = say(&kit, stage, "08_6")?;
        fade_out_all(stage, &[first_label], 0.2)?;
        let n = fact(&f, "C1.wait.x0.9")?
            .parse::<u32>()
            .map_err(|e| fail(e.to_string()))?;
        let next_curve = PLOT.curve(stage, n, curve_color(), BRIGHT_W)?;
        stage.play(fmn::animation::replacement_transform(curve, next_curve).rt(0.5))?;
        curve = next_curve;
        let half = half_line(&kit, stage)?;
        let mut witness = witness_at(stage, n)?;
        fade_in_all(stage, &half, 0.4)?;
        fade_in_all(stage, &witness, 0.4)?;
        let mut label = math(&kit, stage, &format!(r"n=N={n}"), -0.75, 30.0, ink())?;
        let crossing = math(&kit, stage, r"x_N=2^{-1/N}<1", -1.35, 29.0, moving())?;
        fade_in_all(stage, &[label, crossing], 0.4)?;
        line.wait_for(stage, "Below one")?;
        let miss = math(
            &kit,
            stage,
            r"|f_N(x_N)-f(x_N)|=\varepsilon=\tfrac12",
            -2.0,
            24.0,
            gap_color(),
        )?;
        fade_in_all(stage, &[miss], 0.5)?;

        let _line = say(&kit, stage, "08_7")?;
        for (_, key) in &WAITS[1..] {
            let n = fact(&f, key)?.parse::<u32>().map_err(|e| fail(e.to_string()))?;
            next_stage(&kit, stage, &mut curve, &mut witness, &mut label, n)?;
        }
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "08_8")?;
        let a = kit.text(stage, POINTWISE, 34.0, fixed(), v(0.0, 2.8))?;
        let arule = kit.tex(stage, POINT_RULE, 32.0, ink(), &[], v(0.0, 1.95))?;
        let b = kit.text(stage, UNIFORM, 34.0, moving(), v(0.0, 0.8))?;
        let brule = kit.tex(stage, UNIFORM_RULE, 32.0, ink(), &[], v(0.0, -0.05))?;
        let err = kit.tex(stage, ERROR_RULE, 34.0, ink(), &[], v(0.0, -1.2))?;
        fade_in_lagged(stage, &[a, arule, b, brule, err], 0.8, 0.1)?;
        let read = stage.scene().time().to_f64() + READ_S;
        let line = say(&kit, stage, "08_9")?;
        line.wait_for(stage, "Next")?;
        let next = kit.text(
            stage,
            "next: carry continuity through the limit",
            26.0,
            Palette::dim(),
            v(0.0, -2.55),
        )?;
        fade_in_all(stage, &[next], 0.4)?;
        wait_until(stage, read)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_proposed_stage_has_the_drawn_losing_point() {
        for n in [1, 7, 69, 693, 10_000] {
            let x = model::half_height(n).unwrap();
            assert!(0.0 < x && x < 1.0);
            assert_eq!(model::limit(x).unwrap(), 0.0);
            assert!((model::gap(x, n).unwrap() - 0.5).abs() < 1e-11);
        }
        assert_eq!(model::gap(1.0, 693).unwrap(), 0.0);
    }

    #[test]
    fn the_fixed_point_waits_start_with_strict_error_below_half() {
        let f = facts::facts().unwrap();
        for (i, (_, key)) in WAITS.iter().enumerate() {
            let p = facts::decimal_point(facts::WAITS[i]).unwrap();
            let n = model::first_within(p, model::HALF).unwrap();
            assert_eq!(f.get(key), Some(n.to_string().as_str()));
            assert!(model::gap(p.f64(), n).unwrap() < 0.5);
            assert!(model::gap(p.f64(), n - 1).unwrap() >= 0.5);
        }
    }
}
