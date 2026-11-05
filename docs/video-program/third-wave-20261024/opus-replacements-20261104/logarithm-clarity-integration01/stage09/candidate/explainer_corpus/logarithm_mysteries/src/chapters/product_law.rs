//! Native FrankenManim draft of a principal-log product-law counterexample.
//! Unintegrated, uncompiled, unrendered, and not independently reviewed.
//! Proposed cue 04_1 ends after the sum of angles; 04_2 gives the product
//! and chosen principal angle. The retained ElevenLabs take remains whole.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct ProductLaw {
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

fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

/// Format a rational multiple of pi, with a positive denominator.
fn pi_multiple(numerator: i32, denominator: i32) -> String {
    let divisor = gcd(numerator, denominator);
    let n = numerator / divisor;
    let d = denominator / divisor;
    if n == 0 {
        return "0".into();
    }
    let coefficient = match n {
        1 => String::new(),
        -1 => "-".into(),
        _ => n.to_string(),
    };
    if d == 1 {
        format!("{coefficient}π")
    } else {
        format!("{coefficient}π/{d}")
    }
}

impl SceneConstruct for ProductLaw {
    fn name(&self) -> &str {
        "04_principal_product"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // Both unit-modulus inputs have the same exact argument, 3π/4.
        let (numerator, denominator) = (3_i32, 4_i32);
        let total = numerator * 2;
        let residue = (total + denominator).rem_euclid(2 * denominator) - denominator;
        // Use (-π, π] for this value calculation. The selected point is not
        // on the negative-real branch cut, so its principal log is analytic.
        let principal = if residue == -denominator { denominator } else { residue };
        let turn_difference = total - principal;
        let angle = std::f64::consts::PI * f64::from(numerator) / f64::from(denominator);
        let (imaginary, real) = angle.sin_cos();
        let product = (real * real - imaginary * imaginary, 2.0 * real * imaginary);
        let center = v(-3.2, 0.1);
        let radius = 2.0;

        kit.say(stage, "04_1")?;
        let heading = kit.text(stage, "Can chosen logarithms always add?", 34.0, Palette::ink(), v(0.0, 2.9))?;
        let ring = Circle::new().radius(radius).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(center);
        let ring = stage.arena_mut().add(ring);
        let horizontal = stage.arena_mut().add(rect(v(-5.45, 0.08), 4.5, 0.025, ink_style(Palette::faint()))?);
        let vertical = stage.arena_mut().add(rect(v(-3.21, -2.15), 0.025, 4.5, ink_style(Palette::faint()))?);
        let a_position = v(center[0] + radius * real, center[1] + radius * imaginary);
        let point = Circle::new().radius(0.12).style(ink_style(Palette::blue())).build().moved_to(a_position);
        let point = stage.arena_mut().add(point);
        let input_label = kit.text(stage, "a = b", 21.0, Palette::blue(), v(-5.1, 1.95))?;
        let angle_text = kit.text(stage, &format!("Arg(a) = Arg(b) = {}", pi_multiple(numerator, denominator)), 25.0, Palette::blue(), v(2.0, 1.65))?;
        let total_text = kit.text(stage, &format!("Unwrapped sum = {}", pi_multiple(total, denominator)), 25.0, Palette::hero(), v(2.0, 0.85))?;
        let question = kit.text(stage, "What principal angle does the product get?", 25.0, Palette::ink(), v(1.8, -0.15))?;
        appear(stage, &[heading, ring, horizontal, vertical, question])?;
        kit.until_sentence(stage, "04_1", 1)?;
        appear(stage, &[point, input_label, angle_text, total_text])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "04_2")?;
        let result_position = v(center[0] + radius * product.0, center[1] + radius * product.1);
        let result = Circle::new().radius(0.13).style(ink_style(Palette::green())).build().moved_to(result_position);
        let result = stage.arena_mut().add(result);
        let result_label = kit.text(stage, "ab", 21.0, Palette::green(), v(-2.55, -2.24))?;
        let principal_text = kit.text(stage, &format!("Arg(ab) = {}", pi_multiple(principal, denominator)), 25.0, Palette::green(), v(2.0, -1.1))?;
        let difference = kit.text(stage, &format!("Log(a) + Log(b) − Log(ab) = {}i", pi_multiple(turn_difference, denominator)), 24.0, Palette::hero(), v(0.0, -2.9))?;
        appear(stage, &[result, result_label, principal_text])?;
        kit.until_sentence(stage, "04_2", 1)?;
        appear(stage, &[difference])?;
        kit.finish(stage)
    }
}
