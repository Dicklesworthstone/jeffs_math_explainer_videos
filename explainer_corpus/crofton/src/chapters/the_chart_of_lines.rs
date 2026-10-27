//! A line's address, the sheet's joint measure, and a controlled comparison.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::{glide, live_count};
use crate::{
    facts,
    line_chart::{self, Address},
    model, palette, pin_lines,
};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::PI;

pub struct TheChartOfLines {
    pub kit: Kit,
}
const SCALE: f64 = 0.36;
const NOTE: &str = "Uniform area in the angle-distance chart, conditioned on meeting the card. Accepted angles alone are not uniform. Counts are finite seeded samples; displayed dots are a subset.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.4 + SCALE * p[0], 0.35 + SCALE * p[1])
}
fn chart(a: Address) -> Vec3 {
    v(0.8 + 5.4 * a.theta / PI, 0.25 + 2.0 * a.distance / line_chart::reach())
}
fn path(s: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}
fn dot(s: &mut Stage<'_>, p: Vec3, color: Srgb, radius: f64) -> Mob {
    s.arena_mut().add(
        Circle::new()
            .radius(radius)
            .arc_center(p)
            .style(flat(color, 1.))
            .build(),
    )
}
fn txt(k: &Kit, s: &mut Stage<'_>, text: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, text, size, color, v(x, y))
}
fn chord(a: Address) -> Vec<Vec3> {
    a.line()
        .chord()
        .expect("authored address meets card")
        .into_iter()
        .map(physical)
        .collect()
}
fn working() -> Result<(Vec<pin_lines::PinLine>, [usize; 2]), String> {
    let lines = line_chart::samples()?;
    let hits = pin_lines::totals(&lines);
    let f = facts::facts()?;
    for (key, value) in [
        ("lines.even.middle", hits[0]),
        ("lines.even.corner", hits[1]),
        ("lines.count", lines.len()),
    ] {
        if f.get(key) != Some(value.to_string().as_str()) {
            return Err(format!("scene differs from {key}"));
        }
    }
    Ok((lines, hits))
}
impl SceneConstruct for TheChartOfLines {
    fn name(&self) -> &str {
        "17_the_chart_of_lines"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (lines, hits) = working().map_err(fail)?;
        let f = facts::facts().map_err(fail)?;
        say(&k, s, "17_1")?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * SCALE)
                .height(model::CARD_H * SCALE)
                .corner_radius(0.05)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(physical([0., 0.])),
        );
        let sticks =
            pin_lines::STICKS.map(|ends| path(s, ends.into_iter().map(physical).collect(), palette::stick(), 6.));
        let heading = txt(&k, s, "Give every line an address", 36., 0., 3.3, Palette::ink())?;
        let middle = txt(&k, s, "middle", 23., physical([0., 0.])[0], 2.2, Palette::ink())?;
        let corner = txt(&k, s, "corner", 23., physical([-7., 0.])[0], 2.2, Palette::ink())?;
        let old = txt(
            &k,
            s,
            &format!(
                "pins: {} / {}",
                f.get("lines.pins.middle").unwrap(),
                f.get("lines.pins.corner").unwrap()
            ),
            29.,
            3.5,
            0.5,
            Palette::gold(),
        )?;
        fade_in_all(s, &[card, sticks[0], sticks[1], heading, middle, corner, old], 0.6)?;
        k.hold(s)?;
        pause(s, 1.5)?;

        say(&k, s, "17_2")?;
        fade_out_all(s, &[old], 0.3)?;
        let theta = s.add_value_tracker(0.6);
        let distance = s.add_value_tracker(2.);
        let initial = Address {
            theta: 0.6,
            distance: 2.,
        };
        let line = path(s, chord(initial), Palette::ink(), 3.);
        let normal = path(
            s,
            vec![physical([0., 0.]), physical([2. * 0.6_f64.cos(), 2. * 0.6_f64.sin()])],
            Palette::teal(),
            4.,
        );
        let arm = path(s, vec![physical([0., 0.]), physical([2.5, 0.])], Palette::dim(), 1.5);
        let arc = path(
            s,
            (0..=40)
                .map(|i| {
                    let a = 0.6 * i as f64 / 40.;
                    physical([1.3 * a.cos(), 1.3 * a.sin()])
                })
                .collect(),
            Palette::teal(),
            3.,
        );
        let foot = path(
            s,
            vec![physical([0., 0.]), physical(initial.foot())],
            Palette::gold(),
            2.,
        );
        let origin = dot(s, physical([0., 0.]), Palette::gold(), 0.045);
        let pointer = dot(s, chart(initial), Palette::gold(), 0.06);
        let anchor = s.arena_mut().add(VMobject::new());
        s.add_updater(
            anchor,
            move |s, _| {
                let a = Address {
                    theta: s.tracker_value(theta).unwrap_or(0.6),
                    distance: s.tracker_value(distance).unwrap_or(2.),
                };
                let _ = s.set_points(line, polyline(chord(a)).build().points());
                let _ = s.set_points(
                    normal,
                    polyline(vec![
                        physical([0., 0.]),
                        physical([2. * a.theta.cos(), 2. * a.theta.sin()]),
                    ])
                    .build()
                    .points(),
                );
                let _ = s.set_points(
                    arc,
                    polyline(
                        (0..=40)
                            .map(|i| {
                                let t = a.theta * i as f64 / 40.;
                                physical([1.3 * t.cos(), 1.3 * t.sin()])
                            })
                            .collect::<Vec<_>>(),
                    )
                    .build()
                    .points(),
                );
                let _ = s.set_points(
                    foot,
                    polyline(vec![physical([0., 0.]), physical(a.foot())]).build().points(),
                );
                s.move_to(pointer, chart(a), ORIGIN);
            },
            true,
        )?;
        let normal_label = txt(
            &k,
            s,
            "normal angle: direction of the perpendicular",
            26.,
            0.,
            -2.35,
            Palette::teal(),
        )?;
        fade_in_all(s, &[anchor, line, normal, arm, arc, origin, normal_label], 0.5)?;
        s.play(glide(theta, 1.2, 4., linear)?)?;
        pause(s, 2.)?;
        k.hold(s)?;
        pause(s, 1.)?;

        say(&k, s, "17_3")?;
        fade_out_all(s, &[normal_label], 0.3)?;
        let distance_label = txt(
            &k,
            s,
            "signed distance: along / against the normal",
            26.,
            0.,
            -2.35,
            Palette::gold(),
        )?;
        fade_in_all(s, &[foot, distance_label], 0.4)?;
        s.play(glide(distance, -2., 4., linear)?)?;
        pause(s, 2.)?;
        s.play(glide(distance, 0., 2., linear)?)?;
        pause(s, 2.)?;
        s.play(glide(distance, 2., 2., linear)?)?;
        k.hold(s)?;
        pause(s, 1.5)?;

        say(&k, s, "17_4")?;
        fade_out_all(s, &[distance_label], 0.3)?;
        let mut axes = vec![
            path(
                s,
                vec![v(0.8, -1.75), v(6.2, -1.75), v(6.2, 2.25), v(0.8, 2.25), v(0.8, -1.75)],
                Palette::dim(),
                1.5,
            ),
            path(s, vec![v(0.8, 0.25), v(6.2, 0.25)], Palette::dim(), 1.),
            txt(&k, s, "normal angle", 24., 3.5, -2.13, Palette::teal())?,
            txt(&k, s, "signed distance (cm)", 23., 3.5, 2.65, Palette::gold())?,
        ];
        let half = f.get("lines.half_turn").unwrap();
        let reach = f.get("lines.reach").unwrap();
        axes.push(txt(
            &k,
            s,
            &format!("{:.0}°", 0.0_f64.to_degrees()),
            19.,
            0.8,
            -1.95,
            Palette::dim(),
        )?);
        axes.push(txt(&k, s, &format!("{half}°"), 19., 6.1, -1.95, Palette::dim())?);
        axes.push(txt(&k, s, &format!("+{reach}"), 18., 0.38, 2.25, Palette::dim())?);
        axes.push(txt(&k, s, &format!("−{reach}"), 18., 0.38, -1.65, Palette::dim())?);
        fade_in_all(s, &axes, 0.6)?;
        fade_in_all(s, &[pointer], 0.4)?;
        s.play(glide(theta, 0.35, 4., linear)?)?;
        pause(s, 1.5)?;
        s.play(glide(distance, -2., 3., linear)?)?;
        pause(s, 1.5)?;
        s.play(glide(theta, 1.25, 4., linear)?)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "17_5")?;
        stop(s, anchor);
        fade_out_all(s, &[line, normal, arm, arc, foot, pointer], 0.35)?;
        let column_angle = s.add_value_tracker(0.6);
        let shift = s.add_value_tracker(0.);
        let offsets = [-2., -1., 0., 1., 2.].map(|x| x * model::GAP);
        let mut parallel = Vec::new();
        let mut points = Vec::new();
        for d in offsets {
            let a = Address {
                theta: 0.6,
                distance: d,
            };
            parallel.push(path(s, chord(a), Palette::ink(), 2.));
            points.push(dot(s, chart(a), Palette::ink(), 0.04));
        }
        let columns = s.arena_mut().add(VMobject::new());
        let bars = parallel.clone();
        let marks = points.clone();
        s.add_updater(
            columns,
            move |s, _| {
                for ((bar, mark), d) in bars.iter().zip(&marks).zip(offsets) {
                    let a = Address {
                        theta: s.tracker_value(column_angle).unwrap_or(0.6),
                        distance: d + s.tracker_value(shift).unwrap_or(0.),
                    };
                    let _ = s.set_points(*bar, polyline(chord(a)).build().points());
                    s.move_to(*mark, chart(a), ORIGIN);
                }
            },
            true,
        )?;
        fade_in_all(s, &[columns], 0.01)?;
        fade_in_all(s, &parallel, 0.4)?;
        fade_in_all(s, &points, 0.4)?;
        let column_note = txt(
            &k,
            s,
            &format!("selected sheet lines · {} cm gap", f.get("sheet.gap").unwrap()),
            24.,
            0.,
            -2.55,
            Palette::ink(),
        )?;
        fade_in_all(s, &[column_note], 0.4)?;
        s.play(glide(shift, 0.4, 4., linear)?)?;
        pause(s, 2.)?;
        s.play(glide(column_angle, 1.25, 5., linear)?)?;
        pause(s, 2.)?;
        k.hold(s)?;
        pause(s, 1.)?;

        say(&k, s, "17_6")?;
        stop(s, columns);
        fade_out_all(s, &parallel, 0.3)?;
        fade_out_all(s, &points, 0.3)?;
        fade_out_all(s, &[column_note, origin], 0.2)?;
        let mut boundary = Vec::new();
        for i in 0..=180 {
            let theta = PI * i as f64 / 180.;
            boundary.push(chart(Address {
                theta,
                distance: line_chart::half_width(theta),
            }));
        }
        for i in (0..=180).rev() {
            let theta = PI * i as f64 / 180.;
            boundary.push(chart(Address {
                theta,
                distance: -line_chart::half_width(theta),
            }));
        }
        boundary.push(boundary[0]);
        let region = path(s, boundary, Palette::teal(), 3.);
        fade_in_all(s, &[region], 0.6)?;
        let rejected = dot(
            s,
            chart(Address {
                theta: PI / 2.,
                distance: 0.95 * line_chart::reach(),
            }),
            Palette::red(),
            0.065,
        );
        let miss = txt(
            &k,
            s,
            "outside boundary: line misses the card",
            24.,
            0.,
            -2.55,
            Palette::red(),
        )?;
        fade_in_all(s, &[rejected, miss], 0.4)?;
        pause(s, 3.)?;
        fade_out_all(s, &[rejected, miss], 0.3)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "17_7")?;
        let law = txt(
            &k,
            s,
            "equal chart areas → equal chances",
            28.,
            0.,
            -2.55,
            Palette::teal(),
        )?;
        fade_in_all(s, &[law], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "17_8")?;
        fade_out_all(s, &[law], 0.3)?;
        let question = txt(
            &k,
            s,
            "Will the middle stick win again?",
            28.,
            0.,
            -2.55,
            Palette::ink(),
        )?;
        fade_in_all(s, &[question], 0.4)?;
        let thinking = think(&k, s, v(-1.5, 0.4))?;

        say(&k, s, "17_9")?;
        fade_out_all(s, &[question, thinking], 0.3)?;
        let index = s.add_value_tracker(0.);
        let alpha = s.add_value_tracker(1.);
        let experiment_line = path(
            s,
            lines[0].chord().unwrap().into_iter().map(physical).collect(),
            Palette::ink(),
            2.5,
        );
        let experiment_dot = dot(
            s,
            chart(Address::from_line(lines[0]).map_err(fail)?),
            Palette::gold(),
            0.055,
        );
        let driver = s.arena_mut().add(VMobject::new());
        let stream = lines.clone();
        s.add_updater(
            driver,
            move |s, _| {
                let n = (s.tracker_value(index).unwrap_or(0.).floor() as usize).min(stream.len() - 1);
                let line = stream[n];
                let _ = s.set_points(
                    experiment_line,
                    polyline(line.chord().unwrap().into_iter().map(physical).collect::<Vec<_>>())
                        .build()
                        .points(),
                );
                s.move_to(
                    experiment_dot,
                    chart(Address::from_line(line).expect("validated nondegenerate input")),
                    ORIGIN,
                );
            },
            true,
        )?;
        fade_in_all(s, &[driver, experiment_line, experiment_dot], 0.4)?;
        let count = live_count(
            &k,
            s,
            move |s| (s.tracker_value(index).unwrap_or(0.).floor() as usize + 1).min(pin_lines::COUNT),
            alpha,
            30.,
            v(-4.4, -2.35),
        )?;
        let experiment_label = txt(
            &k,
            s,
            "lines counted · selected draws shown",
            23.,
            1.,
            -2.55,
            Palette::dim(),
        )?;
        fade_in_all(s, &[experiment_label], 0.3)?;
        s.play(glide(index, (lines.len() - 1) as f64, 12., linear)?)?;
        stop(s, driver);
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "17_10")?;
        fade_out_all(s, &[count, experiment_label, experiment_line, experiment_dot], 0.4)?;
        let result = txt(
            &k,
            s,
            &format!("this seeded run: middle {} · corner {}", hits[0], hits[1]),
            29.,
            0.,
            -2.55,
            Palette::ink(),
        )?;
        let subset_label = txt(
            &k,
            s,
            &format!("{} dots shown", f.get("lines.chart_shown").unwrap()),
            24.,
            -3.4,
            -1.7,
            Palette::ink(),
        )?;
        fade_in_all(s, &[result, subset_label], 0.6)?;
        let shown = lines.len() / 10;
        let uniform = lines
            .iter()
            .take(shown)
            .map(|line| Address::from_line(*line).map(|a| dot(s, chart(a), Palette::ink(), 0.014)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(fail)?;
        fade_in_all(s, &uniform, 1.)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "17_11")?;
        fade_out_all(s, &[result, subset_label], 0.3)?;
        let pin_inputs = pin_lines::samples().map_err(fail)?;
        let pins = pin_inputs
            .iter()
            .take(shown)
            .map(|line| Address::from_line(*line).map(|a| dot(s, chart(a), Palette::gold(), 0.014)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(fail)?;
        let even_label = txt(
            &k,
            s,
            &format!(
                "uniform chart rule · {} dots shown",
                f.get("lines.chart_shown").unwrap()
            ),
            25.,
            0.,
            -2.55,
            Palette::ink(),
        )?;
        fade_in_all(s, &[even_label], 0.3)?;
        pause(s, 4.)?;
        fade_out_all(s, &uniform, 0.6)?;
        fade_out_all(s, &[even_label], 0.3)?;
        let pin_label = txt(
            &k,
            s,
            &format!("independent pins · {} dots shown", f.get("lines.chart_shown").unwrap()),
            25.,
            0.,
            -2.55,
            Palette::gold(),
        )?;
        fade_in_all(s, &pins, 0.8)?;
        fade_in_all(s, &[pin_label], 0.3)?;
        k.hold(s)?;
        pause(s, 5.)?;

        say(&k, s, "17_12")?;
        fade_out_all(s, &[pin_label], 0.3)?;
        let note = precisely(&k, s, NOTE)?;
        fade_in_all(s, &[note], 0.5)?;
        let deadline = now(s) + (NOTE.chars().count() + 10) as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;
        pause(s, 1.)?;
        k.finish(s)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_and_chart_points_are_computed_from_retained_lines() {
        let (lines, hits) = working().unwrap();
        assert_eq!(hits, [796, 811]);
        assert_eq!(lines.len() / 10, 1000);
        for line in lines {
            let a = Address::from_line(line).unwrap();
            let p = chart(a);
            assert!((0.8..=6.2).contains(&p[0]));
            assert!((-1.75..=2.25).contains(&p[1]));
        }
    }
}
