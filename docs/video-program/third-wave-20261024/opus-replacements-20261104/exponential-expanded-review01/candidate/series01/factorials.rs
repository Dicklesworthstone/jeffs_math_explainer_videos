//! Native staged coefficient-matching chapter; not compiled or rendered.
use crate::series_model::{
    ORDERS, PLOT_SEGMENTS, PLOT_TOP, RADIUS, TAIL_ORDER, coefficient, plot_x, sum, tail_bound, tail_label, tail_ratio,
};
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
    r"\frac{|x|}{k+1}\leq\frac{R}{k+1}\leq\frac12\quad(k+1\geq 2R)",
    r"\sum_{k=N+1}^{\infty}\frac{|x|^k}{k!}\leq\frac{R^{N+1}}{(N+1)!}\frac1{1-R/(N+2)}\quad(N+2>R)",
    r"E(x)=\sum_{k=0}^{\infty}\frac{x^k}{k!},\quad E'=E,\quad E(0)=1",
];
/// The column headers: TeX, because the bundled roman face has no ², ³ or ⁴.
const POWERS: [&str; 4] = ["x", "x^2", "x^3", "x^4"];
fn reveal(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.play(a.rt(0.35))?;
    Ok(())
}
fn reveal_all(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(seconds))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(0.3))?);
    }
    stage.play_prepared(animations)?;
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
        // The boxes land on the sentence that names them.
        k.until_sentence(stage, "02_1", 2)?;
        table.push(k.text(stage, "function", 23.0, Palette::blue(), v(2.6, 1.6))?);
        for (i, x) in xs.iter().copied().enumerate() {
            for (a, b) in [
                (v(x - 0.42, 1.3), v(x + 0.42, 1.3)),
                (v(x + 0.42, 1.3), v(x + 0.42, 1.9)),
                (v(x + 0.42, 1.9), v(x - 0.42, 1.9)),
                (v(x - 0.42, 1.9), v(x - 0.42, 1.3)),
            ] {
                table.push(line(stage, a, b, Palette::dim(), 1.5)?);
            }
            table.push(if i == 0 {
                k.text(stage, TEXT[3], 25.0, Palette::dim(), v(x, 2.25))?
            } else {
                k.tex(stage, POWERS[i - 1], 25.0, Palette::dim(), &[], v(x, 2.25))?
            });
            table.push(k.text(stage, ["1", "a", "b", "c", "d"][i], 25.0, Palette::blue(), v(x, 1.6))?);
        }
        reveal_all(stage, &table, 0.6)?;
        k.until_sentence(stage, "02_1", 3)?;
        let derivative = k.text(stage, TEXT[2], 23.0, Palette::gold(), v(2.6, 0.8))?;
        reveal(stage, derivative)?;
        table.push(derivative);
        for (i, s) in ["a", "2b", "3c", "4d"].iter().enumerate() {
            // A copy of the term leaves its box, moves one column left and picks up its power.
            let moving = k.text(stage, ["a", "b", "c", "d"][i], 25.0, Palette::gold(), v(xs[i + 1], 1.6))?;
            reveal(stage, moving)?;
            let target = k.text(stage, s, 25.0, Palette::gold(), v(xs[i], 0.8))?;
            stage.play(Transform::new(moving, target).rt(0.6))?;
            table.push(moving);
            let a = line(stage, v(xs[i + 1], 1.35), v(xs[i], 1.02), Palette::dim(), 2.0)?;
            reveal(stage, a)?;
            table.push(a);
        }
        k.until_sentence(stage, "02_1", 4)?;
        let question = k.text(stage, TEXT[21], 24.0, Palette::hero(), v(0.0, -2.8))?;
        reveal(stage, question)?;
        k.hold(stage)?;
        k.mark(stage, "predict");
        k.music_breath(stage, 4.0)?;
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
        k.until_sentence(stage, "02_3", 1)?;
        let recurrence = k.tex(stage, TEX[0], 28.0, Palette::ink(), &[], v(0.0, -0.8))?;
        reveal(stage, recurrence)?;
        table.push(recurrence);
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
        k.until_sentence(stage, "02_3", 4)?;
        clear(stage, &table)?;
        let map = |x: f64, y: f64| v(-4.8 + 6.4 * x, -2.0 + 0.8 * y);
        let mut graph = Vec::new();
        for (label, position) in [
            ("0".to_owned(), v(-5.05, -2.3)),
            (RADIUS.to_string(), v(4.8, -2.3)),
            (PLOT_TOP.to_string(), v(-5.1, 2.0)),
            (TEXT[20].to_owned(), v(0.0, 2.55)),
        ] {
            graph.push(k.text(stage, &label, 23.0, Palette::dim(), position)?);
        }
        for (a, b) in [(map(0.0, 0.0), map(RADIUS, 0.0)), (map(0.0, 0.0), map(0.0, PLOT_TOP))] {
            graph.push(line(stage, a, b, Palette::dim(), 2.0)?);
        }
        reveal_all(stage, &graph, 0.4)?;
        // One path per partial sum. The newest stays blue and the one before it dims.
        let mut previous: Option<Mob> = None;
        for n in ORDERS {
            let points: Vec<Vec3> = (0..=PLOT_SEGMENTS).map(|j| map(plot_x(j), sum(plot_x(j), n))).collect();
            let curve = stage
                .arena_mut()
                .add(polyline(points).style(stroke_style(Palette::blue(), 2.5)).build());
            let a = fade_in(stage.arena_mut(), curve, ORIGIN, 1.0)?;
            let mut animations = vec![stage.prepare(a.rt(0.5))?];
            if let Some(old) = previous {
                // set_stroke, not set_color: a recolour also sets fill alpha and would fill the open path.
                let dimmed = old
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.5),
                        ..AnimateArgs::default()
                    })?
                    .set_stroke(Some(Palette::dim()), None, None)?;
                animations.push(stage.prepare(dimmed)?);
            }
            stage.play_prepared(animations)?;
            stage.wait(0.6)?;
            graph.push(curve);
            previous = Some(curve);
        }
        k.hold(stage)?;
        k.say(stage, "02_4")?;
        let warning = k.text(stage, TEXT[22], 25.0, Palette::gold(), v(0.0, -2.9))?;
        reveal(stage, warning)?;
        graph.push(warning);
        k.until_sentence(stage, "02_4", 3)?;
        let proof = k.text(stage, TEXT[23], 25.0, Palette::gold(), v(0.0, -3.35))?;
        reveal(stage, proof)?;
        graph.push(proof);
        k.hold(stage)?;
        clear(stage, &graph)?;
        k.say(stage, "02_5")?;
        // The interval the first sentence names, so this cue does not open on an empty screen.
        let mut proof_card = vec![
            line(stage, v(-2.4, 2.6), v(2.4, 2.6), Palette::blue(), 3.0)?,
            k.tex(stage, "-R", 25.0, Palette::blue(), &[], v(-2.4, 2.3))?,
            k.tex(stage, "R", 25.0, Palette::blue(), &[], v(2.4, 2.3))?,
        ];
        reveal_all(stage, &proof_card, 0.4)?;
        k.until_sentence(stage, "02_5", 1)?;
        let ratio = k.tex(stage, TEX[1], 31.0, Palette::blue(), &[], v(0.0, 1.55))?;
        reveal(stage, ratio)?;
        proof_card.push(ratio);
        k.until_sentence(stage, "02_5", 3)?;
        let bound = k.tex(stage, TEX[2], 28.0, Palette::green(), &[], v(0.0, 0.3))?;
        reveal(stage, bound)?;
        proof_card.push(bound);
        let read = k.text(
            stage,
            &format!(
                "|x| ≤ {RADIUS}, degree {TAIL_ORDER}: ratio ≤ {:.2}, tail ≤ {}",
                tail_ratio(RADIUS, TAIL_ORDER),
                tail_label(tail_bound(RADIUS, TAIL_ORDER).unwrap())
            ),
            25.0,
            Palette::ink(),
            v(0.0, -1.0),
        )?;
        reveal(stage, read)?;
        proof_card.push(read);
        k.until_sentence(stage, "02_5", 5)?;
        let uniform = k.text(stage, TEXT[24], 25.0, Palette::gold(), v(0.0, -2.0))?;
        reveal(stage, uniform)?;
        proof_card.push(uniform);
        k.hold(stage)?;
        k.say(stage, "02_6")?;
        clear(stage, &proof_card)?;
        let end = k.tex(stage, TEX[3], 31.0, Palette::blue(), &[], v(0.0, 0.6))?;
        reveal(stage, end)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::TEXT;
    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("factorials.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("split yields a first part")
    }
    fn assert_in_order(needles: &[&str]) {
        let mut from = 0;
        for &needle in needles {
            let at = scene()[from..]
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} missing or out of order"));
            from += at + needle.len();
        }
    }
    #[test]
    fn each_indexed_string_is_the_one_its_beat_needs() {
        assert_eq!(TEXT[21], "Which coefficient makes the constant column agree?");
        assert_eq!(TEXT[22], "A finite polynomial loses its highest power");
        assert_eq!(TEXT[23], "The infinite tail needs a proof");
        assert_eq!(TEXT[24], "Uniform convergence on every bounded interval");
        assert_in_order(&[
            "let question = k.text(stage, TEXT[21]",
            "let warning = k.text(stage, TEXT[22]",
            r#"until_sentence(stage, "02_4", 3)"#,
            "let proof = k.text(stage, TEXT[23]",
            r#"until_sentence(stage, "02_5", 5)"#,
            "let uniform = k.text(stage, TEXT[24]",
        ]);
    }
    #[test]
    fn the_think_gap_is_authored_and_the_answer_waits() {
        assert_in_order(&[
            "reveal(stage, question)?;",
            "k.hold(stage)?;",
            r#"k.mark(stage, "predict");"#,
            "k.music_breath(stage, 4.0)?;",
            "clear(stage, &[question])?;",
            r#"k.say(stage, "02_2")?;"#,
            "coefficient(n)",
        ]);
    }
}
