//! A one-form sampled on single steps (08) and integrated by adding sampled steps (09).
//! CC3 scratch draft: uncompiled, unrendered, sentence timing provisional until Frederick takes exist.
use explainer_kit::{Kit, Palette, ink_style, v};
use fmn::prelude::*;
use super::opening::{arrow, fade, line};
use crate::facts::{polygon_sum, x_dy_at};

pub struct SignedSteps { pub kit: Kit }
pub struct StepSums { pub kit: Kit }

// The step plane: world x in [0, 4], y in [0, 3], drawn left of the text column.
const PLANE_X: f64 = -6.2;
const PLANE_Y: f64 = -2.4;
const UNIT: f64 = 1.3;
const P: (f64, f64) = (2.0, 1.0);
const STEP: (f64, f64) = (1.0, 0.5);
const MOVED: (f64, f64) = (3.0, 2.0);
const COLUMN: f64 = 3.4;

/// A value as shown on screen, with a true minus sign.
pub(super) fn shown(x: f64) -> String {
    if x < 0.0 { format!("−{}", -x) } else { format!("{x}") }
}

fn s(p: (f64, f64)) -> (f64, f64) { (PLANE_X + UNIT * p.0, PLANE_Y + UNIT * p.1) }
fn step(stage: &mut Stage<'_>, at: (f64, f64), d: (f64, f64), color: Srgb) -> fmn::Result<Vec<Mob>> {
    arrow(stage, s(at), s((at.0 + d.0, at.1 + d.1)), color)
}

impl SceneConstruct for SignedSteps {
    fn name(&self) -> &str { "08_signed_steps" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "A one-form at a point", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut plane = vec![
            line(stage, s((0.0, 0.0)), s((4.2, 0.0)), Palette::dim(), 2.0)?,
            line(stage, s((0.0, 0.0)), s((0.0, 3.1)), Palette::dim(), 2.0)?,
        ];
        for k in [1.0_f64, 2.0, 3.0] {
            plane.push(line(stage, s((k, 0.0)), s((k, 3.0)), Palette::faint(), 1.2)?);
            let (x, y) = s((k, 3.0));
            plane.push(kit.text(stage, &format!("x = {k}"), 20.0, Palette::dim(), v(x, y + 0.3))?);
        }
        let (px, py) = s(P);
        plane.push(stage.arena_mut().add(Circle::new().radius(0.08).style(ink_style(Palette::ink())).build().moved_to(v(px, py))));
        plane.push(kit.text(stage, "(2, 1)", 22.0, Palette::ink(), v(px + 0.35, py - 0.4))?);
        let original = step(stage, P, STEP, Palette::hero())?;
        let form = kit.text(stage, "ω = x dy", 32.0, Palette::hero(), v(COLUMN, 2.15))?;
        let rule = kit.text(stage, "value on a tangent vector = x × vertical component", 21.0, Palette::ink(), v(COLUMN, 1.45))?;
        let worked = kit.text(stage, &format!("at (2, 1), step (1, 1/2): 2 × 1/2 = {}", shown(x_dy_at(P, STEP))), 24.0, Palette::hero(), v(COLUMN, 0.75))?;
        let question = kit.text(stage, "reversed step?   doubled step?", 25.0, Palette::hero(), v(COLUMN, 0.0))?;
        let mut first = vec![title, form];
        first.extend(plane.iter().copied());
        fade(stage, &first, true)?;
        kit.say(stage, "08_1")?;
        kit.until_sentence(stage, "08_1", 2)?;
        fade(stage, &[rule], true)?;
        kit.hold(stage)?;
        kit.say(stage, "08_2")?;
        let mut first_step = original.clone();
        first_step.push(worked);
        fade(stage, &first_step, true)?;
        kit.until_sentence(stage, "08_2", 2)?;
        fade(stage, &[question], true)?;
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        stage.wait(3.0)?;
        kit.say(stage, "08_3")?;
        fade(stage, &[question], false)?;
        // The doubled step lies along the original, so the original leaves first.
        fade(stage, &original, false)?;
        let mut answers = step(stage, P, (-STEP.0, -STEP.1), Palette::green())?;
        answers.extend(step(stage, P, (2.0 * STEP.0, 2.0 * STEP.1), Palette::blue())?);
        answers.push(kit.text(stage, &format!("reversed step: {}", shown(x_dy_at(P, (-STEP.0, -STEP.1)))), 24.0, Palette::green(), v(COLUMN, 0.0))?);
        answers.push(kit.text(stage, &format!("doubled step: {}", shown(x_dy_at(P, (2.0 * STEP.0, 2.0 * STEP.1)))), 24.0, Palette::blue(), v(COLUMN, -0.65))?);
        fade(stage, &answers, true)?;
        kit.until_sentence(stage, "08_3", 1)?;
        let mut sideways = step(stage, P, (1.0, 0.0), Palette::dim())?;
        sideways.push(kit.text(stage, &format!("sideways step (1, 0): {}", shown(x_dy_at(P, (1.0, 0.0)))), 24.0, Palette::dim(), v(COLUMN, -1.3))?);
        fade(stage, &sideways, true)?;
        kit.until_sentence(stage, "08_3", 2)?;
        let mut moved = step(stage, MOVED, STEP, Palette::hero())?;
        moved.push(kit.text(stage, &format!("same step where x = 3: {}", shown(x_dy_at(MOVED, STEP))), 24.0, Palette::hero(), v(COLUMN, -1.95))?);
        fade(stage, &moved, true)?;
        kit.until_sentence(stage, "08_3", 3)?;
        let summary = kit.text(stage, "at a fixed point, not an integral along the arrow", 21.0, Palette::ink(), v(COLUMN, -2.75))?;
        fade(stage, &[summary], true)?;
        kit.finish(stage)
    }
}

// The sampled circle: centre and radius on screen.
const CX: f64 = -3.4;
const CY: f64 = -0.35;
const R: f64 = 2.3;
const COUNTS: [usize; 4] = [4, 8, 16, 64];

fn on_circle(k: usize, n: usize) -> (f64, f64) {
    let t = std::f64::consts::TAU * k as f64 / n as f64;
    (CX + R * t.cos(), CY + R * t.sin())
}

/// The n straight steps, with arrowheads while a step is long enough to carry one.
fn polygon(stage: &mut Stage<'_>, n: usize, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for k in 0..n {
        let (a, b) = (on_circle(k, n), on_circle(k + 1, n));
        if n <= 16 { mobs.extend(arrow(stage, a, b, color)?); } else { mobs.push(line(stage, a, b, color, 2.5)?); }
    }
    Ok(mobs)
}

// Project each straight edge onto the vertical axis. These disjoint strips
// tile this convex, counterclockwise polygon; their signed areas are x_mid dy.
fn strip_vertices(k: usize, n: usize) -> [(f64, f64); 4] {
    let (a, b) = (on_circle(k, n), on_circle(k + 1, n));
    [(CX, a.1), a, b, (CX, b.1)]
}

fn area_strip(stage: &mut Stage<'_>, k: usize, n: usize) -> Vec<Mob> {
    let p = strip_vertices(k, n);
    [[p[0], p[1], p[2]], [p[0], p[2], p[3]]].into_iter()
        .filter(|t| ((t[1].0-t[0].0)*(t[2].1-t[0].1)
                  -(t[2].0-t[0].0)*(t[1].1-t[0].1)).abs() > 1e-10)
        .map(|t| stage.arena_mut().add(Polygon::new(t.into_iter().map(|q| v(q.0,q.1)))
            .style(ink_style(Palette::green()).fill(Palette::green(), 0.22)).build()))
        .collect()
}

impl SceneConstruct for StepSums {
    fn name(&self) -> &str { "09_step_sums" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "Integrating by adding steps", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let mut circle = Vec::new();
        for k in 0..96 { circle.push(line(stage, on_circle(k, 96), on_circle(k + 1, 96), Palette::faint(), 1.5)?); }
        let recipe = kit.text(stage, "add x dy over each step, taken at its midpoint", 23.0, Palette::ink(), v(COLUMN, 2.15))?;
        let vertical_axis = line(stage, (CX, CY-R-0.15), (CX, CY+R+0.15), Palette::dim(), 1.5)?;
        let axis_label = kit.text(stage, "x = 0", 22.0, Palette::dim(), v(CX+0.45, CY+R+0.3))?;
        let mut first = vec![title, vertical_axis, axis_label];
        first.extend(circle.iter().copied());
        fade(stage, &first, true)?;
        kit.say(stage, "09_1")?;
        fade(stage, &[recipe], true)?;
        let rows: Vec<Mob> = COUNTS.iter().enumerate()
            .map(|(i, &n)| kit.text(stage, &format!("{n} steps:  {:.2}", polygon_sum(n)), 26.0,
                                    if n == 64 { Palette::hero() } else { Palette::ink() }, v(COLUMN, 1.3 - 0.7 * i as f64)))
            .collect::<fmn::Result<_>>()?;
        kit.until_sentence(stage, "09_1", 2)?;
        let mut current = polygon(stage, 4, Palette::blue())?;
        for k in 0..4 {
            let (a, b) = (on_circle(k, 4), on_circle(k + 1, 4));
            current.push(stage.arena_mut().add(Circle::new().radius(0.06).style(ink_style(Palette::hero())).build()
                .moved_to(v(0.5 * (a.0 + b.0), 0.5 * (a.1 + b.1)))));
        }
        fade(stage, &current, true)?;
        kit.until_sentence(stage, "09_1", 3)?;
        let mut strips = area_strip(stage, 0, 4);
        fade(stage, &strips, true)?;
        kit.until_sentence(stage, "09_1", 4)?;
        for k in 1..4 {
            let part = area_strip(stage, k, 4);
            fade(stage, &part, true)?;
            strips.extend(part);
        }
        current.extend(strips);
        kit.until_sentence(stage, "09_1", 5)?;
        fade(stage, &[rows[0]], true)?;
        kit.hold(stage)?;
        kit.say(stage, "09_2")?;
        for (i, &n) in COUNTS.iter().enumerate().skip(1) {
            if i > 1 { kit.until_sentence(stage, "09_2", i - 1)?; }
            fade(stage, &current, false)?;
            current = polygon(stage, n, Palette::blue())?;
            for k in 0..n { current.extend(area_strip(stage, k, n)); }
            let mut next = current.clone();
            next.push(rows[i]);
            fade(stage, &next, true)?;
        }
        kit.until_sentence(stage, "09_2", 3)?;
        let exact = kit.text(stage, "each sum = area of the polygon walked", 23.0, Palette::dim(), v(COLUMN, -1.6))?;
        fade(stage, &[exact], true)?;
        kit.until_sentence(stage, "09_2", 4)?;
        let limit = kit.text(stage, "→ π = ∫ over the disk of dx ∧ dy", 26.0, Palette::green(), v(COLUMN, -2.4))?;
        fade(stage, &[limit], true)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plane_and_its_steps_stay_left_of_the_text_column() {
        for p in [(4.2, 0.0), (0.0, 3.1), (4.0, 2.0), (4.0, 2.5), (1.0, 0.5)] {
            let (x, y) = s(p);
            assert!(x <= -0.6 && (-2.5..=1.8).contains(&y), "{p:?} -> {x}, {y}");
        }
    }

    #[test]
    fn the_shaded_strips_tile_each_polygon_with_its_computed_area() {
        for n in COUNTS {
            let mut total = 0.0;
            for k in 0..n {
                let p = strip_vertices(k, n);
                let area: f64 = (0..4).map(|i| {
                    let j=(i+1)%4; p[i].0*p[j].1-p[j].0*p[i].1
                }).sum::<f64>() * 0.5;
                assert!(area >= -1e-10);
                let (a,b)=(on_circle(k,n),on_circle(k+1,n));
                let midpoint_area=((a.0+b.0)*0.5-CX)*(b.1-a.1);
                assert!((area-midpoint_area).abs()<1e-10);
                total += area;
            }
            assert!((total/(R*R)-polygon_sum(n)).abs()<1e-10);
        }
    }

    #[test]
    fn the_shown_values_are_the_spoken_ones() {
        assert_eq!(x_dy_at(P, STEP), 1.0);
        assert_eq!(x_dy_at(P, (-STEP.0, -STEP.1)), -1.0);
        assert_eq!(x_dy_at(P, (2.0 * STEP.0, 2.0 * STEP.1)), 2.0);
        assert_eq!(x_dy_at(P, (1.0, 0.0)), 0.0);
        assert_eq!(x_dy_at(MOVED, STEP), 1.5);
        let shown: Vec<String> = COUNTS.iter().map(|&n| format!("{:.2}", polygon_sum(n))).collect();
        assert_eq!(shown, ["2.00", "2.83", "3.06", "3.14"]);
        assert_eq!(super::shown(-1.0), "−1");
    }

    #[test]
    fn the_circle_clears_the_title_and_the_text_column() {
        const { assert!(CY + R < 2.6 && CY - R > -3.2 && CX + R < 0.0); }
    }
}
