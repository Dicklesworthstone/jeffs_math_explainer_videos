//! One persistent stick, its chart band, and the measure that stays fixed.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::glide;
use crate::{
    facts,
    line_chart::{self, Address},
    model, palette, pin_lines,
    stick_band::{self, Stick},
};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, TextKind, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::{f64::consts::PI, rc::Rc};

pub struct TheSticksBand {
    pub kit: Kit,
}
const SCALE: f64 = 0.36;
const STEPS: usize = 240;
const NOTE: &str = "Uniform chart area, conditioned on meeting the card. Equal-length sticks have equal hit probabilities while fully inside it. Angle is in radians; wrapping a half turn reverses signed distance. Pins use a different law.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.4 + SCALE * p[0], 0.35 + SCALE * p[1])
}
fn chart(theta: f64, p: f64) -> Vec3 {
    v(0.8 + 5.4 * theta / PI, 0.25 + 2. * p / line_chart::reach())
}
fn path(s: &mut Stage<'_>, ps: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(ps).style(line_style(color, width)).build())
}
fn txt(k: &Kit, s: &mut Stage<'_>, text: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, text, size, color, v(x, y))
}
fn dot(s: &mut Stage<'_>, p: Vec3, color: Srgb, r: f64) -> Mob {
    s.arena_mut()
        .add(Circle::new().radius(r).arc_center(p).style(flat(color, 1.)).build())
}
fn edge(stick: Stick, upper: bool) -> Vec<Vec3> {
    (0..=STEPS)
        .map(|i| {
            let t = PI * i as f64 / STEPS as f64;
            chart(t, stick.interval(t)[usize::from(upper)])
        })
        .collect()
}
fn fill_points(stick: Stick) -> Vec<Vec3> {
    let mut p = edge(stick, false);
    p.extend(edge(stick, true).into_iter().rev());
    p
}
fn chord(a: Address) -> Vec<Vec3> {
    a.line()
        .chord()
        .expect("authored line meets card")
        .into_iter()
        .map(physical)
        .collect()
}
#[derive(Clone, Copy)]
struct Controls {
    x: Mob,
    angle: Mob,
}
impl Controls {
    fn stick(self, s: &fmn::mobject::Stage) -> Stick {
        stick_band::example(
            [s.tracker_value(self.x).unwrap_or(0.), 0.],
            s.tracker_value(self.angle).unwrap_or(PI / 2.),
        )
    }
}
fn working() -> Result<(), String> {
    let f = facts::facts()?;
    let poses = [
        stick_band::example([0., 0.], PI / 2.),
        stick_band::example([3., 0.], PI / 2.),
        stick_band::example([0., 0.], 0.),
    ];
    let values = poses.map(|p| facts::fixed(p.area(), 2)).join(",");
    if f.get("chart.stick_poses") != Some(values.as_str())
        || f.get("chart.stick_band") != Some(facts::fixed(poses[0].area(), 2).as_str())
    {
        return Err("scene band differs from facts".into());
    }
    Ok(())
}
impl SceneConstruct for TheSticksBand {
    fn name(&self) -> &str {
        "18_the_sticks_band"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        working().map_err(fail)?;
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        say(&k, s, "18_1")?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * SCALE)
                .height(model::CARD_H * SCALE)
                .corner_radius(0.05)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(physical([0., 0.])),
        );
        let old_sticks =
            pin_lines::STICKS.map(|ends| path(s, ends.into_iter().map(physical).collect(), palette::stick(), 6.));
        let title = txt(&k, s, "The stick's band", 38., 0., 3.3, Palette::ink())?;
        let axes = vec![
            path(
                s,
                vec![
                    chart(0., -line_chart::reach()),
                    chart(PI, -line_chart::reach()),
                    chart(PI, line_chart::reach()),
                    chart(0., line_chart::reach()),
                    chart(0., -line_chart::reach()),
                ],
                Palette::dim(),
                1.5,
            ),
            path(s, vec![chart(0., 0.), chart(PI, 0.)], Palette::dim(), 1.),
            txt(&k, s, "normal angle (radians)", 23., 3.5, -2.13, Palette::teal())?,
            txt(&k, s, "signed distance (cm)", 23., 3.5, 2.65, Palette::gold())?,
            txt(&k, s, "0", 21., 0.8, -1.95, Palette::dim())?,
            k.tex(s, "\\pi", 23., Palette::dim(), &[], v(6.2, -1.95))?,
        ];
        let boundary = path(
            s,
            (0..=STEPS)
                .map(|i| {
                    let t = PI * i as f64 / STEPS as f64;
                    chart(t, line_chart::half_width(t))
                })
                .chain((0..=STEPS).rev().map(|i| {
                    let t = PI * i as f64 / STEPS as f64;
                    chart(t, -line_chart::half_width(t))
                }))
                .collect(),
            Palette::teal(),
            1.5,
        );
        fade_in_all(s, &[card, old_sticks[0], old_sticks[1], title, boundary], 0.5)?;
        fade_in_all(s, &axes, 0.4)?;
        pause(s, 1.5)?;
        fade_out_all(s, &old_sticks, 0.4)?;
        let controls = Controls {
            x: s.add_value_tracker(0.),
            angle: s.add_value_tracker(PI / 2.),
        };
        let start = controls.stick(s);
        let fill = s.arena_mut().add(
            Polygon::new(fill_points(start))
                .style(flat(palette::stick(), 0.22))
                .build(),
        );
        let lower = path(s, edge(start, false), palette::stick(), 2.5);
        let upper = path(s, edge(start, true), palette::stick(), 2.5);
        let stick = path(
            s,
            start.ends().into_iter().map(physical).collect(),
            palette::stick(),
            7.,
        );
        let stick_name = txt(
            &k,
            s,
            &format!("{} cm stick", f.get("stick.length").unwrap()),
            28.,
            -3.4,
            2.25,
            palette::stick(),
        )?;
        let driver = s.arena_mut().add(VMobject::new());
        s.add_updater(
            driver,
            move |s, _| {
                let pose = controls.stick(s);
                let _ = s.set_points(
                    stick,
                    polyline(pose.ends().into_iter().map(physical).collect::<Vec<_>>())
                        .build()
                        .points(),
                );
                let _ = s.set_points(fill, Polygon::new(fill_points(pose)).build().points());
                let _ = s.set_points(lower, polyline(edge(pose, false)).build().points());
                let _ = s.set_points(upper, polyline(edge(pose, true)).build().points());
            },
            true,
        )?;
        fade_in_all(s, &[driver, stick, stick_name], 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "18_2")?;
        let theta = s.add_value_tracker(PI / 2.);
        let distance = s.add_value_tracker(-4.);
        let test_line = path(
            s,
            chord(Address {
                theta: PI / 2.,
                distance: -4.,
            }),
            Palette::ink(),
            2.,
        );
        let column = path(s, vec![chart(PI / 2., -3.), chart(PI / 2., 3.)], palette::stick(), 7.);
        let pointer = dot(s, chart(PI / 2., -4.), Palette::gold(), 0.055);
        let ends = start.ends().map(|p| dot(s, physical(p), Palette::gold(), 0.055));
        let inspect = s.arena_mut().add(VMobject::new());
        s.add_updater(
            inspect,
            move |s, _| {
                let t = s.tracker_value(theta).unwrap_or(PI / 2.);
                let p = s.tracker_value(distance).unwrap_or(0.);
                let [lo, hi] = controls.stick(s).interval(t);
                let _ = s.set_points(column, polyline(vec![chart(t, lo), chart(t, hi)]).build().points());
                let _ = s.set_points(
                    test_line,
                    polyline(chord(Address { theta: t, distance: p })).build().points(),
                );
                s.move_to(pointer, chart(t, p), ORIGIN);
            },
            true,
        )?;
        let column_words = txt(
            &k,
            s,
            "between the endpoint projections → a crossing",
            26.,
            0.,
            -2.65,
            Palette::ink(),
        )?;
        fade_in_all(
            s,
            &[inspect, test_line, column, pointer, ends[0], ends[1], column_words],
            0.5,
        )?;
        s.play(glide(distance, 4., 8., linear)?)?;
        pause(s, 1.5)?;
        s.play(glide(distance, 0., 2., linear)?)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "18_3")?;
        fade_out_all(s, &[column_words], 0.3)?;
        fade_in_all(s, &[fill, lower, upper], 0.7)?;
        s.play(glide(theta, PI / 6., 5., linear)?)?;
        pause(s, 2.)?;
        s.play(glide(theta, 5. * PI / 6., 6., linear)?)?;
        k.hold(s)?;
        pause(s, 2.)?;
        stop(s, inspect);
        fade_out_all(s, &[test_line, column, pointer, ends[0], ends[1]], 0.4)?;

        say(&k, s, "18_4")?;
        let area_words = txt(
            &k,
            s,
            &format!(
                "shadow widths added over a half turn → area {}",
                f.get("chart.stick_band").unwrap()
            ),
            27.,
            0.,
            -2.65,
            palette::stick(),
        )?;
        let units = txt(
            &k,
            s,
            "chart area: centimetres × radians",
            23.,
            0.,
            -3.15,
            Palette::ink(),
        )?;
        fade_in_all(s, &[area_words, units], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "18_5")?;
        fade_out_all(s, &[fill, lower, upper, area_words, units], 0.4)?;
        let q = txt(
            &k,
            s,
            "Slide the stick: what changes in its band?",
            28.,
            0.,
            -2.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[q], 0.3)?;
        let thinking = think(&k, s, v(-5.3, 0.4))?;

        say(&k, s, "18_6")?;
        fade_out_all(s, &[q, thinking], 0.3)?;
        fade_in_all(s, &[fill, lower, upper], 0.4)?;
        let area_alpha = s.add_value_tracker(1.);
        let book = Rc::clone(&k.book);
        let area = s.always_redraw(move |s| {
            let d = DecimalNumber::new(controls.stick(s).area())
                .num_decimal_places(2)
                .include_sign(false)
                .font_size(32.)
                .color(palette::stick())
                .build(&book)
                .expect("bundled digits");
            explainer_kit::glyphs::record(TextKind::Text, d.num_string());
            let m = s.add(d.into_vmob());
            s.move_to(m, v(4.3, -2.7), RIGHT);
            s.set_family_opacity(m, s.tracker_value(area_alpha).unwrap_or(0.));
            m
        });
        s.add_to_scene(area)?;
        let label = txt(&k, s, "area", 28., 2.1, -2.7, palette::stick())?;
        let invariant = txt(
            &k,
            s,
            "column moves; column height stays",
            25.,
            -2.8,
            -2.7,
            Palette::ink(),
        )?;
        fade_in_all(s, &[label, invariant], 0.4)?;
        s.play(glide(controls.x, 3., 7., linear)?)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "18_7")?;
        s.play(glide(controls.x, 0., 3., linear)?)?;
        fade_out_all(s, &[fill, lower, upper, label, invariant], 0.3)?;
        s.play(glide(area_alpha, 0., 0.3, linear)?)?;
        let q2 = txt(
            &k,
            s,
            "Turn the centered stick: will its area change?",
            27.,
            0.,
            -2.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[q2], 0.3)?;
        let thinking2 = think(&k, s, v(-5.3, 0.4))?;

        say(&k, s, "18_8")?;
        fade_out_all(s, &[q2, thinking2], 0.3)?;
        fade_in_all(s, &[fill, lower, upper, label], 0.4)?;
        s.play(glide(area_alpha, 1., 0.3, linear)?)?;
        let turning = txt(
            &k,
            s,
            "the width pattern moves across the angles",
            24.,
            -2.5,
            -2.7,
            Palette::ink(),
        )?;
        fade_in_all(s, &[turning], 0.4)?;
        s.play(glide(controls.angle, 0., 7., linear)?)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "18_9")?;
        fade_out_all(s, &[turning, label, fill, lower, upper], 0.3)?;
        s.play(glide(area_alpha, 0., 0.3, linear)?)?;
        let wrap_theta = s.add_value_tracker(PI - 0.25);
        let wrap_line = path(
            s,
            chord(Address {
                theta: PI - 0.25,
                distance: 2.,
            }),
            Palette::gold(),
            3.,
        );
        let wrapped = stick_band::wrap(Address {
            theta: PI - 0.25,
            distance: 2.,
        });
        let wrap_dot = dot(s, chart(wrapped.theta, wrapped.distance), Palette::gold(), 0.065);
        let seam = s.arena_mut().add(VMobject::new());
        s.add_updater(
            seam,
            move |s, _| {
                let a = Address {
                    theta: s.tracker_value(wrap_theta).unwrap_or(PI),
                    distance: 2.,
                };
                let b = stick_band::wrap(a);
                let _ = s.set_points(wrap_line, polyline(chord(a)).build().points());
                s.move_to(wrap_dot, chart(b.theta, b.distance), ORIGIN);
            },
            true,
        )?;
        let seam_words = txt(
            &k,
            s,
            "edge wrap: the normal reverses, so distance changes sign",
            25.,
            0.,
            -2.7,
            Palette::gold(),
        )?;
        fade_in_all(s, &[seam, wrap_line, wrap_dot, seam_words], 0.5)?;
        s.play(glide(wrap_theta, PI, 3., linear)?)?;
        pause(s, 3.)?;
        s.play(glide(wrap_theta, PI + 0.25, 3., linear)?)?;
        k.hold(s)?;
        pause(s, 3.)?;
        stop(s, seam);
        fade_out_all(s, &[wrap_line, wrap_dot, seam_words], 0.4)?;

        say(&k, s, "18_10")?;
        s.play(glide(controls.angle, PI / 2., 3., linear)?)?;
        let pins = pin_lines::samples().map_err(fail)?;
        let cloud = pins
            .iter()
            .step_by(10)
            .map(|line| {
                let a = Address::from_line(*line).expect("validated retained pins");
                dot(s, chart(a.theta, a.distance), Palette::gold(), 0.013)
            })
            .collect::<Vec<_>>();
        let pin_words = txt(
            &k,
            s,
            &format!(
                "fixed pin distribution · {} sample dots shown",
                f.get("lines.chart_shown").unwrap()
            ),
            24.,
            0.,
            -2.7,
            Palette::gold(),
        )?;
        fade_in_all(s, &cloud, 0.7)?;
        fade_in_all(s, &[fill, lower, upper, pin_words], 0.5)?;
        s.play(glide(controls.x, 3., 7., linear)?)?;
        pause(s, 2.)?;
        s.play(glide(controls.x, 0., 5., linear)?)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "18_11")?;
        fade_out_all(s, &cloud, 0.4)?;
        fade_out_all(s, &[pin_words], 0.3)?;
        let note = precisely(&k, s, NOTE)?;
        fade_in_all(s, &[note], 0.5)?;
        let deadline = now(s) + (NOTE.chars().count() + 10) as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;
        stop(s, driver);
        stop(s, area);
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn displayed_band_uses_actual_endpoints_and_stays_on_chart() {
        working().unwrap();
        for x in [0., 1., 2., 3.] {
            for angle in [0., PI / 6., PI / 2.] {
                let p = stick_band::example([x, 0.], angle);
                assert!(
                    p.ends()
                        .iter()
                        .all(|e| e[0].abs() <= model::CARD_W / 2. && e[1].abs() <= model::CARD_H / 2.)
                );
                for j in 0..=STEPS {
                    let t = PI * j as f64 / STEPS as f64;
                    let [lo, hi] = p.interval(t);
                    assert!(lo >= -line_chart::half_width(t) - 1e-12 && hi <= line_chart::half_width(t) + 1e-12);
                }
            }
        }
    }
}
