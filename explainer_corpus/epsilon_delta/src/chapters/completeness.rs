//! A continuous rational-domain function can change sign without a zero.
//! Exact bisection illustrates the missing limit; real completeness is
//! the separate existence ingredient, not a consequence of the definition.
use super::draw::{fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, say, think_pause, wait_until};
use crate::model::Frac;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

pub struct Completeness {
    pub kit: Kit,
}
pub const RADICAND: i64 = 2;
pub const BISECTION_FACTOR: i64 = 2;
pub const ENDS: [i64; 2] = [1, 2];
pub const SHOWN_STEPS: [usize; 4] = [0, 2, 4, 6];
pub fn value(x: Frac) -> Result<Frac, String> {
    x.times(x)?.minus(Frac::whole(RADICAND))
}
pub fn bracket(steps: usize) -> Result<(Frac, Frac), String> {
    if steps > 24 {
        return Err("illustration is limited to 24 bisections".to_owned());
    }
    let (mut lo, mut hi) = (Frac::whole(ENDS[0]), Frac::whole(ENDS[1]));
    for _ in 0..steps {
        let mid = lo.plus(hi)?.over(Frac::whole(BISECTION_FACTOR))?;
        if value(mid)? < Frac::whole(0) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo, hi))
}
fn text(kit: &Kit, s: &mut Stage<'_>, t: &str, y: f64, size: f64) -> fmn::Result<Mob> {
    kit.text(s, t, size, Palette::ink(), v(0.0, y))
}
fn tex(kit: &Kit, s: &mut Stage<'_>, t: &str, y: f64, size: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.tex(s, t, size, c, &[], v(0.0, y))
}
fn line(s: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> fmn::Result<Mob> {
    Ok(s.arena_mut().add(Line::new(a, b).style(line_style(c, w)).build()?))
}
impl SceneConstruct for Completeness {
    fn name(&self) -> &str {
        "12_what_the_definition_does_not_supply"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [a, b] = ENDS;
        let factor = 2 * b;
        let _line = say(&kit, stage, "12_1")?;
        let title = text(&kit, stage, "A definition is not an existence theorem", 3.4, 33.0)?;
        let domain = tex(
            &kit,
            stage,
            &format!(r"D=\mathbb{{Q}}\cap[{a},{b}],\qquad h(q)=q^2-{RADICAND}"),
            2.1,
            36.0,
            Palette::teal(),
        )?;
        let note = text(&kit, stage, "only rational inputs belong to this domain", 1.2, 26.0)?;
        fade_in_lagged(stage, &[title, domain, note], 0.7, 0.1)?;
        let _line = say(&kit, stage, "12_2")?;
        let va = value(Frac::whole(a)).map_err(fail)?;
        let vb = value(Frac::whole(b)).map_err(fail)?;
        let signs = tex(
            &kit,
            stage,
            &format!(r"h({a})={va}<0,\qquad h({b})={vb}>0"),
            0.1,
            36.0,
            Palette::gold(),
        )?;
        fade_in_all(stage, &[signs], 0.5)?;
        let _line = say(&kit, stage, "12_3")?;
        let bound = tex(
            &kit,
            stage,
            &format!(r"|h(q)-h(r)|=|q-r||q+r|\le {factor}|q-r|"),
            -1.2,
            32.0,
            Palette::ink(),
        )?;
        let delta = tex(
            &kit,
            stage,
            &format!(
                r"\delta=\varepsilon/{factor}:\quad q,r\in D,\ |q-r|<\delta\ \Longrightarrow\ |h(q)-h(r)|<\varepsilon"
            ),
            -2.25,
            28.0,
            Palette::teal(),
        )?;
        fade_in_lagged(stage, &[bound, delta], 0.7, 0.1)?;
        kit.hold(stage)?;
        let _question = say(&kit, stage, "12_4")?;
        let mark = text(&kit, stage, "?", -3.0, 40.0)?;
        fade_in_all(stage, &[mark], 0.3)?;
        think_pause(&kit, stage, v(0.0, -3.0), 0.25)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "12_5")?;
        let absent = text(&kit, stage, "The required zero is outside the domain", 3.35, 33.0)?;
        let root = tex(
            &kit,
            stage,
            &format!(r"h(q)=0\ \Longleftrightarrow\ q=\sqrt{{{RADICAND}}}\notin\mathbb{{Q}}\quad(q>0)"),
            2.1,
            35.0,
            Palette::gold(),
        )?;
        let fraction = tex(
            &kit,
            stage,
            &format!(r"\sqrt{{{RADICAND}}}=p/q\quad\text{{in lowest terms}},\qquad p^2={RADICAND}q^2"),
            0.8,
            32.0,
            Palette::ink(),
        )?;
        let even = tex(
            &kit,
            stage,
            &format!(r"p={RADICAND}k\quad\Longrightarrow\quad q^2={RADICAND}k^2"),
            -0.3,
            34.0,
            Palette::blue(),
        )?;
        let contradiction = text(&kit, stage, "both p and q even contradict lowest terms", -1.5, 28.0)?;
        fade_in_lagged(stage, &[absent, root, fraction, even, contradiction], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 12.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "12_6")?;
        let nested = text(&kit, stage, "Rational brackets, missing rational limit", 3.4, 33.0)?;
        let axis = line(stage, v(-5.0, 1.7), v(5.0, 1.7), Palette::dim(), 3.0)?;
        let labels = [
            kit.text(stage, &a.to_string(), 25.0, Palette::ink(), v(-5.0, 1.2))?,
            kit.text(stage, &b.to_string(), 25.0, Palette::ink(), v(5.0, 1.2))?,
        ];
        fade_in_all(stage, &[nested, axis, labels[0], labels[1]], 0.6)?;
        let mut previous = Vec::new();
        for steps in SHOWN_STEPS {
            let (lo, hi) = bracket(steps).map_err(fail)?;
            let left = -5.0 + 10.0 * (lo.f64() - a as f64) / (b - a) as f64;
            let right = -5.0 + 10.0 * (hi.f64() - a as f64) / (b - a) as f64;
            let segment = line(stage, v(left, 1.7), v(right, 1.7), Palette::teal(), 8.0)?;
            let p = stage.arena_mut().add(
                Dot::new()
                    .point(v(left, 1.7))
                    .radius(0.06)
                    .style(flat(Palette::blue(), 1.0))
                    .build(),
            );
            let q = stage.arena_mut().add(
                Dot::new()
                    .point(v(right, 1.7))
                    .radius(0.06)
                    .style(flat(Palette::gold(), 1.0))
                    .build(),
            );
            let endpoints = tex(
                &kit,
                stage,
                &format!(r"\text{{lower}}={lo},\qquad\text{{upper}}={hi}"),
                0.25,
                31.0,
                Palette::ink(),
            )?;
            let width = hi.minus(lo).map_err(fail)?;
            let width_label = tex(
                &kit,
                stage,
                &format!(r"\text{{after }}{steps}\text{{ bisections: width}}={width}"),
                -0.75,
                30.0,
                Palette::teal(),
            )?;
            if !previous.is_empty() {
                fade_out_all(stage, &previous, 0.2)?;
            }
            previous = vec![segment, p, q, endpoints, width_label];
            fade_in_all(stage, &previous, 0.4)?;
            stage.wait(1.3)?;
        }
        let limit = tex(
            &kit,
            stage,
            &format!(
                r"a_n\uparrow\sqrt{{{RADICAND}}},\qquad b_n\downarrow\sqrt{{{RADICAND}}}\quad\text{{in }}\mathbb{{R}}"
            ),
            -2.0,
            34.0,
            Palette::gold(),
        )?;
        fade_in_all(stage, &[limit], 0.5)?;
        let warning = text(&kit, stage, "no rational limit can satisfy h(q)=0", -3.0, 27.0)?;
        fade_in_all(stage, &[warning], 0.4)?;
        // The narrated cue can end before this final formula and note appear.
        wait_until(stage, stage.scene().time().to_f64() + 6.0)?;
        kit.hold(stage)?;

        fade_all(stage, 0.5)?;
        let _line = say(&kit, stage, "12_7")?;
        let complete = text(
            &kit,
            stage,
            "Completeness supplies the missing existence step",
            3.3,
            32.0,
        )?;
        let real = tex(
            &kit,
            stage,
            &format!(r"D=[{a},{b}]\subset\mathbb{{R}},\qquad h(\sqrt{{{RADICAND}}})=0"),
            1.9,
            36.0,
            Palette::teal(),
        )?;
        let nested_limit = text(
            &kit,
            stage,
            "nested closed intervals with widths tending to zero",
            0.75,
            27.0,
        )?;
        let exists = text(&kit, stage, "a common real point exists", -0.1, 30.0)?;
        let ivt = text(&kit, stage, "continuity then gives the intermediate value", -1.1, 28.0)?;
        fade_in_lagged(stage, &[complete, real, nested_limit, exists, ivt], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 9.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "12_8")?;
        let endtitle = text(&kit, stage, "Ask for the promise you need", 3.0, 35.0)?;
        let definitions = text(&kit, stage, "definitions specify the challenge", 1.8, 29.0)?;
        let estimates = text(&kit, stage, "estimates build a response", 0.65, 29.0)?;
        let existence = text(&kit, stage, "existence needs its own hypotheses", -0.5, 29.0)?;
        fade_in_lagged(stage, &[endtitle, definitions, estimates, existence], 0.7, 0.1)?;
        let _line = say(&kit, stage, "12_9")?;
        let order = text(&kit, stage, "who chooses what, and when?", -2.0, 33.0)?;
        let final_line = text(
            &kit,
            stage,
            "closer is useful; a precise promise is stronger",
            -3.15,
            26.0,
        )?;
        fade_in_lagged(stage, &[order, final_line], 0.7, 0.1)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rational_bisection_keeps_signs_and_halves_width() {
        let (mut last_lo, mut last_hi) = bracket(0).unwrap();
        for n in 1..=20 {
            let (lo, hi) = bracket(n).unwrap();
            assert!(value(lo).unwrap() < Frac::whole(0));
            assert!(value(hi).unwrap() > Frac::whole(0));
            assert!(lo >= last_lo && hi <= last_hi);
            assert_eq!(
                hi.minus(lo).unwrap().times(Frac::whole(2)).unwrap(),
                last_hi.minus(last_lo).unwrap()
            );
            (last_lo, last_hi) = (lo, hi);
        }
        assert!(bracket(25).is_err());
    }
    #[test]
    fn factor_four_bounds_every_sample_pair_in_domain() {
        for i in 20..=40 {
            for j in 20..=40 {
                let x = Frac::whole(i).over(Frac::whole(20)).unwrap();
                let y = Frac::whole(j).over(Frac::whole(20)).unwrap();
                let abs = |z: Frac| if z < Frac::whole(0) { z.negated().unwrap() } else { z };
                let error = abs(value(x).unwrap().minus(value(y).unwrap()).unwrap());
                let bound = abs(x.minus(y).unwrap()).times(Frac::whole(2 * ENDS[1])).unwrap();
                assert!(error <= bound);
            }
        }
    }
}
