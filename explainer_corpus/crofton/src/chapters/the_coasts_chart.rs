//! The coast and its multiplicity chart share the same intersection geometry.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::glide;
use crate::{
    coast_chart::{self, Grid},
    facts, line_chart, model, palette,
};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, TextKind, v};
use fmn::core::rate::linear;
use fmn::prelude::*;
use std::{f64::consts::PI, rc::Rc};

pub struct TheCoastsChart {
    pub kit: Kit,
}
const SCALE: f64 = 0.36;
const NOTE: &str = "Finite-length plane paths; count each traversal. Integrate signed distance over all real values and normal angle over a half turn, in radians, with area dp dθ. The finite chart grid is an approximation, not the identity.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.4 + SCALE * p[0], 0.35 + SCALE * p[1])
}
fn chart(t: f64, p: f64) -> Vec3 {
    v(0.8 + 5.4 * t / PI, 0.25 + 2. * p / line_chart::reach())
}
fn path(s: &mut Stage<'_>, ps: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(ps).style(line_style(color, width)).build())
}
fn text(k: &Kit, s: &mut Stage<'_>, words: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, words, size, color, v(x, y))
}
fn dot(s: &mut Stage<'_>, p: Vec3) -> Mob {
    s.arena_mut().add(
        Circle::new()
            .radius(0.062)
            .arc_center(p)
            .style(flat(Palette::gold(), 1.))
            .build(),
    )
}
fn shade(n: usize, max: usize) -> Srgb {
    let t = n as f32 / max as f32;
    Srgb::from_rgb8((41. + 10. * t) as u8, (79. + 97. * t) as u8, (122. + 122. * t) as u8)
}
fn tiles(s: &mut Stage<'_>, g: &Grid) -> fmn::Result<Vec<Mob>> {
    let max = g.histogram().len() - 1;
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
                let x0 = PI * i as f64 / g.columns as f64;
                let x1 = PI * (i + 1) as f64 / g.columns as f64;
                let y0 = line_chart::reach() * (2. * start as f64 / g.rows as f64 - 1.);
                let y1 = line_chart::reach() * (2. * j as f64 / g.rows as f64 - 1.);
                out.push(
                    s.arena_mut().add(
                        Polygon::new(vec![chart(x0, y0), chart(x1, y0), chart(x1, y1), chart(x0, y1)])
                            .style(flat(shade(n, max), 0.9))
                            .build(),
                    ),
                );
            }
        }
    }
    Ok(out)
}
fn working() -> Result<(), String> {
    let c = model::coast();
    let g = Grid::build(&c, coast_chart::COLUMNS, coast_chart::ROWS);
    let f = facts::facts()?;
    if f.get("chart.grid_total") != Some(facts::fixed(*g.prefix().last().unwrap(), 5).as_str())
        || f.get("chart.exact_total_5dp") != Some(facts::fixed(2. * c.length, 5).as_str())
    {
        return Err("chart data differs from facts".into());
    }
    Ok(())
}
impl SceneConstruct for TheCoastsChart {
    fn name(&self) -> &str {
        "19_the_coasts_chart"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        working().map_err(fail)?;
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let grid = Grid::build(&c, coast_chart::COLUMNS, coast_chart::ROWS);
        let maximum = grid.histogram().len() - 1;
        say(&k, s, "19_1")?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * SCALE)
                .height(model::CARD_H * SCALE)
                .corner_radius(0.05)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(physical([0., 0.])),
        );
        let title = text(&k, s, "The coast's chart", 38., 0., 3.3, Palette::ink())?;
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
            text(&k, s, "normal angle (radians)", 23., 3.5, -2.13, Palette::teal())?,
            text(&k, s, "signed distance (cm)", 23., 3.5, 2.65, Palette::gold())?,
            text(&k, s, "0", 21., 0.8, -1.95, Palette::dim())?,
            k.tex(s, "\\pi", 23., Palette::dim(), &[], v(6.2, -1.95))?,
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
        let stick = path(s, vec![physical([0., -3.]), physical([0., 3.])], palette::stick(), 6.);
        fade_in_all(s, &[card, title, stick, boundary], 0.5)?;
        fade_in_all(s, &axes, 0.4)?;
        pause(s, 1.5)?;
        fade_out_all(s, &[stick], 0.4)?;
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            4.,
        );
        let name = text(&k, s, "the same drawn coast", 27., -3.4, 2.3, palette::coast())?;
        fade_in_all(s, &[coast, name], 0.7)?;
        k.hold(s)?;
        pause(s, 2.)?;

        let samples = [
            grid.representative(&c, 0),
            grid.representative(&c, 1),
            grid.representative(&c, maximum),
        ];
        let mut inspection = Vec::new();
        for (cue, idx) in [("19_2", 0), ("19_3", 2)] {
            say(&k, s, cue)?;
            let list = if idx == 0 { vec![0, 1] } else { vec![2] };
            for sample in list {
                fade_out_all(s, &inspection, 0.3)?;
                inspection.clear();
                let a = samples[sample];
                let crossings = coast_chart::intersections(&c, a);
                inspection.push(path(
                    s,
                    a.line()
                        .chord()
                        .expect("selected line meets card")
                        .into_iter()
                        .map(physical)
                        .collect(),
                    Palette::ink(),
                    2.,
                ));
                inspection.push(dot(s, chart(a.theta, a.distance)));
                for hit in &crossings {
                    inspection.push(dot(s, physical(hit.at)));
                }
                inspection.push(text(
                    &k,
                    s,
                    &format!("selected line: {} crossings", crossings.len()),
                    30.,
                    0.,
                    -2.7,
                    Palette::gold(),
                )?);
                fade_in_all(s, &inspection, 0.5)?;
                pause(s, 4.)?;
            }
            k.hold(s)?;
            pause(s, 2.)?;
        }
        say(&k, s, "19_4")?;
        fade_out_all(s, &inspection, 0.4)?;
        let heat = tiles(s, &grid)?;
        let grid_label = text(
            &k,
            s,
            &format!(
                "computed midpoint grid: {} × {}",
                f.get("chart.grid_columns").unwrap(),
                f.get("chart.grid_rows").unwrap()
            ),
            24.,
            0.,
            -2.7,
            Palette::ink(),
        )?;
        fade_in_all(s, &[grid_label], 0.4)?;
        fade_in_all(s, &heat, 1.)?;
        // Bring the outline forward over the grid, keeping its identity.
        s.remove_from_scene(boundary);
        s.add_to_scene(boundary)?;
        let mut legend = Vec::new();
        for n in 0..=maximum {
            let x = 1.1 + 0.76 * n as f64;
            let style = if n == 0 {
                line_style(Palette::dim(), 1.)
            } else {
                flat(shade(n, maximum), 0.9)
            };
            legend.push(
                s.arena_mut().add(
                    Rectangle::new()
                        .width(0.25)
                        .height(0.22)
                        .style(style)
                        .build()?
                        .moved_to(v(x - 0.2, -2.48)),
                ),
            );
            legend.push(text(&k, s, &n.to_string(), 23., x + 0.12, -2.48, Palette::ink())?);
        }
        // Separate legend and grid disclosure vertically.
        s.move_to(grid_label, v(0., -3.1), ORIGIN);
        fade_in_all(s, &legend, 0.4)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "19_5")?;
        fade_out_all(s, &[grid_label], 0.3)?;
        let stats = vec![
            text(
                &k,
                s,
                &format!("grid: {}% miss the coast", f.get("chart.grid_miss_percent").unwrap()),
                27.,
                -3.2,
                -2.45,
                Palette::ink(),
            )?,
            text(
                &k,
                s,
                &format!(
                    "among grid hits: {}% cross once",
                    f.get("chart.grid_once_percent").unwrap()
                ),
                27.,
                0.,
                -3.1,
                Palette::ink(),
            )?,
        ];
        fade_in_all(s, &stats, 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "19_6")?;
        fade_out_all(s, &stats, 0.3)?;
        let weight = text(
            &k,
            s,
            "patch contribution = chart area × crossing count",
            29.,
            0.,
            -3.05,
            Palette::ink(),
        )?;
        fade_in_all(s, &[weight], 0.5)?;
        let a = samples[2];
        let points = coast_chart::intersections(&c, a);
        let mut selected = vec![
            path(
                s,
                a.line().chord().unwrap().into_iter().map(physical).collect(),
                Palette::ink(),
                2.,
            ),
            dot(s, chart(a.theta, a.distance)),
        ];
        for x in &points {
            selected.push(dot(s, physical(x.at)));
        }
        fade_in_all(s, &selected, 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;

        say(&k, s, "19_7")?;
        fade_out_all(s, &selected, 0.3)?;
        fade_out_all(s, &[weight], 0.3)?;
        let question = text(
            &k,
            s,
            "Add every patch's contribution: what total?",
            30.,
            0.,
            -3.0,
            Palette::ink(),
        )?;
        fade_in_all(s, &[question], 0.4)?;
        let thinking = think(&k, s, v(-5.7, -2.5))?;

        let sum = say(&k, s, "19_8")?;
        fade_out_all(s, &[question, thinking], 0.3)?;
        let progress = s.add_value_tracker(0.);
        let alpha = s.add_value_tracker(1.);
        let prefix = Rc::new(grid.prefix());
        let book = Rc::clone(&k.book);
        let total = s.always_redraw(move |s| {
            let i = (s.tracker_value(progress).unwrap_or(0.).floor() as usize).min(prefix.len() - 1);
            let d = DecimalNumber::new(prefix[i])
                .num_decimal_places(if i == prefix.len() - 1 { 2 } else { 5 })
                .include_sign(false)
                .font_size(32.)
                .color(Palette::gold())
                .build(&book)
                .expect("bundled digits");
            explainer_kit::glyphs::record(TextKind::Text, d.num_string());
            let m = s.add(d.into_vmob());
            s.move_to(m, v(4.1, -3.), RIGHT);
            s.set_family_opacity(m, s.tracker_value(alpha).unwrap_or(0.));
            m
        });
        s.add_to_scene(total)?;
        let sum_label = text(&k, s, "finite grid sum ≈", 29., -0.4, -3., Palette::gold())?;
        let sweep = path(
            s,
            vec![chart(0., -line_chart::reach()), chart(0., line_chart::reach())],
            Palette::gold(),
            3.,
        );
        s.add_updater(
            sweep,
            move |s, m| {
                let t = PI * s.tracker_value(progress).unwrap_or(0.) / coast_chart::COLUMNS as f64;
                let _ = s.set_points(
                    m,
                    polyline(vec![chart(t, -line_chart::reach()), chart(t, line_chart::reach())])
                        .build()
                        .points(),
                );
            },
            true,
        )?;
        fade_in_all(s, &[sum_label, sweep], 0.4)?;
        // Finish before the result phrase; sentence/character timing remains
        // provisional until this chapter has qualified spoken audio.
        let sweep_time = (sum.at("approximately")? - now(s) - 0.25).max(0.1);
        s.play(glide(progress, grid.columns as f64, sweep_time, linear)?)?;
        sum.wait_for(s, "approximately")?;
        k.hold(s)?;
        pause(s, 3.)?;
        stop(s, sweep);
        fade_out_all(s, &[sweep, sum_label], 0.3)?;
        s.play(glide(alpha, 0., 0.3, linear)?)?;
        stop(s, total);

        say(&k, s, "19_9")?;
        let comparison = vec![
            text(
                &k,
                s,
                &format!("grid ≈ {}", f.get("chart.grid_total").unwrap()),
                29.,
                -3.4,
                -2.55,
                Palette::gold(),
            )?,
            text(
                &k,
                s,
                &format!("twice exact length ≈ {}", f.get("chart.exact_total_5dp").unwrap()),
                29.,
                2.6,
                -2.9,
                Palette::teal(),
            )?,
        ];
        fade_out_all(s, &legend, 0.3)?;
        fade_in_all(s, &comparison, 0.6)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "19_10")?;
        fade_out_all(s, &comparison, 0.4)?;
        let reason = text(
            &k,
            s,
            "add every piece's crossings → add every piece's length",
            27.,
            0.,
            -2.8,
            Palette::ink(),
        )?;
        fade_in_all(s, &[reason], 0.5)?;
        let mut highlights = Vec::new();
        for part in &c.parts {
            let mob = path(
                s,
                (0..=80).map(|i| physical(part.seg.point(i as f64 / 80.))).collect(),
                palette::stick(),
                5.,
            );
            highlights.push(mob);
            fade_in_all(s, &[mob], 0.35)?;
            pause(s, 0.15)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        fade_out_all(s, &highlights, 0.5)?;

        say(&k, s, "19_11")?;
        fade_out_all(s, &[reason], 0.3)?;
        let attribution = format!(
            "Cauchy–Crofton length formula · Morgan W. Crofton, {}",
            f.get("crofton.year").unwrap()
        );
        let credit = text(&k, s, &attribution, 25., 0., -2.85, Palette::ink())?;
        fade_in_all(s, &[credit], 0.5)?;
        let deadline = now(s) + attribution.chars().count() as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;

        say(&k, s, "19_12")?;
        fade_out_all(s, &[credit], 0.4)?;
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
    fn displayed_weighted_sum_is_bound_to_the_geometry() {
        working().unwrap();
    }
}
