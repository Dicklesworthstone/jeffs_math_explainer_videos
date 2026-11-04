//! Native FrankenManim draft of modeled exponential applications.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed 04_1/04_2 are the two retained source paragraphs; no audio is cut.
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;

pub struct Applications {
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

impl SceneConstruct for Applications {
    fn name(&self) -> &str { "04_applications" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "04_1")?;
        let heading = kit.text(stage, "One rate pattern, different models", 35.0, Palette::ink(), v(0.0, 2.9))?;
        appear(stage, &[heading])?;
        let mut list = vec![heading];
        let examples = [
            ("Radioactive decay", "Expected count; fixed decay"),
            ("Charging capacitor", "Ideal resistor-capacitor circuit"),
            ("Cooling", "Constant room temperature"),
            ("Reaction", "First-order reaction model"),
            ("Compounding", "Fixed continuous rate; no cash flows"),
        ];
        for (index, (name, assumption)) in examples.iter().enumerate() {
            // Sentence 0 introduces the pattern; each following sentence
            // introduces one of these modeled examples in the same order.
            kit.until_sentence(stage, "04_1", index + 1)?;
            let y = 1.8 - index as f64 * 0.75;
            let row = [
                kit.text(stage, name, 31.0, Palette::blue(), v(-3.4, y))?,
                kit.text(stage, assumption, 28.0, Palette::ink(), v(1.55, y))?,
            ];
            appear(stage, &row)?;
            list.extend(row);
        }
        // Finish the compounding explanation before asking for a prediction.
        kit.hold(stage)?;
        let question = kit.text(stage, "Must growth continue exponentially forever?", 29.0, Palette::hero(), v(0.0, -2.15))?;
        appear(stage, &[question])?;
        stage.wait(3.0)?;

        kit.say(stage, "04_2")?;
        let limit = kit.text(stage, "Changing conditions can invalidate the model", 23.0, Palette::green(), v(0.0, -3.0))?;
        appear(stage, &[limit])?;
        kit.until_sentence(stage, "04_2", 1)?;
        list.extend([question, limit]);
        disappear(stage, &list)?;

        // Both curves start at 0.08. The second is one illustrative logistic
        // model with limiting capacity 1, not a claim about every population.
        let initial=0.08_f64;
        let axis_left=-4.4_f64;
        let axis_bottom=-2.0_f64;
        let axis_width=8.0_f64;
        let axis_height=3.0_f64;
        let duration=5.0_f64;
        let steps=80_u32;
        let mut graph=vec![
            kit.text(stage, "One possible limit: capacity", 33.0, Palette::ink(), v(0.0,2.8))?,
            kit.text(stage, "constant proportional rate", 24.0, Palette::blue(), v(-2.7,1.95))?,
            kit.text(stage, "capacity-limited example", 24.0, Palette::gold(), v(2.25,1.95))?,
            kit.text(stage, "amount", 21.0, Palette::dim(), v(-4.4,1.4))?,
            kit.text(stage, "time →", 22.0, Palette::dim(), v(3.8,-2.55))?,
            kit.text(stage, "capacity", 20.0, Palette::dim(), v(4.45,1.35))?,
        ];
        graph.push(stage.arena_mut().add(Line::new(v(axis_left,axis_bottom),v(axis_left+axis_width,axis_bottom)).style(stroke_style(Palette::dim(),2.5)).build()?));
        graph.push(stage.arena_mut().add(Line::new(v(axis_left,axis_bottom),v(axis_left,axis_bottom+axis_height)).style(stroke_style(Palette::dim(),2.5)).build()?));
        graph.push(stage.arena_mut().add(Line::new(v(axis_left,axis_bottom+axis_height),v(axis_left+axis_width,axis_bottom+axis_height)).style(stroke_style(Palette::dim(),1.0)).build()?));
        let project=|t: f64, y: f64| v(axis_left+axis_width*t/duration,axis_bottom+axis_height*y);
        let growth=|t: f64| initial*t.exp();
        let growth_top=(1.0/initial).ln();
        let limited=|t: f64| 1.0/(1.0+(1.0/initial-1.0)*(-t).exp());
        for i in 1..=steps {
            let t0=duration*f64::from(i-1)/f64::from(steps);
            let t1=duration*f64::from(i)/f64::from(steps);
            graph.push(stage.arena_mut().add(Line::new(project(t0,limited(t0)),project(t1,limited(t1))).style(stroke_style(Palette::gold(),4.0)).build()?));
            // Stop the exponential at the top of the plot instead of
            // falsely flattening it or drawing beyond the chart.
            if t0 < growth_top {
                let end=t1.min(growth_top);
                graph.push(stage.arena_mut().add(Line::new(project(t0,growth(t0)),project(end,growth(end))).style(stroke_style(Palette::blue(),4.0)).build()?));
            }
        }
        appear(stage, &graph)?;
        stage.wait(2.0)?;
        kit.finish(stage)
    }
}
