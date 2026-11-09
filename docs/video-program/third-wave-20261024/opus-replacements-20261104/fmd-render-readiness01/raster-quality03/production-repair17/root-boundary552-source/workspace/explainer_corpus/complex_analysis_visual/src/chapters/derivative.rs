//! Native z² derivative scene draft; not integrated, built or reviewed.
//! Proposed 02_1 and 02_2 are the derivative paragraphs in corrected
//! remaining-draft-source03.txt. The ElevenLabs recording remains whole.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct ComplexDerivative {
    pub kit: Kit,
}

#[derive(Clone, Copy)]
struct Complex { re: f64, im: f64 }
impl Complex {
    fn add(self, other: Self) -> Self { Self { re: self.re + other.re, im: self.im + other.im } }
    fn subtract(self, other: Self) -> Self { Self { re: self.re - other.re, im: self.im - other.im } }
    fn multiply(self, other: Self) -> Self {
        Self { re: self.re * other.re - self.im * other.im, im: self.re * other.im + self.im * other.re }
    }
    fn scale(self, factor: f64) -> Self { Self { re: self.re * factor, im: self.im * factor } }
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn segment(stage: &mut Stage<'_>, start: Vec3, end: Vec3, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(Line::new(start, end).style(stroke_style(color, 4.0)).build()?))
}

fn point(stage: &mut Stage<'_>, center: Vec3, color: Srgb) -> Mob {
    let circle = Circle::new().radius(0.09).style(ink_style(color)).build().moved_to(center);
    stage.arena_mut().add(circle)
}

impl SceneConstruct for ComplexDerivative {
    fn name(&self) -> &str { "02_complex_derivative" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let z = Complex { re: 1.0, im: 1.0 };
        let h = 0.25_f64;
        let steps = [Complex { re: h, im: 0.0 }, Complex { re: 0.0, im: h }];
        let derivative = z.scale(2.0);
        let squared = z.multiply(z);
        let input_center = v(-3.1, -0.1);
        let output_center = v(3.1, -0.1);
        let colors = [Palette::blue(), Palette::gold()];
        let degree = 2_u32;

        kit.say(stage, "02_1")?;
        let mut first = vec![
            kit.text(stage, "One local rule, from every direction", 34.0, Palette::ink(), v(0.0, 2.9))?,
            kit.text(stage, &format!("Example: f(z) = z^{degree}"), 22.0, Palette::dim(), v(0.0, 2.2))?,
            kit.text(stage, &format!("input near z₀ = {} + {}i", z.re, z.im), 23.0, Palette::blue(), v(-3.1, 2.0))?,
            kit.text(stage, &format!("output near z₀² = {}i", squared.im), 23.0, Palette::gold(), v(3.1, 2.0))?,
            kit.text(stage, "Lines: linear approximation; dots: exact finite changes", 21.0, Palette::dim(), v(0.0, -2.0))?,
        ];
        for (index, step) in steps.iter().enumerate() {
            let exact = z.add(*step).multiply(z.add(*step)).subtract(squared);
            let tangent = derivative.multiply(*step);
            let input_end = v(input_center[0] + 4.0 * step.re, input_center[1] + 4.0 * step.im);
            let tangent_end = v(output_center[0] + 2.0 * tangent.re, output_center[1] + 2.0 * tangent.im);
            let exact_end = v(output_center[0] + 2.0 * exact.re, output_center[1] + 2.0 * exact.im);
            first.push(segment(stage, input_center, input_end, colors[index])?);
            first.push(segment(stage, output_center, tangent_end, colors[index])?);
            first.push(point(stage, exact_end, colors[index]));
        }
        let question = kit.text(stage, "What changes where the derivative is zero?", 26.0, Palette::hero(), v(0.0, -2.9))?;
        first.push(question);
        fade(stage, &first, true)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "02_2")?;
        fade(stage, &first, false)?;
        let diagonal = h / 2.0_f64.sqrt();
        let critical_steps = [Complex { re: h, im: 0.0 }, Complex { re: diagonal, im: diagonal }];
        let mut critical = vec![
            kit.text(stage, "At z = 0, the first-order derivative vanishes", 32.0, Palette::ink(), v(0.0, 2.9))?,
            kit.text(stage, "input directions", 24.0, Palette::blue(), v(-3.1, 2.0))?,
            kit.text(stage, "directions after squaring", 24.0, Palette::gold(), v(3.1, 2.0))?,
            kit.text(stage, "Directions magnified separately", 21.0, Palette::dim(), v(0.0, -2.0))?,
            kit.text(stage, &format!("{}° → {}°", 180 / 4, degree * 180 / 4), 25.0, Palette::green(), v(0.0, -1.2))?,
            kit.text(stage, "Holomorphic means differentiable on an open region", 23.0, Palette::hero(), v(0.0, -2.9))?,
        ];
        for (index, step) in critical_steps.iter().enumerate() {
            let image = step.multiply(*step);
            critical.push(segment(stage, input_center, v(input_center[0] + 4.0 * step.re, input_center[1] + 4.0 * step.im), colors[index])?);
            critical.push(segment(stage, output_center, v(output_center[0] + 16.0 * image.re, output_center[1] + 16.0 * image.im), colors[index])?);
        }
        fade(stage, &critical, true)?;
        kit.finish(stage)
    }
}
