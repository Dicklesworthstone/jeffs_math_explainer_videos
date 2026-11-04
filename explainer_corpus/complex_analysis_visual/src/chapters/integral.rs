//! Native FrankenManim draft: a real integral via two upper-half-plane poles.
//! Not integrated, compiled, rendered, listened to, or independently reviewed.
//! Proposed 05_1 is the first retained source paragraph; 05_2 ends after the
//! residue formula; 05_3 contains the sum, result and arc. No audio is cut.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct RealIntegral {
    pub kit: Kit,
}

#[derive(Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn multiply(self, other: Self) -> Self {
        Self { re: self.re * other.re - self.im * other.im, im: self.re * other.im + self.im * other.re }
    }

    fn reciprocal(self) -> Self {
        let norm = self.re * self.re + self.im * self.im;
        Self { re: self.re / norm, im: -self.im / norm }
    }
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

impl SceneConstruct for RealIntegral {
    fn name(&self) -> &str {
        "05_real_integral"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let degree = 4_u32;
        let roots: Vec<Complex> = (0..degree).map(|k| {
            let angle = f64::from(2 * k + 1) * std::f64::consts::PI / f64::from(degree);
            let (im, re) = angle.sin_cos();
            Complex { re, im }
        }).collect();
        let upper: Vec<Complex> = roots.iter().copied().filter(|root| root.im > 0.0).collect();
        if upper.len() != (degree / 2) as usize {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("unexpected upper poles")));
        }
        let residues: Vec<Complex> = upper.iter().map(|&root| {
            let cube = root.multiply(root).multiply(root);
            Complex { re: cube.re * f64::from(degree), im: cube.im * f64::from(degree) }.reciprocal()
        }).collect();
        let sum = residues.iter().fold(Complex { re: 0.0, im: 0.0 }, |sum, value| Complex {
            re: sum.re + value.re, im: sum.im + value.im,
        });
        let result = Complex { re: -2.0 * std::f64::consts::PI * sum.im, im: 2.0 * std::f64::consts::PI * sum.re };
        let radicand = upper.len();
        let expected = std::f64::consts::PI / (radicand as f64).sqrt();
        if result.im.abs() > 1e-10 || (result.re - expected).abs() > 1e-10 {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("residue check failed")));
        }

        kit.say(stage, "05_1")?;
        let heading = kit.text(stage, &format!("∫ dx / (1 + x^{degree}) over the real line"), 31.0, Palette::ink(), v(0.0, 2.9))?;
        let plane = kit.text(stage, "complex input plane", 22.0, Palette::dim(), v(-3.4, -2.8))?;
        let direction = kit.text(stage, "upper arc: counterclockwise", 20.0, Palette::blue(), v(-3.3, 2.2))?;
        let origin = (-3.3_f64, -0.35_f64);
        let scale = 1.1_f64;
        let contour_radius = 2.0_f64;
        let axis = stage.arena_mut().add(Line::new(
            v(origin.0 - contour_radius * scale, origin.1),
            v(origin.0 + contour_radius * scale, origin.1),
        ).style(stroke_style(Palette::dim(), 2.0)).build()?);
        let mut picture = vec![heading, plane, direction, axis];
        for k in 0..48 {
            let first = std::f64::consts::PI * f64::from(k) / 48.0;
            let second = std::f64::consts::PI * f64::from(k + 1) / 48.0;
            let a = v(origin.0 + contour_radius * scale * first.cos(), origin.1 + contour_radius * scale * first.sin());
            let b = v(origin.0 + contour_radius * scale * second.cos(), origin.1 + contour_radius * scale * second.sin());
            picture.push(stage.arena_mut().add(Line::new(a, b).style(stroke_style(Palette::blue(), 3.0)).build()?));
        }
        for root in &roots {
            let point = Circle::new().radius(0.10).style(ink_style(Palette::dim())).build()
                .moved_to(v(origin.0 + scale * root.re, origin.1 + scale * root.im));
            picture.push(stage.arena_mut().add(point));
        }
        let prediction = kit.text(stage, "Which poles lie inside?", 26.0, Palette::hero(), v(2.5, 1.3))?;
        picture.push(prediction);
        appear(stage, &picture)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "05_2")?;
        let mut selected = Vec::new();
        for root in &upper {
            let point = Circle::new().radius(0.13).style(ink_style(Palette::gold())).build()
                .moved_to(v(origin.0 + scale * root.re, origin.1 + scale * root.im));
            selected.push(stage.arena_mut().add(point));
        }
        selected.push(kit.text(stage, &format!("{} poles inside", upper.len()), 25.0, Palette::gold(), v(2.5, 0.4))?);
        selected.push(kit.text(stage, &format!("Residue at r: 1/({degree}r^{})", degree - 1), 24.0, Palette::ink(), v(2.5, -0.4))?);
        appear(stage, &selected)?;

        kit.say(stage, "05_3")?;
        let answer = [
            kit.text(stage, &format!("residue sum = −i√{radicand}/{degree}"), 23.0, Palette::dim(), v(2.5, -1.2))?,
            kit.text(stage, &format!("integral = π/√{radicand}"), 30.0, Palette::green(), v(2.3, -2.0))?,
            kit.text(stage, &format!("|arc| ≤ πR/(R^{degree} − 1), R > 1"), 21.0, Palette::dim(), v(0.0, -3.2))?,
        ];
        appear(stage, &answer)?;
        kit.finish(stage)
    }
}
