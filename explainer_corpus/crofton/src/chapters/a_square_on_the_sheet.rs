//! Closing transfer: the square's crossings, with the probability law explicit.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::{placement, sheet};
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::{PI, SQRT_2};

pub struct ASquareOnTheSheet {
    pub kit: Kit,
}
pub const SIDE: f64 = model::GAP;
pub const POSES: [(f64, f64); 4] = [(0., 0.4), (45., 1.), (30., 0.2), (75., 1.)];
pub const NOTE: &str = "Square side = sheet gap = 2 cm. Uniform half-turn orientation and independent uniform full-gap slide. Two or four crossings almost surely; corner contacts and coincident edges have probability zero. Count all traversals.";
pub fn square() -> model::Curve {
    let h = SIDE / 2.;
    let p = [[-h, -h], [h, -h], [h, h], [-h, h], [-h, -h]];
    model::Curve {
        parts: (0..4)
            .map(|i| model::Part {
                seg: model::Seg::Line {
                    from: p[i],
                    to: p[i + 1],
                },
                start: i as f64 * SIDE,
            })
            .collect(),
        length: 4. * SIDE,
    }
}
pub fn width(beta: f64) -> f64 {
    SIDE * (beta.sin().abs() + beta.cos().abs())
}
pub fn mean_count() -> f64 {
    2. * square().length / (PI * model::GAP)
}
pub fn probability_four() -> f64 {
    (mean_count() - 2.) / 2.
}
/// Closed-segment contacts, including corners, kept separate from transverse crossings.
/// A tolerance identifies displayed degenerate examples; generic tests stay clear of it.
pub fn contacts(beta: f64, slide: f64) -> (Vec<model::P2>, Vec<[model::P2; 2]>) {
    let n = [-beta.sin(), beta.cos()];
    let mut pts: Vec<model::P2> = Vec::new();
    let mut overlap = Vec::new();
    for part in square().parts {
        let model::Seg::Line { from, to } = part.seg else {
            unreachable!()
        };
        let a = from[0] * n[0] + from[1] * n[1];
        let b = to[0] * n[0] + to[1] * n[1];
        for j in -3..=3 {
            let h = slide + f64::from(j) * model::GAP;
            if (a - h).abs() < 1e-10 && (b - h).abs() < 1e-10 {
                overlap.push([from, to]);
                continue;
            }
            if (a - b).abs() < 1e-12 {
                continue;
            }
            let t = (h - a) / (b - a);
            if (-1e-10..=1. + 1e-10).contains(&t) {
                let p = [from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])];
                if !pts.iter().any(|q| (p[0] - q[0]).hypot(p[1] - q[1]) < 1e-9) {
                    pts.push(p);
                }
            }
        }
    }
    (pts, overlap)
}
fn at(p: model::P2, large: bool) -> Vec3 {
    if large {
        v(3. + 0.95 * p[0], 0.5 + 0.95 * p[1])
    } else {
        v(-3.5 + 0.33 * p[0], 0.5 + 0.33 * p[1])
    }
}
fn path(s: &mut Stage<'_>, p: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(p).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, text: &str, x: f64, y: f64, size: f64) -> fmn::Result<Mob> {
    k.text(s, text, size, Palette::ink(), v(x, y))
}
fn zoom_clip(beta: f64, h: f64) -> Option<[model::P2; 2]> {
    let n = [-beta.sin(), beta.cos()];
    let a = 1.8;
    let mut p = Vec::new();
    for x in [-a, a] {
        if n[1].abs() > 1e-12 {
            let y = (h - n[0] * x) / n[1];
            if y.abs() <= a {
                p.push([x, y]);
            }
        }
    }
    for y in [-a, a] {
        if n[0].abs() > 1e-12 {
            let x = (h - n[1] * y) / n[0];
            if x.abs() <= a && !p.iter().any(|q| (x - q[0]).hypot(y - q[1]) < 1e-9) {
                p.push([x, y]);
            }
        }
    }
    if p.len() == 2 { Some([p[0], p[1]]) } else { None }
}
fn coast_sheet(s: &mut Stage<'_>, c: &model::Curve) -> Vec<Mob> {
    let slide = sheet::shown_slide(c, 0.);
    let mut out = Vec::new();
    for j in -5..=5 {
        if let Some([p, q]) = placement::clipped(0., slide + f64::from(j) * model::GAP) {
            out.push(path(s, vec![at(p, false), at(q, false)], Palette::dim(), 1.2));
        }
    }
    for x in sheet::crossings(c, 0., slide) {
        out.push(
            s.arena_mut().add(
                Circle::new()
                    .radius(0.045)
                    .arc_center(at(x.at, false))
                    .style(flat(Palette::gold(), 1.))
                    .build(),
            ),
        );
    }
    out
}
struct Live {
    beta: Mob,
    slide: Mob,
    small: Mob,
    large: Mob,
    driver: Mob,
    objects: Vec<Mob>,
}
impl Live {
    fn new(s: &mut Stage<'_>) -> fmn::Result<Self> {
        let beta = s.add_value_tracker(0.);
        let slide = s.add_value_tracker(0.4);
        let small = s.add_value_tracker(0.);
        let large = s.add_value_tracker(0.);
        let mut views = Vec::new();
        let mut objects = Vec::new();
        for big in [false, true] {
            // Separate open edges keep zero-area fills when the opacity tracker runs.
            let outline: Vec<_> = square()
                .parts
                .iter()
                .map(|part| {
                    path(
                        s,
                        vec![at(part.seg.point(0.), big), at(part.seg.point(1.), big)],
                        Palette::teal(),
                        3.,
                    )
                })
                .collect();
            let lines: Vec<_> = (0..21)
                .map(|_| path(s, vec![at([0., 0.], big), at([0.001, 0.], big)], Palette::dim(), 1.2))
                .collect();
            let dots: Vec<_> = (0..8)
                .map(|_| {
                    s.arena_mut().add(
                        Circle::new()
                            .radius(if big { 0.07 } else { 0.04 })
                            .arc_center(at([0., 0.], big))
                            .style(flat(Palette::gold(), 1.))
                            .build(),
                    )
                })
                .collect();
            let edges: Vec<_> = (0..4)
                .map(|_| path(s, vec![at([0., 0.], big), at([0.001, 0.], big)], Palette::gold(), 5.))
                .collect();
            objects.extend(outline.iter().copied());
            objects.extend(lines.iter().chain(&dots).chain(&edges).copied());
            views.push((big, outline, lines, dots, edges));
        }
        for &m in &objects {
            s.add_to_scene(m)?;
            s.set_family_opacity(m, 0.);
        }
        let driver = s.arena_mut().add(VMobject::new());
        let mut prev = None;
        s.add_updater(
            driver,
            move |s, _| {
                let b = s.tracker_value(beta).unwrap_or(0.);
                let u = s.tracker_value(slide).unwrap_or(0.4);
                let sa = s.tracker_value(small).unwrap_or(0.);
                let la = s.tracker_value(large).unwrap_or(0.);
                if prev == Some((b, u, sa, la)) {
                    return;
                }
                prev = Some((b, u, sa, la));
                let (points, overlaps) = contacts(b, u);
                for (big, outline, lines, dots, edges) in &views {
                    let a = if *big { la } else { sa };
                    for &edge in outline {
                        s.set_family_opacity(edge, a);
                    }
                    for (j, &m) in lines.iter().enumerate() {
                        let h = u + (j as f64 - 10.) * model::GAP;
                        let clipped = if *big {
                            zoom_clip(b, h)
                        } else {
                            placement::clipped(b, h)
                        };
                        if let Some([p, q]) = clipped {
                            let _ = s.set_points(m, polyline(vec![at(p, *big), at(q, *big)]).build().points());
                            s.set_family_opacity(m, a * 0.8);
                        } else {
                            s.set_family_opacity(m, 0.);
                        }
                    }
                    for (j, &m) in dots.iter().enumerate() {
                        if let Some(&p) = points.get(j) {
                            let _ = s.move_to(m, at(p, *big), ORIGIN);
                            s.set_family_opacity(m, if overlaps.is_empty() { a } else { 0. });
                        } else {
                            s.set_family_opacity(m, 0.);
                        }
                    }
                    for (j, &m) in edges.iter().enumerate() {
                        if let Some([p, q]) = overlaps.get(j) {
                            let _ = s.set_points(m, polyline(vec![at(*p, *big), at(*q, *big)]).build().points());
                            s.set_family_opacity(m, a);
                        } else {
                            s.set_family_opacity(m, 0.);
                        }
                    }
                }
            },
            true,
        )?;
        s.add_to_scene(driver)?;
        Ok(Self {
            beta,
            slide,
            small,
            large,
            driver,
            objects,
        })
    }
    fn show(&self, s: &mut Stage<'_>, small: f64, large: f64) -> fmn::Result<()> {
        s.play(placement::glide(self.small, small, 0.3, linear)?)?;
        s.play(placement::glide(self.large, large, 0.3, linear)?)?;
        Ok(())
    }
    fn pose(&self, s: &mut Stage<'_>, degrees: f64, u: f64) -> fmn::Result<()> {
        s.play(placement::glide(self.beta, degrees.to_radians(), 1.8, linear)?)?;
        s.play(placement::glide(self.slide, u, 1.8, linear)?)?;
        Ok(())
    }
    fn end(self, s: &mut Stage<'_>) {
        stop(s, self.driver);
        for m in self.objects {
            s.set_family_opacity(m, 0.);
        }
    }
}
fn panel(k: &Kit, s: &mut Stage<'_>, words: &[String]) -> fmn::Result<Vec<Mob>> {
    words
        .iter()
        .enumerate()
        .map(|(i, x)| label(k, s, x, 3., 1.55 - i as f64 * 0.75, if i == 0 { 28. } else { 24. }))
        .collect()
}
impl SceneConstruct for ASquareOnTheSheet {
    fn name(&self) -> &str {
        "28_a_square_on_the_sheet"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let title = label(&k, s, "A square on the sheet", 0., 3.45, 35.)?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * 0.33)
                .height(model::CARD_H * 0.33)
                .style(line_style(Palette::dim(), 1.5))
                .build()?
                .moved_to(v(-3.5, 0.5)),
        );
        let coast = path(
            s,
            c.parts
                .iter()
                .flat_map(|p| (0..=120).map(move |j| at(p.seg.point(j as f64 / 120.), false)))
                .collect(),
            palette::coast(),
            3.,
        );
        let caption = label(&k, s, "the original 16 × 9 cm card", -3.5, 2.45, 24.)?;
        let opening = coast_sheet(s, &c);
        say(&k, s, "28_1")?;
        fade_in_all(s, &[title, card, coast, caption], 0.5)?;
        fade_in_all(s, &opening, 0.4)?;
        let mut words = panel(
            &k,
            s,
            &[
                format!(
                    "{} crossings → ≈ {} cm",
                    f.get("open.crossings").unwrap(),
                    f.get("open.estimate").unwrap()
                ),
                format!("drawn length ≈ {} cm", f.get("coast.length").unwrap()),
                "One placement gives an estimate".into(),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "28_2")?;
        fade_out_all(s, &words, 0.3)?;
        words = panel(
            &k,
            s,
            &[
                "2 cm gaps → π cm per crossing".into(),
                "Average over uniform turns AND slides".into(),
                "Finite length · all traversals counted".into(),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "28_3")?;
        fade_out_all(s, &words, 0.3)?;
        words = panel(
            &k,
            s,
            &[
                format!(
                    "4 directions + full slides: ≈ {} cm",
                    f.get("ladder.4.coast_estimate").unwrap()
                ),
                "All directions + full slides: length".into(),
                "An average identity, not each placement".into(),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "28_4")?;
        fade_out_all(s, &words, 0.3)?;
        fade_out_all(s, &opening, 0.3)?;
        fade_out_all(s, &[coast], 0.3)?;
        let live = Live::new(s)?;
        live.show(s, 1., 1.)?;
        let zoom = label(&k, s, "same 2 cm square · enlarged", 3., 2.65, 25.)?;
        let law = label(
            &k,
            s,
            "Uniform half-turn · independent uniform slide over one 2 cm gap",
            0.,
            -2.,
            24.,
        )?;
        let selected = label(&k, s, "Selected illustrations, not random samples", 0., -2.6, 22.)?;
        fade_in_all(s, &[zoom, law, selected], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "28_5")?;
        for &(b, u) in &POSES {
            live.pose(s, b, u)?;
            let n = contacts(b.to_radians(), u).0.len();
            let count = label(
                &k,
                s,
                &format!("{n} transverse crossings at this placement"),
                3.,
                -1.45,
                22.,
            )?;
            fade_in_all(s, &[count], 0.25)?;
            pause(s, 1.2)?;
            fade_out_all(s, &[count], 0.25)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "28_6")?;
        live.pose(s, 45., 1.)?;
        let width_label = label(
            &k,
            s,
            &format!(
                "width across lines: 2 to 2√2 ≈ {} cm",
                f.get("square.max_width").unwrap()
            ),
            0.,
            -1.45,
            24.,
        )?;
        let mut width_marks = vec![width_label];
        width_marks.push(path(
            s,
            vec![at([-1., 1.], true), at([1., -1.], true)],
            Palette::ink(),
            1.8,
        ));
        for p in [[-1., 1.], [1., -1.]] {
            width_marks.push(path(
                s,
                vec![
                    at([p[0] - 0.12, p[1] - 0.12], true),
                    at([p[0] + 0.12, p[1] + 0.12], true),
                ],
                Palette::ink(),
                2.,
            ));
        }
        fade_in_all(s, &width_marks, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        fade_out_all(s, &width_marks, 0.3)?;
        say(&k, s, "28_7")?;
        live.pose(s, 45., SQRT_2)?;
        let corner = label(
            &k,
            s,
            &format!(
                "corner contact: {} distinct points",
                f.get("square.corner_contacts").unwrap()
            ),
            3.,
            -1.45,
            23.,
        )?;
        fade_in_all(s, &[corner], 0.4)?;
        pause(s, 4.)?;
        fade_out_all(s, &[corner], 0.3)?;
        live.pose(s, 0., 1.)?;
        let edge = label(&k, s, "coincident edges: infinitely many points", 3., -1.45, 22.)?;
        fade_in_all(s, &[edge], 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        fade_out_all(s, &[edge], 0.3)?;
        say(&k, s, "28_8")?;
        live.pose(s, 45., 1.)?;
        let question = label(&k, s, "How often are there four crossings?", 0., -1.45, 28.)?;
        fade_in_all(s, &[question], 0.4)?;
        let prediction = think(&k, s, v(6., 0.5))?;
        say(&k, s, "28_9")?;
        fade_out_all(s, &[prediction, question, zoom, selected], 0.3)?;
        live.show(s, 1., 0.)?;
        words = panel(
            &k,
            s,
            &[
                format!(
                    "rim = 4 × {} = {} cm",
                    f.get("square.side").unwrap(),
                    f.get("square.rim").unwrap()
                ),
                format!("mean count = 8/π ≈ {}", f.get("square.mean").unwrap()),
                "Exact identity; decimal is rounded".into(),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "28_10")?;
        fade_out_all(s, &words, 0.3)?;
        words = panel(
            &k,
            s,
            &[
                "p = probability of four crossings".into(),
                "2(1 − p) + 4p = 2 + 2p".into(),
                "Baseline two, plus two with probability p".into(),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "28_11")?;
        fade_out_all(s, &words, 0.3)?;
        words = panel(
            &k,
            s,
            &[
                "2 + 2p = 8/π".into(),
                "p = 4/π − 1".into(),
                format!("≈ {}% of placements", f.get("square.four_share_percent").unwrap()),
            ],
        )?;
        fade_in_all(s, &words, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "28_12")?;
        fade_out_all(s, &words, 0.3)?;
        let p = probability_four();
        let x = 0.7;
        let y = 0.5;
        let total = 4.6;
        let mut bars = vec![label(
            &k,
            s,
            "Probability, not a finite sample frequency",
            3.,
            1.7,
            23.,
        )?];
        for (left, len, color) in [
            (x, total * (1. - p), Palette::teal()),
            (x + total * (1. - p), total * p, Palette::gold()),
        ] {
            bars.push(
                s.arena_mut().add(
                    Rectangle::new()
                        .width(len)
                        .height(0.6)
                        .style(flat(color, 0.8))
                        .build()?
                        .moved_to(v(left + len / 2., y)),
                ),
            );
        }
        bars.push(label(
            &k,
            s,
            &format!("2 crossings: ≈ {}%", f.get("square.two_share_percent").unwrap()),
            2.,
            -0.3,
            23.,
        )?);
        bars.push(label(
            &k,
            s,
            &format!("4 crossings: ≈ {}%", f.get("square.four_share_percent").unwrap()),
            4.4,
            -0.85,
            23.,
        )?);
        fade_in_all(s, &bars, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "28_13")?;
        fade_out_all(s, &bars, 0.3)?;
        fade_out_all(s, &[law], 0.3)?;
        live.show(s, 0., 1.)?;
        fade_in_all(s, &[coast, zoom], 0.4)?;
        fade_in_all(s, &opening, 0.4)?;
        let final_label = label(&k, s, "A count · an average · the stated random choice", 0., -1.65, 27.)?;
        fade_in_all(s, &[final_label], 0.4)?;
        let note = precisely(&k, s, NOTE)?;
        let full = now(s) + 0.5;
        fade_in_all(s, &[note], 0.5)?;
        k.hold(s)?;
        wait_until(s, full + (NOTE.chars().count() + 10) as f64 / 15. + 1.)?;
        pause(s, 2.)?;
        live.end(s);
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn width_count_and_conditional_probability() {
        let c = square();
        for i in 0..180 {
            let b = (i as f64 + 0.37) * PI / 180.;
            assert!(width(b) >= SIDE && width(b) < 2. * model::GAP);
            for j in 0..43 {
                let u = (j as f64 + 0.23) * model::GAP / 43.;
                let a = sheet::crossings(&c, b, u);
                let (p, e) = contacts(b, u);
                assert!(e.is_empty());
                assert_eq!(a.len(), p.len());
                assert!([2, 4].contains(&a.len()));
            }
        }
        assert!((probability_four() - (4. / PI - 1.)).abs() < 1e-14);
    }
    #[test]
    fn exceptional_contacts_are_separate_from_transverse_crossings() {
        let (p, e) = contacts(PI / 4., SQRT_2);
        assert_eq!(p.len(), 3);
        assert!(e.is_empty());
        assert_eq!(sheet::crossings(&square(), PI / 4., SQRT_2).len(), 2);
        let (_, e) = contacts(0., 1.);
        assert_eq!(e.len(), 2);
        assert_eq!(POSES.map(|(b, u)| contacts(b.to_radians(), u).0.len()), [2, 4, 2, 4]);
    }
}
