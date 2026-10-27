//! A lucky retained experiment, followed by its sampling variation.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, think, wait_until};
use super::sheet;
use crate::{drop_samples as drops, facts, model, palette, pin_lines::PinLine};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
#[cfg(test)]
use std::f64::consts::PI;

pub struct TenDrops {
    pub kit: Kit,
}
const SCALE: f64 = 0.37;
const NOTE: &str = "Independent drops: uniform direction over a half turn, independent uniform slide over one gap. Population figures are numerical model estimates. Standard deviation describes spread, not a bound or a confidence interval.";
fn physical(p: model::P2) -> Vec3 {
    v(-3.2 + SCALE * p[0], 0.45 + SCALE * p[1])
}
fn path(s: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(pts).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, t: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, t, size, color, v(x, y))
}
fn rectangle(
    s: &mut Stage<'_>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: Srgb,
    opacity: f64,
) -> fmn::Result<Mob> {
    Ok(s.arena_mut().add(
        Rectangle::new()
            .width(width)
            .height(height)
            .style(flat(color, opacity))
            .build()?
            .moved_to(v(x, y)),
    ))
}
fn placement(s: &mut Stage<'_>, c: &model::Curve, d: drops::Drop) -> Vec<Mob> {
    let beta = d.degrees.to_radians();
    let n = [-beta.sin(), beta.cos()];
    let mut out = Vec::new();
    for j in -8..=8 {
        let p = d.slide + j as f64 * model::GAP;
        let foot = [p * n[0], p * n[1]];
        if let Some(chord) = (PinLine {
            pins: [foot, [foot[0] - n[1], foot[1] + n[0]]],
        })
        .chord()
        {
            out.push(path(s, chord.into_iter().map(physical).collect(), Palette::teal(), 1.5));
        }
    }
    for hit in sheet::crossings(c, beta, d.slide) {
        out.push(
            s.arena_mut().add(
                Circle::new()
                    .radius(0.055)
                    .arc_center(physical(hit.at))
                    .style(flat(Palette::gold(), 1.))
                    .build(),
            ),
        );
    }
    out
}
fn histogram(k: &Kit, s: &mut Stage<'_>, hist: &[usize], shares: &[f64]) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let mut sample = Vec::new();
    let mut population = Vec::new();
    let n: usize = hist.iter().sum();
    for (i, &prob) in shares.iter().enumerate().skip(5) {
        let x = 0.45 + (i - 5) as f64 * 0.65;
        let h = hist.get(i).copied().unwrap_or(0) as f64 / n as f64 * 7.;
        if h > 0. {
            sample.push(rectangle(s, x, -0.85 + h / 2., 0.36, h, Palette::teal(), 0.85)?);
        }
        sample.push(label(k, s, &i.to_string(), 23., x, -1.15, Palette::ink())?);
        let y = -0.85 + prob * 7.;
        population.push(path(
            s,
            vec![v(x - 0.26, -0.85), v(x - 0.26, y), v(x + 0.26, y), v(x + 0.26, -0.85)],
            Palette::gold(),
            2.,
        ));
    }
    sample.push(path(s, vec![v(0.05, -0.85), v(6., -0.85)], Palette::dim(), 1.5));
    sample.push(label(k, s, "crossings per drop", 24., 3., -1.65, Palette::ink())?);
    Ok((sample, population))
}
fn spread(k: &Kit, s: &mut Stage<'_>, one: f64, ten: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![label(
        k,
        s,
        "spread of length estimates (cm)",
        25.,
        3.,
        2.2,
        Palette::ink(),
    )?];
    for (row, width, name, color) in [
        (0, one, "one drop", Palette::gold()),
        (1, ten, "mean of ten", Palette::teal()),
    ] {
        let y = 1. - row as f64 * 1.3;
        let dx = width * 0.45;
        out.push(path(s, vec![v(3. - dx, y), v(3. + dx, y)], color, 5.));
        out.push(path(s, vec![v(3., y - 0.18), v(3., y + 0.18)], Palette::ink(), 2.));
        out.push(label(
            k,
            s,
            &format!("{name}: SD {}", facts::fixed(width, 1)),
            25.,
            3.,
            y - 0.5,
            color,
        )?);
    }
    out.push(label(
        k,
        s,
        "bars show ±1 SD, not all outcomes",
        22.,
        3.,
        -1.5,
        Palette::ink(),
    )?);
    Ok(out)
}
impl SceneConstruct for TenDrops {
    fn name(&self) -> &str {
        "21_ten_drops"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = model::coast();
        let draws = drops::samples().map_err(fail)?;
        let counts = drops::counts(&draws, &c);
        let shares = c.count_shares(facts::SHARE_DIRECTIONS, Some(model::GAP));
        say(&k, s, "21_1")?;
        let title = label(&k, s, "What can ten drops tell us?", 36., 0., 3.3, Palette::ink())?;
        let card = rectangle(
            s,
            -3.2,
            0.45,
            model::CARD_W * SCALE,
            model::CARD_H * SCALE,
            palette::card(),
            0.55,
        )?;
        let coast = path(
            s,
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            4.,
        );
        let truth = label(
            &k,
            s,
            &format!("same coast · exact length ≈ {} cm", f.get("coast.length_2dp").unwrap()),
            25.,
            -3.2,
            -1.65,
            Palette::ink(),
        )?;
        fade_in_all(s, &[title, card, coast, truth], 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;
        say(&k, s, "21_2")?;
        let rules = vec![
            label(
                &k,
                s,
                "random direction over a half turn",
                25.,
                3.,
                1.65,
                Palette::teal(),
            )?,
            label(
                &k,
                s,
                "independent random slide over one gap",
                24.,
                3.,
                0.85,
                Palette::gold(),
            )?,
            label(
                &k,
                s,
                "fresh independent choices each drop",
                24.,
                3.,
                0.05,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                &format!("retained seed {}", drops::SEED),
                21.,
                3.,
                -0.75,
                Palette::dim(),
            )?,
        ];
        fade_in_all(s, &rules, 0.5)?;
        k.hold(s)?;
        pause(s, 2.)?;
        say(&k, s, "21_3")?;
        let mut placed = Vec::new();
        let mut row = Vec::new();
        for (i, &d) in draws[..facts::FEW_DROPS].iter().enumerate() {
            fade_out_all(s, &placed, 0.15)?;
            placed = placement(s, &c, d);
            fade_in_all(s, &placed, 0.25)?;
            let number = label(
                &k,
                s,
                &counts[i].to_string(),
                33.,
                -4.95 + i as f64 * 1.1,
                -2.45,
                Palette::gold(),
            )?;
            fade_in_all(s, &[number], 0.2)?;
            row.push(number);
            pause(s, 1.4)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "21_4")?;
        let estimate = label(
            &k,
            s,
            &format!(
                "mean {} × π ≈ {} cm",
                f.get("drops.few.mean").unwrap(),
                f.get("drops.few.estimate").unwrap()
            ),
            34.,
            0.,
            -3.2,
            Palette::gold(),
        )?;
        fade_in_all(s, &[estimate], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "21_5")?;
        fade_out_all(s, &rules, 0.3)?;
        k.hold(s)?;
        let ring = think(&k, s, v(3., 0.3))?;
        fade_out_all(s, &[ring], 0.2)?;
        let spoken = say(&k, s, "21_6")?;
        fade_out_all(s, &rules, 0.3)?;
        fade_out_all(s, &row, 0.3)?;
        fade_out_all(s, &[estimate], 0.3)?;
        let (hist, model_hist) = histogram(&k, s, &drops::histogram(&counts), &shares)?;
        let hist_title = label(
            &k,
            s,
            &format!("{} retained drops · frequency", draws.len()),
            26.,
            3.,
            2.6,
            Palette::teal(),
        )?;
        fade_in_all(s, &hist, 0.6)?;
        fade_in_all(s, &[hist_title], 0.3)?;
        spoken.wait_for(s, "The gold outlines")?;
        let line = say_after_model(&k, s, &model_hist, &f)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "21_7")?;
        fade_out_all(s, &hist, 0.3)?;
        fade_out_all(s, &model_hist, 0.3)?;
        fade_out_all(s, &[hist_title, line], 0.3)?;
        let (_, sd) = model::moments(&shares);
        let one = sd * model::worth(model::GAP);
        let ten = one / (facts::FEW_DROPS as f64).sqrt();
        let spread_picture = spread(&k, s, one, ten)?;
        // Reveal the ten-drop row only after its narration starts.
        fade_in_all(s, &spread_picture[..4], 0.5)?;
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "21_8")?;
        fade_in_all(s, &spread_picture[4..], 0.5)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "21_9")?;
        fade_out_all(s, &spread_picture, 0.3)?;
        let totals = drops::totals(&drops::counts(&drops::repeated().map_err(fail)?, &c));
        let mut dots = Vec::new();
        for (i, &total) in totals.iter().enumerate() {
            let color = if drops::far(total, &c) {
                Palette::red()
            } else {
                Palette::teal()
            };
            dots.push(rectangle(
                s,
                0.3 + (i % 50) as f64 * 0.115,
                1.9 - (i / 50) as f64 * 0.15,
                0.075,
                0.075,
                color,
                1.,
            )?);
        }
        let ensemble = vec![
            label(
                &k,
                s,
                &format!("{} groups · {} drops per group", totals.len(), facts::FEW_DROPS),
                25.,
                3.1,
                2.65,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                "each square = a fresh group estimate",
                23.,
                3.1,
                -1.35,
                Palette::ink(),
            )?,
            label(
                &k,
                s,
                &format!(
                    "red: error > {} cm · {}% of groups",
                    drops::ERROR_CM,
                    f.get("drops.ten.far_percent").unwrap()
                ),
                25.,
                3.1,
                -2.0,
                Palette::red(),
            )?,
        ];
        fade_in_all(s, &dots, 0.7)?;
        fade_in_all(s, &ensemble, 0.5)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "21_10")?;
        let theoretical = label(
            &k,
            s,
            &format!(
                "numerical model: about {}%",
                f.get("drops.ten.far_exact_percent").unwrap()
            ),
            29.,
            0.,
            -2.8,
            Palette::gold(),
        )?;
        let qualifier = label(
            &k,
            s,
            "sum probabilities for totals outside the ±1 cm window",
            23.,
            0.,
            -3.35,
            Palette::ink(),
        )?;
        fade_in_all(s, &[theoretical, qualifier], 0.5)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "21_11")?;
        fade_out_all(s, &dots, 0.3)?;
        fade_out_all(s, &ensemble, 0.3)?;
        fade_out_all(s, &[theoretical, qualifier], 0.3)?;
        let callback = label(
            &k,
            s,
            &format!("first ten: {} cm", f.get("drops.few.estimate").unwrap()),
            28.,
            3.,
            0.7,
            Palette::gold(),
        )?;
        fade_in_all(s, &[callback], 0.5)?;
        let note = precisely(&k, s, NOTE)?;
        fade_in_all(s, &[note], 0.5)?;
        let deadline = now(s) + (NOTE.chars().count() + 10) as f64 / 15. + 2.;
        k.hold(s)?;
        wait_until(s, deadline)?;
        k.finish(s)?;
        Ok(())
    }
}
fn say_after_model(k: &Kit, s: &mut Stage<'_>, hist: &[Mob], f: &explainer_kit::Facts) -> fmn::Result<Mob> {
    // Population bars are numerical estimates; the tiny top bin is called out.
    fade_in_all(s, hist, 0.5)?;
    let note = label(
        k,
        s,
        &format!(
            "gold: model · {} crossings ≈ {}%",
            f.get("drops.top_count").unwrap(),
            f.get("drops.top_count_percent").unwrap()
        ),
        23.,
        3.,
        -2.3,
        Palette::gold(),
    )?;
    fade_in_all(s, &[note], 0.4)?;
    Ok(note)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numerical_population_convolution_preserves_mass_mean_and_variance() {
        let c = model::coast();
        let p = c.count_shares(facts::SHARE_DIRECTIONS, Some(model::GAP));
        let q = model::sum_shares(&p, facts::FEW_DROPS);
        let (m, s) = model::moments(&p);
        let (mm, ss) = model::moments(&q);
        assert!((p.iter().sum::<f64>() - 1.).abs() < 1e-10);
        assert!((q.iter().sum::<f64>() - 1.).abs() < 1e-9);
        assert!((mm - 10. * m).abs() < 1e-8);
        assert!((ss * ss - 10. * s * s).abs() < 1e-8);
        assert!((PI * ss / 10. - PI * s / 10f64.sqrt()).abs() < 1e-8);
    }
}
