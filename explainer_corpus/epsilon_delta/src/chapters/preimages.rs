//! Preimages need no inverse function. Openness is relative to the domain.
use super::draw::{fade_in_all, fade_in_lagged, fade_out_all, flat, line_style, say, think_pause, wait_until};
#[cfg(test)]
use crate::model::Frac;
use explainer_kit::{BACKGROUND, Kit, Palette, fade_all, hex, v};
use fmn::prelude::*;

pub struct Preimages {
    pub kit: Kit,
}
pub const ROOTS: [i64; 2] = [1, 3];
pub fn output_bounds() -> [i64; 2] {
    ROOTS.map(|n| n * n)
}
#[cfg(test)]
pub fn square_in_target(x: Frac) -> Result<bool, String> {
    let y = x.times(x)?;
    let [lo, hi] = output_bounds();
    Ok(y > Frac::whole(lo) && y < Frac::whole(hi))
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
fn end(s: &mut Stage<'_>, at: Vec3, c: Srgb, closed: bool) -> fmn::Result<Mob> {
    let style = if closed {
        flat(c, 1.0)
    } else {
        Style::default().fill(hex(BACKGROUND), 1.0).stroke(c, 3.0, 1.0)
    };
    Ok(s.arena_mut()
        .add(Dot::new().point(at).radius(0.085).style(style).build()))
}
impl SceneConstruct for Preimages {
    fn name(&self) -> &str {
        "11_pull_back_open_sets"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [lo, hi] = output_bounds();
        let [a, b] = ROOTS;
        let _line = say(&kit, stage, "11_1")?;
        let title = text(&kit, stage, "Pull back a whole output set", 3.45, 34.0)?;
        let definition = tex(
            &kit,
            stage,
            r"f^{-1}(U)=\{x\in D:f(x)\in U\}",
            2.3,
            36.0,
            Palette::teal(),
        )?;
        let note = text(&kit, stage, "preimage: all inputs that land in U", 1.4, 28.0)?;
        fade_in_lagged(stage, &[title, definition, note], 0.7, 0.1)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[definition, note], 0.4)?;
        let _line = say(&kit, stage, "11_2")?;
        let formula = tex(
            &kit,
            stage,
            &format!(r"f(x)=x^2,\quad D=\mathbb{{R}},\quad U=({lo},{hi})"),
            2.6,
            34.0,
            Palette::ink(),
        )?;
        let axis = line(stage, v(-5.5, 1.4), v(5.5, 1.4), Palette::dim(), 3.0)?;
        let band = line(stage, v(-4.0, 1.4), v(4.0, 1.4), Palette::gold(), 9.0)?;
        let l = end(stage, v(-4.0, 1.4), Palette::gold(), false)?;
        let h = end(stage, v(4.0, 1.4), Palette::gold(), false)?;
        let ll = kit.text(stage, &lo.to_string(), 25.0, Palette::ink(), v(-4.0, 0.9))?;
        let hl = kit.text(stage, &hi.to_string(), 25.0, Palette::ink(), v(4.0, 0.9))?;
        let output = text(&kit, stage, "output interval", 1.95, 23.0)?;
        fade_in_lagged(stage, &[formula, axis, band, l, h, ll, hl, output], 0.6, 0.07)?;
        let _question = say(&kit, stage, "11_3")?;
        let ask = text(&kit, stage, "?", -1.8, 44.0)?;
        let question = text(&kit, stage, "Must its inputs form a single piece?", -0.6, 28.0)?;
        fade_in_all(stage, &[ask, question], 0.4)?;
        think_pause(&kit, stage, v(0.0, -1.8), 0.4)?;
        fade_out_all(stage, &[ask, question], 0.3)?;
        let _line = say(&kit, stage, "11_4")?;
        let input_axis = line(stage, v(-6.0, -0.6), v(6.0, -0.6), Palette::dim(), 3.0)?;
        fade_in_all(stage, &[input_axis], 0.3)?;
        for (left, right) in [(-b, -a), (a, b)] {
            let segment = line(
                stage,
                v(left as f64 * 1.5, -0.6),
                v(right as f64 * 1.5, -0.6),
                Palette::teal(),
                9.0,
            )?;
            let mut group = vec![segment];
            for n in [left, right] {
                group.push(end(stage, v(n as f64 * 1.5, -0.6), Palette::teal(), false)?);
                group.push(kit.tex(
                    stage,
                    &n.to_string(),
                    25.0,
                    Palette::ink(),
                    &[],
                    v(n as f64 * 1.5, -1.15),
                )?);
            }
            fade_in_all(stage, &group, 0.5)?;
        }
        let preimage = tex(
            &kit,
            stage,
            &format!(r"f^{{-1}}(({lo},{hi}))=(-{b},-{a})\cup({a},{b})"),
            -2.2,
            35.0,
            Palette::teal(),
        )?;
        let endpoints = text(&kit, stage, "endpoints excluded; two open input intervals", -3.1, 26.0)?;
        fade_in_lagged(stage, &[preimage, endpoints], 0.6, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 7.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "11_5")?;
        let title2 = text(&kit, stage, "The same local promise, in set language", 3.45, 33.0)?;
        let first = text(
            &kit,
            stage,
            "continuous ⇒ every open output set has an open preimage",
            2.5,
            27.0,
        )?;
        let ball = tex(
            &kit,
            stage,
            r"a\in f^{-1}(U),\quad B_\varepsilon(f(a))\subset U",
            1.4,
            33.0,
            Palette::gold(),
        )?;
        let pull = tex(
            &kit,
            stage,
            r"B_\delta(a)\cap D\ \subset\ f^{-1}(U)",
            0.4,
            36.0,
            Palette::teal(),
        )?;
        let all = text(
            &kit,
            stage,
            "each preimage point has a whole input neighborhood",
            -0.45,
            26.0,
        )?;
        fade_in_lagged(stage, &[title2, first, ball, pull, all], 0.7, 0.1)?;
        let read = stage.scene().time().to_f64() + 9.0;
        wait_until(stage, read)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[first, ball, pull, all], 0.4)?;
        let _line = say(&kit, stage, "11_6")?;
        let converse = text(&kit, stage, "open preimages ⇒ continuity", 2.4, 29.0)?;
        let choose = tex(
            &kit,
            stage,
            r"U=B_\varepsilon(f(a)),\quad a\in f^{-1}(U)",
            1.3,
            34.0,
            Palette::gold(),
        )?;
        let open = tex(
            &kit,
            stage,
            r"f^{-1}(U)\text{ open in }D\ \Longrightarrow\ \exists\delta>0",
            0.3,
            32.0,
            Palette::teal(),
        )?;
        let local = tex(
            &kit,
            stage,
            r"x\in D,\ |x-a|<\delta\ \Longrightarrow\ |f(x)-f(a)|<\varepsilon",
            -0.8,
            31.0,
            Palette::ink(),
        )?;
        fade_in_lagged(stage, &[converse, choose, open, local], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 10.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "11_7")?;
        let title3 = text(&kit, stage, "A jump fails this test", 3.3, 34.0)?;
        let step = tex(
            &kit,
            stage,
            r"s(x)=0\ (x<0),\qquad s(x)=1\ (x\ge0)",
            2.1,
            34.0,
            Palette::ink(),
        )?;
        let target = tex(&kit, stage, r"s^{-1}((1/2,3/2))=[0,\infty)", 0.9, 37.0, Palette::gold())?;
        fade_in_lagged(stage, &[title3, step, target], 0.7, 0.1)?;
        let ax = line(stage, v(-5.5, -0.4), v(5.5, -0.4), Palette::dim(), 3.0)?;
        let right = line(stage, v(0.0, -0.4), v(5.5, -0.4), Palette::gold(), 9.0)?;
        let closed = end(stage, v(0.0, -0.4), Palette::gold(), true)?;
        let z = text(&kit, stage, "0", -0.95, 25.0)?;
        let no = text(&kit, stage, "no open neighborhood of zero fits inside", -2.0, 28.0)?;
        fade_in_lagged(stage, &[ax, right, closed, z, no], 0.6, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 8.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "11_8")?;
        let domain = text(&kit, stage, "Open in the domain", 3.3, 34.0)?;
        let g = tex(
            &kit,
            stage,
            r"g(x)=\sqrt{x},\qquad D=[0,\infty)",
            2.1,
            35.0,
            Palette::ink(),
        )?;
        let example = tex(
            &kit,
            stage,
            r"g^{-1}((-1,1))=[0,1)\quad\text{open in }D",
            0.9,
            35.0,
            Palette::teal(),
        )?;
        let rel = tex(&kit, stage, r"[0,1)=(-1,1)\cap D", -0.3, 35.0, Palette::blue())?;
        fade_in_lagged(stage, &[domain, g, example, rel], 0.7, 0.1)?;
        let _line = say(&kit, stage, "11_9")?;
        let result = text(&kit, stage, "continuity: preimages of open sets are open", -1.5, 29.0)?;
        let next = text(&kit, stage, "next: what the definition does not supply", -2.7, 26.0)?;
        fade_in_lagged(stage, &[result, next], 0.6, 0.1)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_intervals_equal_the_square_preimage_including_endpoints() {
        for n in -160..=160 {
            let x = Frac::whole(n).over(Frac::whole(40)).unwrap();
            let actual = square_in_target(x).unwrap();
            let [a, b] = ROOTS;
            let in_intervals =
                (x > Frac::whole(-b) && x < Frac::whole(-a)) || (x > Frac::whole(a) && x < Frac::whole(b));
            assert_eq!(actual, in_intervals, "x={x}");
        }
        for n in [-3, -1, 1, 3] {
            assert!(!square_in_target(Frac::whole(n)).unwrap());
        }
    }
}
