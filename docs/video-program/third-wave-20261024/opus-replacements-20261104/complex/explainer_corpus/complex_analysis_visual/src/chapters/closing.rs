//! 15_closing: back to the bump, the one demand and what it bought, and the
//! closing question with its answer (storyboard 15_1–15_4).
//!
//! - Chapter 01's graph and degree-40 recipe; the plane rises behind it with
//!   the poles at ±i and the radius 1 circle.
//! - A strip of five small pictures (06, 09, 11, 13, 14), each lit as it is
//!   named; the first says "on an open region", because one point's
//!   derivative buys nothing.
//! - The prediction: recentre the recipe at 3; the answer is the circle
//!   through ±i, radius √10, and its interval on the real line. Title card.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Plane, dashed, dot, fact, fact_f64, fact_list, fade_in_all, fade_out_all, faded_line, hero, input, line_style,
    output, pole, pole_mark, predict, restroke, say, shift_all, tween_linear, value, wait_until, warn, zero_mark,
};
// the derivative's quotient from 06; the bump's formula, pole names and recipe from 01
use super::one_derivative::QUOTIENT_TEX;
use super::the_bump::{AT_X, BUMP_TEX, POLE_I, POLE_MINUS_I, graph, recipe_points};
use crate::model::{self, C, ZERO};

pub const DEMAND: &str = "the same from every direction";
pub const CAPTIONS: [[&str; 2]; 5] = [
    ["on an open region:", "stretch and turn"],
    ["the boundary", "decides the inside"],
    ["a loop counts", "zeros − poles"],
    ["real integrals", "from residues"],
    ["at most one extension;", "paths can branch"],
];
pub const ASK_PREFIX: &str = "radius at ";
pub const ROOT: &str = "√";
pub const ABOUT: &str = " ≈ ";
pub const TITLE_CARD: &str = "The Hidden Geometry of Complex Functions";

/// The recipe of 15_1 is chapter 01's last: degree 40, as half-degree k.
pub const RECIPE_K: f64 = 20.0;

/// The recap strip: five panels in a row.
pub const STRIP_X: [f64; 5] = [-5.3, -2.65, 0.0, 2.65, 5.3];
pub const STRIP_Y: f64 = -0.2;
pub const PANEL_W: f64 = 2.3;

/// Panel `j`'s plane (the residue panel looks at the upper half plane).
pub fn panel(j: usize) -> Plane {
    let y = if j == 3 { (-0.5, 2.1) } else { (-1.3, 1.3) };
    Plane::new(v(STRIP_X[j], STRIP_Y), PANEL_W, (-1.3, 1.3), y)
}

/// The stretch and turn of the first panel: multiplication by 2 + i.
pub const STRETCH_TURN: C = C::new(2.0, 1.0);
pub const PIECE: f64 = 0.5;
pub const PIECE_AT: C = C::new(-1.15, -1.1);
pub const IMAGE_AT: C = C::new(0.2, -0.5);

/// A small square of side `PIECE` at `at`, anticlockwise, closed, carried by `w ↦ base + m·w`.
pub fn piece(base: C, m: C) -> [C; 5] {
    [ZERO, C::real(PIECE), C::new(PIECE, PIECE), C::new(0.0, PIECE), ZERO].map(|d| base + m * d)
}

/// The illustrative zeros and pole inside the counting panel's loop.
pub const COUNT_ZEROS: [C; 2] = [C::new(0.45, 0.2), C::new(-0.35, 0.45)];
pub const COUNT_POLE: C = C::new(-0.1, -0.5);

/// The closing plane (15_3–15_4): the recipe at 3 and its circle through ±i.
pub fn closing_plane() -> Plane {
    Plane::new(v(-1.6, -0.2), 7.6, (-1.2, 6.8), (-3.4, 3.4))
}
pub const ASK_AT: (f64, f64) = (4.7, 1.2);

/// A text minus sign for a fact printed with an ASCII hyphen.
fn minus(s: &str) -> String {
    s.replacen('-', "−", 1)
}

/// The circle |z − centre| = r in `p`, closed.
fn ring_points(p: &Plane, centre: C, r: f64) -> Vec<Vec3> {
    let mut zs = model::circle(centre, r.max(1e-3), 180);
    zs.push(zs[0]);
    zs.iter().map(|&z| p.p(z)).collect()
}

/// The small picture of panel `j`, detached.
fn panel_picture(stage: &mut Stage<'_>, j: usize) -> fmn::Result<Vec<Mob>> {
    let p = panel(j);
    let mut out = Vec::new();
    match j {
        // 06: a small square, stretched and turned, still a square
        0 => {
            out.push(p.path(stage, &piece(PIECE_AT, C::real(1.0)), input(), 3.0)?);
            out.push(p.path(stage, &piece(IMAGE_AT, STRETCH_TURN), output(), 3.0)?);
            let carry = Arrow::new(p.p(C::new(-0.55, -0.85)), p.p(C::new(0.05, -0.6)))
                .thickness(3.0)
                .color(Palette::dim())
                .build()?;
            out.push(stage.arena_mut().add(carry));
        }
        // 09: the values on the circle decide the centre
        1 => {
            out.push(p.circle(stage, ZERO, 1.0, hero(), 3.0)?);
            for k in 0..12 {
                let z = C::polar(1.0, TAU * f64::from(k) / 12.0);
                let spoke = polyline(vec![p.p(z), p.p(ZERO)])
                    .style(faded_line(Palette::dim(), 1.5, 0.6))
                    .build();
                out.push(stage.arena_mut().add(spoke));
                out.push(dot(stage, p.p(z), 0.05, output()));
            }
            out.push(dot(stage, p.p(ZERO), 0.08, hero()));
        }
        // 11: a loop around zeros and a pole
        2 => {
            out.push(p.circle(stage, ZERO, 1.0, hero(), 3.0)?);
            let along = Arrow::new(p.p(C::polar(1.0, 0.45)), p.p(C::polar(1.0, 0.85)))
                .thickness(3.0)
                .color(hero())
                .build()?;
            out.push(stage.arena_mut().add(along));
            for z in COUNT_ZEROS {
                out.extend(zero_mark(stage, p.p(z)));
            }
            out.extend(pole_mark(stage, p.p(COUNT_POLE)));
        }
        // 13: the half-disk contour over the real line, the upper poles inside
        3 => {
            let r = 1.1;
            let mut half: Vec<C> = (0..=60).map(|i| C::polar(r, PI * f64::from(i) / 60.0)).collect();
            half.push(C::real(r));
            out.push(p.path(stage, &half, hero(), 3.0)?);
            out.push(p.path(stage, &[C::real(-1.25), C::real(1.25)], Palette::ink(), 5.0)?);
            for q in model::quartic_roots().into_iter().filter(|q| q.im > 0.0) {
                out.extend(pole_mark(stage, p.p(q)));
            }
        }
        // 14: disks carried round 0 two ways end on two different values
        _ => {
            let r = 0.85;
            let arc =
                |sign: f64| -> Vec<C> { (0..=40).map(|i| C::polar(r, sign * PI * f64::from(i) / 40.0)).collect() };
            out.push(p.path(stage, &arc(1.0), input(), 3.0)?);
            out.push(p.path(stage, &arc(-1.0), output(), 3.0)?);
            for k in 1..4 {
                let a = PI * f64::from(k) / 4.0;
                out.push(p.circle(stage, C::polar(r, a), 0.3, Palette::faint(), 1.5)?);
                out.push(p.circle(stage, C::polar(r, -a), 0.3, Palette::faint(), 1.5)?);
            }
            out.push(dot(stage, p.p(ZERO), 0.07, warn()));
            out.push(dot(stage, p.p(C::real(r)), 0.07, Palette::ink()));
            out.push(dot(stage, p.p(C::new(-r, 0.14)), 0.07, input()));
            out.push(dot(stage, p.p(C::new(-r, -0.14)), 0.07, output()));
        }
    }
    Ok(out)
}

pub struct Closing {
    pub kit: Kit,
}

impl SceneConstruct for Closing {
    fn name(&self) -> &str {
        "15_closing"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let g = graph();

        // 15_1: chapter 01's bump and recipe; the plane rises with the poles
        let line = say(&kit, stage, "15_1")?;
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
        let bump: Vec<Vec3> = (0..=360)
            .map(|i| {
                let x = g.x.0 + (g.x.1 - g.x.0) * f64::from(i) / 360.0;
                g.p(x, model::bump(x))
            })
            .collect();
        let curve = stage
            .arena_mut()
            .add(polyline(bump).style(line_style(Palette::ink(), 5.0)).build());
        let recipe = stage.arena_mut().add(
            polyline(recipe_points(&g, RECIPE_K))
                .style(line_style(hero(), 4.0))
                .build(),
        );
        let guide = |x: f64| -> fmn::Result<VMobject> {
            DashedLine::new(g.p(x, g.y.0), g.p(x, g.y.1))
                .dash_length(0.09)
                .style(faded_line(Palette::dim(), 2.0, 0.8))
                .build()
        };
        let guides = [stage.arena_mut().add(guide(-1.0)?), stage.arena_mut().add(guide(1.0)?)];
        let formula = kit.tex(stage, BUMP_TEX, 36.0, Palette::ink(), &[], v(-0.6, 3.4))?;
        fade_in_all(stage, &[x_axis, y_axis, curve, formula], 0.6)?;
        fade_in_all(stage, &[recipe], 0.5)?;
        fade_in_all(stage, &guides, 0.4)?;
        line.wait_sentence(stage, 1)?;
        for m in guides {
            restroke(stage, m, hero(), 1.0);
        }
        line.wait_for(stage, "because of two points")?;
        // as in 01_5: the graph's real axis becomes the plane's, same scale
        let y0 = g.p(0.0, 0.0)[1];
        let plane = Plane::new(v(g.center[0], y0 + 0.4), g.width, (-3.0, 3.0), (-1.9, 2.1));
        let lift = plane.p(ZERO)[1] - y0;
        shift_all(
            stage,
            &[x_axis, y_axis, curve, recipe, guides[0], guides[1]],
            0.0,
            lift,
            0.8,
        )?;
        restroke(stage, curve, Palette::ink(), 0.35);
        let vertical = stage.arena_mut().add(
            polyline(vec![plane.p(C::new(0.0, -1.6)), plane.p(C::new(0.0, 1.8))])
                .style(faded_line(input(), 2.0, 0.7))
                .build(),
        );
        fade_in_all(stage, &[vertical], 0.5)?;
        line.wait_for(stage, "at i and minus i")?;
        let poles: Vec<f64> = fact_list("bump.poles")?;
        let mut pole_mobs = Vec::new();
        for &im in &poles {
            pole_mobs.extend(pole_mark(stage, plane.p(C::new(0.0, im))));
        }
        pole_mobs.push(kit.text(stage, POLE_I, 28.0, pole(), plane.p(C::new(0.3, poles[0])))?);
        pole_mobs.push(kit.text(stage, POLE_MINUS_I, 28.0, pole(), plane.p(C::new(0.35, poles[1])))?);
        fade_in_all(stage, &pole_mobs, 0.5)?;
        let r0 = fact_f64("bump.radius0")?;
        let ring = stage.arena_mut().add(
            polyline(ring_points(&plane, ZERO, r0))
                .style(line_style(hero(), 5.0))
                .build(),
        );
        fade_in_all(stage, &[ring], 0.5)?;
        wait_until(stage, line.end())?;

        // 15_2: one demand, and five things it bought, each lit as named
        kit.hold(stage)?;
        let line = say(&kit, stage, "15_2")?;
        let mut gone = vec![x_axis, y_axis, curve, recipe, formula, vertical, ring];
        gone.extend(guides);
        gone.extend(&pole_mobs);
        fade_out_all(stage, &gone, 0.5)?;
        let borders: Vec<Mob> = (0..5)
            .map(|j| panel(j).border(stage, Palette::faint()))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &borders, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let quotient = kit.tex(stage, QUOTIENT_TEX, 32.0, Palette::ink(), &[], v(-2.4, 2.3))?;
        let demand = kit.text(stage, DEMAND, 22.0, hero(), v(2.4, 2.3))?;
        fade_in_all(stage, &[quotient], 0.4)?;
        fade_in_all(stage, &[demand], 0.4)?;
        let mut strip = Vec::new();
        for (j, (&x, &border)) in STRIP_X.iter().zip(&borders).enumerate() {
            match j {
                0 => line.wait_sentence(stage, 2)?,
                1 => line.wait_sentence(stage, 3)?,
                2 => line.wait_sentence(stage, 4)?,
                3 => line.wait_for(stage, "and it turns a real integral")?,
                _ => line.wait_sentence(stage, 5)?,
            }
            let picture = panel_picture(stage, j)?;
            let [first, second] = CAPTIONS[j];
            let caption = [
                kit.text(stage, first, 18.0, Palette::ink(), v(x, -1.7))?,
                kit.text(stage, second, 18.0, Palette::ink(), v(x, -2.05))?,
            ];
            restroke(stage, border, hero(), 1.0);
            fade_in_all(stage, &picture, 0.5)?;
            fade_in_all(stage, &caption, 0.4)?;
            strip.extend(picture);
            strip.extend(caption);
        }
        wait_until(stage, line.end())?;

        // 15_3: recentre the recipe at 3: how far will it reach?
        kit.hold(stage)?;
        let line = say(&kit, stage, "15_3")?;
        let mut gone = vec![quotient, demand];
        gone.extend(&borders);
        gone.extend(&strip);
        fade_out_all(stage, &gone, 0.5)?;
        let cp = closing_plane();
        let mut frame = vec![cp.border(stage, Palette::faint())?];
        frame.extend(cp.axes(stage)?);
        let mut poles_here = Vec::new();
        for &im in &poles {
            poles_here.extend(pole_mark(stage, cp.p(C::new(0.0, im))));
        }
        poles_here.push(kit.text(stage, POLE_I, 28.0, pole(), cp.p(C::new(0.4, poles[0])))?);
        poles_here.push(kit.text(stage, POLE_MINUS_I, 28.0, pole(), cp.p(C::new(0.45, poles[1])))?);
        let old = stage.arena_mut().add(
            polyline(ring_points(&cp, ZERO, r0))
                .style(faded_line(hero(), 3.0, 0.45))
                .build(),
        );
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &poles_here, 0.4)?;
        fade_in_all(stage, &[old], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let a = fact_f64("recentre3.centre")?;
        let centre = dot(stage, cp.p(C::real(a)), 0.09, hero());
        let centre_name = kit.text(
            stage,
            &format!("{AT_X}{}", fact("recentre3.centre")?),
            24.0,
            hero(),
            cp.p(C::new(a, -0.45)),
        )?;
        fade_in_all(stage, &[centre, centre_name], 0.4)?;
        let ask = format!("{ASK_PREFIX}{}?", fact("recentre3.centre")?);
        let asked = predict(&kit, stage, &line, &[ask.as_str()], v(ASK_AT.0, ASK_AT.1))?;

        // 15_4: to the poles, √10 away: the circle through ±i and its interval
        let line = say(&kit, stage, "15_4")?;
        fade_out_all(stage, &asked, 0.3)?;
        let r3 = fact_f64("recentre3.radius")?;
        let spokes: Vec<Mob> = poles
            .iter()
            .map(|&im| dashed(stage, cp.p(C::real(a)), cp.p(C::new(0.0, im)), hero()))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &spokes, 0.5)?;
        line.wait_for(stage, "the square root of ten")?;
        let says = kit.text(
            stage,
            &format!(
                "{ROOT}{}{ABOUT}{}",
                fact("recentre3.radius_sq")?,
                fact("recentre3.radius")?
            ),
            24.0,
            hero(),
            cp.p(radius_label_at(a, poles[0])),
        )?;
        fade_in_all(stage, &[says], 0.4)?;
        let grown = stage.add_value_tracker(0.02);
        let ring = stage.arena_mut().add(
            polyline(ring_points(&cp, C::real(a), 0.02))
                .style(line_style(hero(), 5.0))
                .build(),
        );
        stage.add_to_scene(ring)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let r = value(s, grown);
                let _ = s.set_points(ring, polyline(ring_points(&cp, C::real(a), r)).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(grown, r3, 1.4)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_sentence(stage, 1)?;
        let ends: Vec<f64> = fact_list("recentre3.ends")?;
        let printed: Vec<&str> = fact("recentre3.ends")?.split(',').collect();
        let reach = cp.path(stage, &[C::real(ends[0]), C::real(ends[1])], hero(), 10.0)?;
        let mut end_marks = Vec::new();
        for (k, &x) in ends.iter().enumerate() {
            end_marks.push(cp.path(stage, &[C::new(x, -0.15), C::new(x, 0.15)], hero(), 4.0)?);
            end_marks.push(kit.text(stage, &minus(printed[k]), 22.0, hero(), cp.p(C::new(x, -0.45)))?);
        }
        fade_in_all(stage, &[reach], 0.5)?;
        fade_in_all(stage, &end_marks, 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 1.5)?;
        let mut all = vec![old, centre, centre_name, says, ring, reach];
        all.extend(frame);
        all.extend(poles_here);
        all.extend(spokes);
        all.extend(end_marks);
        fade_out_all(stage, &all, 0.8)?;
        // the title card, held over the music
        let title = kit.text(stage, TITLE_CARD, 44.0, Palette::ink(), v(0.0, 0.2))?;
        fade_in_all(stage, &[title], 0.8)?;
        kit.music_breath(stage, 3.0)?;
        fade_out_all(stage, &[title], 0.8)?;
        kit.finish(stage)
    }
}

/// Where "√10 ≈ 3.162" sits: off the middle of the spoke from the centre
/// `a` to the pole at `i·im`, on the side away from the real axis.
pub fn radius_label_at(a: f64, im: f64) -> C {
    let mid = C::new(a / 2.0, im / 2.0);
    let along = C::new(-a, im).scale(1.0 / a.hypot(im));
    let away = along * C::new(0.0, -im.signum());
    mid + away.scale(0.55)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(key: &str) -> &'static str {
        crate::facts::facts().unwrap().get(key).unwrap()
    }

    #[test]
    fn the_answer_is_the_distance_to_the_poles() {
        let a: f64 = get("recentre3.centre").parse().unwrap();
        assert_eq!(a, model::RECENTRE_AT);
        let sq: f64 = get("recentre3.radius_sq").parse().unwrap();
        assert_eq!(sq, a * a + 1.0);
        let r: f64 = get("recentre3.radius").parse().unwrap();
        assert!((r - sq.sqrt()).abs() < 5e-4 && (r - model::bump_radius(a)).abs() < 5e-4);
        let ends: Vec<f64> = get("recentre3.ends").split(',').map(|s| s.parse().unwrap()).collect();
        assert!((ends[0] - (a - r)).abs() < 6e-3 && (ends[1] - (a + r)).abs() < 6e-3);
        // the circle passes through both poles
        for im in [1.0, -1.0] {
            assert!(((C::new(0.0, im) - C::real(a)).abs() - model::bump_radius(a)).abs() < 1e-12);
        }
    }

    #[test]
    fn the_closing_picture_fits() {
        let cp = closing_plane();
        let a = model::RECENTRE_AT;
        let r = model::bump_radius(a);
        for z in [
            C::real(a - r),
            C::real(a + r),
            C::new(a, r),
            C::new(a, -r),
            C::new(0.0, 1.0),
        ] {
            assert!(cp.contains(z));
        }
        let f = cp.frame;
        assert!(f.top() < 3.3 && f.bottom() > -3.8 && f.center[0] - f.width / 2.0 > -6.9);
        // the question card sits right of the plane
        assert!(f.center[0] + f.width / 2.0 < ASK_AT.0 - 1.6);
        // the radius label is inside the window, clear of the circle's centre and the pole
        let at = radius_label_at(a, 1.0);
        assert!(cp.contains(at) && at.im > 0.6 && (at - C::new(0.0, 1.0)).abs() > 1.0);
        // the end labels fit
        assert!(cp.contains(C::new(a - r, -0.45)) && cp.contains(C::new(a + r, -0.45)));
    }

    #[test]
    fn the_strip_panels_fit_and_say_what_they_show() {
        for p in (0..STRIP_X.len()).map(panel) {
            assert!((p.frame.height - PANEL_W).abs() < 1e-12);
            assert!(p.frame.center[0].abs() + PANEL_W / 2.0 < 6.9);
            assert!(p.frame.bottom() > -1.6 && p.frame.top() < 1.2);
        }
        for w in STRIP_X.windows(2) {
            assert!(w[1] - w[0] > PANEL_W + 0.2);
        }
        // the stretch-and-turn picture: a square goes to a square, turned and stretched
        let (src, img) = (piece(PIECE_AT, C::real(1.0)), piece(IMAGE_AT, STRETCH_TURN));
        let p0 = panel(0);
        for z in src.iter().chain(&img) {
            assert!(p0.contains(*z));
        }
        let (s1, s2) = (img[1] - img[0], img[3] - img[0]);
        assert!((s1.re * s2.re + s1.im * s2.im).abs() < 1e-12);
        assert!((s1.abs() - PIECE * STRETCH_TURN.abs()).abs() < 1e-12 && (s1.abs() - s2.abs()).abs() < 1e-12);
        // the counting panel's marks are inside its loop; the residue panel's poles are inside the half disk
        for z in COUNT_ZEROS.iter().chain([&COUNT_POLE]) {
            assert!(z.abs() < 0.85);
        }
        let upper: Vec<C> = model::quartic_roots().into_iter().filter(|q| q.im > 0.0).collect();
        assert_eq!(upper.len(), 2);
        for q in upper {
            assert!(q.abs() < 1.1 && panel(3).contains(q));
        }
    }
}
