//! The wedge cancellation worked once on the unit cube (11) and both sides of the theorem checked there (12):
//! ω = g dz∧dx with g = y² + xz, dω = 2y dx∧dy∧dz. CC3 scratch draft: uncompiled, unrendered, sentence
//! timing provisional until Frederick takes exist.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow, fade, line};
use crate::facts::{Q, cube};

pub struct CubeDerivative { pub kit: Kit }
pub struct CubeCheck { pub kit: Kit }

const COLUMN: f64 = 2.9;
const TRIAD: (f64, f64) = (-6.6, -3.2);

/// Oblique projection of the unit cube: x to the right, y back and up, z up.
fn pt(x: f64, y: f64, z: f64) -> (f64, f64) { (-5.9 + 2.4 * x + 1.3 * y, -2.0 + 0.9 * y + 2.4 * z) }

/// A fraction as shown on screen, with a true minus sign.
fn q(x: Q) -> String { x.to_string().replacen('-', "−", 1) }

fn wireframe(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for (a, b) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].iter().zip([(1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]) {
        for y in [0.0, 1.0] { mobs.push(line(stage, pt(a.0, y, a.1), pt(b.0, y, b.1), Palette::blue(), 2.2)?); }
        mobs.push(line(stage, pt(a.0, 0.0, a.1), pt(a.0, 1.0, a.1), Palette::blue(), 2.2)?);
    }
    // The axis triad sits apart from the cube so no axis is drawn across a face.
    for d in [(0.8, 0.0), (0.55, 0.38), (0.0, 0.8)] {
        mobs.extend(arrow(stage, TRIAD, (TRIAD.0 + d.0, TRIAD.1 + d.1), Palette::dim())?);
    }
    Ok(mobs)
}

fn triad_labels(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    [((1.05, 0.0), "x"), ((0.75, 0.55), "y"), ((0.0, 1.05), "z")].iter()
        .map(|&(d, name)| kit.text(stage, name, 22.0, Palette::dim(), v(TRIAD.0 + d.0, TRIAD.1 + d.1)))
        .collect()
}

/// The face where y is fixed at `y`, traced over the wireframe.
fn y_face(stage: &mut Stage<'_>, y: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let c = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    (0..4).map(|k| line(stage, pt(c[k].0, y, c[k].1), pt(c[(k + 1) % 4].0, y, c[(k + 1) % 4].1), color, 4.0)).collect()
}

/// A flow arrow through the centre of the face y = `y`, pointing along +y (`dir` 1) or −y (`dir` −1).
fn through(stage: &mut Stage<'_>, y: f64, dir: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    arrow(stage, pt(0.5, y, 0.5), pt(0.5, y + 0.6 * dir, 0.5), color)
}

impl SceneConstruct for CubeDerivative {
    fn name(&self) -> &str { "11_cube_derivative" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "The cancellation, worked on a cube", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut first = vec![title];
        first.extend(wireframe(stage)?);
        first.extend(triad_labels(&kit, stage)?);
        fade(stage, &first, true)?;
        kit.say(stage, "11_1")?;
        kit.until_sentence(stage, "11_1", 1)?;
        let form = kit.tex(stage, r"\omega=g\,dz\wedge dx", 38.0, Palette::hero(), &[], v(COLUMN, 2.2))?;
        let g = kit.tex(stage, r"g=y^2+xz", 34.0, Palette::hero(), &[], v(COLUMN, 1.5))?;
        fade(stage, &[form, g], true)?;
        kit.until_sentence(stage, "11_1", 2)?;
        let mut flow = through(stage, 1.0, 1.0, Palette::green())?;
        flow.extend(through(stage, 0.0, 1.0, Palette::green())?);
        flow.push(kit.text(stage, "on a face with y fixed: flow in the y direction", 22.0, Palette::dim(), v(COLUMN, 0.85))?);
        fade(stage, &flow, true)?;
        kit.hold(stage)?;
        kit.say(stage, "11_2")?;
        let d_form = kit.tex(stage, r"d\omega=dg\wedge dz\wedge dx", 34.0, Palette::ink(), &[], v(COLUMN, 0.15))?;
        fade(stage, &[d_form], true)?;
        kit.until_sentence(stage, "11_2", 1)?;
        let dg = kit.tex(stage, r"dg=z\,dx+2y\,dy+x\,dz", 34.0, Palette::ink(), &[], v(COLUMN, -0.55))?;
        fade(stage, &[dg], true)?;
        kit.until_sentence(stage, "11_2", 2)?;
        let question = kit.text(stage, "Which parts survive the wedge?", 26.0, Palette::hero(), v(COLUMN, -1.3))?;
        fade(stage, &[question], true)?;
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        stage.wait(3.0)?;
        kit.say(stage, "11_3")?;
        fade(stage, &[question], false)?;
        let dead = [
            kit.tex(stage, r"z\,dx\wedge dz\wedge dx=0", 32.0, Palette::red(), &[], v(COLUMN - 0.6, -1.3))?,
            kit.text(stage, "dx twice", 21.0, Palette::red(), v(COLUMN + 2.6, -1.3))?,
            kit.tex(stage, r"x\,dz\wedge dz\wedge dx=0", 32.0, Palette::red(), &[], v(COLUMN - 0.6, -1.95))?,
            kit.text(stage, "dz twice", 21.0, Palette::red(), v(COLUMN + 2.6, -1.95))?,
        ];
        fade(stage, &dead, true)?;
        kit.until_sentence(stage, "11_3", 1)?;
        let survivor = kit.tex(stage, r"2y\,dy\wedge dz\wedge dx", 32.0, Palette::green(), &[], v(COLUMN - 1.3, -2.65))?;
        fade(stage, &[survivor], true)?;
        kit.until_sentence(stage, "11_3", 2)?;
        let swaps = kit.text(stage, "dx to the front: two swaps, sign unchanged", 21.0, Palette::dim(), v(COLUMN, -3.3))?;
        fade(stage, &[swaps], true)?;
        kit.until_sentence(stage, "11_3", 3)?;
        let ordered = kit.tex(stage, r"=2y\,dx\wedge dy\wedge dz", 32.0, Palette::green(), &[], v(COLUMN + 1.4, -2.65))?;
        fade(stage, &[ordered], true)?;
        kit.finish(stage)
    }
}

impl SceneConstruct for CubeCheck {
    fn name(&self) -> &str { "12_cube_check" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (inside, top, bottom) = cube();
        let title = kit.text(stage, "Both sides, on the cube", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let derivative = kit.tex(stage, r"d\omega=2y\,dx\wedge dy\wedge dz", 34.0, Palette::ink(), &[], v(COLUMN, 2.2))?;
        let mut first = vec![title, derivative];
        first.extend(wireframe(stage)?);
        first.extend(triad_labels(&kit, stage)?);
        fade(stage, &first, true)?;
        kit.say(stage, "12_1")?;
        kit.until_sentence(stage, "12_1", 1)?;
        let interior = kit.text(stage, &format!("inside: ∫ 2y over the cube = {}", q(inside)), 25.0, Palette::ink(), v(COLUMN, 1.5))?;
        fade(stage, &[interior], true)?;
        kit.until_sentence(stage, "12_1", 2)?;
        let others = kit.text(stage, "faces with x or z fixed: dz ∧ dx = 0", 23.0, Palette::dim(), v(COLUMN, 0.8))?;
        fade(stage, &[others], true)?;
        kit.until_sentence(stage, "12_1", 3)?;
        let mut faces = y_face(stage, 1.0, Palette::green())?;
        faces.extend(y_face(stage, 0.0, Palette::hero())?);
        let (bx, by) = pt(0.5, 1.0, 1.0);
        faces.push(kit.text(stage, "y = 1: g = 1 + xz", 22.0, Palette::green(), v(bx, by + 0.4))?);
        let (fx, _) = pt(0.5, 0.0, 0.0);
        faces.push(kit.text(stage, "y = 0: g = xz", 22.0, Palette::hero(), v(fx, -2.45))?);
        fade(stage, &faces, true)?;
        kit.hold(stage)?;
        kit.say(stage, "12_2")?;
        let mut upper = through(stage, 1.0, 1.0, Palette::green())?;
        upper.push(kit.text(stage, &format!("y = 1, outward +y: ∫∫ (1 + xz) dx dz = {}", q(top)), 24.0, Palette::green(), v(COLUMN, 0.0))?);
        fade(stage, &upper, true)?;
        kit.until_sentence(stage, "12_2", 1)?;
        let mut lower = through(stage, 0.0, -1.0, Palette::hero())?;
        lower.push(kit.text(stage, &format!("y = 0, outward −y: −∫∫ xz dx dz = {}", q(bottom)), 24.0, Palette::hero(), v(COLUMN, -0.7))?);
        fade(stage, &lower, true)?;
        kit.until_sentence(stage, "12_2", 2)?;
        let sum = kit.text(stage, &format!("{} − {} = {}", q(top), q(bottom.neg()), q(top.add(bottom))), 32.0, Palette::green(), v(COLUMN, -1.5))?;
        fade(stage, &[sum], true)?;
        kit.until_sentence(stage, "12_2", 3)?;
        let cancel = kit.text(stage, "the xz part is the same on both faces and cancels", 23.0, Palette::ink(), v(COLUMN, -2.3))?;
        fade(stage, &[cancel], true)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shown_fractions_are_the_spoken_ones() {
        let (inside, top, bottom) = cube();
        assert_eq!([q(inside), q(top), q(bottom), q(bottom.neg())], ["1", "5/4", "−1/4", "1/4"]);
        assert_eq!(top.add(bottom), inside);
    }

    #[test]
    fn the_cube_and_its_arrows_stay_left_of_the_text_column() {
        for &(x, y, z) in &[(1.0, 1.0, 1.0), (0.5, 1.6, 0.5), (0.5, -0.6, 0.5), (0.0, 0.0, 0.0)] {
            let (sx, sy) = pt(x, y, z);
            assert!(sx < -1.9 && (-2.2..=1.4).contains(&sy), "{x},{y},{z} -> {sx},{sy}");
        }
        assert!(TRIAD.0 > -7.0 && TRIAD.1 > -3.6);
    }
}
