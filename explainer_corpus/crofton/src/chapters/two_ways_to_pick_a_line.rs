//! Chapter 16: identical sticks, a different explicitly specified line law.
//! Every count uses all retained pin pairs. The rapid animation shows selected
//! draws, not a claim that all ten thousand were individually displayed.
use super::draw::{fade_in_all, fade_out_all, fail, flat, line_style, now, pause, say, stop, think, wait_until};
use super::placement::{glide, live_count};
use crate::{facts, model, palette, pin_lines};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

pub struct TwoWaysToPickALine {
    pub kit: Kit,
}

const SCALE: f64 = 0.44;
const NOTE: &str = "Independent uniform pins on this rectangle; count their full line. These are finite seeded counts, not exact probabilities. Every draw is counted; selected draws are shown.";

fn point(p: model::P2) -> Vec3 {
    v(-2.9 + SCALE * p[0], 0.35 + SCALE * p[1])
}
fn path(s: &mut Stage<'_>, ps: &[model::P2], color: Srgb, width: f64) -> Mob {
    s.arena_mut().add(
        polyline(ps.iter().copied().map(point).collect::<Vec<_>>())
            .style(line_style(color, width))
            .build(),
    )
}
fn dot(s: &mut Stage<'_>, p: model::P2) -> Mob {
    s.arena_mut().add(
        Circle::new()
            .radius(0.065)
            .arc_center(point(p))
            .style(flat(Palette::gold(), 1.))
            .build(),
    )
}
fn txt(k: &Kit, s: &mut Stage<'_>, text: &str, size: f64, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, text, size, color, v(x, y))
}
fn working() -> Result<(Vec<pin_lines::PinLine>, [usize; 2]), String> {
    let lines = pin_lines::samples()?;
    let hits = pin_lines::totals(&lines);
    let f = facts::facts()?;
    for (key, value) in [
        ("lines.count", lines.len().to_string()),
        ("lines.pins.middle", hits[0].to_string()),
        ("lines.pins.corner", hits[1].to_string()),
    ] {
        if f.get(key) != Some(value.as_str()) {
            return Err(format!("picture differs from {key}"));
        }
    }
    Ok((lines, hits))
}

impl SceneConstruct for TwoWaysToPickALine {
    fn name(&self) -> &str {
        "16_two_ways_to_pick_a_line"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (lines, hits) = working().map_err(fail)?;
        say(&k, stage, "16_1")?;
        let card = stage.arena_mut().add(
            Rectangle::new()
                .width(model::CARD_W * SCALE)
                .height(model::CARD_H * SCALE)
                .corner_radius(0.06)
                .style(flat(palette::card(), 0.55))
                .build()?
                .moved_to(point([0., 0.])),
        );
        let coast = model::coast();
        let shore = path(
            stage,
            &coast.upto(coast.length, super::draw::EVERY),
            palette::coast(),
            3.,
        );
        let heading = txt(&k, stage, "How are we choosing the line?", 35., 0., 3.2, Palette::ink())?;
        let old_slide = txt(&k, stage, "even slide", 30., 3.65, 1.75, Palette::ink())?;
        let old_turn = txt(&k, stage, "even turn", 30., 3.65, 0.9, Palette::ink())?;
        fade_in_all(stage, &[card, shore, heading, old_slide, old_turn], 0.6)?;
        k.hold(stage)?;
        pause(stage, 1.5)?;

        say(&k, stage, "16_2")?;
        fade_out_all(stage, &[old_slide, old_turn], 0.35)?;
        let rule = txt(&k, stage, "independent uniform pins", 25., 3.65, 1.9, Palette::gold())?;
        let full = txt(&k, stage, "extend the full line", 25., 3.65, 1.15, Palette::ink())?;
        fade_in_all(stage, &[rule, full], 0.4)?;
        let index = stage.add_value_tracker(0.);
        let initial = lines[0];
        let pins = initial.pins.map(|p| dot(stage, p));
        let line = path(
            stage,
            &initial.chord().ok_or_else(|| fail("missing chord".into()))?,
            Palette::ink(),
            2.,
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let frames = lines.clone();
        stage.add_updater(
            anchor,
            move |s, _| {
                let i = (s.tracker_value(index).unwrap_or(0.).floor() as usize).min(frames.len() - 1);
                let sample = frames[i];
                for (mob, p) in pins.into_iter().zip(sample.pins) {
                    s.move_to(mob, point(p), ORIGIN);
                }
                if let Some(chord) = sample.chord() {
                    let shape = polyline(chord.into_iter().map(point).collect::<Vec<_>>()).build();
                    let _ = s.set_points(line, shape.points());
                }
            },
            true,
        )?;
        fade_in_all(stage, &[anchor, pins[0], pins[1]], 0.4)?;
        pause(stage, 1.)?;
        fade_in_all(stage, &[line], 0.6)?;
        pause(stage, 2.)?;
        for i in 1..4 {
            stage.play(glide(index, i as f64, 0.25, linear)?)?;
            pause(stage, 1.6)?;
        }
        k.hold(stage)?;
        pause(stage, 1.)?;

        say(&k, stage, "16_3")?;
        fade_out_all(stage, &[shore, line, pins[0], pins[1], full], 0.4)?;
        let sticks = pin_lines::STICKS.map(|ends| path(stage, &ends, palette::stick(), 7.));
        let middle = txt(
            &k,
            stage,
            "middle",
            24.,
            point(pin_lines::MIDDLE[0])[0],
            2.6,
            Palette::ink(),
        )?;
        let corner = txt(
            &k,
            stage,
            "corner",
            24.,
            point(pin_lines::CORNER[0])[0],
            2.6,
            Palette::ink(),
        )?;
        let equal = txt(&k, stage, "2 cm each · same direction", 25., 3.65, 0.95, Palette::ink())?;
        let count_note = txt(
            &k,
            stage,
            &format!("{} fresh pin pairs", lines.len()),
            26.,
            3.65,
            0.05,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[sticks[0], sticks[1], middle, corner, equal, count_note], 0.5)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "16_4")?;
        let question = txt(&k, stage, "Middle, corner, or a tie?", 28., 0., -2.35, Palette::ink())?;
        fade_in_all(stage, &[question], 0.4)?;
        let thinking = think(&k, stage, v(3.65, -1.25))?;

        say(&k, stage, "16_5")?;
        fade_out_all(stage, &[question, thinking, count_note], 0.35)?;
        let selected = txt(
            &k,
            stage,
            "All counted · selected draws shown",
            23.,
            0.,
            -2.35,
            Palette::dim(),
        )?;
        let total_label = txt(&k, stage, "pairs counted", 23., 3.65, -0.3, Palette::ink())?;
        fade_in_all(stage, &[selected, total_label], 0.3)?;
        stage.play(glide(index, 0., 0.01, linear)?)?;
        fade_in_all(stage, &[line, pins[0], pins[1]], 0.4)?;
        let alpha = stage.add_value_tracker(1.);
        let total = live_count(
            &k,
            stage,
            move |s| (s.tracker_value(index).unwrap_or(0.).floor() as usize + 1).min(pin_lines::COUNT),
            alpha,
            42.,
            v(4.5, -1.15),
        )?;
        stage.play(glide(index, (lines.len() - 1) as f64, 12., linear)?)?;
        stop(stage, anchor);
        k.hold(stage)?;
        pause(stage, 2.)?;

        let spoken = say(&k, stage, "16_6")?;
        fade_out_all(stage, &[line, pins[0], pins[1], total_label, total, selected], 0.4)?;
        let result_title = txt(
            &k,
            stage,
            "Crossings · this seeded run",
            24.,
            3.65,
            0.05,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[result_title], 0.3)?;
        let mut results = vec![result_title];
        for (i, (name, n, phrase)) in [("middle", hits[0], "two thousand"), ("corner", hits[1], "One thousand")]
            .into_iter()
            .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            let y = -0.7 - i as f64 * 0.9;
            let label = txt(&k, stage, &format!("{name}: {n}"), 31., 3.65, y, Palette::ink())?;
            let length = 4.0 * n as f64 / hits[0] as f64;
            let bar = stage.arena_mut().add(
                Line::new(v(1.65, y - 0.35), v(1.65 + length, y - 0.35))
                    .style(line_style(palette::stick(), 7.))
                    .build()?,
            );
            fade_in_all(stage, &[label, bar], 0.5)?;
            results.extend([label, bar]);
        }
        k.hold(stage)?;
        pause(stage, 3.)?;

        say(&k, stage, "16_7")?;
        let examples = [
            pin_lines::PinLine {
                pins: [[-2., -2.], [2., 2.]],
            },
            pin_lines::PinLine {
                pins: [[-7., 3.], [-6., 4.]],
            },
        ];
        let mut illustration = Vec::new();
        for example in examples {
            let ends = example.chord().ok_or_else(|| fail("illustration missed card".into()))?;
            let span = path(stage, &ends, Palette::ink(), 2.5);
            fade_in_all(stage, &[span], 0.4)?;
            illustration.push(span);
            for (a, b) in [(0.15, 0.7), (0.3, 0.9), (0.05, 0.55)] {
                let at = |u: f64| {
                    [
                        ends[0][0] + u * (ends[1][0] - ends[0][0]),
                        ends[0][1] + u * (ends[1][1] - ends[0][1]),
                    ]
                };
                let pair = [dot(stage, at(a)), dot(stage, at(b))];
                fade_in_all(stage, &pair, 0.3)?;
                pause(stage, 1.1)?;
                fade_out_all(stage, &pair, 0.2)?;
            }
        }
        let family = txt(
            &k,
            stage,
            "Illustration: long and short runs across the card",
            24.,
            0.,
            -2.35,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[family], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "16_8")?;
        illustration.push(family);
        fade_out_all(stage, &illustration, 0.4)?;
        let lesson = txt(&k, stage, "Random needs a recipe.", 31., 0., -2.35, Palette::ink())?;
        fade_in_all(stage, &[lesson], 0.4)?;
        k.hold(stage)?;
        pause(stage, 2.)?;

        say(&k, stage, "16_9")?;
        fade_out_all(stage, &[lesson], 0.3)?;
        let note = precisely(&k, stage, NOTE)?;
        fade_in_all(stage, &[note], 0.5)?;
        let deadline = now(stage) + NOTE.chars().count() as f64 / 15. + 2.;
        k.hold(stage)?;
        wait_until(stage, deadline)?;
        // All original sticks and results survive into the next comparison.
        let _ = (results, note);
        pause(stage, 1.)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picture_uses_all_seeded_draws_and_equal_sticks() {
        let (samples, hits) = working().unwrap();
        assert_eq!(samples.len(), pin_lines::COUNT);
        assert_eq!(hits, [2178, 1273]);
        for [a, b] in pin_lines::STICKS {
            assert_eq!(a[0], b[0]);
            assert_eq!((a[1] - b[1]).abs(), 2.);
        }
    }
}
