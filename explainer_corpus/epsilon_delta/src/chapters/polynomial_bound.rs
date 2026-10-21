//! Chapter 07: three terms share an error budget. Davis's cubic is
//! recentred, powers are capped in a unit window, and their coefficients
//! become one sufficient bound. The sample bars illustrate the algebra;
//! the inequalities, not the sample, prove the guarantee for every input.

use explainer_kit::{Facts, Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    fact, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, moving, say, think_pause, wait_until,
};
use super::limit_that_jumps::read_s;
use super::old_objection::tex_frac;
use super::shrinking_region::glow;
use crate::{facts, model};

pub struct PolynomialBound {
    pub kit: Kit,
}

const TITLE: &str = "One budget for every term";
const GENERAL: &str = "Any polynomial, near any point";
const SAMPLE: &str = "illustration at t = 1/2";
const NEXT: &str = "next: fixed and moving choices";
const FINITE: &str = "a finite expansion, no derivatives";
const BAR_X: f64 = -5.9;
const BAR_W: f64 = 5.2;
const BAR_H: f64 = 0.24;
const ROW_Y: [f64; 3] = [0.85, -0.05, -0.95];
const SAMPLE_T: f64 = 0.5;
#[cfg(test)]
const WAIT_FOR: &[(&str, &str)] = &[
    ("07_1", "Put h equal"),
    ("07_2", "Bound its size"),
    ("07_3", "Each higher power"),
    ("07_5", "So choose delta"),
    ("07_6", "Every h inside"),
    ("07_7", "Add the sizes"),
    ("07_8", "If zero"),
    ("07_9", "Now return"),
];

/// Math strings are built from the same facts that bind the narration.
struct Rows {
    setting: String,
    h: String,
    change: String,
    sizes: [String; 3],
    caps: [String; 3],
    total: String,
    delta: String,
    guarantee: String,
    note: String,
}

impl Rows {
    fn new(f: &Facts) -> fmn::Result<Self> {
        let a = fact(f, "C7.a")?;
        let power = fact(f, "C7.degree")?;
        let square = fact(f, "C7.power2")?;
        let c1 = fact(f, "C7.linear")?;
        let c2 = fact(f, "C7.square")?;
        let c3 = fact(f, "C7.cube")?;
        let bound = fact(f, "D1.bound")?;
        Ok(Self {
            setting: format!(
                r"P(x)=x^{{{power}}}+{}x^{{{square}}},\quad P({a})={}",
                fact(f, "C7.quadratic")?,
                fact(f, "C7.value")?
            ),
            h: format!(r"h=x-{a},\qquad t=|h|"),
            change: format!(r"P({a}+h)-P({a})=h^{{{power}}}+{c2}h^{{{square}}}+{c1}h"),
            sizes: [
                format!(r"t^{{{power}}}"),
                format!(r"{c2}t^{{{square}}}"),
                format!(r"{c1}t"),
            ],
            caps: [r"\le t".into(), format!(r"\le {c2}t"), format!(r"\le {c1}t")],
            total: format!(r"({c3}+{c2}+{c1})t={bound}t"),
            delta: format!(r"\delta=\min(1,\varepsilon/{bound})"),
            guarantee: format!(r"|P({a}+h)-P({a})|\le {bound}t<{bound}\delta\le\varepsilon"),
            note: format!(
                // the short credit; the full title and author are in the
                // storyboard row and the README's sources
                "Davis, Monthly {} ({}), pp. {}–{}",
                fact(f, "C7.note.volume")?,
                fact(f, "C7.note.year")?,
                fact(f, "C7.note.first")?,
                fact(f, "C7.note.last")?
            ),
        })
    }
}

fn math(kit: &Kit, stage: &mut Stage<'_>, text: &str, at: Vec3, size: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, text, size, color, &[], at)
}

fn bar(stage: &mut Stage<'_>, x: f64, y: f64, width: f64, style: Style) -> fmn::Result<Mob> {
    let shape = rect(v(x, y - BAR_H / 2.0), width, BAR_H, style)?;
    Ok(stage.arena_mut().add(shape))
}

/// Lengths normalized to the total linear budget C*t. They are computed
/// from the recentered model, not picked to make a pleasing picture.
fn bar_shares(t: f64) -> fmn::Result<([f64; 3], [f64; 3])> {
    if !(0.0 < t && t <= 1.0) {
        return Err(fail("bar illustration needs 0 < t <= 1".into()));
    }
    let c = model::recenter(&[-6, 0, 5, 1], 1).map_err(fail)?;
    let total = model::local_bound(&c) as f64;
    let caps = [
        c[3].unsigned_abs() as f64 / total,
        c[2].unsigned_abs() as f64 / total,
        c[1].unsigned_abs() as f64 / total,
    ];
    Ok(([caps[0] * t * t, caps[1] * t, caps[2]], caps))
}

impl SceneConstruct for PolynomialBound {
    fn name(&self) -> &str {
        "07_the_polynomial_bound"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let rows = Rows::new(&f)?;
        let ink = Palette::ink();
        let colors = [Palette::hero(), moving(), Palette::teal()];
        let mut body = Vec::new();

        let line = say(&kit, stage, "07_1")?;
        let title = kit.text(stage, TITLE, 34.0, ink, v(0.0, 3.35))?;
        let setting = math(&kit, stage, &rows.setting, v(0.0, 2.65), 32.0, ink)?;
        fade_in_lagged(stage, &[title, setting], 0.7, 0.1)?;
        body.push(setting);
        line.wait_for(stage, "Put h equal")?;
        let h = math(&kit, stage, &rows.h, v(0.0, 2.0), 30.0, ink)?;
        fade_in_all(stage, &[h], 0.4)?;
        body.push(h);

        let line = say(&kit, stage, "07_2")?;
        let change = math(&kit, stage, &rows.change, v(0.0, 1.4), 28.0, ink)?;
        fade_in_all(stage, &[change], 0.5)?;
        body.push(change);
        line.wait_for(stage, "Bound its size")?;
        let (actual, caps) = bar_shares(SAMPLE_T)?;
        let mut bounds = Vec::new();
        // the sample at t = 1/2 (its bars, their outlines and its label) has
        // done its job once the stacked bar holds the total: it goes as 07_6
        // begins, before the guarantee comes in; the terms and caps stay
        let mut sample_parts = Vec::new();
        let mut cap_texts = Vec::new();
        for i in 0..ROW_Y.len() {
            let term = kit.tex_left(stage, &rows.sizes[i], 27.0, colors[i], &[], v(BAR_X, ROW_Y[i]))?;
            let filled = bar(stage, BAR_X, ROW_Y[i] - 0.35, BAR_W * actual[i], flat(colors[i], 0.9))?;
            fade_in_all(stage, &[term, filled], 0.3)?;
            body.push(term);
            sample_parts.push(filled);
            let cap = kit.tex_left(stage, &rows.caps[i], 27.0, ink, &[], v(-3.95, ROW_Y[i]))?;
            let outline = bar(
                stage,
                BAR_X,
                ROW_Y[i] - 0.35,
                BAR_W * caps[i],
                line_style(colors[i], 2.0),
            )?;
            bounds.extend([cap, outline]);
            cap_texts.push(cap);
            sample_parts.push(outline);
        }
        let sample = kit.text(stage, SAMPLE, 20.0, Palette::dim(), v(-3.3, -1.65))?;
        fade_in_all(stage, &[sample], 0.3)?;
        sample_parts.push(sample);

        let line = say(&kit, stage, "07_3")?;
        let unit = math(&kit, stage, r"0\le t<1", v(3.1, 0.6), 32.0, ink)?;
        fade_in_all(stage, &[unit], 0.4)?;
        body.push(unit);
        line.wait_for(stage, "Each higher power")?;
        let powers = math(&kit, stage, r"t^k\le t\quad(k\ge1)", v(3.1, -0.05), 30.0, ink)?;
        fade_in_all(stage, &[powers], 0.4)?;
        body.push(powers);
        fade_in_lagged(stage, &bounds, 0.8, 0.15)?;
        body.extend(cap_texts);

        let _question = say(&kit, stage, "07_4")?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), v(3.1, -1.25))?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, v(3.1, -1.25), 0.45)?;

        let line = say(&kit, stage, "07_5")?;
        fade_out_all(stage, &[ask], 0.2)?;
        let total = math(&kit, stage, &rows.total, v(-3.3, -2.3), 30.0, ink)?;
        fade_in_all(stage, &[total], 0.4)?;
        body.push(total);
        let mut x = BAR_X;
        for (share, color) in caps.into_iter().zip(colors) {
            let chunk = bar(stage, x, -2.75, BAR_W * share, flat(color, 0.9))?;
            fade_in_all(stage, &[chunk], 0.25)?;
            body.push(chunk);
            x += BAR_W * share;
        }
        line.wait_for(stage, "So choose delta")?;
        let delta = math(&kit, stage, &rows.delta, v(3.1, -1.0), 30.0, moving())?;
        fade_in_all(stage, &[delta], 0.5)?;
        body.push(delta);

        let line = say(&kit, stage, "07_6")?;
        fade_out_all(stage, &sample_parts, 0.3)?;
        line.wait_for(stage, "Every h inside")?;
        let inside = math(&kit, stage, r"0\le t<\delta", v(3.1, -1.7), 28.0, moving())?;
        let proof = math(&kit, stage, &rows.guarantee, v(3.1, -2.3), 26.0, ink)?;
        fade_in_lagged(stage, &[inside, proof], 0.7, 0.15)?;
        body.extend([inside, proof]);
        // This example is visible but not recited; leave at least 3 s.
        let example = math(
            &kit,
            stage,
            &format!(
                r"\varepsilon={}:\quad\delta={}",
                tex_frac(&fact(&f, "S.eps")?),
                tex_frac(&fact(&f, "D1.delta")?)
            ),
            v(3.1, -3.05),
            26.0,
            moving(),
        )?;
        fade_in_all(stage, &[example], 0.4)?;
        body.push(example);
        wait_until(stage, stage.scene().time().to_f64() + 3.0)?;
        kit.hold(stage)?;

        // The concrete picture has earned the general formula. Clear it
        // before reusing its positions; never crossfade different text.
        fade_out_all(stage, &body, 0.4)?;
        fade_out_all(stage, &[title], 0.2)?;
        let line = say(&kit, stage, "07_7")?;
        let general = kit.text(stage, GENERAL, 34.0, ink, v(0.0, 3.35))?;
        let expansion = math(
            &kit,
            stage,
            r"P(a+h)-P(a)=c_1h+c_2h^2+\cdots+c_mh^m",
            v(0.0, 2.25),
            36.0,
            ink,
        )?;
        fade_in_lagged(stage, &[general, expansion], 0.7, 0.1)?;
        line.wait_for(stage, "Add the sizes")?;
        let sum = math(
            &kit,
            stage,
            r"C=|c_1|+|c_2|+\cdots+|c_m|",
            v(0.0, 1.25),
            36.0,
            Palette::hero(),
        )?;
        fade_in_all(stage, &[sum], 0.5)?;

        let line = say(&kit, stage, "07_8")?;
        let positive = math(
            &kit,
            stage,
            r"C>0:\quad\delta=\min(1,\varepsilon/C)",
            v(0.0, 0.25),
            34.0,
            moving(),
        )?;
        let proof = math(
            &kit,
            stage,
            r"|h|<\delta\ \Longrightarrow\ |P(a+h)-P(a)|\le C|h|<C\delta\le\varepsilon",
            v(0.0, -0.6),
            28.0,
            ink,
        )?;
        fade_in_lagged(stage, &[positive, proof], 0.7, 0.1)?;
        line.wait_for(stage, "If zero")?;
        let zero = math(
            &kit,
            stage,
            r"C=0:\quad P(a+h)-P(a)=0,\quad\text{any }\delta>0",
            v(0.0, -1.55),
            32.0,
            ink,
        )?;
        fade_in_all(stage, &[zero], 0.5)?;

        let line = say(&kit, stage, "07_9")?;
        let finite = kit.text(stage, FINITE, 24.0, Palette::dim(), v(0.0, -2.25))?;
        let note = kit.text(stage, &rows.note, 19.0, Palette::dim(), v(0.0, -2.9))?;
        fade_in_lagged(stage, &[finite, note], 0.7, 0.1)?;
        let notes_up = stage.scene().time().to_f64();
        line.wait_for(stage, "Now return")?;
        let next = kit.text(stage, NEXT, 23.0, ink, v(0.0, -3.45))?;
        fade_in_all(stage, &[next], 0.4)?;
        let next_up = stage.scene().time().to_f64();
        glow(&kit, stage, v(0.0, 1.25), 0.4)?;
        kit.hold(stage)?;
        // each stays until read, counted once from its completed fade: the
        // line, the glow, the hold and the rest that follows are part of that
        // time, not added to it
        let read = (notes_up + read_s(FINITE) + read_s(&rows.note)).max(next_up + read_s(NEXT));
        wait_until(stage, read.max(stage.scene().time().to_f64() + 1.0))?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_picture_waits_for_one_spoken_phrase() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for &(id, phrase) in WAIT_FOR {
            assert_eq!(script.text(id).unwrap().matches(phrase).count(), 1, "{id}: {phrase}");
        }
    }

    #[test]
    fn bars_are_the_three_terms_and_not_arbitrary_lengths() {
        for t in [0.0001, 0.1, 0.5, 0.9, 1.0] {
            let (actual, caps) = bar_shares(t).unwrap();
            assert!((caps.iter().sum::<f64>() - 1.0).abs() < 1e-14);
            assert!(actual.iter().zip(caps).all(|(&a, b)| 0.0 < a && a <= b));
            let drawn_sum = actual.iter().sum::<f64>() * 22.0 * t;
            let polynomial_sum = t * t * t + 8.0 * t * t + 13.0 * t;
            assert!((drawn_sum - polynomial_sum).abs() < 1e-12);
        }
        assert!(bar_shares(0.0).is_err());
        assert!(bar_shares(1.1).is_err());
    }

    #[test]
    fn the_source_example_and_constant_case_agree_with_model() {
        let f = facts::facts().unwrap();
        assert_eq!(f.get("C7.value"), Some("6"));
        assert_eq!(f.get("D1.bound"), Some("22"));
        assert_eq!(f.get("D1.delta"), Some("1/220"));
        let constant = model::recenter(&[7], -3).unwrap();
        assert_eq!(model::local_bound(&constant), 0);
        assert_eq!(model::davis_delta(0, model::Ratio::tolerance(1, 10).unwrap()), (1, 1));
    }
}
