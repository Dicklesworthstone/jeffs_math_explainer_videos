//! Retained running averages versus the spread of independent averages.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::glide;
use crate::{drop_samples as drops, facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

pub struct AThousandDrops {
    pub kit: Kit,
}
const NOTE: &str = "Independent drops with uniform half-turn direction and independent uniform slide over one gap. SD is the root mean squared deviation from the mean, not an error bound or coverage interval. Population SD and the threshold use a numerical model.";
fn label(k: &Kit, s: &mut Stage<'_>, t: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, t, size, color, v(x, y))
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
    v(-3.5 + 0.32 * p[0], 0.7 + 0.32 * p[1])
}
fn sample_point(n: usize, estimate: f64) -> Vec3 {
    v(0.4 + 5.5 * n as f64 / 1000., -0.8 + (estimate - 24.) * 0.7)
}
fn sd_point(n: usize, sd: f64) -> Vec3 {
    v(0.4 + 5.5 * n as f64 / 2200., -0.8 + sd * 5.6)
}
pub fn running_estimates(counts: &[usize]) -> Vec<f64> {
    let mut total = 0;
    counts
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            total += count;
            drops::estimate(total, i + 1)
        })
        .collect()
}
pub fn mean_sd(single: f64, n: usize) -> f64 {
    assert!(n > 0);
    single / (n as f64).sqrt()
}
impl SceneConstruct for AThousandDrops {
    fn name(&self) -> &str {
        "22_a_thousand_drops"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let counts = drops::counts(&drops::samples().map_err(fail)?, &c);
        let estimates = running_estimates(&counts);
        let shares = c.count_shares(facts::SHARE_DIRECTIONS, Some(model::GAP));
        let single = model::moments(&shares).1 * model::worth(model::GAP);
        let threshold = (single / 0.1).powi(2).ceil() as usize;
        say(&k, s, "22_1")?;
        let title = label(
            &k,
            s,
            "What changes when we keep dropping?",
            36.,
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
                .moved_to(v(-3.5, 0.7)),
        );
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            4.,
        );
        let truth = label(
            &k,
            s,
            &format!("same coast · length ≈ {} cm", f.get("coast.length_2dp").unwrap()),
            24.,
            -3.5,
            -1.05,
            Palette::ink(),
        )?;
        let rules = label(
            &k,
            s,
            "fresh direction + fresh slide each time",
            23.,
            -3.5,
            -1.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[title, card, coast, truth, rules], 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;

        say(&k, s, "22_2")?;
        let mut sample = vec![
            path(s, vec![v(0.4, 2.), v(0.4, -0.8), v(5.9, -0.8)], Palette::dim(), 1.5),
            label(&k, s, "running length estimate (cm)", 25., 3.2, 2.55, Palette::teal())?,
            label(&k, s, "drops used", 23., 3.2, -1.5, Palette::ink())?,
            label(&k, s, "10", 21., 0.45, -1.12, Palette::ink())?,
            label(&k, s, "1,000", 21., 5.9, -1.12, Palette::ink())?,
            label(&k, s, "24", 20., 0.05, -0.8, Palette::ink())?,
            label(&k, s, "28", 20., 0.05, 2., Palette::ink())?,
            path(
                s,
                vec![sample_point(10, c.length), sample_point(1000, c.length)],
                Palette::gold(),
                1.8,
            ),
        ];
        fade_in_all(s, &sample, 0.5)?;
        let trace = path(
            s,
            vec![sample_point(10, estimates[9]), sample_point(11, estimates[10])],
            Palette::teal(),
            3.,
        );
        let marker = dot(s, sample_point(10, estimates[9]), Palette::teal());
        let progress = s.add_value_tracker(10.);
        let values = estimates.clone();
        let driver = s.arena_mut().add(VMobject::new());
        s.add_updater(
            driver,
            move |s, _| {
                let n = (s.tracker_value(progress).unwrap_or(10.).round() as usize).clamp(11, 1000);
                let points: Vec<_> = (10..=n).map(|j| sample_point(j, values[j - 1])).collect();
                let _ = s.set_points(trace, polyline(points).build().points());
                s.move_to(marker, sample_point(n, values[n - 1]), ORIGIN);
            },
            true,
        )?;
        fade_in_all(s, &[driver, trace, marker], 0.3)?;
        s.play(glide(progress, 1000., 12., linear)?)?;
        stop(s, driver);
        sample.extend([driver, trace, marker]);
        k.hold(s)?;
        pause(s, 3.)?;

        let spoken = say(&k, s, "22_3")?;
        spoken.wait_for(s, "twenty-five point nine centimetres")?;
        let observed = label(
            &k,
            s,
            &format!("retained run: {} cm", f.get("drops.many.estimate").unwrap()),
            29.,
            0.,
            -2.45,
            Palette::teal(),
        )?;
        fade_in_all(s, &[observed], 0.4)?;
        let caution = label(
            &k,
            s,
            "one observed average · no guaranteed error bar",
            24.,
            0.,
            -3.1,
            Palette::ink(),
        )?;
        fade_in_all(s, &[caution], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "22_4")?;
        fade_out_all(s, &sample, 0.4)?;
        fade_out_all(s, &[observed, caution], 0.4)?;
        let mut spread = vec![
            label(&k, s, "repeated independent averages", 26., 3.2, 2.6, Palette::gold())?,
            label(
                &k,
                s,
                &format!("model SD for 1,000 drops: {} cm", f.get("drops.many.spread").unwrap()),
                27.,
                3.2,
                1.75,
                Palette::gold(),
            )?,
            label(
                &k,
                s,
                "square deviations → average → square root",
                22.,
                3.2,
                0.85,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                "population model, not this run's error",
                23.,
                3.2,
                -0.1,
                Palette::ink(),
            )?,
        ];
        fade_in_all(s, &spread, 0.5)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "22_5")?;
        let variance = vec![
            label(
                &k,
                s,
                "independent deviations: cross-products average to zero",
                25.,
                0.,
                -2.35,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                "variance of sum = n × variance of one drop",
                26.,
                0.,
                -3.05,
                Palette::teal(),
            )?,
        ];
        fade_in_all(s, &variance, 0.5)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "22_6")?;
        fade_out_all(s, &variance, 0.35)?;
        let formula = vec![
            label(
                &k,
                s,
                "σ = one-drop SD · n = drops averaged",
                22.,
                3.2,
                -1.1,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                "variance of average = n σ² / n² = σ² / n",
                27.,
                0.,
                -2.35,
                Palette::ink(),
            )?,
            label(&k, s, "SD of average = σ / √n", 30., 0., -3.05, Palette::gold())?,
        ];
        fade_in_all(s, &formula, 0.5)?;
        k.hold(s)?;
        pause(s, 5.)?;

        say(&k, s, "22_7")?;
        fade_out_all(s, &spread, 0.4)?;
        fade_out_all(s, &formula, 0.3)?;
        spread.clear();
        spread.push(path(
            s,
            vec![v(0.4, 2.), v(0.4, -0.8), v(5.9, -0.8)],
            Palette::dim(),
            1.5,
        ));
        spread.push(label(&k, s, "numerical model SD (cm)", 25., 3.2, 2.6, Palette::gold())?);
        spread.push(label(&k, s, "independent drops n", 23., 3.2, -1.5, Palette::ink())?);
        for n in [100, 1000] {
            spread.push(label(
                &k,
                s,
                &n.to_string(),
                20.,
                sd_point(n, 0.)[0],
                -1.12,
                Palette::ink(),
            )?);
        }
        for sd in [0., 0.2, 0.4] {
            spread.push(label(
                &k,
                s,
                &facts::fixed(sd, 1),
                20.,
                0.05,
                sd_point(0, sd)[1],
                Palette::ink(),
            )?);
        }
        spread.push(path(
            s,
            // Leave the target crossing for the answer after the prediction.
            (100..=1200)
                .step_by(2)
                .map(|n| sd_point(n, mean_sd(single, n)))
                .collect(),
            Palette::gold(),
            3.,
        ));
        let at_thousand = dot(s, sd_point(1000, mean_sd(single, 1000)), Palette::teal());
        spread.push(at_thousand);
        let hundred = label(
            &k,
            s,
            &format!("100 drops: SD {}", facts::fixed(mean_sd(single, 100), 2)),
            23.,
            -3.5,
            -2.35,
            Palette::ink(),
        )?;
        let four_hundred = label(
            &k,
            s,
            &format!("400 drops: SD {}", facts::fixed(mean_sd(single, 400), 2)),
            23.,
            -3.5,
            -2.95,
            Palette::ink(),
        )?;
        fade_in_all(s, &spread, 0.5)?;
        fade_in_all(s, &[hundred, four_hundred], 0.5)?;
        k.hold(s)?;
        pause(s, 4.)?;

        say(&k, s, "22_8")?;
        let target = path(s, vec![sd_point(100, 0.1), sd_point(2200, 0.1)], Palette::red(), 2.);
        let target_name = label(&k, s, "target SD = 0.1 cm", 24., 3.2, 0.45, Palette::red())?;
        let question = label(&k, s, "How many independent drops?", 28., 0., -3.55, Palette::ink())?;
        fade_in_all(s, &[target, target_name, question], 0.4)?;
        let thinking = think(&k, s, v(4.7, -2.5))?;

        let spoken = say(&k, s, "22_9")?;
        spoken.wait_for(s, "About two thousand")?;
        fade_out_all(s, &[thinking, question], 0.3)?;
        let continuation = path(
            s,
            (1200..=2200)
                .step_by(2)
                .map(|n| sd_point(n, mean_sd(single, n)))
                .collect(),
            Palette::gold(),
            3.,
        );
        let two_thousand = label(&k, s, "2000", 20., sd_point(2000, 0.)[0], -1.12, Palette::ink())?;
        spread.extend([continuation, two_thousand]);
        let answer = dot(s, sd_point(threshold, mean_sd(single, threshold)), Palette::red());
        let threshold_text = label(
            &k,
            s,
            &format!("model minimum: {} independent drops", f.get("drops.for_tenth").unwrap()),
            28.,
            0.,
            -3.55,
            Palette::red(),
        )?;
        fade_in_all(s, &[continuation, two_thousand, answer, threshold_text], 0.4)?;
        let calculation = label(&k, s, "n ≥ (unrounded σ / 0.1)²", 24., 3.2, 1.6, Palette::ink())?;
        fade_in_all(s, &[calculation], 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;

        say(&k, s, "22_10")?;
        fade_out_all(s, &spread, 0.4)?;
        fade_out_all(
            s,
            &[
                hundred,
                four_hundred,
                target,
                target_name,
                answer,
                threshold_text,
                calculation,
            ],
            0.4,
        )?;
        let callback = vec![
            label(
                &k,
                s,
                &format!("retained run: {} cm", f.get("drops.many.estimate").unwrap()),
                28.,
                3.2,
                2.0,
                Palette::teal(),
            )?,
            label(
                &k,
                s,
                &format!("population SD: {} cm", f.get("drops.many.spread").unwrap()),
                28.,
                3.2,
                1.15,
                Palette::gold(),
            )?,
            label(
                &k,
                s,
                "identity: L = ½ ∫∫ crossings dp dθ",
                26.,
                3.2,
                0.2,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                "sample variation belongs to the estimate",
                23.,
                3.2,
                -0.75,
                Palette::ink(),
            )?,
        ];
        fade_in_all(s, &callback, 0.5)?;
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
    fn running_prefixes_and_unrounded_threshold_match_the_experiment() {
        let c = model::coast();
        let counts = drops::counts(&drops::samples().unwrap(), &c);
        let values = running_estimates(&counts);
        for n in 1..=counts.len() {
            assert!((values[n - 1] - drops::estimate(counts[..n].iter().sum(), n)).abs() < 1e-12);
        }
        let p = c.count_shares(facts::SHARE_DIRECTIONS, Some(model::GAP));
        let sd = model::moments(&p).1 * model::worth(model::GAP);
        assert!(mean_sd(sd, 1964) > 0.1 && mean_sd(sd, 1965) <= 0.1);
        for n in 1..=1000 {
            assert!((mean_sd(sd, 4 * n) * 2. - mean_sd(sd, n)).abs() < 1e-12);
        }
        assert!(((4.4_f64 / 0.1).powi(2).ceil() as usize) < 1965);
    }
}
