//! Native FrankenManim draft of three exponential constructions.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed cues 03_1/03_2 split the first retained source paragraph at a
//! sentence boundary; 03_3 and 03_4 use its next two paragraphs. No audio is cut.
use explainer_kit::{ink_style, stroke_style, Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct ThreeDefinitions {
    pub kit: Kit,
}

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for ThreeDefinitions {
    fn name(&self) -> &str { "03_three_definitions" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let x = crate::facts::SAMPLE_X;
        let counts = crate::facts::COUNTS;
        let limit_samples: Vec<f64> = counts.iter().map(|&n| crate::facts::limit_sample(x, n)).collect();
        let order = crate::facts::SERIES_ORDER;
        let partial_sum = crate::facts::series(x, order);
        let value = x.exp();
        if limit_samples.iter().any(|value| !value.is_finite()) || !partial_sum.is_finite() || !value.is_finite() {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("nonfinite exponential example")));
        }
        if x != 1.0 || limit_samples.iter().any(|&sample| !(2.0..=3.0).contains(&sample)) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("convergence plot requires x=1 and values in [2,3]")));
        }

        kit.say(stage, "03_1")?;
        let heading = kit.text(stage, "Three ways to make one function", 35.0, Palette::ink(), v(0.0, 2.9))?;
        let example = kit.tex(stage, &format!(r"\left(1+\frac{{x}}{{n}}\right)^n,\quad x={x}"), 34.0, Palette::blue(), &[], v(-3.2, 1.9))?;
        appear(stage, &[heading, example])?;
        // Use the exact same computed samples as the table: each new dot
        // shows the values crowding together, without revealing the limit.
        let left = 0.35_f64;
        let width = 4.9_f64;
        let map = |sample: f64, index: usize| v(left + width * (sample - 2.0), 1.1 - index as f64 * 0.52);
        let mut chart = vec![
            kit.text(stage, "value →", 24.0, Palette::dim(), v(2.8, -2.65))?,
            kit.text(stage, "2", 21.0, Palette::dim(), v(left, -2.2))?,
            kit.text(stage, "3", 21.0, Palette::dim(), v(left + width, -2.2))?,
        ];
        chart.push(stage.arena_mut().add(Line::new(v(left, -1.95), v(left + width, -1.95)).style(stroke_style(Palette::dim(), 2.0)).build()?));
        appear(stage, &chart)?;
        let mut previous = None;
        for (index, (&n, &sample)) in counts.iter().zip(&limit_samples).enumerate() {
            let row = kit.text(stage, &format!("n = {n}:  {sample:.5}"), 26.0, Palette::ink(), v(-3.3, 1.1 - index as f64 * 0.52))?;
            let point = map(sample, index);
            let dot = stage.arena_mut().add(Circle::new().radius(0.075).style(ink_style(Palette::hero())).build().moved_to(point));
            let mut reveal = vec![row, dot];
            chart.push(dot);
            if let Some(before) = previous {
                let line = stage.arena_mut().add(Line::new(before, point).style(stroke_style(Palette::blue(), 3.0)).build()?);
                reveal.push(line);
                chart.push(line);
            }
            appear(stage, &reveal)?;
            previous = Some(point);
        }
        let question = kit.text(stage, "As n grows, what is the limit?", 27.0, Palette::hero(), v(2.6, 1.4))?;
        kit.hold(stage)?;
        disappear(stage, &chart)?;
        appear(stage, &[question])?;
        stage.wait(3.0)?;

        kit.say(stage, "03_2")?;
        let general = kit.tex(stage, r"\lim_{n\to\infty}\left(1+\frac{x}{n}\right)^n=e^x", 28.0, Palette::green(), &[], v(2.2, 0.7))?;
        let limit = kit.tex(stage, &format!(r"e^{{{x}}}\approx {value:.5}"), 30.0, Palette::green(), &[], v(2.6, 0.05))?;
        appear(stage, &[general, limit])?;

        kit.say(stage, "03_3")?;
        let rate = kit.tex(stage, r"f\prime=f,\quad f(0)=1", 33.0, Palette::gold(), &[], v(2.4, -0.7))?;
        appear(stage, &[rate])?;
        kit.until_sentence(stage, "03_3", 1)?;
        let series = [
            kit.tex(stage, r"\sum_{k=0}^{\infty}\frac{x^k}{k!}", 35.0, Palette::blue(), &[], v(2.4, -1.5))?,
        ];
        appear(stage, &series)?;

        kit.say(stage, "03_4")?;
        kit.until_sentence(stage, "03_4", 1)?;
        let check = kit.text(stage, &format!("Series through k={order}: {partial_sum:.5}"), 27.0, Palette::ink(), v(0.0, -2.25))?;
        appear(stage, &[check])?;
        kit.until_sentence(stage, "03_4", 2)?;
        let connection = kit.text(stage, "Same derivative and starting value; one function", 27.0, Palette::hero(), v(0.0, -3.0))?;
        appear(stage, &[connection])?;
        kit.finish(stage)
    }
}
