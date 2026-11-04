//! 09_boundary: Cauchy's integral formula as the payoff of loop sums, shown
//! as an average (storyboard 09_1–09_9). The bump's colour picture stays on
//! the left throughout. Its samples on a circle appear twice: as dots on the
//! circle, and as their values in a value plane on the right, where the
//! average is the visible centroid. On radius ½ the centroid sits on 1
//! (8 and 16 samples). On radius 2 the circle has swallowed both poles and
//! the centroid falls to 0. Turning each value back by its own angle
//! (once, then twice) moves the centroid to r·c₁ and r²·c₂, which reads the
//! recipe's coefficients off the boundary. The chapter ends on the opening
//! puzzle answered: the recipe at 0 works out to the poles, and recentred at
//! 1 it works on (−0.41, 2.41).

use std::f64::consts::TAU;

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, dot, fact, fact_f64, fade_in_all, fade_out_all, faded_line, hero, input, line_style, marks,
    names_now, output, pause, pole, precisely_now, predict, say, tween_linear, value, wait_until,
};
use crate::model::{self, C, Func, ZERO};

pub const SPLIT_TEX: &str = r"\frac{f(z)}{z-a} = \frac{f(a)}{z-a} + \frac{f(z)-f(a)}{z-a}";
pub const REST: &str = "the second piece is holomorphic: its loop integral is 0";
pub const RESULT_TEX: &str = r"\oint \frac{f(z)}{z-a}\,dz = 2\pi i\, f(a)";
pub const FORMULA_TEX: &str = r"f(a) = \frac{1}{2\pi i}\oint \frac{f(z)}{z-a}\,dz";
pub const ON_CIRCLE_TEX: &str = r"z = a + r e^{i\theta}:\quad dz = i\,(z-a)\,d\theta";
pub const AVERAGE_TEX: &str = r"f(a) = \frac{1}{2\pi}\int_0^{2\pi} f(a + r e^{i\theta})\,d\theta";
pub const AVERAGE: &str = "the value at the centre = the average on the circle";
pub const VALUES: &str = "the samples' values";
pub const AVG_OF: &str = "average of ";
pub const COLON: &str = ": ";
pub const ASK: [&str; 1] = ["average = 1?"];
pub const LIMIT: &str = "limit: ";
pub const SHARES: &str = "bump at 0, then each pole: ";
pub const NEEDS: &str =
    "Cauchy's formula needs f holomorphic on an open set containing the closed disk: on the circle and inside it";
pub const COEF_TEX: &str = r"c_n = \frac{1}{r^n}\cdot \mathrm{avg}_k\; f(a + r e^{i\theta_k})\, e^{-i n \theta_k}";
pub const TURNED: [&str; 2] = ["each turned back by its angle", "each turned back by twice its angle"];
pub const READ: [&str; 2] = [
    r"c_1 = \frac{\mathrm{average}}{r} \approx ",
    r"c_2 = \frac{\mathrm{average}}{r^2} \approx ",
];
pub const ASK_RECENTRE: [&str; 2] = ["recipe centred at 1:", "works from ? to ?"];
pub const WORKS: &str = "works on ";
pub const CHECK: &str = " terms at ";

/// "average of 8: 1.0039": how many samples, then what they average to.
fn average_says(samples: &str, mean: &str) -> String {
    format!("{AVG_OF}{samples}{COLON}{mean}")
}

/// The bump's picture, [−2.5, 2.5]², on the left.
pub fn picture() -> Plane {
    Plane::new(v(-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5))
}

/// Where the samples' values are drawn (09_3–09_5).
pub fn values_plane() -> Plane {
    Plane::new(v(4.0, 0.3), 5.0, (-0.55, 1.55), (-0.55, 0.55))
}

/// Where the turned values are drawn (09_6).
pub fn turned_plane() -> Plane {
    Plane::new(v(4.0, 0.15), 3.4, (-1.5, 1.5), (-1.5, 1.5))
}

/// The real graph of 09_7–09_9.
pub fn graph() -> Frame2 {
    Frame2 {
        center: v(3.9, -0.6),
        width: 5.6,
        height: 3.0,
        x: (-1.6, 3.0),
        y: (-0.5, 1.5),
    }
}

/// The bump at a complex point.
pub fn bump_at(z: C) -> C {
    Func::Bump.eval(z).unwrap_or(ZERO)
}

/// The `n` sample points on the circle of radius `r` about 0.
pub fn samples(r: f64, n: usize) -> Vec<C> {
    model::circle(ZERO, r, n)
}

/// The mean of some points.
pub fn centroid(ws: &[C]) -> C {
    ws.iter().fold(ZERO, |a, &b| a + b).scale(1.0 / ws.len().max(1) as f64)
}

/// The values on the circle of radius `r`, each turned back by `s` times its
/// own angle (`s` = 1 reads r·c₁, `s` = 2 reads r²·c₂).
pub fn turned_values(r: f64, n: usize, s: f64) -> Vec<C> {
    (0..n)
        .map(|k| {
            let th = TAU * k as f64 / n as f64;
            bump_at(C::polar(r, th)) * C::polar(1.0, -s * th)
        })
        .collect()
}

/// A function of x drawn in `g`, cut into pieces wherever it leaves the window.
pub fn clipped(g: &Frame2, f: impl Fn(f64) -> f64) -> Vec<Vec<Vec3>> {
    let n = 600;
    let mut pieces: Vec<Vec<Vec3>> = Vec::new();
    let mut current: Vec<Vec3> = Vec::new();
    for k in 0..=n {
        let x = g.x.0 + (g.x.1 - g.x.0) * f64::from(k) / f64::from(n);
        let y = f(x);
        if y.is_finite() && y >= g.y.0 && y <= g.y.1 {
            current.push(g.p(x, y));
        } else if !current.is_empty() {
            pieces.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        pieces.push(current);
    }
    pieces.into_iter().filter(|p| p.len() >= 2).collect()
}

fn signed(s: &str) -> String {
    s.replace('-', "−")
}

pub struct Boundary {
    pub kit: Kit,
}

impl SceneConstruct for Boundary {
    fn name(&self) -> &str {
        "09_boundary"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = picture();
        let right = 4.0;

        // 09_1: divide by z − a; split; the rest integrates to zero
        let line = say(&kit, stage, "09_1")?;
        let painter = Painter::build(stage, Some(Func::Bump), p, 1.0)?;
        let border = p.border(stage, Palette::faint())?;
        painter.show(stage, 1.0)?;
        let poles = marks(stage, Func::Bump, &p);
        fade_in_all(stage, &[border], 0.3)?;
        fade_in_all(stage, &poles, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let centre = dot(stage, p.p(ZERO), 0.08, Palette::ink());
        let a_name = kit.tex(stage, "a", 28.0, Palette::ink(), &[], p.p(C::new(0.18, -0.2)))?;
        let half = p.circle(stage, ZERO, 0.5, hero(), 5.0)?;
        let beyond = p.circle(stage, ZERO, 0.8, Palette::faint(), 2.0)?;
        fade_in_all(stage, &[centre, a_name, half, beyond], 0.5)?;
        let split = kit.tex(stage, SPLIT_TEX, 34.0, Palette::ink(), &[], v(2.0, 3.2))?;
        fade_in_all(stage, &[split], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let rest = kit.text(stage, REST, 20.0, Palette::dim(), v(right, 2.2))?;
        fade_in_all(stage, &[rest], 0.4)?;
        line.wait_sentence(stage, 4)?;
        let result = kit.tex(stage, RESULT_TEX, 36.0, hero(), &[], v(right, 1.1))?;
        fade_in_all(stage, &[result], 0.5)?;

        // 09_2: on a circle the formula is an average
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_2")?;
        fade_out_all(stage, &[split, rest, beyond], 0.3)?;
        let formula = kit.tex(stage, FORMULA_TEX, 34.0, Palette::ink(), &[], v(2.0, 3.2))?;
        let on_circle = kit.tex(stage, ON_CIRCLE_TEX, 28.0, Palette::dim(), &[], v(right, 0.0))?;
        fade_in_all(stage, &[formula, on_circle], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let average_tex = kit.tex(stage, AVERAGE_TEX, 32.0, hero(), &[], v(right, -1.1))?;
        let average = kit.text(stage, AVERAGE, 22.0, Palette::ink(), v(right, -2.0))?;
        fade_in_all(stage, &[average_tex, average], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["Cauchy's integral formula"])?;

        // 09_3: the bump on radius ½: 8 samples, then 16
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_3")?;
        fade_out_all(stage, &[result, on_circle, average_tex, average, search], 0.4)?;
        let vp = values_plane();
        let mut vp_frame = vec![vp.border(stage, output())?];
        vp_frame.extend(vp.axes(stage)?);
        vp_frame.push(kit.text(
            stage,
            VALUES,
            22.0,
            output(),
            v(vp.frame.center[0], vp.frame.top() + 0.25),
        )?);
        let peak = fact_f64("bump.peak")?;
        vp_frame.push(dot(stage, vp.p(C::real(peak)), 0.05, Palette::dim()));
        vp_frame.push(kit.text(
            stage,
            fact("bump.peak")?,
            20.0,
            Palette::dim(),
            vp.p(C::new(peak, -0.12)),
        )?);
        fade_in_all(stage, &vp_frame, 0.4)?;
        line.wait_sentence(stage, 1)?;
        let [few, many] = model::AVERAGE_SAMPLES;
        let counts: Vec<&str> = fact("cauchy.samples")?.split(',').collect();
        let (dots8, mean8) = sample_round(stage, &p, &vp, 0.5, few)?;
        let avg8 = kit.text(
            stage,
            &average_says(counts[0], fact("cauchy.r_half.n8")?),
            24.0,
            hero(),
            v(right, -1.4),
        )?;
        fade_in_all(stage, &[avg8], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let mut gone = dots8;
        gone.push(mean8);
        fade_out_all(stage, &gone, 0.3)?;
        let (dots16, mean16) = sample_round(stage, &p, &vp, 0.5, many)?;
        let avg16 = kit.text(
            stage,
            &average_says(counts[1], fact("cauchy.r_half.n16")?),
            24.0,
            hero(),
            v(right, -1.95),
        )?;
        fade_in_all(stage, &[avg16], 0.4)?;

        // 09_4: radius 2: is the average still 1?
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_4")?;
        let mut gone = dots16;
        gone.extend([mean16, avg8, avg16]);
        fade_out_all(stage, &gone, 0.3)?;
        line.wait_sentence(stage, 0)?;
        let two = p.circle(stage, ZERO, 2.0, hero(), 5.0)?;
        fade_in_all(stage, &[two], 0.5)?;
        let (dots2, mean2) = sample_round_hidden(stage, &p, &vp, 2.0, many)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right, -2.0))?;

        // 09_5: zero: each swallowed pole pulls the average down by one half
        let line = say(&kit, stage, "09_5")?;
        fade_out_all(stage, &asked, 0.3)?;
        fade_in_all(stage, &[mean2], 0.4)?;
        let avg2 = kit.text(
            stage,
            &average_says(counts[1], &signed(fact("cauchy.r_two.n16")?)),
            24.0,
            hero(),
            v(right, -1.35),
        )?;
        fade_in_all(stage, &[avg2], 0.4)?;
        let limit = kit.text(
            stage,
            &format!("{LIMIT}{}", fact("cauchy.r_two.limit")?),
            24.0,
            hero(),
            v(right, -1.8),
        )?;
        fade_in_all(stage, &[limit], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let share = signed(fact("cauchy.pole_share")?);
        let mut flash = Vec::new();
        for z in [model::I, -model::I] {
            flash.push(p.circle(stage, z, 0.22, pole(), 5.0)?);
            flash.push(kit.text(stage, &share, 24.0, pole(), p.p(z + C::new(0.62, 0.0)))?);
        }
        fade_in_all(stage, &flash, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let tally = kit.text(
            stage,
            &format!(
                "{SHARES}{} {} {} = {}",
                fact("bump.peak")?,
                share.replace('−', "− "),
                share.replace('−', "− "),
                fact("cauchy.r_two.limit")?
            ),
            22.0,
            Palette::ink(),
            v(right, -2.25),
        )?;
        fade_in_all(stage, &[tally], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let needs = precisely_now(&kit, stage, NEEDS)?;
        wait_until(stage, line.end())?;

        // 09_6: turn the samples back by their angles: the coefficients
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_6")?;
        let mut gone = vec![two, mean2, avg2, limit, tally, needs, formula];
        gone.extend(dots2);
        gone.extend(flash);
        gone.extend(vp_frame);
        fade_out_all(stage, &gone, 0.4)?;
        let coef_tex = kit.tex(stage, COEF_TEX, 32.0, Palette::ink(), &[], v(2.0, 3.25))?;
        let tp = turned_plane();
        let mut tp_frame = vec![tp.border(stage, output())?];
        tp_frame.extend(tp.axes(stage)?);
        fade_in_all(stage, &[coef_tex], 0.4)?;
        fade_in_all(stage, &tp_frame, 0.3)?;
        let n = many;
        let on_circle: Vec<Mob> = samples(0.5, n)
            .iter()
            .map(|&z| dot(stage, p.p(z), 0.05, input()))
            .collect();
        let vals: Vec<Mob> = turned_values(0.5, n, 0.0)
            .iter()
            .map(|&w| dot(stage, tp.p(w), 0.06, output()))
            .collect();
        let mean = stage.arena_mut().add(
            Circle::new()
                .radius(0.12)
                .style(line_style(hero(), 4.0))
                .build()
                .moved_to(tp.p(centroid(&turned_values(0.5, n, 0.0)))),
        );
        fade_in_all(stage, &on_circle, 0.3)?;
        fade_in_all(stage, &vals, 0.3)?;
        fade_in_all(stage, &[mean], 0.3)?;
        let s = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let moving = vals.clone();
        stage.add_updater(
            anchor,
            move |a, _| {
                let sv = value(a, s);
                let ws = turned_values(0.5, n, sv);
                for (m, w) in moving.iter().zip(&ws) {
                    a.move_to(*m, tp.p(*w), ORIGIN);
                }
                a.move_to(mean, tp.p(centroid(&ws)), ORIGIN);
            },
            true,
        )?;
        line.wait_sentence(stage, 1)?;
        let c: Vec<&str> = fact("cauchy.coef")?.split(',').collect();
        let turned_once = kit.text(stage, TURNED[0], 20.0, Palette::dim(), v(right, -1.95))?;
        fade_in_all(stage, &[turned_once], 0.3)?;
        stage.play(tween_linear(s, 1.0, 2.2)?)?;
        line.wait_sentence(stage, 2)?;
        let read1 = kit.tex(
            stage,
            &format!("{}{}", READ[0], c[1]),
            26.0,
            hero(),
            &[],
            v(right, -2.55),
        )?;
        fade_in_all(stage, &[read1], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let turned_twice = kit.text(stage, TURNED[1], 20.0, Palette::dim(), v(right, -1.95))?;
        fade_out_all(stage, &[turned_once], 0.2)?;
        fade_in_all(stage, &[turned_twice], 0.3)?;
        stage.play(tween_linear(s, 2.0, 2.2)?)?;
        let read2 = kit.tex(
            stage,
            &format!("{}{}", READ[1], c[2]),
            26.0,
            hero(),
            &[],
            v(right, -3.4),
        )?;
        fade_in_all(stage, &[read2], 0.4)?;
        stage.clear_updaters(anchor, true);
        line.wait_sentence(stage, 4)?;
        let list = kit.tex(
            stage,
            &format!(
                r"c_0 \approx {},\; c_1 \approx {},\; c_2 \approx {},\; c_3 \approx {},\; c_4 \approx {}",
                c[0], c[1], c[2], c[3], c[4]
            ),
            26.0,
            Palette::ink(),
            &[],
            v(-3.3, -3.45),
        )?;
        fade_in_all(stage, &[list], 0.5)?;
        wait_until(stage, line.end())?;

        // 09_7: the opening puzzle answered: holomorphic out to the poles, radius 1
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_7")?;
        let mut gone = vec![coef_tex, mean, read1, read2, turned_twice, list, half];
        gone.extend(on_circle);
        gone.extend(vals);
        gone.extend(tp_frame);
        fade_out_all(stage, &gone, 0.4)?;
        let g = graph();
        let mut axes_g = vec![
            stage.arena_mut().add(
                polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
                    .style(faded_line(Palette::dim(), 2.0, 0.9))
                    .build(),
            ),
            stage.arena_mut().add(
                polyline(vec![g.p(0.0, g.y.0), g.p(0.0, g.y.1)])
                    .style(faded_line(Palette::dim(), 2.0, 0.9))
                    .build(),
            ),
        ];
        for piece in clipped(&g, model::bump) {
            axes_g.push(
                stage
                    .arena_mut()
                    .add(polyline(piece).style(line_style(Palette::ink(), 4.0)).build()),
            );
        }
        fade_in_all(stage, &axes_g, 0.5)?;
        let recipe0: Vec<Mob> = clipped(&g, |x| model::bump_recipe0(x, 40))
            .into_iter()
            .map(|pc| {
                stage
                    .arena_mut()
                    .add(polyline(pc).style(line_style(hero(), 3.0)).build())
            })
            .collect();
        fade_in_all(stage, &recipe0, 0.5)?;
        line.wait_sentence(stage, 2)?;
        let unit = p.circle(stage, ZERO, fact_f64("bump.radius0")?, hero(), 5.0)?;
        fade_in_all(stage, &[unit], 0.6)?;
        line.wait_sentence(stage, 3)?;
        let glow0 = stage.arena_mut().add(
            polyline(vec![g.p(-1.0, 0.0), g.p(1.0, 0.0)])
                .style(faded_line(hero(), 12.0, 0.55))
                .build(),
        );
        fade_in_all(stage, &[glow0], 0.5)?;

        // 09_8: recentre at 1: how far now?
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_8")?;
        fade_out_all(stage, &recipe0, 0.3)?;
        fade_out_all(stage, &[glow0], 0.3)?;
        let slide = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let t = value(a, slide);
                let c = C::real(t);
                let r = (1.0 - t).max(1e-3);
                a.move_to(centre, p.p(c), ORIGIN);
                a.move_to(a_name, p.p(c + C::new(0.18, -0.2)), ORIGIN);
                let pts: Vec<Vec3> = (0..=120)
                    .map(|k| p.p(c + C::polar(r, TAU * f64::from(k) / 120.0)))
                    .collect();
                let _ = a.set_points(unit, polyline(pts).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(slide, 1.0, 1.8)?)?;
        stage.clear_updaters(anchor, true);
        let asked = predict(&kit, stage, &line, &ASK_RECENTRE, v(3.9, 2.2))?;

        // 09_9: radius √2 through ±i; the recipe at 1 hugs the bump on (−0.41, 2.41)
        let line = say(&kit, stage, "09_9")?;
        fade_out_all(stage, &asked, 0.3)?;
        let radius = fact_f64("recentre1.radius")?;
        let grow = stage.add_value_tracker(1e-3);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, grow).max(1e-3);
                let pts: Vec<Vec3> = (0..=160)
                    .map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))
                    .collect();
                let _ = a.set_points(unit, polyline(pts).build().points());
            },
            true,
        )?;
        let rt = line.until(stage, "So the new recipe", 1.6)?;
        stage.play(tween_linear(grow, radius, rt)?)?;
        stage.clear_updaters(anchor, true);
        let (left, right_end) = (fact_f64("recentre1.left")?, fact_f64("recentre1.right")?);
        let glow1 = stage.arena_mut().add(
            polyline(vec![g.p(left, 0.0), g.p(right_end, 0.0)])
                .style(faded_line(hero(), 12.0, 0.55))
                .build(),
        );
        let works = kit.text(
            stage,
            &format!(
                "{WORKS}({}, {})",
                signed(fact("recentre1.left")?),
                fact("recentre1.right")?
            ),
            24.0,
            hero(),
            v(3.9, 1.45),
        )?;
        let recipe1: Vec<Mob> = clipped(&g, |x| model::bump_recipe(1.0, x, model::RECENTRE_CHECK_TERMS))
            .into_iter()
            .map(|pc| {
                stage
                    .arena_mut()
                    .add(polyline(pc).style(line_style(hero(), 3.0)).build())
            })
            .collect();
        fade_in_all(stage, &recipe1, 0.6)?;
        fade_in_all(stage, &[glow1, works], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let terms = fact("recentre1.check.terms")?;
        let points: Vec<&str> = fact("recentre1.check.at")?.split(',').collect();
        let check = [
            kit.text(
                stage,
                &format!("{terms}{CHECK}{}{COLON}{}", points[0], fact("recentre1.check.at2")?),
                22.0,
                Palette::ink(),
                v(3.9, 2.55),
            )?,
            kit.text(
                stage,
                &format!("{terms}{CHECK}{}{COLON}{}", points[1], fact("recentre1.check.at2_6")?),
                22.0,
                Palette::ink(),
                v(3.9, 2.0),
            )?,
        ];
        fade_in_all(stage, &check, 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![border, centre, a_name, unit, glow1, works, check[0], check[1]];
        all.extend(poles);
        all.extend(axes_g);
        all.extend(recipe1);
        fade_out_all(stage, &all, 0.5)?;
        painter.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

/// `n` samples on the circle of radius `r`: dots on the picture, their values
/// in `vp`, and the centroid ring. Returns the dots and the ring.
fn sample_round(stage: &mut Stage<'_>, p: &Plane, vp: &Plane, r: f64, n: usize) -> fmn::Result<(Vec<Mob>, Mob)> {
    let (dots, mean) = sample_round_hidden(stage, p, vp, r, n)?;
    pause(stage, 0.3)?;
    fade_in_all(stage, &[mean], 0.4)?;
    Ok((dots, mean))
}

/// As [`sample_round`], but the centroid ring is built and left hidden.
fn sample_round_hidden(stage: &mut Stage<'_>, p: &Plane, vp: &Plane, r: f64, n: usize) -> fmn::Result<(Vec<Mob>, Mob)> {
    let zs = samples(r, n);
    let ws: Vec<C> = zs.iter().map(|&z| bump_at(z)).collect();
    let mut dots: Vec<Mob> = zs.iter().map(|&z| dot(stage, p.p(z), 0.06, input())).collect();
    let shown: Vec<Mob> = ws.iter().map(|&w| dot(stage, vp.p(w), 0.06, output())).collect();
    fade_in_all(stage, &dots, 0.4)?;
    fade_in_all(stage, &shown, 0.4)?;
    dots.extend(shown);
    let mean = stage.arena_mut().add(
        Circle::new()
            .radius(0.12)
            .style(line_style(hero(), 4.0))
            .build()
            .moved_to(vp.p(centroid(&ws))),
    );
    Ok((dots, mean))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(key: &str) -> f64 {
        crate::facts::facts().unwrap().get(key).unwrap().parse().unwrap()
    }

    #[test]
    fn the_centroids_drawn_are_the_averages_said() {
        let at = |r: f64, n: usize| centroid(&samples(r, n).iter().map(|&z| bump_at(z)).collect::<Vec<_>>());
        let [few, many] = model::AVERAGE_SAMPLES;
        assert!((at(0.5, few).re - num("cauchy.r_half.n8")).abs() < 5e-5);
        assert!((at(0.5, many).re - num("cauchy.r_half.n16")).abs() < 5e-7);
        assert!((at(2.0, many).re - num("cauchy.r_two.n16")).abs() < 5e-7);
        assert!(at(0.5, few).im.abs() < 1e-12 && at(2.0, many).im.abs() < 1e-12);
        // the limit said is what more samples close in on
        assert!((at(2.0, 256).re - num("cauchy.r_two.limit")).abs() < 1e-9);
    }

    #[test]
    fn the_captions_keep_their_words_around_the_facts() {
        assert_eq!(average_says("8", "1.0039"), "average of 8: 1.0039");
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("cauchy.samples"), Some("8,16"));
        assert_eq!(f.get("recentre1.check.terms"), Some("60"));
        assert_eq!(f.get("recentre1.check.at"), Some("2,2.6"));
        assert_eq!(f.get("cauchy.r_two.limit"), Some("0"));
    }

    #[test]
    fn turning_back_reads_the_coefficients() {
        let c: Vec<f64> = crate::facts::facts()
            .unwrap()
            .get("cauchy.coef")
            .unwrap()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        for (s, k) in [(0.0, 0usize), (1.0, 1), (2.0, 2)] {
            let mean = centroid(&turned_values(0.5, model::AVERAGE_SAMPLES[1], s));
            assert!((mean.re / 0.5f64.powi(k as i32) - c[k]).abs() < 1e-3, "c{k}");
        }
    }

    #[test]
    fn every_point_drawn_stays_in_its_window() {
        let (p, vp, tp) = (picture(), values_plane(), turned_plane());
        for r in [0.5, 2.0] {
            for z in samples(r, model::AVERAGE_SAMPLES[1]) {
                assert!(p.contains(z) && vp.contains(bump_at(z)), "radius {r}");
            }
        }
        for k in 0..=20 {
            for w in turned_values(0.5, model::AVERAGE_SAMPLES[1], f64::from(k) / 10.0) {
                assert!(tp.contains(w));
            }
        }
        // the recentred circle fits the picture and the graph shows its interval
        let r1 = num("recentre1.radius");
        assert!(1.0 + r1 < p.frame.x.1 && 1.0 - r1 > p.frame.x.0);
        let g = graph();
        assert!(g.x.0 < num("recentre1.left") && g.x.1 > num("recentre1.right"));
        for piece in clipped(&g, |x| model::bump_recipe(1.0, x, model::RECENTRE_CHECK_TERMS)) {
            for q in piece {
                assert!(q[1] >= g.bottom() - 1e-9 && q[1] <= g.top() + 1e-9);
            }
        }
        // the right column clears the picture
        assert!(p.frame.right() < vp.frame.left() && p.frame.right() < tp.frame.left());
        assert!(p.frame.right() < g.left());
    }
}
