//! A calibrated six-direction sheet: orientation bias and placement variation stay separate.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::{self, glide};
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::{PI, TAU};

pub struct TheLongimeter {
    pub kit: Kit,
}
pub const DIRECTIONS: usize = 6;
pub const POSES: [[f64; 2]; 3] = [[0.137, 0.271], [0.537, 0.671], [1.037, 0.471]];
const NOTE: &str = "Finite rectifiable paths; count all traversals. Fixed-direction means use uniform full-gap slides. Exact calibration also averages orientation uniformly over a half turn. A single placement has no stated error bound.";
pub fn gap_mm() -> f64 {
    2. * DIRECTIONS as f64 / PI
}
pub fn gap_cm() -> f64 {
    gap_mm() / 10.
}
pub fn slide_mean_mm(c: &model::Curve) -> f64 {
    c.fixed_directions_estimate(DIRECTIONS, gap_cm()) * 10.
}
/// Analytic line/arc intersections. Endpoints/tangencies excluded: zero-measure offsets.
pub fn intersections(c: &model::Curve, beta: f64, h: f64) -> Vec<model::P2> {
    let n = [-beta.sin(), beta.cos()];
    let dot = |p: model::P2| p[0] * n[0] + p[1] * n[1];
    let psi = n[1].atan2(n[0]);
    let mut out = Vec::new();
    for part in &c.parts {
        match part.seg {
            model::Seg::Line { from, to } => {
                let (a, b) = (dot(from), dot(to));
                if (a - h) * (b - h) < 0. {
                    let t = (h - a) / (b - a);
                    out.push([from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])]);
                }
            }
            model::Seg::Arc {
                centre,
                radius,
                from,
                to,
            } => {
                let q = (h - dot(centre)) / radius;
                if q.abs() < 1. {
                    for theta in [psi + q.acos(), psi - q.acos()] {
                        let u = (theta - from.min(to)).rem_euclid(TAU);
                        if u > 0. && u < (to - from).abs() {
                            out.push([centre[0] + radius * theta.cos(), centre[1] + radius * theta.sin()]);
                        }
                    }
                }
            }
        }
    }
    out
}
/// Translation in gap units, shared by the entire sheet; no independent family draws.
pub fn points(c: &model::Curve, shift: model::P2, grids: usize) -> Vec<model::P2> {
    let mut out = Vec::new();
    for j in 0..DIRECTIONS {
        if j % 3 >= grids {
            continue;
        }
        let beta = PI * j as f64 / DIRECTIONS as f64;
        let offset = (-beta.sin() * shift[0] + beta.cos() * shift[1]) * gap_cm();
        for index in -30..=30 {
            out.extend(intersections(c, beta, offset + f64::from(index) * gap_cm()));
        }
    }
    out
}
fn physical(p: model::P2) -> Vec3 {
    v(-3.5 + 0.32 * p[0], 0.65 + 0.32 * p[1])
}
fn path(s: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(pts).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, words: &str, x: f64, y: f64, size: f64) -> fmn::Result<Mob> {
    k.text(s, words, size, Palette::ink(), v(x, y))
}
struct Sheet {
    x: Mob,
    y: Mob,
    grids: Mob,
    alpha: Mob,
    driver: Mob,
    lines: Vec<Mob>,
    dots: Vec<Mob>,
    count: Mob,
}
impl Sheet {
    fn new(k: &Kit, s: &mut Stage<'_>, c: &model::Curve) -> fmn::Result<Self> {
        let x = s.add_value_tracker(POSES[0][0]);
        let y = s.add_value_tracker(POSES[0][1]);
        let grids = s.add_value_tracker(0.);
        let alpha = s.add_value_tracker(0.);
        let colors = [Palette::teal(), Palette::gold(), Palette::ink()];
        let lines: Vec<_> = (0..6)
            .flat_map(|j| (0..61).map(move |_| j))
            .map(|j| path(s, vec![physical([0., 0.]), physical([0.001, 0.])], colors[j % 3], 0.65))
            .collect();
        let dots: Vec<_> = (0..400)
            .map(|_| {
                s.arena_mut().add(
                    Circle::new()
                        .radius(0.022)
                        .arc_center(physical([0., 0.]))
                        .style(flat(Palette::gold(), 1.))
                        .build(),
                )
            })
            .collect();
        for &m in lines.iter().chain(&dots) {
            s.add_to_scene(m)?;
            s.set_family_opacity(m, 0.);
        }
        let driver = s.arena_mut().add(VMobject::new());
        let ls = lines.clone();
        let ds = dots.clone();
        let curve = c.clone();
        let mut previous = None;
        s.add_updater(
            driver,
            move |s, _| {
                let shift = [s.tracker_value(x).unwrap_or(0.), s.tracker_value(y).unwrap_or(0.)];
                let g = s.tracker_value(grids).unwrap_or(0.) as usize;
                let a = s.tracker_value(alpha).unwrap_or(0.);
                // Preserve identities and cached geometry throughout still holds.
                if previous == Some((shift, g, a)) {
                    return;
                }
                previous = Some((shift, g, a));
                for j in 0..6 {
                    let beta = PI * j as f64 / 6.;
                    let offset = (-beta.sin() * shift[0] + beta.cos() * shift[1]) * gap_cm();
                    for index in -30..=30 {
                        let mob = ls[j * 61 + (index + 30) as usize];
                        if j % 3 < g
                            && let Some([p, q]) = placement::clipped(beta, offset + f64::from(index) * gap_cm())
                        {
                            let _ = s.set_points(mob, polyline(vec![physical(p), physical(q)]).build().points());
                            s.set_family_opacity(mob, a * 0.28);
                        } else {
                            s.set_family_opacity(mob, 0.);
                        }
                    }
                }
                let pts = points(&curve, shift, g);
                assert!(pts.len() <= ds.len(), "crossing pool exhausted");
                for (i, &mob) in ds.iter().enumerate() {
                    if let Some(&p) = pts.get(i) {
                        let _ = s.move_to(mob, physical(p), ORIGIN);
                        s.set_family_opacity(mob, a);
                    } else {
                        s.set_family_opacity(mob, 0.);
                    }
                }
            },
            true,
        )?;
        s.add_to_scene(driver)?;
        let curve = c.clone();
        let count = placement::live_count(
            k,
            s,
            move |s| {
                points(
                    &curve,
                    [s.tracker_value(x).unwrap_or(0.), s.tracker_value(y).unwrap_or(0.)],
                    s.tracker_value(grids).unwrap_or(0.) as usize,
                )
                .len()
            },
            alpha,
            42.,
            v(3.3, 1.1),
        )?;
        Ok(Self {
            x,
            y,
            grids,
            alpha,
            driver,
            lines,
            dots,
            count,
        })
    }
    fn show_grids(&self, s: &mut Stage<'_>, n: usize) -> fmn::Result<()> {
        s.play(glide(self.grids, n as f64, 0.1, linear)?)?;
        s.play(glide(self.alpha, 1., 0.5, linear)?)?;
        Ok(())
    }
    fn shift(&self, s: &mut Stage<'_>, pose: model::P2) -> fmn::Result<()> {
        placement::at_once(
            s,
            &[],
            &[],
            vec![glide(self.x, pose[0], 5., linear)?, glide(self.y, pose[1], 5., linear)?],
            5.,
        )?;
        Ok(())
    }
    fn hide(&self, s: &mut Stage<'_>) -> fmn::Result<()> {
        s.play(glide(self.alpha, 0., 0.4, linear)?)?;
        Ok(())
    }
    fn end(self, s: &mut Stage<'_>) {
        stop(s, self.driver);
        stop(s, self.count);
        for mob in self.lines.into_iter().chain(self.dots).chain([self.count]) {
            s.set_family_opacity(mob, 0.);
        }
    }
}
impl SceneConstruct for TheLongimeter {
    fn name(&self) -> &str {
        "24_the_longimeter"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let title = label(&k, s, "A counting sheet", 0., 3.45, 36.)?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * 0.32)
                .height(model::CARD_H * 0.32)
                .style(line_style(Palette::dim(), 1.5))
                .build()?,
        );
        s.move_to(card, v(-3.5, 0.65), ORIGIN);
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            3.,
        );
        let estimate = label(
            &k,
            s,
            &format!("six directions · {} cm", f.get("ladder.6.coast_estimate").unwrap()),
            3.1,
            2.25,
            27.,
        )?;
        let band = label(
            &k,
            s,
            &format!(
                "about {}% to +{}%",
                f.get("ladder.6.low_percent").unwrap(),
                f.get("ladder.6.high_percent").unwrap()
            ),
            3.1,
            1.35,
            25.,
        )?;
        let rule = label(&k, s, "after full slide averaging", 3.1, 0.5, 23.)?;
        say(&k, s, "24_1")?;
        fade_in_all(s, &[title, card, coast, estimate, band, rule], 0.6)?;
        k.hold(s)?;
        pause(s, 3.)?;
        fade_out_all(s, &[estimate, band, rule], 0.4)?;
        let sheet = Sheet::new(&k, s, &c)?;
        let spoken = say(&k, s, "24_2")?;
        let count_word = label(&k, s, "actual crossings", 3.1, 1.7, 25.)?;
        fade_in_all(s, &[count_word], 0.3)?;
        let mut grid_labels = Vec::new();
        for (n, anchor, words) in [
            (1, "Start with a square grid", "0° + 90°"),
            (2, "another turned thirty degrees,", "30° + 120°"),
            (3, "the other way", "−30° + 60°"),
        ] {
            spoken.wait_for(s, anchor)?;
            sheet.show_grids(s, n)?;
            let m = label(&k, s, words, -3.5, -1.25 - (n as f64 - 1.) * 0.42, 22.)?;
            fade_in_all(s, &[m], 0.3)?;
            grid_labels.push(m);
            pause(s, 1.)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        sheet.hide(s)?;
        fade_out_all(s, &grid_labels, 0.3)?;
        fade_out_all(s, &[count_word], 0.3)?;
        say(&k, s, "24_3")?;
        let fixed = label(&k, s, "fixed direction: mean count = T(β) / d", 2.8, 2.25, 24.)?;
        let def = label(&k, s, "T(β): total across-line projection travel", 2.8, 1.35, 22.)?;
        let slide = label(&k, s, "average uniformly over a full gap", 2.8, 0.45, 23.)?;
        fade_in_all(s, &[fixed, def, slide], 0.4)?;
        k.hold(s)?;
        pause(s, 3.)?;
        fade_out_all(s, &[fixed, def, slide], 0.3)?;
        let spoken = say(&k, s, "24_4")?;
        spoken.wait_for(s, "two times the length over pi")?;
        let turn = label(&k, s, "orientation mean of T(β) = 2L / π", 2.9, 2.25, 25.)?;
        let family = label(&k, s, "mean per family = 2L / (πd)", 2.9, 1.3, 27.)?;
        let units = label(&k, s, "L: path length in mm · d: gap in mm", 2.9, 0.35, 23.)?;
        fade_in_all(s, &[turn, family, units], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "24_5")?;
        fade_out_all(s, &[turn, family], 0.3)?;
        let total = label(&k, s, "6 × 2L / (πd) = 12L / (πd)", 2.9, 2.15, 29.)?;
        let marg = label(&k, s, "turn uniformly + uniform offset for each family", 2.9, 1.25, 21.)?;
        fade_in_all(s, &[total, marg], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "24_6")?;
        let question = label(&k, s, "What gap makes count read millimetres?", 0., -2.4, 29.)?;
        fade_in_all(s, &[question], 0.3)?;
        let thinking = think(&k, s, v(3., -1.1))?;
        let spoken = say(&k, s, "24_7")?;
        spoken.wait_for(s, "Twelve over pi millimetres")?;
        fade_out_all(s, &[question, thinking, total, marg], 0.3)?;
        let gap = label(
            &k,
            s,
            &format!("d = 12/π mm ≈ {} mm", f.get("steinhaus.gap_mm").unwrap()),
            2.9,
            2.2,
            29.,
        )?;
        let cancel = label(&k, s, "12L / (π × 12/π) = L", 2.9, 1.2, 29.)?;
        let expected = label(&k, s, "expected count = numerical length in mm", 2.9, -0.5, 23.)?;
        fade_in_all(s, &[gap, cancel, expected], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        let spoken = say(&k, s, "24_8")?;
        fade_out_all(s, &[cancel, units, expected], 0.3)?;
        let round = label(&k, s, "exact spacing used here · printed value rounded", 2.9, 0.1, 21.)?;
        fade_in_all(s, &[round, count_word], 0.3)?;
        spoken.wait_for(s, "Let's slide")?;
        sheet.show_grids(s, 3)?;
        pause(s, 3.)?;
        sheet.shift(s, POSES[1])?;
        pause(s, 4.)?;
        k.hold(s)?;
        say(&k, s, "24_9")?;
        sheet.shift(s, POSES[2])?;
        pause(s, 4.)?;
        sheet.shift(s, POSES[0])?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "24_10")?;
        fade_out_all(s, &[round], 0.3)?;
        let mean = label(
            &k,
            s,
            &format!("fixed-six slide mean: {} mm", f.get("steinhaus.coast_mean_mm").unwrap()),
            2.9,
            -0.15,
            25.,
        )?;
        let truth = label(
            &k,
            s,
            &format!(
                "actual coast length: {} mm",
                f.get("steinhaus.coast_length_mm").unwrap()
            ),
            2.9,
            -1.05,
            25.,
        )?;
        let separate = label(&k, s, "direction bias ≠ placement variation", 0., -2.25, 25.)?;
        fade_in_all(s, &[mean, truth, separate], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "24_11")?;
        fade_out_all(s, &[separate], 0.3)?;
        let source = label(&k, s, "Grid directions: C. Staecker’s reconstruction", 0., -2.1, 22.)?;
        fade_in_all(s, &[source], 0.3)?;
        let note = precisely(&k, s, NOTE)?;
        let full = now(s) + 0.5;
        fade_in_all(s, &[note], 0.5)?;
        k.hold(s)?;
        wait_until(s, full + (NOTE.chars().count() + 10) as f64 / 15. + 1.)?;
        pause(s, 2.)?;
        sheet.end(s);
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn displayed_intersections_match_native_line_counter_and_calibration() {
        let c = model::coast();
        for pose in POSES {
            let mut count = 0;
            for j in 0..6 {
                let b = PI * j as f64 / 6.;
                let h0 = (-b.sin() * pose[0] + b.cos() * pose[1]) * gap_cm();
                for i in -30..=30 {
                    let h = h0 + f64::from(i) * gap_cm();
                    let pts = intersections(&c, b, h);
                    assert_eq!(pts.len(), c.meetings(b, h));
                    for p in &pts {
                        assert!((-b.sin() * p[0] + b.cos() * p[1] - h).abs() < 1e-10);
                    }
                    count += pts.len();
                }
            }
            assert_eq!(count, points(&c, pose, 3).len());
            assert!(count < 400);
        }
        assert!((6. * 2. / (PI * gap_mm()) - 1.).abs() < 1e-14);
        assert_ne!(points(&c, POSES[0], 3).len(), points(&c, POSES[1], 3).len());
    }
    #[test]
    fn fixed_means_and_orientation_calibration_are_different() {
        let c = model::coast();
        let sum: f64 = (0..6).map(|j| c.travel(j as f64 * 30.) / gap_cm()).sum();
        assert!((sum - slide_mean_mm(&c)).abs() < 1e-10);
        assert!((sum - c.length * 10.).abs() > 0.1);
        let (lo, hi) = super::super::fixed_directions::bounds(6);
        assert!(sum / (c.length * 10.) >= lo && sum / (c.length * 10.) <= hi);
        assert!((12. / (PI * 3.82) - 1.).abs() > 0.00001);
    }
}
