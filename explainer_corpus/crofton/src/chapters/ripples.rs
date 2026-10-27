//! One beach changes; every other part, its endpoints and the card persist.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::{placement, sheet, the_longimeter};
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::PI;

pub struct Ripples {
    pub kit: Kit,
}
const NOTE: &str = "Finite rectifiable drawing; count all traversals. Uniform half-turn direction and independent uniform full-gap slide. Rounded lengths are approximations. Equal rounded divider totals do not mean equal walks.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.5 + 0.33 * p[0], 0.5 + 0.33 * p[1])
}
fn path(s: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(pts).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, text: &str, x: f64, y: f64, size: f64) -> fmn::Result<Mob> {
    k.text(s, text, size, Palette::ink(), v(x, y))
}
/// Facts write rational multiples as `7/4pi`; put π in the numerator on screen.
fn display_pi(value: &str) -> String {
    value
        .split(' ')
        .map(|term| match term.strip_suffix("pi") {
            Some(coefficient) => match coefficient.split_once('/') {
                Some((numerator, denominator)) => format!("{numerator}π/{denominator}"),
                None => format!("{coefficient}π"),
            },
            None => term.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
/// Supplied n equal semicircles occupy the original beach's diameter.
pub fn beach_point(c: &model::Curve, n: usize, u: f64) -> model::P2 {
    let seg = c.parts[model::BEACH].seg;
    let a = seg.point(0.);
    let b = seg.point(1.);
    let length = seg.len();
    if n == 0 {
        return seg.point(u);
    }
    let z = (u * n as f64).min(n as f64);
    let j = (z.floor() as usize).min(n - 1);
    let t = z - j as f64;
    let radius = length / (2. * n as f64);
    let along = (2. * j as f64 + 1. - (PI * t).cos()) * radius;
    let up = radius * (PI * t).sin();
    let d = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
    [a[0] + d[0] * along - d[1] * up, a[1] + d[1] * along + d[0] * up]
}
fn blend(c: &model::Curve, state: f64, u: f64) -> model::P2 {
    let (a, b, t) = if state <= 1. {
        (0, 7, state)
    } else {
        (7, 14, state - 1.)
    };
    let p = beach_point(c, a, u);
    let q = beach_point(c, b, u);
    [p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]
}
fn walk(s: &mut Stage<'_>, c: &model::Curve, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let w = c.walk(2.);
    let mut pts: Vec<_> = w.landings.iter().map(|x| physical(x.0)).collect();
    pts.push(physical(c.ends().1));
    let m = path(s, pts, color, 2.5);
    fade_in_all(s, &[m], 0.5)?;
    let mut out = vec![m];
    for &(p, _) in &w.landings {
        let dot = s.arena_mut().add(
            Circle::new()
                .radius(0.042)
                .arc_center(physical(p))
                .style(flat(color, 1.))
                .build(),
        );
        s.add_to_scene(dot)?;
        out.push(dot);
    }
    Ok(out)
}
impl SceneConstruct for Ripples {
    fn name(&self) -> &str {
        "26_ripples"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let c = model::coast();
        let r = sheet::rippled(&c, 7).map_err(fail)?;
        let f = facts::facts().map_err(fail)?;
        let beach_exact = display_pi(f.get("scale.beach_exact").unwrap());
        let length_exact = display_pi(f.get("scale.rippled_length_exact").unwrap());
        let rounded_smooth = f.get("dividers.2_0.length").unwrap();
        let rounded_rippled = f.get("dividers.2_0.rippled_length").unwrap();
        // The shared rounded label is valid only while these two walks round alike.
        assert_eq!(rounded_smooth, rounded_rippled);
        let title = label(&k, s, "Which drawing did we measure?", 0., 3.45, 35.)?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * 0.33)
                .height(model::CARD_H * 0.33)
                .style(line_style(Palette::dim(), 1.5))
                .build()?
                .moved_to(v(-3.5, 0.5)),
        );
        let mut rest = vec![title, card];
        for (i, p) in c.parts.iter().enumerate() {
            if i != model::BEACH {
                rest.push(path(
                    s,
                    (0..=120).map(|j| physical(p.seg.point(j as f64 / 120.))).collect(),
                    palette::coast(),
                    3.,
                ));
            }
        }
        let a = c.parts[model::BEACH].seg.point(0.);
        let b = c.parts[model::BEACH].seg.point(1.);
        let zoom = move |p: model::P2| v(0.75 + 1.4 * (p[0] - a[0]), 1.25 + 1.4 * (p[1] - a[1]));
        let beach = path(s, vec![physical(a), physical(b)], Palette::gold(), 3.5);
        let large = path(s, vec![zoom(a), zoom(b)], Palette::gold(), 4.);
        let zoom_label = label(&k, s, "same beach · enlarged", 3.2, 2.5, 25.)?;
        rest.extend([beach, large, zoom_label]);
        let state = s.add_value_tracker(0.);
        let driver = s.arena_mut().add(VMobject::new());
        let copy = c.clone();
        let mut previous = None;
        s.add_updater(
            driver,
            move |s, _| {
                let t = s.tracker_value(state).unwrap_or(0.);
                if previous == Some(t) {
                    return;
                }
                previous = Some(t);
                let pts: Vec<_> = (0..=560).map(|j| blend(&copy, t, j as f64 / 560.)).collect();
                let _ = s.set_points(
                    beach,
                    polyline(pts.iter().copied().map(physical).collect::<Vec<_>>())
                        .build()
                        .points(),
                );
                let _ = s.set_points(
                    large,
                    polyline(pts.into_iter().map(zoom).collect::<Vec<_>>()).build().points(),
                );
            },
            true,
        )?;
        s.add_to_scene(driver)?;
        let spoken = say(&k, s, "26_1")?;
        fade_in_all(s, &rest, 0.5)?;
        let baseline = path(s, vec![zoom(a), zoom(b)], Palette::dim(), 1.5);
        fade_in_all(s, &[baseline], 0.3)?;
        let caption = label(
            &k,
            s,
            &format!("fixed endpoints · {} cm apart", f.get("scale.beach").unwrap()),
            3.2,
            0.65,
            25.,
        )?;
        fade_in_all(s, &[caption], 0.3)?;
        spoken.wait_for(s, "seven semicircles")?;
        s.play(placement::glide(state, 1., 3., linear)?)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "26_2")?;
        let mut explanation = vec![
            label(
                &k,
                s,
                &format!(
                    "{} × π × {} = {beach_exact} cm",
                    f.get("scale.ripples").unwrap(),
                    f.get("scale.radius").unwrap()
                ),
                3.2,
                -0.1,
                28.,
            )?,
            label(
                &k,
                s,
                &format!("beach length ≈ {} cm", f.get("scale.rippled_beach").unwrap()),
                3.2,
                -0.75,
                28.,
            )?,
            label(&k, s, "arc / diameter = π/2", 3.2, -1.4, 27.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "26_3")?;
        for &mob in &explanation {
            s.set_family_opacity(mob, 0.);
        }
        s.play(placement::glide(state, 2., 3., linear)?)?;
        fade_out_all(s, &explanation, 0.3)?;
        explanation = vec![
            label(
                &k,
                s,
                &format!(
                    "{} × π × {} = {beach_exact} cm",
                    f.get("scale.ripples_fine").unwrap(),
                    f.get("scale.fine_radius").unwrap()
                ),
                3.2,
                -0.1,
                28.,
            )?,
            label(&k, s, "same total · smaller semicircles", 3.2, -0.8, 25.)?,
            label(&k, s, "replacement, not ripples inside ripples", 3.2, -1.45, 21.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        for &mob in &explanation {
            s.set_family_opacity(mob, 0.);
        }
        s.play(placement::glide(state, 1., 2., linear)?)?;
        say(&k, s, "26_4")?;
        fade_out_all(s, &explanation, 0.3)?;
        let original = walk(s, &c, Palette::dim())?;
        let changed = walk(s, &r, Palette::red())?;
        let table = vec![
            label(&k, s, "2 cm dividers, including last short chord", 3.2, 0.05, 21.)?,
            label(
                &k,
                s,
                &format!("smooth: ≈ {} cm", f.get("scale.divider_original_4dp").unwrap()),
                3.2,
                -0.6,
                27.,
            )?,
            label(
                &k,
                s,
                &format!("rippled: ≈ {} cm", f.get("scale.divider_rippled_4dp").unwrap()),
                3.2,
                -1.25,
                27.,
            )?,
            label(&k, s, &format!("both round to {rounded_smooth} cm"), 3.2, -1.9, 25.)?,
            label(&k, s, "gray: smooth walk · red: rippled walk", -3.5, -1.4, 21.)?,
        ];
        fade_in_all(s, &table, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "26_5")?;
        let question = label(&k, s, "Will the sheet recover the added arc length?", 0., -2.55, 27.)?;
        fade_in_all(s, &[question], 0.3)?;
        let thinking = think(&k, s, v(-3.5, -2.3))?;
        say(&k, s, "26_6")?;
        fade_out_all(s, &[thinking, question], 0.3)?;
        fade_out_all(s, &table, 0.3)?;
        fade_out_all(s, &original, 0.3)?;
        fade_out_all(s, &changed, 0.3)?;
        let identity = vec![
            label(
                &k,
                s,
                &format!("E[count] = L/π ≈ {}", f.get("scale.rippled_mean_crossings").unwrap()),
                3.2,
                -0.1,
                28.,
            )?,
            label(
                &k,
                s,
                &format!("π × E[count] = L ≈ {} cm", f.get("scale.rippled_length").unwrap()),
                3.2,
                -0.8,
                27.,
            )?,
            label(&k, s, &format!("L = {length_exact} cm"), 3.2, -1.5, 27.)?,
        ];
        fade_in_all(s, &identity, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "26_7")?;
        fade_out_all(s, &identity, 0.3)?;
        let crossing_line = path(
            s,
            vec![zoom([a[0] - 0.1, a[1] + 0.4]), zoom([b[0] + 0.1, a[1] + 0.4])],
            Palette::teal(),
            2.,
        );
        let small_line = path(
            s,
            placement::clipped(0., a[1] + 0.4)
                .unwrap()
                .into_iter()
                .map(physical)
                .collect(),
            Palette::teal(),
            2.,
        );
        let selected = label(&k, s, "selected line · beach crossings: 0", 3.2, -0.25, 24.)?;
        fade_in_all(s, &[crossing_line, small_line, selected], 0.4)?;
        pause(s, 3.)?;
        let height = s.add_value_tracker(a[1] + 0.4);
        let linked = s.arena_mut().add(VMobject::new());
        s.add_updater(
            linked,
            move |s, _| {
                let h = s.tracker_value(height).unwrap_or(a[1] + 0.4);
                let _ = s.set_points(
                    crossing_line,
                    polyline(vec![zoom([a[0] - 0.1, h]), zoom([b[0] + 0.1, h])])
                        .build()
                        .points(),
                );
                if let Some(ends) = placement::clipped(0., h) {
                    let _ = s.set_points(
                        small_line,
                        polyline(ends.into_iter().map(physical).collect::<Vec<_>>())
                            .build()
                            .points(),
                    );
                }
            },
            true,
        )?;
        s.add_to_scene(linked)?;
        s.set_family_opacity(selected, 0.);
        s.play(placement::glide(height, a[1] + 0.125, 2., linear)?)?;
        stop(s, linked);
        fade_out_all(s, &[selected], 0.2)?;
        let hits: Vec<_> = the_longimeter::intersections(&r, 0., a[1] + 0.125)
            .into_iter()
            .filter(|p| p[0] > a[0] && p[0] < b[0])
            .collect();
        assert_eq!(hits.len().to_string(), f.get("scale.selected_beach_crossings").unwrap());
        let mut dots = Vec::new();
        for p in hits {
            for (at, radius) in [(zoom(p), 0.037), (physical(p), 0.027)] {
                dots.push(
                    s.arena_mut().add(
                        Circle::new()
                            .radius(radius)
                            .arc_center(at)
                            .style(flat(Palette::ink(), 1.))
                            .build(),
                    ),
                );
            }
        }
        fade_in_all(s, &dots, 0.3)?;
        let fourteen = label(
            &k,
            s,
            &format!(
                "selected line · beach crossings: {}",
                f.get("scale.selected_beach_crossings").unwrap()
            ),
            3.2,
            -0.25,
            24.,
        )?;
        let sum = label(&k, s, "count each crossing, not just each hit line", 3.2, -1.0, 22.)?;
        fade_in_all(s, &[fourteen, sum], 0.3)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "26_8")?;
        fade_out_all(s, &[crossing_line, small_line, fourteen, sum], 0.3)?;
        fade_out_all(s, &dots, 0.3)?;
        let means = vec![
            label(
                &k,
                s,
                &format!("one ripple: E[count] = {}", f.get("scale.one_ripple_mean").unwrap()),
                3.2,
                -0.1,
                27.,
            )?,
            label(
                &k,
                s,
                &format!("seven ripples: E[count] = {}", f.get("scale.beach_mean").unwrap()),
                3.2,
                -0.8,
                27.,
            )?,
            label(
                &k,
                s,
                &format!("one ripple: P(any hit) ≤ {}", f.get("scale.one_ripple_mean").unwrap()),
                3.2,
                -1.5,
                26.,
            )?,
            label(&k, s, "under the uniform turn + slide law", 3.2, -2.15, 22.)?,
        ];
        fade_in_all(s, &means, 0.4)?;
        k.hold(s)?;
        pause(s, 7.)?;
        say(&k, s, "26_9")?;
        fade_out_all(s, &means, 0.3)?;
        let comparison = vec![
            label(
                &k,
                s,
                &format!("sheet expectation: ≈ {} cm", f.get("scale.rippled_length").unwrap()),
                3.2,
                -0.1,
                28.,
            )?,
            label(
                &k,
                s,
                &format!("2 cm divider walk: ≈ {rounded_rippled} cm"),
                3.2,
                -0.8,
                28.,
            )?,
            label(&k, s, "a changed drawing has a changed length", 3.2, -1.5, 22.)?,
        ];
        fade_in_all(s, &comparison, 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "26_10")?;
        fade_out_all(s, &comparison, 0.3)?;
        let history = vec![
            label(&k, s, "Vistula: a historical illustration", 3.2, -0.05, 25.)?,
            label(&k, s, "Steinhaus, quoted by Mandelbrot (1967)", 3.2, -0.65, 21.)?,
            label(&k, s, "increased detail → much greater length", 3.2, -1.25, 22.)?,
            label(&k, s, "not a measurement made in this film", 3.2, -1.85, 22.)?,
        ];
        fade_in_all(s, &history, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "26_11")?;
        fade_out_all(s, &history, 0.3)?;
        fade_in_all(s, &comparison, 0.3)?;
        let note = precisely(&k, s, NOTE)?;
        let full = now(s) + 0.5;
        fade_in_all(s, &[note], 0.5)?;
        k.hold(s)?;
        wait_until(s, full + (NOTE.chars().count() + 10) as f64 / 15. + 1.)?;
        pause(s, 2.)?;
        stop(s, driver);
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pi_display_keeps_the_denominator_outside_the_numerator() {
        assert_eq!(display_pi("7/4pi"), "7π/4");
        assert_eq!(display_pi("4 + 1819/240pi"), "4 + 1819π/240");
        assert_eq!(display_pi("2 + 3/8pi"), "2 + 3π/8");
        assert_eq!(display_pi("-2pi"), "-2π");
    }
    #[test]
    fn parametrized_beach_matches_each_native_arc_and_keeps_ends() {
        let c = model::coast();
        for n in [7, 14] {
            let r = sheet::rippled(&c, n).unwrap();
            for j in 0..n {
                for i in 0..=20 {
                    let t = i as f64 / 20.;
                    let a = beach_point(&c, n, (j as f64 + t) / n as f64);
                    let b = r.parts[model::BEACH + j].seg.point(t);
                    assert!((a[0] - b[0]).hypot(a[1] - b[1]) < 1e-12);
                }
            }
            assert!((r.length - (4. + 1819. * PI / 240.)).abs() < 1e-12);
            assert_eq!(r.parts.len(), c.parts.len() + n - 1);
        }
    }
    #[test]
    fn rounded_divider_totals_hide_real_changes() {
        let c = model::coast();
        let r = sheet::rippled(&c, 7).unwrap();
        let a = c.walk(2.);
        let b = r.walk(2.);
        assert_eq!(a.steps(), b.steps());
        assert!((a.length() - b.length()).abs() > 0.01);
        assert_eq!(facts::fixed(a.length(), 1), facts::fixed(b.length(), 1));
        assert!(b.length() < r.length);
    }
}
