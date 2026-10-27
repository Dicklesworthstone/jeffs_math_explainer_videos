//! Equal direction spacing controls orientation error only after full slide averaging.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::{self, Pose, glide};
use super::sheet;
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::PI;

pub struct FixedDirections {
    pub kit: Kit,
}
const NOTE: &str = "For finite positive rectifiable path length, count all traversals. Average uniformly over a full gap at each of m equally spaced directions. Put t = π/(2m): t cot(t) ≤ estimate/length ≤ t/sin(t). Percentages are rounded.";

/// The geometric measurement of a unit segment, with every slide averaged.
pub fn ratio(m: usize, alpha: f64) -> f64 {
    assert!(m > 0);
    (0..m)
        .map(|j| (alpha - PI * j as f64 / m as f64).sin().abs())
        .sum::<f64>()
        * PI
        / (2. * m as f64)
}
/// On a sector of width pi/m, the sine sum is csc(t) cos(alpha-t).
/// Its minimum is at an aligned direction; maximum at the sector midpoint.
pub fn bounds(m: usize) -> (f64, f64) {
    assert!(m > 0);
    let t = PI / (2. * m as f64);
    (if m == 1 { 0. } else { t / t.tan() }, t / t.sin())
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, color, v(x, y))
}
fn path(s: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}
fn dot(s: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    s.arena_mut().add(
        Circle::new()
            .radius(0.055)
            .arc_center(at)
            .style(flat(color, 1.))
            .build(),
    )
}
fn physical(p: model::P2) -> Vec3 {
    v(-3.5 + 0.32 * p[0], 0.65 + 0.32 * p[1])
}
fn plot(m: usize, alpha: f64, value: f64) -> Vec3 {
    let (lo, hi) = if m == 4 { (-6., 4.) } else { (-25., 15.) };
    v(0.6 + 5.3 * alpha / (PI / 2.), -0.7 + 2.7 * (value - lo) / (hi - lo))
}
struct Demo {
    stick: Mob,
    tracker: Mob,
    driver: Mob,
    bars: Vec<Mob>,
    chart: Vec<Mob>,
}
impl Demo {
    fn new(k: &Kit, s: &mut Stage<'_>, m: usize) -> fmn::Result<Self> {
        let tracker = s.add_value_tracker(0.);
        let stick = path(s, vec![physical([-3., 0.]), physical([3., 0.])], Palette::gold(), 5.);
        let bars: Vec<_> = (0..m)
            .map(|_| path(s, vec![v(-5.6, -1.6), v(-5.599, -1.6)], Palette::teal(), 5.))
            .collect();
        let mut chart = vec![
            text(
                k,
                s,
                "orientation error after full slide averages",
                23.,
                3.2,
                2.65,
                Palette::ink(),
            )?,
            path(s, vec![v(0.6, 2.), v(0.6, -0.7), v(5.9, -0.7)], Palette::dim(), 1.4),
            path(s, vec![plot(m, 0., 0.), plot(m, PI / 2., 0.)], Palette::dim(), 1.2),
            text(k, s, "stick direction", 22., 3.2, -1.25, Palette::ink())?,
            text(k, s, "0°", 20., 0.6, -1., Palette::ink())?,
            text(k, s, "90°", 20., 5.9, -1., Palette::ink())?,
            text(k, s, "0%", 20., 0.2, plot(m, 0., 0.)[1], Palette::ink())?,
            text(k, s, "across-line projection lengths", 22., -3.5, -1.1, Palette::ink())?,
        ];
        for j in 0..m {
            chart.push(text(
                k,
                s,
                &format!("{}°", 180 * j / m),
                19.,
                -6.15,
                -1.65 - j as f64 * 0.38,
                Palette::ink(),
            )?);
        }
        let curve = path(
            s,
            (0..=360)
                .map(|j| {
                    let a = PI * j as f64 / 720.;
                    plot(m, a, (ratio(m, a) - 1.) * 100.)
                })
                .collect(),
            Palette::teal(),
            2.8,
        );
        let marker = dot(s, plot(m, 0., (ratio(m, 0.) - 1.) * 100.), Palette::gold());
        chart.extend([curve, marker]);
        let driver = s.arena_mut().add(VMobject::new());
        let moving_bars = bars.clone();
        s.add_updater(
            driver,
            move |s, _| {
                let a = s.tracker_value(tracker).unwrap_or(0.);
                let ends = [
                    physical([-3. * a.cos(), -3. * a.sin()]),
                    physical([3. * a.cos(), 3. * a.sin()]),
                ];
                let _ = s.set_points(stick, polyline(ends.to_vec()).build().points());
                for (j, &bar) in moving_bars.iter().enumerate() {
                    let length = super::straight_sticks::LENGTH * (a - PI * j as f64 / m as f64).sin().abs();
                    let y = -1.65 - j as f64 * 0.38;
                    let _ = s.set_points(
                        bar,
                        polyline(vec![v(-5.6, y), v(-5.6 + length * 0.64 + 1e-8, y)])
                            .build()
                            .points(),
                    );
                }
                s.move_to(marker, plot(m, a, (ratio(m, a) - 1.) * 100.), ORIGIN);
            },
            true,
        )?;
        let mut all = chart.clone();
        all.extend(&bars);
        all.extend([stick, driver]);
        fade_in_all(s, &all, 0.5)?;
        Ok(Self {
            stick,
            tracker,
            driver,
            bars,
            chart,
        })
    }
    fn turn(&self, s: &mut Stage<'_>, alpha: f64, seconds: f64) -> fmn::Result<()> {
        s.play(glide(self.tracker, alpha, seconds, linear)?)?;
        Ok(())
    }
    fn end(self, s: &mut Stage<'_>) -> fmn::Result<()> {
        stop(s, self.driver);
        let mut all = self.chart;
        all.extend(self.bars);
        all.extend([self.stick, self.driver]);
        fade_out_all(s, &all, 0.4)
    }
}
impl SceneConstruct for FixedDirections {
    fn name(&self) -> &str {
        "23_fixed_directions"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        say(&k, s, "23_1")?;
        let title = text(
            &k,
            s,
            "Can planned directions replace random turns?",
            34.,
            0.,
            3.3,
            Palette::ink(),
        )?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * 0.32)
                .height(model::CARD_H * 0.32)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(v(-3.5, 0.65)),
        );
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            4.,
        );
        let directions = text(&k, s, "0°    45°    90°    135°", 28., 3.2, 1.8, Palette::gold())?;
        let rule = text(
            &k,
            s,
            "full slide average in EACH direction",
            24.,
            0.,
            -3.25,
            Palette::ink(),
        )?;
        fade_in_all(s, &[title, card, coast, directions, rule], 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "23_2")?;
        fade_out_all(s, &[directions], 0.3)?;
        let heading = text(
            &k,
            s,
            "computed averages over every slide",
            23.,
            3.2,
            2.1,
            Palette::ink(),
        )?;
        fade_in_all(s, &[heading], 0.3)?;
        let beta = s.add_value_tracker(0.);
        let slide = s.add_value_tracker(0.);
        let lines: Vec<_> = (0..25)
            .map(|_| path(s, vec![physical([0., 0.]), physical([0.001, 0.])], Palette::dim(), 1.4))
            .collect();
        let dots: Vec<_> = (0..40).map(|_| dot(s, physical([0., 0.]), Palette::gold())).collect();
        let driver = s.arena_mut().add(VMobject::new());
        let ls = lines.clone();
        let ds = dots.clone();
        let curve = c.clone();
        s.add_updater(
            driver,
            move |s, _| {
                let pose = Pose::new(
                    s.tracker_value(beta).unwrap_or(0.),
                    s.tracker_value(slide).unwrap_or(0.),
                );
                let actual = placement::lines_at(pose);
                let cross = sheet::crossings(&curve, pose.beta, pose.slide);
                for (i, &mob) in ls.iter().enumerate() {
                    if let Some((_, ends)) = actual.get(i) {
                        let _ = s.set_points(mob, polyline(ends.iter().copied().map(physical)).build().points());
                        s.set_family_opacity(mob, 1.);
                    } else {
                        s.set_family_opacity(mob, 0.);
                    }
                }
                for (i, &mob) in ds.iter().enumerate() {
                    if let Some(crossing) = cross.get(i) {
                        s.move_to(mob, physical(crossing.at), ORIGIN);
                        s.set_family_opacity(mob, 1.);
                    } else {
                        s.set_family_opacity(mob, 0.);
                    }
                }
            },
            true,
        )?;
        for &mob in lines.iter().chain(dots.iter()) {
            s.add_to_scene(mob)?;
        }
        s.add_to_scene(driver)?;
        let mut rows = vec![heading];
        for (i, degrees) in facts::SHOWN_DIRECTIONS.into_iter().enumerate() {
            s.play(glide(beta, f64::from(degrees).to_radians(), 0.6, linear)?)?;
            s.play(glide(slide, model::GAP * (i + 1) as f64, 2.4, linear)?)?;
            let row = text(
                &k,
                s,
                &format!(
                    "{degrees}°  →  {} crossings",
                    f.get(&format!("turn.{degrees}.slide_mean")).unwrap()
                ),
                24.,
                3.2,
                1.35 - i as f64 * 0.7,
                Palette::teal(),
            )?;
            fade_in_all(s, &[row], 0.3)?;
            rows.push(row);
            pause(s, 0.5)?;
        }
        k.hold(s)?;
        pause(s, 2.)?;
        stop(s, driver);
        let mut moving = lines;
        moving.extend(dots);
        moving.push(driver);
        fade_out_all(s, &moving, 0.4)?;

        let spoken = say(&k, s, "23_3")?;
        spoken.wait_for(s, "twenty-five point five three")?;
        let estimate = text(
            &k,
            s,
            &format!(
                "four-direction estimate: {} cm",
                f.get("ladder.4.coast_estimate").unwrap()
            ),
            25.,
            0.,
            -2.1,
            Palette::gold(),
        )?;
        let error = text(
            &k,
            s,
            &format!(
                "exact coast length ≈ {} cm · error {}%",
                f.get("coast.length_2dp").unwrap(),
                f.get("ladder.4.coast_error_percent").unwrap()
            ),
            24.,
            0.,
            -2.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[estimate, error], 0.4)?;
        k.hold(s)?;
        pause(s, 3.)?;
        fade_out_all(s, &rows, 0.4)?;
        fade_out_all(s, &[estimate, error, coast], 0.4)?;

        say(&k, s, "23_4")?;
        let length = text(
            &k,
            s,
            &format!("same {} cm stick", f.get("stick.length").unwrap()),
            24.,
            -3.5,
            2.4,
            Palette::gold(),
        )?;
        fade_in_all(s, &[length], 0.3)?;
        let demo = Demo::new(&k, s, 4)?;
        k.hold(s)?;
        demo.turn(s, PI / 2., 9.)?;
        pause(s, 2.)?;
        say(&k, s, "23_5")?;
        demo.turn(s, 0., 2.)?;
        pause(s, 3.)?;
        demo.turn(s, PI / 8., 3.)?;
        k.hold(s)?;
        pause(s, 3.)?;
        let spoken = say(&k, s, "23_6")?;
        spoken.wait_for(s, "about five point two percent low")?;
        let band = text(
            &k,
            s,
            &format!(
                "rounded extremes: {}% to +{}%",
                f.get("ladder.4.low_percent").unwrap(),
                f.get("ladder.4.high_percent").unwrap()
            ),
            26.,
            3.2,
            -2.25,
            Palette::gold(),
        )?;
        fade_in_all(s, &[band], 0.4)?;
        demo.turn(s, PI / 2., 7.)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "23_7")?;
        demo.end(s)?;
        fade_out_all(s, &[band, length], 0.3)?;
        fade_in_all(s, &[coast], 0.4)?;
        let formula = text(
            &k,
            s,
            "whole ratio = length-weighted local ratios",
            24.,
            3.2,
            1.8,
            Palette::ink(),
        )?;
        let weights = text(
            &k,
            s,
            "positive lengths add · repeated travel counts",
            23.,
            3.2,
            0.8,
            Palette::ink(),
        )?;
        let scoped = text(
            &k,
            s,
            "finite rectifiable paths · whole path counted",
            23.,
            3.2,
            -0.2,
            Palette::gold(),
        )?;
        let parts: Vec<_> = c
            .parts
            .iter()
            .enumerate()
            .map(|(i, p)| {
                path(
                    s,
                    (0..=40).map(|j| physical(p.seg.point(j as f64 / 40.))).collect(),
                    if i % 2 == 0 { Palette::gold() } else { Palette::teal() },
                    4.,
                )
            })
            .collect();
        fade_in_all(s, &parts, 0.8)?;
        fade_in_all(s, &[formula, weights, scoped], 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        fade_out_all(s, &parts, 0.4)?;
        fade_out_all(s, &[formula, weights, scoped], 0.4)?;

        say(&k, s, "23_8")?;
        let question = text(
            &k,
            s,
            "Only two directions at right angles: how far off?",
            29.,
            0.,
            -2.55,
            Palette::ink(),
        )?;
        fade_in_all(s, &[question], 0.4)?;
        let thinking = think(&k, s, v(3.2, 0.5))?;
        let spoken = say(&k, s, "23_9")?;
        spoken.wait_for(s, "About twenty-one point five percent low")?;
        fade_out_all(s, &[thinking, question, coast], 0.3)?;
        fade_in_all(s, &[length], 0.3)?;
        let demo = Demo::new(&k, s, 2)?;
        let band = text(
            &k,
            s,
            &format!(
                "rounded extremes: {}% to +{}%",
                f.get("ladder.2.low_percent").unwrap(),
                f.get("ladder.2.high_percent").unwrap()
            ),
            25.,
            3.2,
            -2.1,
            Palette::gold(),
        )?;
        fade_in_all(s, &[band], 0.4)?;
        demo.turn(s, PI / 4., 4.)?;
        pause(s, 3.)?;
        demo.turn(s, PI / 2., 4.)?;
        k.hold(s)?;
        pause(s, 2.)?;
        demo.end(s)?;
        fade_out_all(s, &[band], 0.3)?;

        let spoken = say(&k, s, "23_10")?;
        let along = path(s, vec![physical([-3., 0.]), physical([3., 0.])], Palette::gold(), 5.);
        let guides: Vec<_> = [-1.3, 0.7, 2.7]
            .into_iter()
            .map(|y| path(s, vec![physical([-7.5, y]), physical([7.5, y])], Palette::dim(), 1.4))
            .collect();
        let zero = text(
            &k,
            s,
            "parallel stick: almost every slide misses",
            24.,
            3.2,
            1.5,
            Palette::ink(),
        )?;
        let range = text(
            &k,
            s,
            &format!(
                "one direction: {}% to about +{}%",
                f.get("ladder.1.low_percent").unwrap(),
                f.get("ladder.1.high_percent").unwrap()
            ),
            25.,
            3.2,
            0.5,
            Palette::gold(),
        )?;
        fade_in_all(s, &guides, 0.4)?;
        fade_in_all(s, &[along, zero, range], 0.4)?;
        spoken.wait_for(s, "a perpendicular stick")?;
        fade_out_all(s, &[zero], 0.3)?;
        let angle = s.add_value_tracker(0.);
        let driver = s.arena_mut().add(VMobject::new());
        s.add_updater(
            driver,
            move |s, _| {
                let a = s.tracker_value(angle).unwrap_or(0.);
                let _ = s.set_points(
                    along,
                    polyline(vec![
                        physical([-3. * a.cos(), -3. * a.sin()]),
                        physical([3. * a.cos(), 3. * a.sin()]),
                    ])
                    .build()
                    .points(),
                );
            },
            true,
        )?;
        s.add_to_scene(driver)?;
        s.play(glide(angle, PI / 2., 3., linear)?)?;
        stop(s, driver);
        let across = text(
            &k,
            s,
            "perpendicular stick: maximum projection",
            24.,
            3.2,
            1.5,
            Palette::ink(),
        )?;
        fade_in_all(s, &[across], 0.3)?;
        k.hold(s)?;
        pause(s, 3.)?;
        fade_out_all(s, &guides, 0.4)?;
        fade_out_all(s, &[along, across, range, length], 0.4)?;

        say(&k, s, "23_11")?;
        fade_in_all(s, &[coast], 0.4)?;
        let answer = text(
            &k,
            s,
            &format!(
                "same coast · four directions · {} cm",
                f.get("ladder.4.coast_estimate").unwrap()
            ),
            25.,
            3.2,
            1.8,
            Palette::teal(),
        )?;
        let caveat = text(
            &k,
            s,
            "direction bound ≠ single-placement guarantee",
            23.,
            3.2,
            0.7,
            Palette::ink(),
        )?;
        fade_in_all(s, &[answer, caveat], 0.4)?;
        fade_out_all(s, &[rule], 0.3)?;
        let note = precisely(&k, s, NOTE)?;
        let full = now(s) + 0.5;
        fade_in_all(s, &[note], 0.5)?;
        k.hold(s)?;
        wait_until(s, full + (NOTE.chars().count() + 10) as f64 / 15. + 1.)?;
        pause(s, 2.)?;
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_follow_all_sector_angles_and_endpoints() {
        for m in 1..=24 {
            let (lo, hi) = bounds(m);
            assert!((ratio(m, 0.) - lo).abs() < 1e-12);
            assert!((ratio(m, PI / (2. * m as f64)) - hi).abs() < 1e-12);
            for i in 0..=2000 {
                let a = 2. * PI * i as f64 / 2000.;
                let q = ratio(m, a);
                assert!(q >= lo - 1e-12 && q <= hi + 1e-12);
                assert!((q - ratio(m, a + PI / m as f64)).abs() < 1e-12);
            }
        }
        assert!((bounds(4).1 - 1.) * 100. > 2.6); // nearest rounding is not an outward bound
    }
    #[test]
    fn segment_geometry_and_coast_share_the_measure() {
        for m in [1, 2, 4, 6] {
            for angle in [0_f64, 13., 22.5, 45., 89.] {
                let a = angle.to_radians();
                let p = model::Seg::Line {
                    from: [-3. * a.cos(), -3. * a.sin()],
                    to: [3. * a.cos(), 3. * a.sin()],
                };
                let c = model::Curve {
                    parts: vec![model::Part { seg: p, start: 0. }],
                    length: 6.,
                };
                for gap in [0.5, 2., 5.] {
                    assert!((c.fixed_directions_estimate(m, gap) / 6. - ratio(m, a)).abs() < 1e-12);
                }
            }
            let c = model::coast();
            let q = c.fixed_directions_estimate(m, model::GAP) / c.length;
            let (lo, hi) = bounds(m);
            assert!(q >= lo && q <= hi);
        }
    }
}
