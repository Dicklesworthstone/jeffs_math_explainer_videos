//! Native staged coefficient-matching chapter; not compiled or rendered.
use crate::series_model::{ORDERS, PLOT_SEGMENTS, PLOT_TOP, RADIUS, TAIL_ORDER, coefficient, plot_x, sum, tail_bound};
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;
pub struct Factorials {
    pub kit: Kit,
}
pub const TEXT: &[&str] = &[
    "Where the factorials come from",
    "function",
    "derivative",
    "constant",
    "x",
    "x²",
    "x³",
    "x⁴",
    "1",
    "a",
    "b",
    "c",
    "d",
    "a = ?",
    "2b",
    "3c",
    "4d",
    "1/2",
    "1/6",
    "1/24",
    "partial sums on fixed axes",
    "Which coefficient makes the constant column agree?",
    "A finite polynomial loses its highest power",
    "The infinite tail needs a proof",
    "Uniform convergence on every bounded interval",
    "0",
    "1.5",
    "5",
];
pub const TEX: &[&str] = &[
    r"a_{k+1}=\frac{a_k}{k+1},\quad a_0=1",
    r"\frac{|x|}{k+1}\leq\frac{R}{k+1}\leq\frac12",
    r"\sum_{k=N+1}^{\infty}\frac{|x|^k}{k!}\leq\frac{R^{N+1}}{(N+1)!}\frac1{1-R/(N+2)}",
    r"E(x)=\sum_{k=0}^{\infty}\frac{x^k}{k!},\quad E'=E,\quad E(0)=1",
];
fn reveal(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.play(a.rt(0.35))?;
    Ok(())
}
fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    for &m in mobs {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        stage.play(a.rt(0.12))?;
    }
    Ok(())
}
fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Line::new(a, b).style(stroke_style(color, width)).build()?))
}
impl SceneConstruct for Factorials {
    fn name(&self) -> &str {
        "02_factorials"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "02_1")?;
        let title = k.text(stage, TEXT[0], 35.0, Palette::ink(), v(0.0, 3.1))?;
        reveal(stage, title)?;
        let mut table = Vec::new();
        let xs = [-3.9, -2.8, -1.7, -0.6, 0.5];
        for (label, y, color) in [("function", 1.6, Palette::blue()), ("derivative", 0.8, Palette::gold())] {
            let m = k.text(stage, label, 23.0, color, v(2.6, y))?;
            reveal(stage, m)?;
            table.push(m);
        }
        for (i, x) in xs.iter().copied().enumerate() {
            for (a, b) in [
                (v(x - 0.42, 1.3), v(x + 0.42, 1.3)),
                (v(x + 0.42, 1.3), v(x + 0.42, 1.9)),
                (v(x + 0.42, 1.9), v(x - 0.42, 1.9)),
                (v(x - 0.42, 1.9), v(x - 0.42, 1.3)),
            ] {
                let m = line(stage, a, b, Palette::dim(), 1.5)?;
                reveal(stage, m)?;
                table.push(m);
            }
            for (s, y, col) in [
                (TEXT[3 + i], 2.25, Palette::dim()),
                (["1", "a", "b", "c", "d"][i], 1.6, Palette::blue()),
            ] {
                let m = k.text(stage, s, 25.0, col, v(x, y))?;
                reveal(stage, m)?;
                table.push(m);
            }
        }
        k.until_sentence(stage, "02_1", 3)?;
        for (i, s) in ["a", "2b", "3c", "4d"].iter().enumerate() {
            let m = k.text(stage, s, 25.0, Palette::gold(), v(xs[i], 0.8))?;
            reveal(stage, m)?;
            table.push(m);
            let a = line(stage, v(xs[i + 1], 1.35), v(xs[i], 1.02), Palette::dim(), 2.0)?;
            reveal(stage, a)?;
            table.push(a);
        }
        let question = k.text(stage, TEXT[22], 24.0, Palette::hero(), v(0.0, -2.8))?;
        reveal(stage, question)?;
        k.hold(stage)?;
        stage.wait(4.0)?;
        clear(stage, &[question])?;
        k.say(stage, "02_2")?;
        for (n, sentence) in [(1, 0), (2, 3), (3, 5)] {
            k.until_sentence(stage, "02_2", sentence)?;
            let label = k.text(
                stage,
                &format!("{} = {}", ["", "a", "b", "c"][n], coefficient(n)),
                25.0,
                Palette::green(),
                v(xs[n], 0.05),
            )?;
            reveal(stage, label)?;
            table.push(label);
        }
        k.say(stage, "02_3")?;
        k.until_sentence(stage, "02_3", 2)?;
        let d = k.text(
            stage,
            &format!("d = {}", coefficient(4)),
            25.0,
            Palette::green(),
            v(xs[4], 0.05),
        )?;
        reveal(stage, d)?;
        table.push(d);
        let recurrence = k.tex(stage, TEX[0], 28.0, Palette::ink(), &[], v(0.0, -0.8))?;
        reveal(stage, recurrence)?;
        table.push(recurrence);
        k.until_sentence(stage, "02_3", 4)?;
        clear(stage, &table)?;
        let map = |x: f64, y: f64| v(-4.8 + 6.4 * x, -2.0 + 0.8 * y);
        let mut graph = Vec::new();
        for (label, position) in [
            ("0", v(-5.05, -2.3)),
            ("1.5", v(4.8, -2.3)),
            ("5", v(-5.1, 2.0)),
            ("partial sums on fixed axes", v(0.0, 2.55)),
        ] {
            let m = k.text(stage, label, 23.0, Palette::dim(), position)?;
            reveal(stage, m)?;
            graph.push(m);
        }
        for (a, b) in [(map(0.0, 0.0), map(RADIUS, 0.0)), (map(0.0, 0.0), map(0.0, PLOT_TOP))] {
            let m = line(stage, a, b, Palette::dim(), 2.0)?;
            reveal(stage, m)?;
            graph.push(m);
        }
        for n in ORDERS {
            let color = if n == 4 { Palette::blue() } else { Palette::dim() };
            let mut animations = Vec::new();
            for j in 0..PLOT_SEGMENTS {
                let x = plot_x(j);
                let z = plot_x(j + 1);
                let m = line(stage, map(x, sum(x, n)), map(z, sum(z, n)), color, 2.5)?;
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                animations.push(stage.prepare(a.rt(0.5))?);
                graph.push(m);
            }
            stage.play_prepared(animations)?;
        }
        k.hold(stage)?;
        k.say(stage, "02_4")?;
        let warning = k.text(stage, TEXT[23], 25.0, Palette::gold(), v(0.0, -2.9))?;
        reveal(stage, warning)?;
        k.until_sentence(stage, "02_4", 3)?;
        k.hold(stage)?;
        // One parallel wipe avoids a per-segment delay for the 300 line pieces.
        let mut wipes = Vec::new();
        for m in graph {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            wipes.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(wipes)?;
        clear(stage, &[warning])?;
        k.say(stage, "02_5")?;
        let ratio = k.tex(stage, TEX[1], 31.0, Palette::blue(), &[], v(0.0, 1.7))?;
        k.until_sentence(stage, "02_5", 2)?;
        reveal(stage, ratio)?;
        let bound = k.tex(stage, TEX[2], 28.0, Palette::green(), &[], v(0.0, 0.3))?;
        k.until_sentence(stage, "02_5", 3)?;
        reveal(stage, bound)?;
        let read = k.text(
            stage,
            &format!(
                "|x| ≤ {RADIUS}, degree {TAIL_ORDER}: tail ≤ {:.5}",
                tail_bound(RADIUS, TAIL_ORDER).unwrap()
            ),
            25.0,
            Palette::ink(),
            v(0.0, -1.0),
        )?;
        reveal(stage, read)?;
        k.hold(stage)?;
        k.say(stage, "02_6")?;
        clear(stage, &[ratio, bound, read])?;
        let end = k.tex(stage, TEX[3], 31.0, Palette::blue(), &[], v(0.0, 0.6))?;
        reveal(stage, end)?;
        k.finish(stage)
    }
}
