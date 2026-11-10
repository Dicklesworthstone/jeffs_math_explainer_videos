//! Chapter 15: the five descriptions, each run backward, and the point on the circle that leads to the companion
//! film. It stops short of that film's ruler examples, columns of logs, winding and branch cuts. "One point, many
//! turns" and "The logarithm has more to reveal" come from the earlier draft's `closing.rs`. Author source, not built
//! or rendered.
use super::ending_draw::{Plane, dot, fade_in_all, fade_out_all, line_of_math, line_style, live, swap, title, walk};
use crate::turning_model as t;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use std::f64::consts::TAU;

pub struct LogBridge {
    pub kit: Kit,
}

const NAME_X: f64 = -6.3;
const MATH_X: f64 = 3.2;
const ROW_TOP: f64 = 2.15;
const ROW_STEP: f64 = 0.9;
const WINDING_SECONDS: f64 = 2.4;

/// Review 721 W7: 15_3 opens on the new difficulty, not on a question 15_2 has just answered.
const OPENING: &str = "so far, each question had one answer";

/// Each row: the forward description and its backward partner. Row 3 carries the slope at zero, without which
/// 2 to the x obeys the same law (review 721 M11). Row 4 names its unit: positions are natural logarithms only in E's
/// own unit, and 07's slide rule used base ten (review 721 H3).
const ROWS: [(&str, &str, &str, Colour); 5] = [
    (
        "a series of factorials",
        r"\sum_{k}\frac{x^k}{k!}",
        r"\ln(1+u)=u-\frac{u^2}{2}+\frac{u^3}{3}-\cdots,\quad -1<u\leq 1",
        Colour::Gold,
    ),
    (
        "a product of tiny multipliers",
        r"\left(1+\frac{x}{n}\right)^n",
        r"\text{rate}=n\left(x^{1/n}-1\right)",
        Colour::Teal,
    ),
    (
        "a rate law with one solution",
        r"y'=y,\quad y(0)=1",
        r"\text{time to reach}\ x:\quad T'(x)=\frac{1}{x}",
        Colour::Green,
    ),
    (
        "adding inputs multiplies outputs",
        r"E(a+b)=E(a)\,E(b),\quad E'(0)=1",
        r"\ln(ab)=\ln a+\ln b",
        Colour::Blue,
    ),
    (
        "a ruler that multiplies by sliding",
        r"\text{unit of}\ E:\ \text{value at}\ x=E(x)",
        r"\text{unit of}\ E:\ \text{position of}\ x=\ln x",
        Colour::Hero,
    ),
];

#[derive(Clone, Copy)]
enum Colour {
    Gold,
    Teal,
    Green,
    Blue,
    Hero,
}

fn colour(c: Colour) -> Srgb {
    match c {
        Colour::Gold => Palette::gold(),
        Colour::Teal => Palette::teal(),
        Colour::Green => Palette::green(),
        Colour::Blue => Palette::blue(),
        Colour::Hero => Palette::hero(),
    }
}
fn row_y(i: usize) -> f64 {
    ROW_TOP - ROW_STEP * i as f64
}
fn plane() -> Plane {
    Plane {
        centre: v(-3.3, -0.25),
        scale: 1.85,
        half_w: 3.0,
        half_h: 2.55,
    }
}

/// 15_1: five descriptions of one object.
fn count_what_we_built(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "15_1")?;
    let heading = title(k, stage, "Five descriptions, one object")?;
    fade_in_all(stage, &[heading], 0.5)?;
    let mut names = Vec::new();
    let mut forward = Vec::new();
    for (i, (name, math, _, c)) in ROWS.into_iter().enumerate() {
        k.until_sentence(stage, "15_1", i + 1)?;
        let row = [
            k.text_left(stage, name, 28.0, colour(c), v(NAME_X, row_y(i)))?,
            line_of_math(k, stage, math, 30.0, colour(c), MATH_X, row_y(i))?,
        ];
        fade_in_all(stage, &row, 0.4)?;
        names.push(row[0]);
        forward.push(row[1]);
    }
    k.hold(stage)?;
    Ok((heading, names, forward))
}

/// One row's forward formula gives way to its backward partner.
fn backward(k: &Kit, stage: &mut Stage<'_>, row: usize, forward: Mob) -> fmn::Result<Mob> {
    let (_, _, math, c) = ROWS[row];
    let size = if row == 0 || row == 2 { 27.0 } else { 30.0 };
    let new = line_of_math(k, stage, math, size, colour(c), MATH_X, row_y(row))?;
    swap(stage, &[forward], &[new], 0.5)?;
    Ok(new)
}

/// 15_2: each description runs backward, in the order the voice takes them: ruler, then the slide rule's longer unit
/// with the law that adds positions, then law, multipliers and series.
fn run_it_backward(k: &Kit, stage: &mut Stage<'_>, heading: Mob, forward: &[Mob]) -> fmn::Result<(Mob, Vec<Mob>)> {
    k.say(stage, "15_2")?;
    let backward_heading = title(k, stage, "The same five, run backward")?;
    swap(stage, &[heading], &[backward_heading], 0.5)?;

    let mut shown = Vec::new();
    for (row, sentence) in [(4, 1), (3, 2), (2, 3), (1, 4)] {
        k.until_sentence(stage, "15_2", sentence)?;
        shown.push(backward(k, stage, row, forward[row])?);
        if row == 4 {
            // Review 721 H3: one base-ten unit of the slide rule spans ln 10 of E's units.
            k.until_sentence(stage, "15_2", 2)?;
            let longer = line_of_math(
                k,
                stage,
                &format!(
                    r"\text{{slide rule: the same marks, a unit}}\ {}\ \text{{times as long}}",
                    t::five(10.0_f64.ln())
                ),
                23.0,
                Palette::dim(),
                MATH_X,
                row_y(4) - 0.6,
            )?;
            fade_in_all(stage, &[longer], 0.4)?;
            shown.push(longer);
        }
    }

    k.until_sentence(stage, "15_2", 5)?;
    let mut table = vec![line_of_math(
        k,
        stage,
        &format!(r"x={:.0}:", t::ROOT_INPUT),
        24.0,
        Palette::teal(),
        -5.9,
        -2.75,
    )?];
    for (i, n) in t::ROOT_CELLS.into_iter().enumerate() {
        table.push(line_of_math(
            k,
            stage,
            &format!(r"n={n}:\ {}", t::five(t::backward_rate(t::ROOT_INPUT, n))),
            24.0,
            Palette::teal(),
            -4.25 + 2.05 * i as f64,
            -2.75,
        )?);
    }
    table.push(line_of_math(
        k,
        stage,
        &format!(r"\ln 2={}", t::five(t::ROOT_INPUT.ln())),
        24.0,
        Palette::hero(),
        5.85,
        -2.75,
    )?);
    fade_in_all(stage, &table, 0.5)?;
    k.until_sentence(stage, "15_2", 6)?;
    shown.push(backward(k, stage, 0, forward[0])?);
    k.hold(stage)?;
    shown.extend(table);
    Ok((backward_heading, shown))
}

/// 15_3: one output, many inputs, and the hand-off to the companion film.
fn one_point_many_turns(k: &Kit, stage: &mut Stage<'_>, heading: Mob, names: &[Mob], last: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "15_3")?;
    fade_out_all(stage, names, 0.4)?;
    fade_out_all(stage, last, 0.4)?;
    let new_heading = title(k, stage, "One point, many turns")?;
    swap(stage, &[heading], &[new_heading], 0.5)?;
    let question = k.text(stage, OPENING, 28.0, Palette::hero(), v(3.4, 2.3))?;
    fade_in_all(stage, &[question], 0.5)?;

    k.until_sentence(stage, "15_3", 1)?;
    let real = [
        line_of_math(k, stage, r"x\ \to\ E(x)>0", 33.0, Palette::green(), 3.4, 1.35)?,
        k.text(
            stage,
            "every positive number: exactly one input",
            24.0,
            Palette::dim(),
            v(3.4, 0.7),
        )?,
    ];
    fade_in_all(stage, &real, 0.5)?;

    k.until_sentence(stage, "15_3", 2)?;
    let axes = plane().axes(stage)?;
    let circle = plane().unit_circle(stage);
    let destination = dot(stage, plane().p(t::ONE), 0.12, Palette::hero());
    let same = line_of_math(k, stage, r"E(0)=E(2\pi i)=1", 33.0, Palette::blue(), 3.4, -0.3)?;
    fade_in_all(stage, &[axes, circle, destination, same], 0.5)?;

    k.until_sentence(stage, "15_3", 3)?;
    let clock = stage.add_value_tracker(0.0);
    let turn = live(stage, clock, |s, now| {
        let points = plane().arc(0.0, now.max(1e-3));
        s.add(polyline(points).style(line_style(Palette::gold(), 5.0)).build())
    })?;
    walk(stage, clock, TAU, WINDING_SECONDS)?;
    stage.clear_updaters(turn, true);
    stage.scene_mut().bring_to_front(&[destination])?;

    k.until_sentence(stage, "15_3", 4)?;
    let more = k.text(
        stage,
        "The logarithm has more to reveal",
        30.0,
        Palette::green(),
        v(3.4, -1.5),
    )?;
    fade_in_all(stage, &[more], 0.6)
}

impl SceneConstruct for LogBridge {
    fn name(&self) -> &str {
        "15_logarithm_bridge"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (heading, names, forward) = count_what_we_built(&k, stage)?;
        let (heading, shown) = run_it_backward(&k, stage, heading, &forward)?;
        one_point_many_turns(&k, stage, heading, &names, &shown)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ending_draw::{assert_in_order, scene_source};
    const SOURCE: &str = include_str!("log_bridge.rs");

    #[test]
    fn the_cues_run_in_film_order() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.say(stage, "15_1")?;"#,
                r#"k.say(stage, "15_2")?;"#,
                r#"k.say(stage, "15_3")?;"#,
            ],
        );
    }
    use super::{OPENING, ROWS};

    #[test]
    fn the_table_arrives_before_the_series_row_it_precedes_in_the_voice() {
        assert_in_order(
            SOURCE,
            &[
                "for (row, sentence) in [(4, 1), (3, 2), (2, 3), (1, 4)]",
                r#"k.until_sentence(stage, "15_2", 2)?;"#,
                "t::five(10.0_f64.ln())",
                r#"k.until_sentence(stage, "15_2", 5)?;"#,
                "t::backward_rate(t::ROOT_INPUT, n)",
                r#"k.until_sentence(stage, "15_2", 6)?;"#,
                "backward(k, stage, 0, forward[0])",
            ],
        );
    }
    #[test]
    fn the_rows_pin_the_exponential_and_name_the_ruler_unit() {
        // Review 721 M11: without the slope at zero, 2 to the x also turns adding into multiplying.
        assert!(ROWS[3].1.contains("E'(0)=1"));
        let two = |x: f64| 2.0_f64.powf(x);
        assert!((two(0.7 + 1.1) - two(0.7) * two(1.1)).abs() < 1e-12);
        assert!(((two(1e-6) - two(-1e-6)) / 2e-6 - 1.0).abs() > 0.3);
        // Review 721 H3: both ruler rows name the unit in which positions are natural logarithms.
        assert!(ROWS[4].1.contains(r"\text{unit of}\ E") && ROWS[4].2.contains(r"\text{unit of}\ E"));
    }
    #[test]
    fn the_last_cue_opens_on_the_new_difficulty() {
        // Review 721 W7.
        assert_eq!(OPENING, "so far, each question had one answer");
        assert!(!scene_source(SOURCE).contains("what was the input?"));
    }
    #[test]
    fn the_circle_arrives_with_its_sentence() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "15_3", 2)?;"#,
                "plane().unit_circle(stage)",
                r#"k.until_sentence(stage, "15_3", 3)?;"#,
                "walk(stage, clock, TAU",
                r#"k.until_sentence(stage, "15_3", 4)?;"#,
                "The logarithm has more to reveal",
            ],
        );
    }
    fn teaching_source(source: &'static str) -> String {
        scene_source(source)
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
            .to_ascii_lowercase()
    }

    #[test]
    fn topic_scan_ignores_documentation_but_keeps_displayed_words() {
        let comment = teaching_source("//! branch; rule of 70; matrix; winding number");
        let shown = teaching_source(r#"let label = "branch; rule of 70; matrix; winding number";"#);
        for word in ["branch", "rule of 70", "matrix", "winding number"] {
            assert!(!comment.contains(word));
            assert!(shown.contains(word));
        }
    }

    #[test]
    fn the_bridge_does_not_retell_the_companion_film() {
        let scene = teaching_source(SOURCE);
        for word in ["branch", "rule of 70", "matrix", "winding number"] {
            assert!(!scene.contains(word), "{word}");
        }
    }
}
