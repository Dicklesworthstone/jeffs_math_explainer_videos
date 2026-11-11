//! 01_the_bump: a perfectly smooth curve whose recipe at zero stops at one
//! for a reason the real line cannot show (storyboard 01_1–01_6). The
//! partial sums are drawn from `model::bump_recipe0` as one gold curve that
//! morphs degree by degree; the readouts at 1.1 and the radius come from
//! `facts`.

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Plane, dot, fact, fact_f64, fact_list, fade_in_all, fade_out_all, faded_line, hero, input, line_style, pause, pole,
    pole_mark, portrait, predict, restroke, say, shift_all, tween_linear, value, wait_until, warn, zero_mark,
};
use crate::model::{self, C, Func, I, ZERO};

pub const TITLE: &str = "the bump";
pub const SERIES_TEX: &str = r"1 - x^2 + x^4 - x^6 + \cdots";
pub const BUMP_TEX: &str = r"\frac{1}{1 + x^2}";
pub const DEGREE: &str = "terms up to degree";
pub const RECIPE: &str = "recipe ≈ ";
pub const BUMP_IS: &str = "bump ≈ ";
pub const ASK: [&str; 2] = ["what stops the recipe", "at x = 1?"];
pub const POLE_I: &str = "i";
pub const POLE_MINUS_I: &str = "−i";
pub const RADIUS: &str = "radius ";
pub const AT_X: &str = "x = ";
pub const COMING_UP: &str = "coming up";

/// The real graph's window (storyboard 01_1; chapter 15 returns to it).
pub fn graph() -> Frame2 {
    Frame2 {
        center: v(-1.3, -0.55),
        width: 8.2,
        height: 4.6,
        x: (-3.0, 3.0),
        y: (-0.6, 1.6),
    }
}

/// The partial sum with `k` = degree/2 continuous: the sum through
/// (−x²)^⌊k⌋ plus the next term scaled by the fraction, so one curve morphs
/// from each degree to the next.
pub fn morphing_recipe(x: f64, k: f64) -> f64 {
    let j = k.max(0.0).floor();
    let base = model::bump_recipe0(x, 2 * j as u32);
    base + (k - j) * (-x * x).powi(j as i32 + 1)
}

/// The curve's points: x outward from 0 both ways until the curve leaves
/// the window (the last point is cut at the window's edge), even in x.
pub fn recipe_points(g: &Frame2, k: f64) -> Vec<Vec3> {
    let n = 300;
    let mut right = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let x = g.x.1 * i as f64 / n as f64;
        let y = morphing_recipe(x, k);
        if y < g.y.0 || y > g.y.1 {
            right.push((x, y.clamp(g.y.0, g.y.1)));
            break;
        }
        right.push((x, y));
    }
    let mut pts: Vec<Vec3> = right.iter().rev().map(|&(x, y)| g.p(-x, y)).collect();
    pts.extend(right.iter().skip(1).map(|&(x, y)| g.p(x, y)));
    pts
}

pub struct TheBump {
    pub kit: Kit,
}

impl SceneConstruct for TheBump {
    fn name(&self) -> &str {
        "01_the_bump"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let g = graph();

        // 01_1: the curve draws left to right; a dot rides it; its peak is 1
        let line = say(&kit, stage, "01_1")?;
        let x_axis = stage.arena_mut().add(
            polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let y_axis = stage.arena_mut().add(
            polyline(vec![g.p(0.0, g.y.0), g.p(0.0, g.y.1)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let title = kit.text(stage, TITLE, 30.0, Palette::ink(), v(-2.3, 3.4))?;
        let formula = kit.tex(stage, BUMP_TEX, 36.0, Palette::ink(), &[], v(-0.6, 3.4))?;
        fade_in_all(stage, &[x_axis, y_axis, title, formula], 0.6)?;
        let drawn = stage.add_value_tracker(g.x.0);
        let bump_pts = |to: f64| -> Vec<Vec3> {
            let n = 360;
            (0..=n)
                .map(|i| {
                    let x = g.x.0 + (to - g.x.0) * i as f64 / n as f64;
                    g.p(x, model::bump(x))
                })
                .collect()
        };
        let curve = stage.arena_mut().add(
            polyline(bump_pts(g.x.0 + 1e-3))
                .style(line_style(Palette::ink(), 5.0))
                .build(),
        );
        let rider = dot(stage, g.p(g.x.0, model::bump(g.x.0)), 0.08, hero());
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_to_scene(curve)?;
        stage.add_to_scene(rider)?;
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let to = value(s, drawn).max(g.x.0 + 1e-3);
                if to == last {
                    return;
                }
                let n = 360;
                let pts: Vec<Vec3> = (0..=n)
                    .map(|i| {
                        let x = g.x.0 + (to - g.x.0) * i as f64 / n as f64;
                        g.p(x, model::bump(x))
                    })
                    .collect();
                let _ = s.set_points(curve, polyline(pts).build().points());
                s.move_to(rider, g.p(to, model::bump(to)), ORIGIN);
                last = to;
            },
            true,
        )?;
        line.wait_sentence(stage, 1)?;
        let rt = line.until(stage, "It has no corners", 3.0)?;
        stage.play(tween_linear(drawn, g.x.1, rt)?)?;
        stage.clear_updaters(anchor, true);
        let _ = stage.set_points(curve, polyline(bump_pts(g.x.1)).build().points());
        let peak = kit.text(
            stage,
            fact("bump.peak")?,
            26.0,
            Palette::ink(),
            v(g.p(0.0, 1.0)[0] + 0.3, g.p(0.0, 1.0)[1] + 0.25),
        )?;
        fade_in_all(stage, &[peak], 0.4)?;
        fade_out_all(stage, &[rider], 0.3)?;

        // 01_2: the recipe near zero; one gold curve through degrees 2, 4, 6
        kit.hold(stage)?;
        let line = say(&kit, stage, "01_2")?;
        let series = kit.tex(stage, SERIES_TEX, 36.0, hero(), &[], v(-1.3, 2.55))?;
        fade_in_all(stage, &[series], 0.5)?;
        let k = stage.add_value_tracker(1.0);
        let recipe = stage
            .arena_mut()
            .add(polyline(recipe_points(&g, 1.0)).style(line_style(hero(), 5.0)).build());
        let morph = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(morph)?;
        let mut last_k = f64::NAN;
        stage.add_updater(
            morph,
            move |s, _| {
                let kv = value(s, k);
                if kv == last_k {
                    return;
                }
                let _ = s.set_points(recipe, polyline(recipe_points(&g, kv)).build().points());
                last_k = kv;
            },
            true,
        )?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[recipe], 0.5)?;
        for (deg, phrase) in [(4.0, "plus x to the fourth"), (6.0, "minus x to the sixth")] {
            line.wait_for(stage, phrase)?;
            stage.play(tween_linear(k, deg / 2.0, 0.9)?)?;
        }
        let degree_label = kit.text(stage, DEGREE, 24.0, Palette::dim(), v(4.9, 1.2))?;
        let degree = kit.live_value(
            stage,
            move |s| 2.0 * value(s, k).ceil(),
            0,
            false,
            30.0,
            hero(),
            v(4.9, 0.7),
            ORIGIN,
        )?;
        fade_in_all(stage, &[degree_label], 0.4)?;

        // 01_3: degree 10, 20, 40; it works inside ±1 and falls apart just past it
        kit.hold(stage)?;
        let line = say(&kit, stage, "01_3")?;
        let guide = |x: f64| -> fmn::Result<VMobject> {
            Ok(DashedLine::new(g.p(x, g.y.0), g.p(x, g.y.1))
                .dash_length(0.09)
                .style(faded_line(Palette::dim(), 2.0, 0.8))
                .build()?)
        };
        let guides = [stage.arena_mut().add(guide(-1.0)?), stage.arena_mut().add(guide(1.0)?)];
        fade_in_all(stage, &guides, 0.4)?;
        for target in [5.0, 10.0, 20.0] {
            stage.play(tween_linear(k, target, 1.2)?)?;
            pause(stage, 0.3)?;
        }
        line.wait_for(stage, "At one point one")?;
        let at = g.p(model::BUMP_READ_AT, model::bump(model::BUMP_READ_AT));
        let mark = dot(stage, at, 0.07, Palette::ink());
        let recipe_says = kit.text(
            stage,
            &format!("{RECIPE}{}", fact("bump.recipe40.at1_1")?),
            28.0,
            warn(),
            v(4.9, -0.4),
        )?;
        let bump_says = kit.text(
            stage,
            &format!("{BUMP_IS}{}", fact("bump.at1_1")?),
            28.0,
            Palette::ink(),
            v(4.9, -1.0),
        )?;
        let at_label = kit.text(
            stage,
            &format!("{AT_X}{}", fact("bump.read_at")?),
            24.0,
            Palette::dim(),
            v(4.9, 0.1),
        )?;
        fade_in_all(stage, &[mark, at_label, recipe_says], 0.4)?;
        line.wait_sentence(stage, 3)?;
        fade_in_all(stage, &[bump_says], 0.4)?;

        // 01_4: nothing happens to the curve at one; what stops the recipe?
        kit.hold(stage)?;
        let line = say(&kit, stage, "01_4")?;
        for m in guides {
            restroke(stage, m, hero(), 1.0);
        }
        stage.clear_updaters(morph, true);
        let asked = predict(&kit, stage, &line, &ASK, v(4.9, 3.0))?;

        // 01_5: the answer is off the line; poles at ±i; a circle of radius 1
        let line = say(&kit, stage, "01_5")?;
        let mut leaving = vec![
            recipe,
            series,
            degree_label,
            degree,
            recipe_says,
            bump_says,
            at_label,
            mark,
            peak,
        ];
        leaving.extend(asked);
        leaving.extend(guides);
        fade_out_all(stage, &leaving, 0.5)?;
        // the graph's real axis becomes the plane's: same x scale, y = 0 kept in place
        let y0 = g.p(0.0, 0.0)[1];
        let unit = g.width / (g.x.1 - g.x.0);
        let plane = Plane::new(v(g.center[0], y0 + 0.4), g.width, (-3.0, 3.0), (-1.9, 2.1));
        debug_assert!((plane.unit() - unit).abs() < 1e-9);
        let lift = plane.p(ZERO)[1] - y0;
        shift_all(stage, &[x_axis, curve, y_axis], 0.0, lift, 0.8)?;
        restroke(stage, curve, Palette::ink(), 0.35);
        line.wait_for(stage, "Let x be a complex number")?;
        let vertical = stage.arena_mut().add(
            polyline(vec![plane.p(C::new(0.0, -1.6)), plane.p(C::new(0.0, 1.8))])
                .style(faded_line(input(), 2.0, 0.7))
                .build(),
        );
        fade_in_all(stage, &[vertical], 0.6)?;
        line.wait_for(stage, "They sit at i")?;
        let poles: Vec<f64> = fact_list("bump.poles")?;
        let mut pole_mobs = Vec::new();
        for &im in &poles {
            pole_mobs.extend(pole_mark(stage, plane.p(C::new(0.0, im))));
        }
        let label_i = kit.text(stage, POLE_I, 28.0, pole(), plane.p(C::new(0.3, poles[0] - 0.3)))?;
        let label_mi = kit.text(stage, POLE_MINUS_I, 28.0, pole(), plane.p(C::new(0.35, poles[1] + 0.3)))?;
        pole_mobs.extend([label_i, label_mi]);
        fade_in_all(stage, &pole_mobs, 0.5)?;
        line.wait_for(stage, "A recipe built at zero")?;
        let radius = stage.add_value_tracker(0.02);
        let ring = stage.arena_mut().add(
            polyline(circle_pts(&plane, 0.02))
                .style(line_style(hero(), 5.0))
                .build(),
        );
        stage.add_to_scene(ring)?;
        let grow = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(grow)?;
        stage.add_updater(
            grow,
            move |s, _| {
                let r = value(s, radius);
                let _ = s.set_points(ring, polyline(circle_pts(&plane, r)).build().points());
            },
            true,
        )?;
        let r0 = fact_f64("bump.radius0")?;
        stage.play(tween_linear(radius, r0, 1.6)?)?;
        stage.clear_updaters(grow, true);
        line.wait_for(stage, "That circle has radius one")?;
        let radius_segment = stage.arena_mut().add(
            polyline(vec![plane.p(ZERO), plane.p(C::real(r0))])
                .style(line_style(hero(), 9.0))
                .build(),
        );
        let radius_label = kit.text(
            stage,
            &format!("{RADIUS}{}", fact("bump.radius0")?),
            26.0,
            hero(),
            plane.p(C::new(1.55, -0.45)),
        )?;
        fade_in_all(stage, &[radius_segment, radius_label], 0.5)?;

        // 01_6: off-line points decide the line; previews of what is coming
        kit.hold(stage)?;
        let line = say(&kit, stage, "01_6")?;
        line.wait_sentence(stage, 2)?;
        let thumbs = previews(&kit, stage)?;
        for chunk in thumbs.chunks(thumbs.len().div_ceil(3)) {
            fade_in_all(stage, chunk, 0.5)?;
            pause(stage, 0.6)?;
        }
        line.wait_sentence(stage, 3)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            x_axis,
            y_axis,
            curve,
            vertical,
            ring,
            radius_segment,
            radius_label,
            title,
            formula,
        ];
        all.extend(pole_mobs);
        all.extend(thumbs);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

/// The circle |z| = r in the plane, closed.
fn circle_pts(plane: &Plane, r: f64) -> Vec<Vec3> {
    let mut zs = model::circle(ZERO, r.max(1e-3), 180);
    zs.push(zs[0]);
    zs.iter().map(|&z| plane.p(z)).collect()
}

/// Three small previews at the right: a colour picture (z²), a loop with an
/// arrow turning around zero, a half-disk contour (storyboard 01_6).
fn previews(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    let x = 5.6;
    let small = Plane::new(v(x, 2.0), 1.5, (-1.2, 1.2), (-1.2, 1.2));
    let pic = portrait(stage, Func::Square, &small, 1.0)?;
    out.push(pic.group);
    let lp = Plane::new(v(x, 0.0), 1.5, (-1.2, 1.2), (-1.2, 1.2));
    out.push(lp.circle(stage, ZERO, 0.8, hero(), 4.0)?);
    let arrow = Arrow::new(lp.p(ZERO), lp.p(C::polar(0.8, 0.9)))
        .buff(0.0)
        .thickness(3.0)
        .color(Palette::ink())
        .build()?;
    out.push(stage.arena_mut().add(arrow));
    let hp = Plane::new(v(x, -2.0), 1.5, (-1.2, 1.2), (-0.6, 1.8));
    let mut half: Vec<C> = (0..=60)
        .map(|i| C::polar(1.0, std::f64::consts::PI * f64::from(i) / 60.0))
        .collect();
    half.push(C::real(1.0));
    out.push(hp.path(stage, &half, hero(), 4.0)?);
    out.extend(zero_mark(stage, hp.p(I.scale(0.7))));
    let caption = kit.text(stage, COMING_UP, 22.0, Palette::dim(), v(x, 3.15))?;
    out.insert(0, caption);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_morph_passes_through_every_partial_sum() {
        for d in [2u32, 4, 6, 40] {
            for x in [0.3, 0.9, 1.05] {
                let a = morphing_recipe(x, f64::from(d / 2));
                assert!((a - model::bump_recipe0(x, d)).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn the_drawn_recipe_hugs_the_bump_inside_one_and_leaves_beyond() {
        let g = graph();
        let pts = recipe_points(&g, 20.0);
        let (x_end, _) = g.data(*pts.last().unwrap());
        assert!(
            x_end > 1.0 && x_end < 1.2,
            "the degree-40 curve leaves the window just past 1: {x_end}"
        );
        assert!((morphing_recipe(0.5, 20.0) - model::bump(0.5)).abs() < 1e-6);
        // what 01_3 says at 1.1: about 25, against less than one half
        assert!((morphing_recipe(1.1, 20.0) - 25.2).abs() < 0.1);
    }

    #[test]
    fn the_plane_keeps_the_graphs_scale() {
        let g = graph();
        let plane = Plane::new(v(g.center[0], 0.0), g.width, (-3.0, 3.0), (-1.9, 2.1));
        assert!((plane.unit() - g.width / 6.0).abs() < 1e-12);
    }
}
