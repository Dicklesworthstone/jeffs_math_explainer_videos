//! The same six seeded simulations under all three dispatch policies.

use explainer_kit::{Facts, Frame2, Kit, Palette, Timed, thousands, v};
use fmn::prelude::*;

use crate::finite_routes::{clear, rectangle, show, text_title};
use crate::service_tail::LOAD;

const SERVERS: u32 = 24;
const ARRIVALS: u32 = 30_000;
const WARM_UP: u32 = 5_000;
const SEEDS: usize = 6;
const AXIS_PERIOD: f64 = 0.25;
const POLICY: [&str; 3] = ["First address", "Shorter of two", "Shortest of all"];

struct Refresh {
    period: f64,
    percentiles: [[f64; SEEDS]; 3],
}

impl Refresh {
    fn means(&self) -> [f64; 3] {
        self.percentiles.map(|row| row.into_iter().sum::<f64>() / SEEDS as f64)
    }

    fn shorter_wins(&self) -> usize {
        self.percentiles[0]
            .into_iter()
            .zip(self.percentiles[1])
            .filter(|(first, shorter)| shorter < first)
            .count()
    }
}

fn replay() -> Vec<Refresh> {
    let data: Vec<_> = include_str!("replay.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect();
    assert_eq!(data.len(), 24);
    data.as_chunks::<3>()
        .0
        .iter()
        .map(|chunk| {
            let mut period = 0.0;
            let percentiles = std::array::from_fn(|i| {
                let mut columns = chunk[i].split('\t');
                let parsed: f64 = columns
                    .next()
                    .expect("refresh period")
                    .parse()
                    .expect("numeric refresh period");
                if i == 0 {
                    period = parsed;
                } else {
                    assert_eq!(period, parsed);
                }
                assert_eq!(columns.next(), Some(["d1", "p2c", "jsq"][i]));
                let waits = std::array::from_fn(|_| {
                    columns
                        .next()
                        .expect("per-seed percentile")
                        .parse::<f64>()
                        .expect("numeric percentile")
                });
                assert!(columns.next().is_none() && waits.iter().all(|v| v.is_finite() && *v > 0.0));
                waits
            });
            Refresh { period, percentiles }
        })
        .collect()
}

fn uniform_tie_mean() -> f64 {
    let mut columns = include_str!("tie-sensitivity.tsv").trim_end().split('\t');
    assert_eq!(columns.next(), Some("2.0"));
    let values: [f64; SEEDS] = std::array::from_fn(|_| {
        columns
            .next()
            .expect("one wait per replay seed")
            .parse()
            .expect("numeric wait")
    });
    assert!(columns.next().is_none() && values.iter().all(|value| value.is_finite() && *value > 0.0));
    values.into_iter().sum::<f64>() / SEEDS as f64
}

pub(super) fn facts(out: &mut Facts) -> Result<(), String> {
    let data = replay();
    let source = "queue-refresh-replay02.json, recomputed from six per-seed percentiles";
    let fresh = data[0].means();
    out.pin("replay.fresh_first", format!("{:.2}", fresh[0]), "30.25", source)?;
    out.pin("replay.fresh_shorter", format!("{:.2}", fresh[1]), "6.87", source)?;
    out.pin("replay.fresh_all", format!("{:.2}", fresh[2]), "3.10", source)?;
    let quarter = data[1].means();
    out.pin("replay.quarter_all", format!("{:.2}", quarter[2]), "10.19", source)?;
    out.pin("replay.quarter_shorter", format!("{:.2}", quarter[1]), "7.17", source)?;
    let arrivals_per_refresh = |row: &Refresh| f64::from(SERVERS) * LOAD * row.period;
    out.pin(
        "replay.quarter_arrivals",
        format!("{:.1}", arrivals_per_refresh(&data[1])),
        "5.1",
        source,
    )?;
    let two = data[4].means();
    out.pin("replay.two_all", format!("{:.1}", two[2]), "45.3", source)?;
    out.pin("replay.two_shorter", format!("{:.1}", two[1]), "8.7", source)?;
    out.pin(
        "replay.two_all_uniform_ties",
        format!("{:.2}", uniform_tie_mean()),
        "26.02",
        "Same six tapes under uniform ties, guarded power-of-two-tie-sensitivity01",
    )?;
    out.pin(
        "replay.two_arrivals",
        format!("{:.1}", arrivals_per_refresh(&data[4])),
        "40.8",
        source,
    )?;
    let last = data[7].means();
    out.pin("replay.thirty_shorter", format!("{:.2}", last[1]), "27.99", source)?;
    out.pin("replay.thirty_first", format!("{:.2}", last[0]), "30.25", source)?;
    out.pin("replay.thirty_all", format!("{:.2}", last[2]), "504.37", source)?;
    out.pin("replay.thirty_two_wins", data[7].shorter_wins(), "4", source)?;
    if !(fresh[2] < fresh[1] && fresh[1] < fresh[0] && quarter[2] > quarter[1] && two[2] > two[0] && last[1] < last[0])
    {
        return Err("policy comparisons disagree with the seeded replay".into());
    }
    Ok(())
}

fn colors() -> [Srgb; 3] {
    [Palette::teal(), Palette::hero(), Palette::red()]
}

pub fn fresh(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let data = replay();
    let row = &data[0];
    kit.say(stage, "06_1")?;
    let title = text_title(kit, stage, "Compare policies on the same arrivals")?;
    let params = kit.text(
        stage,
        &format!(
            "{SERVERS} servers    ·    {:.0}% nominal load    ·    {SEEDS} seeds",
            100.0 * LOAD
        ),
        25.0,
        Palette::hero(),
        v(0.0, 2.48),
    )?;
    let denominator = kit.text(
        stage,
        &format!(
            "{} arrivals per seed, discard the first {}",
            thousands(ARRIVALS),
            thousands(WARM_UP)
        ),
        20.0,
        Palette::dim(),
        v(0.0, 1.94),
    )?;
    show(stage, &[title, params, denominator], 0.55)?;
    let dot_key = kit.text(
        stage,
        "One dot gives one run's 99th-percentile wait",
        22.0,
        Palette::ink(),
        v(0.0, -2.62),
    )?;

    kit.say(stage, "06_2")?;
    let axis = Frame2 {
        center: v(1.03, 0.06),
        width: 7.2,
        height: 2.1,
        x: (0.0, 40.0),
        y: (-1.05, 1.05),
    };
    let color = colors();
    for i in 0..3 {
        let y = 1.04 - i as f64 * 0.92;
        let name = kit.text_left(stage, POLICY[i], 22.0, color[i], v(-5.65, y))?;
        let rail = stage.arena_mut().add(
            polyline(vec![v(axis.left(), y), v(axis.right(), y)])
                .style(Style::default().stroke(Palette::dim(), 2.0, 0.54))
                .build(),
        );
        show(stage, &[name, rail], 0.48)?;
        let mut dots = Vec::new();
        for (j, value) in row.percentiles[i].into_iter().enumerate() {
            let x = axis.p(value, 0.0)[0];
            let pt = Dot::new()
                .point(v(x, y + (j as f64 - 2.5) * 0.039))
                .radius(0.059)
                .color(color[i])
                .build();
            dots.push(stage.arena_mut().add(pt));
        }
        show(stage, &dots, 0.53)?;
    }
    for t in [0.0, 10.0, 20.0, 30.0, 40.0] {
        let label = kit.text(
            stage,
            &format!("{t:.0}"),
            17.0,
            Palette::dim(),
            v(axis.p(t, 0.0)[0], -1.55),
        )?;
        show(stage, &[label], 0.25)?;
    }
    show(stage, &[dot_key], 0.57)?;

    kit.say(stage, "06_3")?;
    let means = row.means();
    for (i, mean) in means.into_iter().enumerate() {
        let y = 1.04 - i as f64 * 0.92;
        let text = kit.text(stage, &format!("{mean:.2}"), 28.0, color[i], v(5.43, y))?;
        show(stage, &[text], 0.54)?;
    }
    let units = kit.text(
        stage,
        "Mean of the six percentiles, in mean service times",
        21.0,
        Palette::teal(),
        v(0.0, -1.97),
    )?;
    show(stage, &[units], 0.54)?;
    kit.finish(stage)
}

fn line_segment(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> Mob {
    let mark = polyline(vec![from, to])
        .style(Style::default().stroke(color, 3.5, 0.95))
        .build();
    stage.arena_mut().add(mark)
}

// Preserve actual measurement spacings while retaining the fresh t=0 case.
fn refresh_x(period: f64) -> f64 {
    (1.0 + period / AXIS_PERIOD).log2()
}

fn reveal_points(stage: &mut Stage<'_>, chart: &Frame2, all: &[Refresh], first: usize, last: usize) -> fmn::Result<()> {
    let color = colors();
    for i in first..=last {
        let mut arrivals = Vec::new();
        let values = all[i].means();
        for col in 0..3 {
            let point = chart.p(refresh_x(all[i].period), values[col].log10());
            if i > 0 {
                let prev = chart.p(refresh_x(all[i - 1].period), all[i - 1].means()[col].log10());
                arrivals.push(line_segment(stage, prev, point, color[col]));
            }
            let tip = Dot::new().point(point).radius(0.063).color(color[col]).build();
            arrivals.push(stage.arena_mut().add(tip));
        }
        show(stage, &arrivals, 0.43)?;
    }
    Ok(())
}

fn stage_baseline(kit: &Kit, stage: &mut Stage<'_>, data: &[Refresh]) -> fmn::Result<Frame2> {
    let chart = Frame2 {
        center: v(-0.20, 0.19),
        width: 9.95,
        height: 3.48,
        x: (0.0, refresh_x(data.last().expect("refresh runs").period)),
        y: (2.5_f64.log10(), 590_f64.log10()),
    };
    let header = text_title(kit, stage, "Old queue lengths change the result")?;
    let label = kit.text(
        stage,
        "Mean p99 wait across six simulations",
        22.0,
        Palette::ink(),
        v(0.0, 2.60),
    )?;
    let guide = kit.text(
        stage,
        "Mean service times, logarithmic scale",
        17.0,
        Palette::dim(),
        v(-3.77, 2.13),
    )?;
    let color = colors();
    let mut mobs = vec![header, label, guide];
    for (i, name) in POLICY.into_iter().enumerate() {
        mobs.push(kit.text(stage, name, 18.0, color[i], v((i as f64 - 1.0) * 3.52, -2.76))?);
    }
    for value in [3.0_f64, 10.0, 30.0, 100.0, 300.0] {
        let y = chart.p(0.0, value.log10())[1];
        let rail = polyline(vec![v(chart.left(), y), v(chart.right(), y)])
            .style(Style::default().stroke(Palette::dim(), 1.55, 0.4))
            .build();
        mobs.push(stage.arena_mut().add(rail));
        mobs.push(kit.text(stage, &format!("{value:.0}"), 19.0, Palette::dim(), v(-5.60, y))?);
    }
    for observation in data {
        let x = chart.p(refresh_x(observation.period), 1.0)[0];
        mobs.push(kit.text(
            stage,
            &observation.period.to_string(),
            19.0,
            Palette::ink(),
            v(x, -1.89),
        )?);
    }
    mobs.push(kit.tex(
        stage,
        r"\text{Refresh period }t\text{ in mean service times}\qquad\text{spacing }\log_2(1+4t)",
        19.0,
        Palette::dim(),
        &[],
        v(0.0, -2.27),
    )?);
    show(stage, &mobs, 0.57)?;
    Ok(chart)
}

pub fn stale(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let data = replay();
    kit.say(stage, "07_1")?;
    let title = text_title(kit, stage, "One published row can attract several jobs")?;
    let label = kit.text(
        stage,
        "Illustration between two updates",
        21.0,
        Palette::dim(),
        v(0.0, 2.54),
    )?;
    let bulletin = kit.text(stage, "Published counts: 0, 1, 1", 30.0, Palette::hero(), v(0.0, 1.37))?;
    let row = rectangle(stage, v(-1.9, -0.07), [2.05, 0.51], Palette::teal(), 0.10)?;
    let lane = kit.text(stage, "Actual lane A", 21.0, Palette::teal(), v(-1.90, -0.07))?;
    show(stage, &[title, label, bulletin, row, lane], 0.53)?;
    for n in 1..=3 {
        let tile = rectangle(
            stage,
            v(-1.03 + 1.0 * n as f64, -1.06),
            [0.78, 0.49],
            Palette::ink(),
            0.13,
        )?;
        let number = kit.text(
            stage,
            &n.to_string(),
            19.0,
            Palette::ink(),
            v(-1.03 + 1.0 * n as f64, -1.06),
        )?;
        show(stage, &[tile, number], 0.54)?;
    }
    let grow = kit.text(
        stage,
        "New jobs enter the actual queue. The published row stays fixed.",
        21.0,
        Palette::ink(),
        v(0.0, -2.25),
    )?;
    show(stage, &[grow], 0.56)?;
    clear(kit, stage)?;

    kit.say(stage, "07_2")?;
    let chart = stage_baseline(kit, stage, &data)?;
    reveal_points(stage, &chart, &data, 0, 1)?;
    let quarter = data[1].means();
    let at_quarter = kit.text(
        stage,
        &format!("At {}: two {:.2}, all {:.2}", data[1].period, quarter[1], quarter[2]),
        21.0,
        Palette::hero(),
        v(0.0, 1.93),
    )?;
    show(stage, &[at_quarter], 0.47)?;

    kit.say(stage, "07_3")?;
    let clear_quote = fade_out(stage.arena_mut(), at_quarter, ORIGIN, 1.0)?.rt(0.34);
    stage.play(clear_quote)?;
    reveal_points(stage, &chart, &data, 2, 4)?;
    let mid = data[4].means();
    let at_two = kit.text(
        stage,
        &format!("At {}: two {:.1}, all {:.1}", data[4].period, mid[1], mid[2]),
        21.0,
        Palette::hero(),
        v(0.0, 1.93),
    )?;
    let tie_rule = kit.text(
        stage,
        "All-server ties keep the first address when eligible, else the lowest index",
        17.0,
        Palette::dim(),
        v(0.0, -3.25),
    )?;
    show(stage, &[at_two, tie_rule], 0.51)?;
    kit.until_sentence(stage, "07_3", 3)?;
    let uniform = uniform_tie_mean();
    let alternate_point = Dot::new()
        .point(chart.p(refresh_x(data[4].period), uniform.log10()))
        .radius(0.083)
        .color(Palette::blue())
        .build();
    let alternate = stage.arena_mut().add(alternate_point);
    let changed = kit.text(
        stage,
        &format!("Only ties change: all-server p99 {:.1} → {:.2}", mid[2], uniform),
        21.0,
        Palette::blue(),
        v(0.0, 1.93),
    )?;
    let fade = fade_out(stage.arena_mut(), at_two, ORIGIN, 1.0)?.rt(0.25);
    stage.play(fade)?;
    show(stage, &[changed, alternate], 0.5)?;

    kit.say(stage, "07_4")?;
    let mut dismiss = Vec::new();
    for marker in [changed, alternate] {
        let fade = fade_out(stage.arena_mut(), marker, ORIGIN, 1.0)?.rt(0.36);
        dismiss.push(stage.prepare(fade)?);
    }
    stage.play_prepared(dismiss)?;
    reveal_points(stage, &chart, &data, 5, 7)?;
    let last = data[7].means();
    let at_last = kit.text(
        stage,
        &format!(
            "At {}: first {:.2}, two {:.2}, all {:.2}",
            data[7].period, last[0], last[1], last[2]
        ),
        21.0,
        Palette::hero(),
        v(0.0, 1.93),
    )?;
    show(stage, &[at_last], 0.54)?;
    let wins = kit.text(
        stage,
        &format!(
            "Two choices had lower p99 in {} of {} paired runs",
            data[7].shorter_wins(),
            SEEDS
        ),
        21.0,
        Palette::hero(),
        v(0.0, -3.66),
    )?;
    show(stage, &[wins], 0.54)?;
    kit.finish(stage)
}

pub(super) fn outcome(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let data = replay();
    let fresh = data[0].means();
    let last = data[7].means();
    let heading = text_title(kit, stage, "The comparison changes with the information")?;
    let mut rows = vec![heading];
    rows.push(kit.text_left(stage, "Refresh period", 25.0, Palette::ink(), v(-5.65, 1.66))?);
    for (i, name) in POLICY.into_iter().enumerate() {
        rows.push(kit.text(stage, name, 21.0, colors()[i], v(-1.95 + i as f64 * 3.1, 1.66))?);
    }
    for (row, values) in [(0, fresh), (7, last)] {
        let y = if row == 0 { 0.56 } else { -0.52 };
        let period = if row == 0 {
            "Fresh".into()
        } else {
            data[row].period.to_string()
        };
        rows.push(kit.text_left(stage, &period, 26.0, Palette::ink(), v(-5.65, y))?);
        for (i, val) in values.into_iter().enumerate() {
            rows.push(kit.text(
                stage,
                &format!("{val:.2}"),
                29.0,
                colors()[i],
                v(-1.95 + i as f64 * 3.1, y),
            )?);
        }
    }
    let foot = kit.text(
        stage,
        "Mean of six per-run p99 waits, in mean service times",
        19.0,
        Palette::dim(),
        v(0.0, -1.69),
    )?;
    rows.push(foot);
    rows.push(kit.text(
        stage,
        "All-server ties favor the first address when eligible, otherwise the lowest index",
        18.0,
        Palette::dim(),
        v(0.0, -2.30),
    )?);
    show(stage, &rows, 0.57)
}
