//! Changing spacing while preserving the random-drop measure and the coast.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::{placement, the_longimeter};
use crate::{drop_samples, facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

pub struct TheGap {
    pub kit: Kit,
}
const NOTE: &str = "Finite rectifiable path; count all traversals. Uniform half-turn direction and independent uniform full-gap slide. Population SD is numerical, not an error bound or coverage interval. Chords can equal straight arcs.";
pub fn count(c: &model::Curve, degrees: f64, slide: f64, gap: f64) -> usize {
    (-8..=8)
        .map(|i| c.meetings(degrees.to_radians(), slide + f64::from(i) * gap))
        .sum()
}
/// The oracle restarts the same seed for each gap. Reuse the stored pairs;
/// for these gaps scaling by 1, 2, 4 is exact in binary64, without RNG calls.
pub fn retained_counts(c: &model::Curve, gap: f64) -> Result<Vec<usize>, String> {
    Ok(drop_samples::samples()?
        .iter()
        .map(|d| count(c, d.degrees, d.slide * (gap / model::GAP), gap))
        .collect())
}
fn physical(p: model::P2) -> Vec3 {
    v(-3.5 + 0.33 * p[0], 0.5 + 0.33 * p[1])
}
fn path(s: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(polyline(pts).style(line_style(color, width)).build())
}
fn label(k: &Kit, s: &mut Stage<'_>, words: &str, x: f64, y: f64, size: f64) -> fmn::Result<Mob> {
    k.text(s, words, size, Palette::ink(), v(x, y))
}
struct Sheet {
    gap: Mob,
    alpha: Mob,
    driver: Mob,
    lines: Vec<Mob>,
    dots: Vec<Mob>,
}
impl Sheet {
    fn new(s: &mut Stage<'_>, c: &model::Curve) -> fmn::Result<Self> {
        let gap = s.add_value_tracker(model::GAP);
        let alpha = s.add_value_tracker(0.);
        let lines: Vec<_> = (0..17)
            .map(|_| path(s, vec![physical([0., 0.]), physical([0.001, 0.])], Palette::teal(), 1.2))
            .collect();
        let dots: Vec<_> = (0..40)
            .map(|_| {
                s.arena_mut().add(
                    Circle::new()
                        .radius(0.043)
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
        let c = c.clone();
        let mut previous = None;
        s.add_updater(
            driver,
            move |s, _| {
                let g = s.tracker_value(gap).unwrap_or(model::GAP);
                let a = s.tracker_value(alpha).unwrap_or(0.);
                if previous == Some((g, a)) {
                    return;
                }
                previous = Some((g, a));
                let beta = 35_f64.to_radians();
                let mut pts = Vec::new();
                for (j, &mob) in ls.iter().enumerate() {
                    let h = (j as f64 - 8. + 0.37) * g;
                    if let Some([p, q]) = placement::clipped(beta, h) {
                        let _ = s.set_points(mob, polyline(vec![physical(p), physical(q)]).build().points());
                        s.set_family_opacity(mob, a * 0.55);
                    } else {
                        s.set_family_opacity(mob, 0.);
                    }
                    pts.extend(the_longimeter::intersections(&c, beta, h));
                }
                assert!(pts.len() <= ds.len());
                for (i, &m) in ds.iter().enumerate() {
                    if let Some(&p) = pts.get(i) {
                        let _ = s.move_to(m, physical(p), ORIGIN);
                        s.set_family_opacity(m, a);
                    } else {
                        s.set_family_opacity(m, 0.);
                    }
                }
            },
            true,
        )?;
        s.add_to_scene(driver)?;
        Ok(Self {
            gap,
            alpha,
            driver,
            lines,
            dots,
        })
    }
    fn set(&self, s: &mut Stage<'_>, gap: f64) -> fmn::Result<()> {
        s.play(placement::glide(self.gap, gap, 3., linear)?)?;
        Ok(())
    }
    fn visibility(&self, s: &mut Stage<'_>, a: f64) -> fmn::Result<()> {
        s.play(placement::glide(self.alpha, a, 0.4, linear)?)?;
        Ok(())
    }
    fn end(self, s: &mut Stage<'_>) {
        stop(s, self.driver);
        for m in self.lines.into_iter().chain(self.dots) {
            s.set_family_opacity(m, 0.);
        }
    }
}
fn histogram(k: &Kit, s: &mut Stage<'_>, c: &model::Curve) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let mut sample = vec![label(k, s, "filled: 1,000 retained drops", 3.1, 2.75, 23.)?];
    let mut population = vec![label(k, s, "gold: numerical population probabilities", 3.1, 2.35, 20.)?];
    let x = |length: f64| 0.3 + length * 0.073;
    for (row, (gap, key)) in facts::WIDE_GAPS.into_iter().enumerate() {
        let base = 1.4 - row as f64 * 1.45;
        let counts = retained_counts(c, gap).map_err(fail)?;
        let hist = drop_samples::histogram(&counts);
        let shares = c.count_shares(facts::SHARE_DIRECTIONS, Some(gap));
        for (i, &p) in shares.iter().enumerate() {
            let xx = x(i as f64 * model::worth(gap));
            let h = hist.get(i).copied().unwrap_or(0) as f64 / counts.len() as f64 * 1.7;
            if h > 0. {
                sample.push(
                    s.arena_mut().add(
                        Rectangle::new()
                            .width(0.11)
                            .height(h)
                            .style(flat(Palette::teal(), 0.85))
                            .build()?
                            .moved_to(v(xx, base + h / 2.)),
                    ),
                );
            }
            if p > 0. {
                population.push(path(
                    s,
                    vec![
                        v(xx - 0.085, base),
                        v(xx - 0.085, base + p * 1.7),
                        v(xx + 0.085, base + p * 1.7),
                        v(xx + 0.085, base),
                    ],
                    Palette::gold(),
                    1.8,
                ));
            }
        }
        let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64 * model::worth(gap);
        sample.push(path(s, vec![v(x(0.), base), v(x(80.), base)], Palette::dim(), 1.));
        for tick in [0., 40., 80.] {
            sample.push(path(
                s,
                vec![v(x(tick), base), v(x(tick), base - 0.08)],
                Palette::ink(),
                1.,
            ));
        }
        sample.push(path(
            s,
            vec![v(x(c.length), base - 0.08), v(x(c.length), base + 0.75)],
            Palette::ink(),
            1.2,
        ));
        sample.push(label(
            k,
            s,
            &format!("{key} cm gap · finite mean {} cm", facts::fixed(mean, 1)),
            3.1,
            base - 0.28,
            21.,
        )?);
    }
    for tick in [0., 40., 80.] {
        sample.push(label(k, s, &facts::fixed(tick, 0), x(tick), -2.15, 20.)?);
    }
    sample.push(label(k, s, "bar height: fraction of drops", -3.5, -2.1, 21.)?);
    sample.push(label(k, s, "length estimate (cm)", 3.1, -2.55, 20.)?);
    sample.push(label(
        k,
        s,
        &format!("white marks: true length ≈ {} cm", facts::fixed(c.length, 3)),
        3.1,
        -2.95,
        20.,
    )?);
    Ok((sample, population))
}
fn walk(s: &mut Stage<'_>, c: &model::Curve, step: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let w = c.walk(step);
    let mut pts: Vec<_> = w.landings.iter().map(|x| x.0).collect();
    pts.push(c.at(c.length));
    let mut out = Vec::new();
    for pair in pts.windows(2) {
        let m = path(s, pair.iter().copied().map(physical).collect(), color, 2.5);
        fade_in_all(s, &[m], 0.09)?;
        out.push(m);
    }
    Ok(out)
}
impl SceneConstruct for TheGap {
    fn name(&self) -> &str {
        "25_the_gap"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let c = model::coast();
        let f = facts::facts().map_err(fail)?;
        let title = label(&k, s, "Choose a spacing", 0., 3.45, 36.)?;
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
            c.upto(c.length, 0.025).into_iter().map(physical).collect(),
            palette::coast(),
            3.,
        );
        let selected = label(&k, s, "selected illustration · same coast", -3.5, -1.35, 21.)?;
        let sheet = Sheet::new(s, &c)?;
        let spoken = say(&k, s, "25_1")?;
        fade_in_all(s, &[title, card, coast, selected], 0.5)?;
        sheet.visibility(s, 1.)?;
        let mut gap_label = label(&k, s, "gap: 2 cm", 3., 1.5, 36.)?;
        fade_in_all(s, &[gap_label], 0.3)?;
        for (gap, anchor) in [(4., "four centimetres"), (8., "then eight")] {
            spoken.wait_for(s, anchor)?;
            sheet.set(s, gap)?;
            fade_out_all(s, &[gap_label], 0.2)?;
            gap_label = label(&k, s, &format!("gap: {} cm", facts::fixed(gap, 0)), 3., 1.5, 36.)?;
            fade_in_all(s, &[gap_label], 0.2)?;
            pause(s, 2.)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "25_2")?;
        let law = label(&k, s, "direction uniform on a half turn", 3., 0.6, 24.)?;
        let offset = label(&k, s, "independent slide uniform on [0, gap)", 3., -0.2, 23.)?;
        fade_in_all(s, &[law, offset], 0.4)?;
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "25_3")?;
        fade_out_all(s, &[gap_label, law, offset], 0.3)?;
        let mut worths = Vec::new();
        for (row, (gap, key)) in facts::WIDE_GAPS.into_iter().enumerate() {
            let exact = ["π", "2π", "4π"][row];
            worths.push(label(
                &k,
                s,
                &format!(
                    "{key} cm gap: {exact} ≈ {} cm per crossing",
                    f.get(&format!("gap.{key}.worth")).unwrap()
                ),
                3.,
                2. - row as f64 * 0.75,
                25.,
            )?);
            assert!(gap > 0.);
        }
        let question = label(&k, s, "Does the average length change?", 0., -2.3, 28.)?;
        fade_in_all(s, &worths, 0.4)?;
        fade_in_all(s, &[question], 0.3)?;
        let thinking = think(&k, s, v(3., -1.1))?;
        say(&k, s, "25_4")?;
        fade_out_all(s, &[thinking, question], 0.3)?;
        fade_out_all(s, &worths, 0.3)?;
        let formula = label(&k, s, "2L / (πd) × πd / 2 = L", 3., 2.35, 30.)?;
        let def = label(&k, s, "L: path length · d: gap (both cm)", 3., 1.7, 23.)?;
        let mut means = vec![formula, def];
        for (row, (_, key)) in facts::WIDE_GAPS.into_iter().enumerate() {
            means.push(label(
                &k,
                s,
                &format!(
                    "{key} cm gap → mean ≈ {} crossings",
                    f.get(&format!("gap.{key}.mean")).unwrap()
                ),
                3.,
                0.95 - row as f64 * 0.65,
                24.,
            )?);
        }
        means.push(label(
            &k,
            s,
            &format!("same expectation ≈ {} cm", f.get("coast.length_3dp").unwrap()),
            3.,
            -1.5,
            27.,
        )?);
        fade_in_all(s, &means, 0.4)?;
        k.hold(s)?;
        pause(s, 5.)?;
        say(&k, s, "25_5")?;
        sheet.visibility(s, 0.)?;
        fade_out_all(s, &means, 0.3)?;
        let (sample, population) = histogram(&k, s, &c)?;
        fade_in_all(s, &sample, 0.5)?;
        k.hold(s)?;
        pause(s, 6.)?;
        let spoken = say(&k, s, "25_6")?;
        fade_in_all(s, &population, 0.5)?;
        spoken.wait_for(s, "Its standard deviations")?;
        fade_out_all(s, &sample, 0.3)?;
        fade_out_all(s, &population, 0.3)?;
        let mut spreads = vec![
            label(&k, s, "numerical population SD (cm)", 3., 2.5, 25.)?,
            label(&k, s, "SD = √mean squared deviation", 3., 1.85, 24.)?,
        ];
        for (row, (gap, key)) in facts::WIDE_GAPS.into_iter().enumerate() {
            let sd = model::moments(&c.count_shares(facts::SHARE_DIRECTIONS, Some(gap))).1 * model::worth(gap);
            let y = 1. - row as f64 * 1.1;
            spreads.push(path(
                s,
                vec![v(3. - sd * 0.13, y), v(3. + sd * 0.13, y)],
                Palette::gold(),
                4.,
            ));
            spreads.push(path(s, vec![v(3., y - 0.12), v(3., y + 0.12)], Palette::ink(), 2.));
            spreads.push(label(
                &k,
                s,
                &format!(
                    "{key} cm gap · SD {}",
                    f.get(&format!("gap.{key}.one_drop_spread")).unwrap()
                ),
                3.,
                y - 0.35,
                22.,
            )?);
        }
        spreads.push(label(&k, s, "spread, not an error guarantee", 3., -2.25, 24.)?);
        fade_in_all(s, &spreads, 0.4)?;
        k.hold(s)?;
        // Keep the earlier thirteen-second inspection budget after the words,
        // while the diagram itself now arrives during its explanation.
        pause(s, 13.)?;
        say(&k, s, "25_7")?;
        k.hold(s)?;
        pause(s, 4.)?;
        fade_out_all(s, &spreads, 0.3)?;
        fade_in_all(s, &sample, 0.3)?;
        fade_in_all(s, &population, 0.3)?;
        pause(s, 5.)?;
        say(&k, s, "25_8")?;
        fade_out_all(s, &sample, 0.3)?;
        fade_out_all(s, &population, 0.3)?;
        fade_out_all(s, &[selected], 0.3)?;
        let original = walk(s, &c, 2., Palette::red())?;
        let result = label(
            &k,
            s,
            &format!("2 cm step → {} cm", f.get("dividers.2_0.length").unwrap()),
            3.,
            1.9,
            30.,
        )?;
        let inequality = label(&k, s, "chord length ≤ replaced arc length", 3., 0.9, 25.)?;
        let equality = label(&k, s, "equality is possible on straight parts", 3., 0.1, 23.)?;
        fade_in_all(s, &[result, inequality, equality], 0.4)?;
        k.hold(s)?;
        pause(s, 4.)?;
        say(&k, s, "25_9")?;
        let question = label(&k, s, "1 cm steps: a longer total?", 0., -2.2, 29.)?;
        fade_in_all(s, &[question], 0.3)?;
        let thinking = think(&k, s, v(3., -1.1))?;
        let spoken = say(&k, s, "25_10")?;
        fade_out_all(s, &[question, thinking, result, inequality, equality], 0.3)?;
        fade_out_all(s, &original, 0.3)?;
        let mut shown = Vec::new();
        for (step, key, anchor) in [(1., "1_0", "one centimetre walk"), (0.5, "0_5", "Half a centimetre")] {
            spoken.wait_for(s, anchor)?;
            fade_out_all(s, &shown, 0.3)?;
            shown = walk(s, &c, step, Palette::red())?;
            let value = label(
                &k,
                s,
                &format!(
                    "{} cm step → {} cm",
                    facts::fixed(step, 1),
                    f.get(&format!("dividers.{key}.length")).unwrap()
                ),
                3.,
                1.6,
                30.,
            )?;
            shown.push(value);
            fade_in_all(s, &[value], 0.3)?;
            pause(s, 4.)?;
        }
        k.hold(s)?;
        pause(s, 3.)?;
        say(&k, s, "25_11")?;
        fade_out_all(s, &shown, 0.3)?;
        let reference = walk(s, &c, 2., Palette::dim())?;
        let exception = walk(s, &c, 1.9, Palette::red())?;
        let legend = label(&k, s, "red: 1.9 cm · gray: 2 cm", -3.5, -1.4, 23.)?;
        let mut table = vec![label(&k, s, "step (cm)       total (cm)", 3., 2.35, 24.)?];
        table.push(legend);
        for (row, (step, key)) in facts::DIVIDER_STEPS.into_iter().enumerate() {
            table.push(label(
                &k,
                s,
                &format!(
                    "{}                   {}",
                    facts::fixed(step, 1),
                    f.get(&format!("dividers.{key}.length")).unwrap()
                ),
                3.,
                1.7 - row as f64 * 0.53,
                25.,
            )?);
        }
        table.push(label(
            &k,
            s,
            &format!("2.0 and 1.9 cm: {} whole steps", f.get("dividers.1_9.steps").unwrap()),
            3.,
            -1.45,
            24.,
        )?);
        table.push(label(&k, s, "totals include the last short chord", 3., -2.05, 22.)?);
        fade_in_all(s, &table, 0.4)?;
        k.hold(s)?;
        pause(s, 6.)?;
        say(&k, s, "25_12")?;
        s.set_family_opacity(legend, 0.);
        let callback = label(
            &k,
            s,
            &format!(
                "sheet expectation ≈ {} cm at every gap",
                f.get("coast.length_3dp").unwrap()
            ),
            -3.5,
            -1.4,
            23.,
        )?;
        fade_in_all(s, &[callback], 0.4)?;
        let note = precisely(&k, s, NOTE)?;
        let full = now(s) + 0.5;
        fade_in_all(s, &[note], 0.5)?;
        k.hold(s)?;
        wait_until(s, full + (NOTE.chars().count() + 10) as f64 / 15. + 1.)?;
        pause(s, 2.)?;
        sheet.end(s);
        let _ = (reference, exception);
        k.finish(s)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reused_drops_and_crossing_geometry_preserve_measure_scaling() {
        let c = model::coast();
        let draws = drop_samples::samples().unwrap();
        assert_eq!(
            retained_counts(&c, model::GAP).unwrap(),
            drop_samples::counts(&draws, &c)
        );
        for (gap, _) in facts::WIDE_GAPS {
            let counts = retained_counts(&c, gap).unwrap();
            assert_eq!(counts.len(), draws.len());
            for (d, &n) in draws.iter().zip(&counts) {
                let slide = d.slide * (gap / model::GAP);
                assert_eq!(slide / gap, d.slide / model::GAP);
                let pts: usize = (-8..=8)
                    .map(|j| {
                        the_longimeter::intersections(&c, d.degrees.to_radians(), slide + f64::from(j) * gap).len()
                    })
                    .sum();
                assert_eq!(pts, n);
            }
        }
    }
    #[test]
    fn divider_chords_have_the_claimed_scope() {
        let c = model::coast();
        for (step, _) in facts::DIVIDER_STEPS {
            let w = c.walk(step);
            for pair in w.landings.windows(2) {
                let chord = (pair[1].0[0] - pair[0].0[0]).hypot(pair[1].0[1] - pair[0].0[1]);
                assert!((chord - step).abs() < 1e-8);
                assert!(chord <= pair[1].1 - pair[0].1 + 1e-8);
            }
            assert!(w.length() < c.length);
        }
        let a = c.walk(1.9);
        let b = c.walk(2.);
        assert_eq!(a.steps(), b.steps());
        assert!(a.length() < b.length());
        let straight = model::Curve {
            parts: vec![model::Part {
                seg: model::Seg::Line {
                    from: [0., 0.],
                    to: [6., 0.],
                },
                start: 0.,
            }],
            length: 6.,
        };
        assert!((straight.walk(2.).length() - straight.length).abs() < 1e-12);
    }
}
