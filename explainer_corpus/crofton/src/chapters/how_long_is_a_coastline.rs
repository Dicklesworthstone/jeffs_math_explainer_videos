//! Empirical scaling is distinct from the length of our finite drawing.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, think, wait_until};
use super::sheet;
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct HowLongIsACoastline {
    pub kit: Kit,
}
/// Mandelbrot, Science156 (1967), pp636–637, reporting Richardson's empirical fit.
pub const REPORTED_D: f64 = 1.25;
pub const STEPS: [f64; 4] = [1., 0.5, 0.25, 0.125];
pub fn fitted_ratio(relative_step: f64) -> f64 {
    relative_step.powf(1. - REPORTED_D)
}
pub fn lengths(c: &model::Curve) -> [f64; 4] {
    STEPS.map(|g| c.walk(g).length())
}
const NOTE: &str = "Richardson scaling is empirical over measured scales, not an exact ratio for each observation or a claim at every scale. Sheet identity: finite rectifiable drawing, all traversals, uniform half-turn and independent full-gap slide.";
fn at(p: model::P2) -> Vec3 {
    v(-3.5 + 0.33 * p[0], 0.6 + 0.33 * p[1])
}
fn path(s: &mut Stage<'_>, p: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(p).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, text: &str, x: f64, y: f64, size: f64) -> fmn::Result<Mob> {
    k.text(s, text, size, Palette::ink(), v(x, y))
}
fn walk(s: &mut Stage<'_>, c: &model::Curve, step: f64) -> fmn::Result<Vec<Mob>> {
    let w = c.walk(step);
    let mut p: Vec<_> = w.landings.iter().map(|x| at(x.0)).collect();
    p.push(at(c.ends().1));
    let m = path(s, p, Palette::gold(), 2.5);
    let mut out = vec![m];
    for &(p, _) in &w.landings {
        out.push(
            s.arena_mut().add(
                Circle::new()
                    .radius(0.025)
                    .arc_center(at(p))
                    .style(flat(Palette::gold(), 1.))
                    .build(),
            ),
        );
    }
    fade_in_all(s, &out, 0.6)?;
    Ok(out)
}
fn graph_point(i: usize, ratio: f64) -> Vec3 {
    v(1. + i as f64 * 1.4, 0.05 + (ratio - 1.) * 2.2)
}
impl SceneConstruct for HowLongIsACoastline {
    fn name(&self) -> &str {
        "27_how_long_is_a_coastline"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let c = model::coast();
        let f = facts::facts().map_err(fail)?;
        let dimension = f.get("richardson.west_coast_d").unwrap();
        let exponent = facts::fixed(dimension.parse::<f64>().map_err(|e| fail(e.to_string()))? - 1., 2);
        let totals = lengths(&c);
        let title = label(&k, s, "How long is a coastline?", 0., 3.45, 35.)?;
        let card = s.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * 0.33)
                .height(model::CARD_H * 0.33)
                .style(line_style(Palette::dim(), 1.5))
                .build()?
                .moved_to(v(-3.5, 0.6)),
        );
        let coast = path(
            s,
            c.parts
                .iter()
                .flat_map(|p| (0..=120).map(move |j| at(p.seg.point(j as f64 / 120.))))
                .collect(),
            palette::coast(),
            3.,
        );
        let caption = label(&k, s, "our fixed drawing · centimetres", -3.5, 2.45, 25.)?;
        say(&k, s, "27_1")?;
        fade_in_all(s, &[title, card, coast, caption], 0.5)?;
        let mut w = walk(s, &c, STEPS[0])?;
        let mut readout = label(
            &k,
            s,
            &format!("step 1 cm → ≈ {} cm", facts::fixed(totals[0], 2)),
            -3.5,
            -1.25,
            26.,
        )?;
        let mut explanation = vec![
            label(&k, s, "Walk along the same drawn coast", 3., 1.2, 27.)?,
            label(&k, s, "Sum chords, including the final short one", 3., 0.45, 22.)?,
        ];
        fade_in_all(s, &[readout], 0.4)?;
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "27_2")?;
        fade_out_all(s, &w, 0.3)?;
        fade_out_all(s, &[readout], 0.3)?;
        w = walk(s, &c, STEPS[1])?;
        readout = label(
            &k,
            s,
            &format!("step 0.5 cm → ≈ {} cm", facts::fixed(totals[1], 2)),
            -3.5,
            -1.25,
            26.,
        )?;
        let gain = label(
            &k,
            s,
            &format!("increase ≈ {}%", f.get("dividers.halving_percent").unwrap()),
            3.,
            -0.45,
            31.,
        )?;
        fade_in_all(s, &[readout, gain], 0.4)?;
        explanation.push(gain);
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "27_3")?;
        fade_out_all(s, &explanation, 0.3)?;
        explanation = vec![
            label(&k, s, "Richardson's map measurements", 3., 1.2, 29.)?,
            label(&k, s, "As reported by Mandelbrot (1967)", 3., 0.5, 24.)?,
            label(&k, s, "Smaller steps: level off or keep growing?", 3., -0.35, 24.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        let prediction = think(&k, s, v(-3.5, -2.3))?;
        say(&k, s, "27_4")?;
        fade_out_all(s, &[prediction], 0.3)?;
        fade_out_all(s, &explanation, 0.3)?;
        explanation = vec![
            label(&k, s, "Growth across the measured scales", 3., 1.2, 28.)?,
            label(&k, s, "An empirical scaling relation", 3., 0.45, 28.)?,
            label(&k, s, "Not identical ratios in every observation", 3., -0.3, 23.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "27_5")?;
        fade_out_all(s, &explanation, 0.3)?;
        explanation = vec![
            label(&k, s, "L(G) ≈ M × G^(1 − D)", 3., 1.4, 30.)?,
            label(&k, s, "G: step · L: measured length", 3., 0.65, 25.)?,
            label(&k, s, &format!("M: scale factor · D ≈ {dimension}"), 3., -0.1, 25.)?,
            label(&k, s, "West coast of Britain: reported fit", 3., -0.85, 23.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "27_6")?;
        fade_out_all(s, &explanation, 0.3)?;
        explanation = vec![
            label(&k, s, "Halve G: ratio ≈ 2^(D − 1)", 3., 1.4, 28.)?,
            label(
                &k,
                s,
                &format!(
                    "With D = {dimension}: 2^{exponent} ≈ {}",
                    f.get("richardson.halving_factor_3dp").unwrap()
                ),
                3.,
                0.65,
                27.,
            )?,
            label(
                &k,
                s,
                &format!("about {}% longer", f.get("richardson.halving_percent").unwrap()),
                3.,
                -0.1,
                31.,
            )?,
            label(&k, s, "Consequence of the fit, not raw data", 3., -0.85, 23.)?,
        ];
        fade_in_all(s, &explanation, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "27_7")?;
        fade_out_all(s, &explanation, 0.3)?;
        let mut chart = vec![
            label(&k, s, "Length / starting length · not measured data", 3.1, 2.5, 23.)?,
            label(&k, s, "teal: fitted law · gold: our divider sums", 3.1, 2.05, 21.)?,
        ];
        for ratio in [1., 1.2, 1.4, 1.6] {
            let y = graph_point(0, ratio)[1];
            chart.push(path(s, vec![v(0.9, y), v(5.35, y)], Palette::dim(), 0.8));
            chart.push(label(&k, s, &facts::fixed(ratio, 1), 0.55, y, 19.)?);
        }
        for (i, g) in STEPS.iter().enumerate() {
            chart.push(label(&k, s, &facts::fixed(*g, 3), graph_point(i, 1.)[0], -0.4, 20.)?);
        }
        chart.push(label(
            &k,
            s,
            "step / starting step · successive halvings",
            3.1,
            -0.85,
            22.,
        )?);
        chart.push(label(
            &k,
            s,
            "starting length = 1 · our starting step = 1 cm",
            3.1,
            -1.35,
            20.,
        )?);
        let fit = path(
            s,
            (0..4).map(|i| graph_point(i, fitted_ratio(STEPS[i]))).collect(),
            Palette::teal(),
            3.,
        );
        let smooth = path(
            s,
            (0..4).map(|i| graph_point(i, totals[i] / totals[0])).collect(),
            Palette::gold(),
            3.,
        );
        chart.extend([fit, smooth]);
        fade_in_all(s, &chart, 0.6)?;
        k.hold(s)?;
        pause(s, 7.)?;
        say(&k, s, "27_8")?;
        for i in 2..4 {
            fade_out_all(s, &w, 0.3)?;
            fade_out_all(s, &[readout], 0.3)?;
            w = walk(s, &c, STEPS[i])?;
            readout = label(
                &k,
                s,
                &format!(
                    "step {} cm → ≈ {} cm",
                    facts::fixed(STEPS[i], 3),
                    facts::fixed(totals[i], 2)
                ),
                -3.5,
                -1.25,
                24.,
            )?;
            fade_in_all(s, &[readout], 0.4)?;
            pause(s, 5.)?;
        }
        let limit = label(
            &k,
            s,
            &format!("drawn arc length ≈ {} cm", f.get("coast.length_2dp").unwrap()),
            -3.5,
            -1.9,
            24.,
        )?;
        fade_in_all(s, &[limit], 0.3)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "27_9")?;
        fade_out_all(s, &chart, 0.3)?;
        let history = vec![
            label(&k, s, "Mandelbrot · Science · 1967", 3., 1.5, 28.)?,
            label(&k, s, "How Long Is the Coast of Britain?", 3., 0.75, 26.)?,
            label(
                &k,
                s,
                "Statistical Self-Similarity and Fractional Dimension",
                3.,
                0.,
                22.,
            )?,
            label(&k, s, "Observed scaling ≠ proof at all smaller scales", 3., -0.8, 22.)?,
        ];
        fade_in_all(s, &history, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "27_10")?;
        fade_out_all(s, &history, 0.3)?;
        fade_out_all(s, &w, 0.3)?;
        fade_out_all(s, &[readout, limit], 0.3)?;
        let r = sheet::rippled(&c, 7).map_err(fail)?;
        let beach = path(
            s,
            r.parts[model::BEACH..model::BEACH + 7]
                .iter()
                .flat_map(|p| (0..=50).map(move |j| at(p.seg.point(j as f64 / 50.))))
                .collect(),
            Palette::gold(),
            3.5,
        );
        let comparison = vec![
            label(
                &k,
                s,
                &format!("smooth drawing: ≈ {} cm", f.get("coast.length").unwrap()),
                3.,
                1.4,
                27.,
            )?,
            label(
                &k,
                s,
                &format!("rippled drawing: ≈ {} cm", f.get("scale.rippled_length").unwrap()),
                3.,
                0.65,
                27.,
            )?,
            label(&k, s, "Each has its own sheet expectation", 3., -0.1, 25.)?,
            label(&k, s, "The drawing determines what is measured", 3., -0.85, 23.)?,
            label(
                &k,
                s,
                "blue: straight beach · gold: ripple replacement",
                -3.5,
                -1.25,
                20.,
            )?,
        ];
        fade_in_all(s, &[beach], 0.4)?;
        fade_in_all(s, &comparison, 0.4)?;
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
    fn fitted_law_halving_and_units_cancel() {
        for i in 0..100 {
            let g = 0.01 + i as f64 / 17.;
            assert!((fitted_ratio(g / 2.) / fitted_ratio(g) - 2f64.powf(0.25)).abs() < 1e-12);
        }
        assert!((fitted_ratio(0.5) - 1.19).abs() > 0.0007);
        assert!((fitted_ratio(0.25) - fitted_ratio(0.5).powi(2)).abs() < 1e-12);
    }
    #[test]
    fn drawn_walks_are_bounded_and_not_the_fitted_law() {
        let c = model::coast();
        let l = lengths(&c);
        for (g, total) in STEPS.into_iter().zip(l) {
            let w = c.walk(g);
            assert!(total < c.length);
            assert!(w.last < g);
            for q in w.landings.windows(2) {
                assert!(((q[0].0[0] - q[1].0[0]).hypot(q[0].0[1] - q[1].0[1]) - g).abs() < 1e-8);
            }
        }
        assert!((l[1] / l[0] - 1.) * 100. > 1.);
        assert!((l[1] / l[0] - 1.) * 100. < 3.);
        assert!(l[3] / l[0] < 1.04);
    }
}
