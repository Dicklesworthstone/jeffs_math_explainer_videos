//! Native FrankenManim storyboard draft for the exponential opening.
//! Not integrated, compiled, rendered, listened to, or independently reviewed.
//! Cue 00_1 ends before the answer; 00_2 begins with the revealed six. The
//! original ElevenLabs full take must be retained when that boundary is made.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct SlideRule { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(seconds))?);
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

fn slide(stage: &mut Stage<'_>, mobs: &[Mob], distance: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = mob.animate().set_anim_args(AnimateArgs {
            run_time: Some(2.0), ..AnimateArgs::default()
        })?.shift(v(distance, 0.0))?;
        animations.push(stage.prepare(animation)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for SlideRule {
    fn name(&self) -> &str { "00_slide_rule" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let left = -4.7_f64;
        let span = 9.0_f64;
        let x = |n: f64| left + span * n.log10();
        let (a, b) = (crate::facts::SLIDE_A, crate::facts::SLIDE_B);
        let product = a * b;
        let offset = x(a) - x(1.0);
        // This mathematical identity is checked again by a separate oracle
        // before any production render; this assertion is only a local guard.
        if (x(b) + offset - x(product)).abs() > 1e-10 {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("slide-rule alignment")));
        }

        kit.say(stage, "00_1")?;
        let heading = kit.text(stage, "How can sliding multiply?", 37.0, Palette::ink(), v(0.0, 2.7))?;
        let d_label = kit.text(stage, "D", 25.0, Palette::dim(), v(-5.5, -0.5))?;
        let c_label = kit.text(stage, "C", 25.0, Palette::blue(), v(-5.5, 1.0))?;
        let d_rail = stage.arena_mut().add(rect(v(left, -0.3), span * 0.91, 0.05, ink_style(Palette::dim()))?);
        let mut fixed = vec![heading, d_label, c_label, d_rail];
        for n in 1..=8 {
            let mark = stage.arena_mut().add(rect(v(x(f64::from(n)), -0.48), 0.025, 0.22, ink_style(Palette::dim()))?);
            fixed.push(mark);
            // Keep the product label hidden until the prediction hold ends.
            if f64::from(n) != product {
                fixed.push(kit.text(stage, &n.to_string(), 23.0, Palette::ink(), v(x(f64::from(n)), -0.8))?);
            }
        }
        appear(stage, &fixed, 0.6)?;
        let c_rail = stage.arena_mut().add(rect(v(left, 1.15), span * 3.0_f64.log10(), 0.05, ink_style(Palette::blue()))?);
        let mut moving = vec![c_rail];
        for n in 1..=3 {
            let mark = stage.arena_mut().add(rect(v(x(f64::from(n)), 0.97), 0.025, 0.22, ink_style(Palette::blue()))?);
            moving.push(mark);
            moving.push(kit.text(stage, &n.to_string(), 23.0, Palette::blue(), v(x(f64::from(n)), 0.65))?);
        }
        appear(stage, &moving, 0.5)?;
        // The scratch sentence boundary is interpolated; real audio requires listening.
        kit.until_sentence(stage, "00_1", 1)?;
        slide(stage, &moving, offset)?;
        let question = kit.text(stage, "What is beneath C's 3?", 28.0, Palette::hero(), v(0.0, -1.7))?;
        appear(stage, &[question], 0.4)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "00_2")?;
        let six = kit.text(stage, &format!("{product}"), 25.0, Palette::green(), v(x(product), -0.8))?;
        let answer = kit.text(stage, &format!("{a} × {b} = {product}"), 32.0, Palette::green(), v(0.0, -2.6))?;
        appear(stage, &[six, answer], 0.6)?;
        // Start the distance explanation at the next sentence; scratch starts
        // are estimates and the full recordings still require listening.
        kit.until_sentence(stage, "00_2", 1)?;
        fixed.extend(moving);
        fixed.extend([question, six, answer]);
        disappear(stage, &fixed)?;
        let start = -3.5_f64;
        let first_width = offset;
        let second_width = x(b) - x(1.0);
        let first = stage.arena_mut().add(rect(v(start, 0.4), first_width, 0.20, ink_style(Palette::blue()))?);
        let second = stage.arena_mut().add(rect(v(start + first_width, 0.4), second_width, 0.20, ink_style(Palette::gold()))?);
        let graphic = [
            kit.text(stage, "Sliding adds distances", 37.0, Palette::ink(), v(0.0, 2.5))?,
            first, second,
            kit.text(stage, "log10(2)", 26.0, Palette::blue(), v(start + first_width/2.0, 1.1))?,
            kit.text(stage, "log10(3)", 26.0, Palette::gold(), v(start + first_width + second_width/2.0, 1.1))?,
            kit.text(stage, "log10(2) + log10(3) = log10(6)", 29.0, Palette::ink(), v(0.0, -0.7))?,
            kit.text(stage, "Adding lengths multiplies the marked numbers", 25.0, Palette::green(), v(0.0, -1.7))?,
        ];
        appear(stage, &graphic, 0.6)?;
        // These sentence marks are provisional review timing, not heard sync.
        kit.until_sentence(stage, "00_2", 2)?;
        disappear(stage, &graphic)?;
        let title = kit.text(stage, "One function, many appearances", 37.0, Palette::ink(), v(0.0, 2.3))?;
        appear(stage, &[title], 0.4)?;
        for (index, (label, color)) in [
            ("Growth", Palette::blue()),
            ("Decay", Palette::gold()),
            ("Circuits", Palette::green()),
            ("Heat", Palette::hero()),
            ("Rotation", Palette::ink()),
        ].iter().enumerate() {
            let x = (index as f64 - 2.0) * 2.5;
            let item = kit.text(stage, label, 30.0, *color, v(x, 0.4))?;
            appear(stage, &[item], 0.4)?;
            stage.wait(0.55)?;
        }
        kit.until_sentence(stage, "00_2", 3)?;
        let takeaway = kit.text(stage, "One underlying idea", 33.0, Palette::green(), v(0.0, -1.5))?;
        appear(stage, &[takeaway], 0.4)?;
        kit.finish(stage)
    }
}
