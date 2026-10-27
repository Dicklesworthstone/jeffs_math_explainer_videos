//! The integral is a compact description of the chart operations already seen.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::glide;
use crate::{
    coast_chart, facts,
    line_chart::{self, Address},
    model, palette,
};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::f64::consts::{PI, TAU};

pub struct TheFormula {
    pub kit: Kit,
}
pub const CAUCHY_REPORTED_YEAR: usize = 1832;
const SCALE: f64 = 0.36;
const NOTE: &str = "Rectifiable plane paths; length and crossings count traversals. Signed p spans the real line; normal angle spans [0, π), in radians. For the centred coin, |p| < r gives two crossings; tangencies have zero chart area.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.4 + SCALE * p[0], 0.35 + SCALE * p[1])
}
fn chart(t: f64, p: f64) -> Vec3 {
    v(0.8 + 5.4 * t / PI, 0.25 + 2. * p / line_chart::reach())
}
fn path(s: &mut Stage<'_>, ps: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(ps).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, color, v(x, y))
}
fn dot(s: &mut Stage<'_>, p: Vec3) -> Mob {
    s.arena_mut().add(
        Circle::new()
            .radius(0.055)
            .arc_center(p)
            .style(flat(Palette::gold(), 1.))
            .build(),
    )
}
/// Analytic intersections with the centred circle. The represented radius and
/// distance determine the exact branch; no tolerance thickens the tangent band.
pub fn coin_hits(a: Address, r: f64) -> Vec<model::P2> {
    assert!(r.is_finite() && r > 0. && a.distance.is_finite() && a.theta.is_finite());
    if a.distance.abs() > r {
        return vec![];
    }
    let foot = a.foot();
    if a.distance.abs() == r {
        return vec![foot];
    }
    let h = ((r - a.distance.abs()) * (r + a.distance.abs())).sqrt();
    [-h, h]
        .map(|q| [foot[0] - q * a.theta.sin(), foot[1] + q * a.theta.cos()])
        .to_vec()
}
pub fn coin_band(r: f64) -> f64 {
    assert!(r > 0. && r.is_finite());
    (r - (-r)) * PI
}
pub fn selected_coin_counts() -> String {
    let r = model::COIN / 2.;
    [0., r, 1.5 * r]
        .map(|distance| coin_hits(Address { theta: 0., distance }, r).len().to_string())
        .join(",")
}
fn coast_tiles(s: &mut Stage<'_>, g: &coast_chart::Grid) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for i in 0..g.columns {
        let mut j = 0;
        while j < g.rows {
            let n = g.cells[i * g.rows + j];
            let start = j;
            while j < g.rows && g.cells[i * g.rows + j] == n {
                j += 1;
            }
            if let Some(n) = n.filter(|n| *n > 0) {
                let lo = line_chart::reach() * (2. * start as f64 / g.rows as f64 - 1.);
                let hi = line_chart::reach() * (2. * j as f64 / g.rows as f64 - 1.);
                let a = PI * i as f64 / g.columns as f64;
                let b = PI * (i + 1) as f64 / g.columns as f64;
                out.push(
                    s.arena_mut().add(
                        Polygon::new(vec![chart(a, lo), chart(b, lo), chart(b, hi), chart(a, hi)])
                            .style(flat(Palette::teal(), 0.16 + 0.11 * n as f64))
                            .build(),
                    ),
                );
            }
        }
    }
    Ok(out)
}
fn inspection(s: &mut Stage<'_>, a: Address, hits: Vec<model::P2>) -> Vec<Mob> {
    let mut out = vec![
        path(
            s,
            a.line()
                .chord()
                .expect("illustration stays inside card")
                .into_iter()
                .map(physical)
                .collect(),
            Palette::ink(),
            2.,
        ),
        dot(s, chart(a.theta, a.distance)),
    ];
    out.extend(hits.into_iter().map(|p| dot(s, physical(p))));
    out
}
fn working() -> Result<(), String> {
    let f = facts::facts()?;
    let band = coin_band(model::COIN / 2.);
    if f.get("chart.coin_band") != Some(facts::fixed(band, 2).as_str())
        || f.get("chart.coin_total") != Some(facts::fixed(2. * band, 2).as_str())
        || f.get("formula.coin_counts") != Some(selected_coin_counts().as_str())
    {
        return Err("formula illustration differs from its computed facts".into());
    }
    Ok(())
}
impl SceneConstruct for TheFormula {
    fn name(&self) -> &str {
        "20_the_formula"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        working().map_err(fail)?;
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let grid = coast_chart::Grid::build(&c, coast_chart::COLUMNS, coast_chart::ROWS);
        say(&k, s, "20_1")?;
        let title = label(
            &k,
            s,
            "The formula is the picture, written down",
            36.,
            0.,
            3.3,
            Palette::ink(),
        )?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * SCALE)
                .height(model::CARD_H * SCALE)
                .corner_radius(0.05)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(physical([0., 0.])),
        );
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            4.,
        );
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
            label(&k, s, "normal angle θ (radians)", 22., 3.5, -2.1, Palette::teal())?,
            label(&k, s, "signed distance p (cm)", 22., 3.5, 2.65, Palette::gold())?,
            k.tex(s, "0", 22., Palette::dim(), &[], v(0.8, -1.96))?,
            k.tex(s, "\\pi", 22., Palette::dim(), &[], v(6.2, -1.96))?,
        ];
        let boundary = path(
            s,
            (0..=240)
                .map(|i| {
                    let t = PI * i as f64 / 240.;
                    chart(t, line_chart::half_width(t))
                })
                .chain((0..=240).rev().map(|i| {
                    let t = PI * i as f64 / 240.;
                    chart(t, -line_chart::half_width(t))
                }))
                .collect(),
            Palette::teal(),
            1.5,
        );
        fade_in_all(s, &[title, card, coast, boundary], 0.5)?;
        fade_in_all(s, &axes, 0.4)?;
        let heat = coast_tiles(s, &grid)?;
        fade_in_all(s, &heat, 0.7)?;
        let approx = label(
            &k,
            s,
            "same finite grid · shading = crossing count",
            23.,
            0.,
            -2.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[approx], 0.4)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "20_2")?;
        fade_out_all(s, &[approx], 0.3)?;
        let a = grid.representative(&c, 1);
        let chosen = inspection(
            s,
            a,
            coast_chart::intersections(&c, a).into_iter().map(|x| x.at).collect(),
        );
        fade_in_all(s, &chosen, 0.5)?;
        let n = k.tex(s, "n(\\theta,p)", 42., Palette::gold(), &[], v(0., -2.8))?;
        fade_in_all(s, &[n], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "20_3")?;
        fade_out_all(s, &chosen, 0.3)?;
        fade_out_all(s, &[n], 0.3)?;
        let inner = k.tex(
            s,
            "\\int_{-\\infty}^{\\infty} n(\\theta,p)\\,dp",
            40.,
            Palette::gold(),
            &[],
            v(0., -2.8),
        )?;
        let sweep = path(
            s,
            vec![
                chart(a.theta, -line_chart::reach()),
                chart(a.theta, line_chart::reach()),
            ],
            Palette::gold(),
            3.,
        );
        fade_in_all(s, &[inner, sweep], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "20_4")?;
        fade_out_all(s, &[inner], 0.3)?;
        let formula = k.tex(
            s,
            "L=\\frac{1}{2}\\int_0^{\\pi}\\int_{-\\infty}^{\\infty} n(\\theta,p)\\,dp\\,d\\theta",
            38.,
            Palette::ink(),
            &[],
            v(0., -2.85),
        )?;
        fade_in_all(s, &[formula], 0.6)?;
        let turn = s.add_value_tracker(0.);
        s.add_updater(
            sweep,
            move |s, m| {
                let t = s.tracker_value(turn).unwrap_or(0.);
                let _ = s.set_points(
                    m,
                    polyline(vec![chart(t, -line_chart::reach()), chart(t, line_chart::reach())])
                        .build()
                        .points(),
                );
            },
            true,
        )?;
        s.play(glide(turn, PI, 5., linear)?)?;
        stop(s, sweep);
        fade_out_all(s, &[sweep], 0.3)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "20_5")?;
        let identity = label(
            &k,
            s,
            "exact identity · finite grid estimates it",
            25.,
            -3.3,
            2.3,
            Palette::gold(),
        )?;
        fade_in_all(s, &[identity], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "20_6")?;
        fade_out_all(s, &[formula, identity], 0.4)?;
        let shadow=k.tex(s,"\\frac{1}{\\pi}\\int_0^{\\pi}\\left(\\int_{-\\infty}^{\\infty}n(\\theta,p)\\,dp\\right)d\\theta=\\frac{2L}{\\pi}",32.,Palette::ink(),&[],v(0.,-2.65))?;
        let average = label(
            &k,
            s,
            &format!(
                "coast: mean total shadow ≈ {} cm",
                f.get("crofton.mean_reach_3dp").unwrap()
            ),
            25.,
            0.,
            -3.3,
            Palette::gold(),
        )?;
        let credit_text = format!(
            "Cauchy, {} · reported by Barbier ({})",
            f.get("cauchy.reported_year").unwrap(),
            f.get("barbier.year").unwrap()
        );
        let credit = label(&k, s, &credit_text, 23., -3.3, 2.3, Palette::ink())?;
        fade_in_all(s, &[shadow, average, credit], 0.5)?;
        let deadline = now(s) + credit_text.chars().count() as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;
        pause(s, 2.)?;

        say(&k, s, "20_7")?;
        fade_out_all(s, &heat, 0.4)?;
        fade_out_all(s, &[shadow, average, credit, coast], 0.4)?;
        let r = model::COIN / 2.;
        let coin = path(
            s,
            (0..=240)
                .map(|i| {
                    let t = TAU * i as f64 / 240.;
                    physical([r * t.cos(), r * t.sin()])
                })
                .collect(),
            palette::coast(),
            4.,
        );
        let rim = label(
            &k,
            s,
            &format!("coin rim ≈ {} cm", f.get("coin.rim").unwrap()),
            28.,
            -3.4,
            2.3,
            palette::coast(),
        )?;
        let question = label(
            &k,
            s,
            "What will the weighted chart total be?",
            29.,
            0.,
            -2.9,
            Palette::ink(),
        )?;
        fade_in_all(s, &[coin, rim, question], 0.5)?;
        let thinking = think(&k, s, v(-5.7, -2.5))?;

        say(&k, s, "20_8")?;
        fade_out_all(s, &[question, thinking], 0.3)?;
        let answer = label(
            &k,
            s,
            &format!("weighted chart total ≈ {}", f.get("chart.coin_total").unwrap()),
            32.,
            0.,
            -2.85,
            Palette::gold(),
        )?;
        fade_in_all(s, &[answer], 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;

        let interior = say(&k, s, "20_9")?;
        fade_out_all(s, &[answer], 0.3)?;
        let band = s.arena_mut().add(
            Polygon::new(vec![chart(0., -r), chart(PI, -r), chart(PI, r), chart(0., r)])
                .style(flat(Palette::teal(), 0.6))
                .build(),
        );
        let band_label = label(&k, s, "interior band: n = 2", 28., 3.5, -0.45, Palette::teal())?;
        fade_in_all(s, &[band, band_label], 0.5)?;
        let moving = s.add_value_tracker(-0.75 * r);
        let angle = s.add_value_tracker(0.);
        let line = path(
            s,
            vec![
                physical([-0.75 * r, -model::CARD_H / 2.]),
                physical([-0.75 * r, model::CARD_H / 2.]),
            ],
            Palette::ink(),
            2.,
        );
        let mark = dot(s, chart(0., -0.75 * r));
        let first = coin_hits(
            Address {
                theta: 0.,
                distance: -0.75 * r,
            },
            r,
        );
        let dots = [dot(s, physical(first[0])), dot(s, physical(first[1]))];
        s.add_updater(
            line,
            move |s, m| {
                let p = s.tracker_value(moving).unwrap_or(0.);
                let theta = s.tracker_value(angle).unwrap_or(0.);
                let _ = s.set_points(
                    m,
                    polyline(
                        Address { theta, distance: p }
                            .line()
                            .chord()
                            .expect("interior coin line meets card")
                            .into_iter()
                            .map(physical),
                    )
                    .build()
                    .points(),
                );
            },
            true,
        )?;
        s.add_updater(
            mark,
            move |s, m| {
                let p = s.tracker_value(moving).unwrap_or(0.);
                let theta = s.tracker_value(angle).unwrap_or(0.);
                s.move_to(m, chart(theta, p), ORIGIN);
            },
            true,
        )?;
        for (i, m) in dots.into_iter().enumerate() {
            s.add_updater(
                m,
                move |s, m| {
                    let p = s.tracker_value(moving).unwrap_or(0.);
                    let theta = s.tracker_value(angle).unwrap_or(0.);
                    s.move_to(m, physical(coin_hits(Address { theta, distance: p }, r)[i]), ORIGIN);
                },
                true,
            )?;
        }
        fade_in_all(s, &[line, mark, dots[0], dots[1]], 0.4)?;
        s.play(glide(moving, 0.75 * r, 5., linear)?)?;
        interior.wait_for(s, "Turning the line")?;
        s.play(glide(angle, PI * 0.9, 5., linear)?)?;
        for m in [line, mark, dots[0], dots[1]] {
            s.clear_updaters(m, true);
        }
        k.hold(s)?;
        pause(s, 3.)?;

        let exceptional = say(&k, s, "20_10")?;
        fade_out_all(s, &[line, mark, dots[0], dots[1], band_label], 0.3)?;
        let edge = path(s, vec![chart(0., r), chart(PI, r)], Palette::gold(), 2.);
        let tangent = Address { theta: 0., distance: r };
        let touch = inspection(s, tangent, coin_hits(tangent, r));
        let touch_label = label(
            &k,
            s,
            "tangent: one contact · boundary has zero area",
            27.,
            0.,
            -2.85,
            Palette::gold(),
        )?;
        fade_in_all(s, &touch, 0.5)?;
        fade_in_all(s, &[edge, touch_label], 0.5)?;
        pause(s, 4.)?;
        exceptional.wait_for(s, "Farther away")?;
        fade_out_all(s, &touch, 0.3)?;
        fade_out_all(s, &[touch_label], 0.3)?;
        let outside = Address {
            theta: 0.,
            distance: 1.5 * r,
        };
        let miss = inspection(s, outside, coin_hits(outside, r));
        let miss_label = label(&k, s, "outside the band: no contact", 27., 0., -2.85, Palette::ink())?;
        fade_in_all(s, &miss, 0.5)?;
        fade_in_all(s, &[miss_label], 0.4)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "20_11")?;
        fade_out_all(s, &miss, 0.3)?;
        fade_out_all(s, &[miss_label, edge], 0.3)?;
        let rows = [
            k.tex(s, "\\text{band area}=2\\pi", 32., Palette::teal(), &[], v(-2.9, -2.55))?,
            k.tex(
                s,
                "\\text{weighted total}=2(2\\pi)=4\\pi",
                32.,
                Palette::gold(),
                &[],
                v(2.6, -2.55),
            )?,
            k.tex(s, "L=\\frac{1}{2}(4\\pi)=2\\pi", 34., Palette::ink(), &[], v(0., -3.25))?,
        ];
        fade_in_all(s, &rows, 0.5)?;
        let width_label = label(
            &k,
            s,
            &format!("height = {} cm · width = π radians", f.get("coin.width").unwrap()),
            24.,
            3.5,
            -0.45,
            Palette::ink(),
        )?;
        fade_in_all(s, &[width_label], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "20_12")?;
        fade_out_all(s, &rows, 0.4)?;
        fade_out_all(s, &[width_label], 0.3)?;
        let callbacks = vec![
            label(
                &k,
                s,
                &format!("coin: {} per sheet", f.get("coin.crossings").unwrap()),
                26.,
                -4.7,
                -2.55,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                &format!(
                    "stick: about {}% total shadow",
                    f.get("stick.mean_reach_percent").unwrap()
                ),
                25.,
                1.7,
                -2.55,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                &format!(
                    "coast: mean {} per sheet · {} cm spacing",
                    f.get("crofton.mean_crossings").unwrap(),
                    f.get("sheet.gap").unwrap()
                ),
                26.,
                0.,
                -3.2,
                Palette::ink(),
            )?,
        ];
        fade_in_all(s, &callbacks, 0.5)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "20_13")?;
        fade_out_all(s, &callbacks, 0.4)?;
        let note = precisely(&k, s, NOTE)?;
        fade_in_all(s, &[note], 0.5)?;
        let deadline = now(s) + (NOTE.chars().count() + 10) as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circle_crossings_match_both_equations_and_strict_branches() {
        for i in 0..=180 {
            for j in -20_i32..=20 {
                let a = Address {
                    theta: PI * i as f64 / 180.,
                    distance: j as f64 / 10.,
                };
                let hits = coin_hits(a, 1.);
                assert_eq!(
                    hits.len(),
                    if j.abs() < 10 {
                        2
                    } else if j.abs() == 10 {
                        1
                    } else {
                        0
                    }
                );
                for p in hits {
                    assert!((p[0].hypot(p[1]) - 1.).abs() < 1e-12);
                    assert!((p[0] * a.theta.cos() + p[1] * a.theta.sin() - a.distance).abs() < 1e-12);
                }
            }
        }
        assert_eq!(
            coin_hits(
                Address {
                    theta: 0.,
                    distance: f64::from_bits(1f64.to_bits() - 1)
                },
                1.
            )
            .len(),
            2
        );
        assert_eq!(
            coin_hits(
                Address {
                    theta: 0.,
                    distance: f64::from_bits(1f64.to_bits() + 1)
                },
                1.
            )
            .len(),
            0
        );
    }
    #[test]
    fn band_counts_measure_not_boundary_thickness() {
        for r in [0.1, 1., 3.5] {
            assert!((2. * coin_band(r) - 2. * TAU * r).abs() < 1e-12);
        }
        assert_eq!(selected_coin_counts(), "2,1,0");
        assert!(coin_band(1.) != 2. * coin_band(1.));
    }
    #[test]
    fn formula_scene_uses_bound_facts() {
        working().unwrap();
    }
}
