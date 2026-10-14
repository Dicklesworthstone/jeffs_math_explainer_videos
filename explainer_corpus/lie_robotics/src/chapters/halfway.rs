//! Chapter 04, halfway: two headings nearly straight back (04_1), what is
//! halfway between them (04_2), the average that points the wrong way and
//! the ruler's seam (04_3), the ruler laid flat against the circle where the
//! arithmetic is honest (04_4), and the one trick that the rest of the video
//! repeats, which becomes the badge (04_5).

use std::f64::consts::TAU;

use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Ask, BADGE_CORNER, Badge, Choice, MINUS, QuizLook, fact_f64, fade_in_all, fade_in_lagged, flat, group, leave,
    line_style, say, value, your_turn,
};
use crate::model::{self, DEG};
use crate::palette;

pub struct Halfway {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const AHEAD: &str = "ahead";
const SEAM: &str = "seam";
const RULER_ENDS: [&str; 3] = ["–180", "0", "180"];
const STEP: &str = "step";
const HALF: &str = "half";
const CURVED: &str = "curved";
const FLAT_WORD: &str = "flat";
const WRAP_BACK: &str = "wrap back";
const THE_FLAT_PIECE: &str = "the flat piece:";
const TANGENT_SPACE: &str = "the tangent space";
const THE_RULER: &str = "the ruler";
/// How far under the flat ruler's end labels its own name stands.
const RULER_NAME_DROP: f64 = 0.95;
/// The size and width of 04_4's "for programmers" panel: its longest comment
/// line is 29 characters of typewriter face.
const CODE_SIZE: f64 = 22.0;
const CODE_WIDTH: f64 = 5.8;

/// 04_4's "for programmers" panel (R2 A-02: the convention of "halfway" in
/// plain words): the wrap, and the comment that says what it does with the
/// facts' own numbers: the difference of the headings, the bound, the step.
/// The panel is in the typewriter face, where a minus is the ASCII hyphen.
fn code_lines() -> fmn::Result<Vec<String>> {
    let (a, b) = (fact_f64("G-H.a")?, fact_f64("G-H.b")?);
    let (bound, gap) = (fact_f64("G-H.halfway")?, fact_f64("G-H.gap")?);
    let raw = (b - a).round() as i64;
    let (bound, gap) = (bound.round() as i64, gap.round() as i64);
    Ok(vec![
        "d = wrap(b - a)".to_owned(),
        "# bring the difference into".to_owned(),
        format!("# -{bound}...{bound}, the shorter way"),
        format!("# round: {raw} becomes +{gap}"),
    ])
}

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 5] = ["04_1", "04_2", "04_3", "04_4", "04_5"];

/// 04_2's quiz: what heading is halfway between 170 and −170? The keys are
/// the heading in degrees as `G-Q.04_2.answer` prints it.
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: "halfway between\n170° and –170°?",
    choices: &[
        Choice { key: "0", label: "0°" },
        Choice {
            key: "90",
            label: "90°",
        },
        Choice {
            key: "180",
            label: "180°",
        },
    ],
};

/// The circle of headings: its centre and radius on the screen. Heading 0 is
/// straight ahead (up), headings grow to the left (counter-clockwise).
const CENTRE: [f64; 2] = [-2.8, 0.75];
/// Small enough that the flat ruler, as long as the circle is round, and its
/// end labels stay inside the safe area (at 1.3 the "–180" ran off the frame).
const RADIUS: f64 = 1.2;
/// The two heading labels stand this far outward along x, so that "170°" and
/// "–170°" read as two numbers and not as one range.
const LABEL_A_SHIFT: f64 = -0.35;
const LABEL_B_SHIFT: f64 = 0.4;
/// Where "step 20°" stands against the second heading's dot on the flat piece.
const STEP_SHIFT: [f64; 2] = [1.6, 0.35];
/// Where the quiz card stands.
const CARD_AT: [f64; 2] = [4.35, 0.3];
const CARD_SCALE: f64 = 0.92;
/// The flat ruler sits this far under the circle.
const RULER_DROP: f64 = 1.05;

/// The direction of heading `h` (degrees) on the screen.
fn dir(h: f64) -> (f64, f64) {
    let t = h * DEG;
    (-t.sin(), t.cos())
}

/// A point on the circle (or on one of radius `r` about its centre) at
/// heading `h`.
fn on_circle(h: f64, r: f64) -> Vec3 {
    let (dx, dy) = dir(h);
    v(CENTRE[0] + r * dx, CENTRE[1] + r * dy)
}

/// Where the number beside heading `h` stands: outside the circle, `shift`
/// to the side.
fn label_spot(h: f64, shift: f64) -> Vec3 {
    let p = on_circle(h, RADIUS + 0.6);
    v(p[0] + shift, p[1])
}

/// The flat ruler's point for the value `u` in −180 to 180: a strip as long
/// as the circle is round, under it.
fn flat_point(u: f64) -> Vec3 {
    v(CENTRE[0] + u / 360.0 * TAU * RADIUS, CENTRE[1] - RADIUS - RULER_DROP)
}

/// The ruler `t` of the way (0 flat, 1 wrapped onto the circle) at value `u`:
/// the value lands on the heading of the same number.
fn ruler_point(u: f64, t: f64) -> Vec3 {
    let (s, c) = (flat_point(u), on_circle(u, RADIUS));
    v(s[0] + (c[0] - s[0]) * t, s[1] + (c[1] - s[1]) * t)
}

/// A robot's three corners, centred at `c`, nose along heading `h`.
fn robot_points(c: Vec3, h: f64) -> [Vec3; 3] {
    let (dx, dy) = dir(h);
    let (nx, ny) = (-dy, dx);
    [
        v(c[0] + 0.30 * dx, c[1] + 0.30 * dy),
        v(c[0] - 0.18 * dx + 0.17 * nx, c[1] - 0.18 * dy + 0.17 * ny),
        v(c[0] - 0.18 * dx - 0.17 * nx, c[1] - 0.18 * dy - 0.17 * ny),
    ]
}

/// The tangent ruler: where the circle's flat piece at heading `touch`
/// reads the step `d` (degrees): a straight line along the tangent, one
/// degree an arc-degree long.
fn tangent_point(touch: f64, d: f64) -> Vec3 {
    let p = on_circle(touch, RADIUS);
    let t = touch * DEG;
    // the tangent in the direction headings grow
    let (tx, ty) = (-t.cos(), -t.sin());
    let s = RADIUS * d * DEG;
    v(p[0] + s * tx, p[1] + s * ty)
}

/// A dashed arrow from `from` to `to` in `color`, as one group.
fn dashed_arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let n = 7;
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let head = 0.22;
    let end = v(to[0] - head * ux, to[1] - head * uy);
    let mut parts = Vec::new();
    for k in 0..n {
        let (t0, t1) = ((k as f64 + 0.1) / n as f64, (k as f64 + 0.6) / n as f64);
        let at = |t: f64| v(from[0] + (end[0] - from[0]) * t, from[1] + (end[1] - from[1]) * t);
        let d = polyline(vec![at(t0), at(t1)]).style(line_style(color, 4.0)).build();
        parts.push(stage.arena_mut().add(d));
    }
    let tri = Polygon::new([
        to,
        v(end[0] - 0.5 * head * uy, end[1] + 0.5 * head * ux),
        v(end[0] + 0.5 * head * uy, end[1] - 0.5 * head * ux),
    ])
    .style(flat(color, 1.0))
    .build();
    parts.push(stage.arena_mut().add(tri));
    group(stage, &parts)
}

/// A solid arrow from the circle's centre out to heading `h`.
fn heading_arrow(stage: &mut Stage<'_>, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let a = Arrow::new(v(CENTRE[0], CENTRE[1]), on_circle(h, RADIUS - 0.08))
        .buff(0.0)
        .style(line_style(color, 3.5))
        .build()?;
    Ok(stage.arena_mut().add(a))
}

/// A green tick centred at `at`, as strokes.
fn tick_at(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let r = 0.2;
    let t = polyline(vec![
        v(at[0] - 0.9 * r, at[1]),
        v(at[0] - 0.25 * r, at[1] - 0.7 * r),
        v(at[0] + r, at[1] + 0.8 * r),
    ])
    .style(line_style(palette::good(), 6.0))
    .build();
    stage.arena_mut().add(t)
}

impl SceneConstruct for Halfway {
    fn name(&self) -> &str {
        "04_halfway"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, orange, warn) = (palette::turn(), palette::flat(), palette::warn());
        let (a, b) = (fact_f64("G-H.a")?, fact_f64("G-H.b")?);
        let (gap, halfway) = (fact_f64("G-H.gap")?, fact_f64("G-H.halfway")?);
        let mean = fact_f64("G-H.mean")?;
        let minus = |x: f64| format!("{}", x.round() as i64).replace('-', &MINUS.to_string());

        // the circle of headings, its four marks, and the robot at its centre
        let circle = stage.arena_mut().add(
            Circle::new()
                .radius(RADIUS)
                .arc_center(v(CENTRE[0], CENTRE[1]))
                .style(line_style(Palette::faint(), 3.0))
                .build(),
        );
        let mut marks = vec![circle];
        for k in 0..4 {
            let h = model::wrap_deg(90.0 * f64::from(k));
            let (inner, outer) = (on_circle(h, RADIUS - 0.1), on_circle(h, RADIUS + 0.1));
            marks.push(
                stage.arena_mut().add(
                    polyline(vec![inner, outer])
                        .style(line_style(Palette::dim(), 3.0))
                        .build(),
                ),
            );
        }
        let ahead = kit.text(
            stage,
            AHEAD,
            24.0,
            Palette::dim(),
            v(CENTRE[0], CENTRE[1] + RADIUS + 0.45),
        )?;
        marks.push(ahead);
        let heading = stage.add_value_tracker(0.0);
        let robot = stage.always_redraw(move |s| {
            let pts = robot_points(v(CENTRE[0], CENTRE[1]), value(s, heading));
            s.add(Polygon::new(pts).style(flat(palette::steel(), 1.0)).build())
        });

        // 04_1: two headings, nearly straight back
        let line = say(&kit, stage, BEATS[0])?;
        fade_in_all(stage, &marks, 0.6)?;
        line.wait_for(stage, "Take two headings")?;
        stage.add_to_scene(robot)?;
        line.wait_for(stage, "A hundred and seventy degrees to the left")?;
        let arrow_a = heading_arrow(stage, a, blue)?;
        let label_a = kit.text(
            stage,
            &format!("{}°", minus(a)),
            30.0,
            blue,
            label_spot(a, LABEL_A_SHIFT),
        )?;
        play!(stage; fade_in(stage.arena_mut(), arrow_a, ORIGIN, 1.0)?.rt(0.6));
        fade_in_all(stage, &[label_a], 0.3)?;
        line.wait_for(stage, "And a hundred and seventy to the right")?;
        let arrow_b = heading_arrow(stage, b, blue)?;
        let label_b = kit.text(
            stage,
            &format!("{}°", minus(b)),
            30.0,
            blue,
            label_spot(b, LABEL_B_SHIFT),
        )?;
        play!(stage; fade_in(stage.arena_mut(), arrow_b, ORIGIN, 1.0)?.rt(0.6));
        fade_in_all(stage, &[label_b], 0.3)?;

        // 04_2: what heading is halfway between them?
        let card = your_turn(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;

        // 04_3: the average is zero, straight ahead; the honest halfway is
        // straight back; the ruler's ends do not meet
        let line = say(&kit, stage, BEATS[2])?;
        leave(stage, &card.mobs(), &[], 0.4)?;
        // TeX takes its own minus: the ASCII hyphen, never the en dash
        let tex = |x: f64| format!("{}", x.round() as i64);
        let sum = format!(r"\frac{{{} + ({})}}{{2}} = {}", tex(a), tex(b), tex(mean));
        let equation = kit.tex(stage, &sum, 56.0, Palette::ink(), &[], v(3.6, 2.4))?;
        let mean_arrow = dashed_arrow(stage, v(CENTRE[0], CENTRE[1]), on_circle(mean, RADIUS - 0.05), warn)?;
        line.wait_for(stage, "Average the numbers")?;
        play!(stage; write(stage.arena(), equation).rt(1.4));
        line.wait_for(stage, "Straight ahead")?;
        fade_in_all(stage, &[mean_arrow], 0.5)?;
        // the ruler, flat under the circle, and its three end labels
        line.wait_for(stage, "But both arrows point almost straight back")?;
        let wrap = stage.add_value_tracker(0.0);
        let ruler = stage.always_redraw(move |s| {
            let t = value(s, wrap);
            let t = t * t * (3.0 - 2.0 * t);
            let pts: Vec<Vec3> = (0..=72)
                .map(|k| ruler_point(-180.0 + 360.0 * f64::from(k) / 72.0, t))
                .collect();
            s.add(polyline(pts).style(line_style(Palette::ink(), 5.0)).build())
        });
        stage.add_to_scene(ruler)?;
        let ends: Vec<Mob> = RULER_ENDS
            .iter()
            .zip([-180.0, 0.0, 180.0])
            .map(|(&text, u)| {
                let p = flat_point(u);
                kit.text(stage, text, 24.0, Palette::dim(), v(p[0], p[1] - 0.4))
            })
            .collect::<Result<_, _>>()?;
        // the strip is named when it first stands there, and the name stays
        // while it bends up (R2 M-02)
        let ruler_name = {
            let p = flat_point(0.0);
            kit.text(stage, THE_RULER, 26.0, Palette::ink(), v(p[0], p[1] - RULER_NAME_DROP))?
        };
        let mut shown_ends = ends.clone();
        shown_ends.push(ruler_name);
        fade_in_all(stage, &shown_ends, 0.5)?;
        line.wait_for(stage, "The true halfway is")?;
        let right = tick_at(stage, on_circle(halfway, RADIUS + 0.55));
        fade_in_all(stage, &[right], 0.4)?;
        // wrapped on, the ruler's two ends meet at the back; the robot, told
        // to go from one heading to the other by the numbers, spins the long
        // way round
        stage.set_tracker_value(heading, a)?;
        // its three end labels are for the flat ruler; wrapped they would
        // float under the circle
        leave(stage, &ends, &[], 0.4)?;
        play!(stage;
            tween(wrap, 1.0, 2.2)?,
            tween(heading, b, 2.6)?,
            fade_out(stage.arena_mut(), ruler_name, ORIGIN, 1.0)?.rt(1.4),
        );
        let seam = {
            let (p, q) = (on_circle(halfway, RADIUS - 0.3), on_circle(halfway, RADIUS + 0.3));
            let m = polyline(vec![p, q]).style(line_style(warn, 5.0)).build();
            stage.arena_mut().add(m)
        };
        let seam_word = kit.text(stage, SEAM, 24.0, warn, on_circle(halfway, RADIUS + 0.95))?;
        fade_in_all(stage, &[seam, seam_word], 0.4)?;

        // 04_4: peel the ruler off; lay it flat against the circle with its
        // zero on the first heading; measure to the second
        let line = say(&kit, stage, BEATS[3])?;
        line.wait_for(stage, "So peel the ruler off")?;
        leave(stage, &[seam, seam_word, right, mean_arrow, equation], &[], 0.4)?;
        play!(stage; tween(wrap, 0.0, 1.0)?);
        stage.clear_updaters(ruler, true);
        let reach = stage.add_value_tracker(0.0);
        let tangent = stage.always_redraw(move |s| {
            let r = value(s, reach).clamp(0.0, 1.0);
            let pts: Vec<Vec3> = (0..=72)
                .map(|k| tangent_point(a, r * (-180.0 + 360.0 * f64::from(k) / 72.0)))
                .collect();
            s.add(polyline(pts).style(line_style(orange, 5.0)).build())
        });
        stage.add_to_scene(tangent)?;
        leave(stage, &[ruler], &[], 0.5)?;
        play!(stage; tween(reach, 1.0, 1.2)?);
        let zero = kit.text(stage, "0", 24.0, orange, {
            let p = tangent_point(a, 0.0);
            v(p[0] - 0.05, p[1] - 0.4)
        })?;
        fade_in_all(stage, &[zero], 0.3)?;
        line.wait_for(stage, "Now measure to the second")?;
        let second = tangent_point(a, gap);
        let second_dot = stage
            .arena_mut()
            .add(Dot::new().point(second).radius(0.09).style(flat(blue, 1.0)).build());
        // beside the dot, above the line: the heading labels hang below it
        let step = kit.text(
            stage,
            &format!("{} {}°", STEP, minus(gap)),
            28.0,
            blue,
            v(second[0] + STEP_SHIFT[0], second[1] + STEP_SHIFT[1]),
        )?;
        fade_in_all(stage, &[second_dot, step], 0.4)?;
        line.wait_for(stage, "Take half")?;
        let half_at = tangent_point(a, gap / 2.0);
        let half_dot = stage
            .arena_mut()
            .add(Dot::new().point(half_at).radius(0.1).style(flat(orange, 1.0)).build());
        let half_label = kit.text(
            stage,
            &format!("{} {}°", HALF, minus(gap / 2.0)),
            28.0,
            orange,
            v(half_at[0] - 0.4, half_at[1] + 0.5),
        )?;
        fade_in_all(stage, &[half_dot, half_label], 0.4)?;
        line.wait_for(stage, "Ten degrees on from the first")?;
        // the half, wrapped back onto the circle: it lands on the tick
        let land = on_circle(a + gap / 2.0, RADIUS);
        let to = stage.add_value_tracker(0.0);
        let mover = stage.always_redraw(move |s| {
            let t = value(s, to);
            let p = v(
                half_at[0] + (land[0] - half_at[0]) * t,
                half_at[1] + (land[1] - half_at[1]) * t,
            );
            s.add(
                Dot::new()
                    .point(p)
                    .radius(0.12)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(mover)?;
        play!(stage; tween(to, 1.0, 1.1)?);
        let code = code_lines()?;
        let code_refs: Vec<&str> = code.iter().map(String::as_str).collect();
        let tag_panel = CodePanel::build(&kit, stage, &code_refs, CODE_SIZE, CODE_WIDTH)?;
        play!(stage; tag_panel.slide_in(0.8)?);

        // 04_5: the whole trick; the circle and its line become the badge
        let line = say(&kit, stage, BEATS[4])?;
        let everything = [
            circle,
            ahead,
            arrow_a,
            arrow_b,
            label_a,
            label_b,
            tangent,
            zero,
            second_dot,
            step,
            half_dot,
            half_label,
            mover,
            tag_panel.group,
            right,
            robot,
        ];
        let mut gone: Vec<Mob> = marks.clone();
        gone.extend(everything);
        leave(stage, &gone, &[], 0.6)?;
        let big = Badge::build(stage, v(0.0, 0.5), 4.0)?;
        fade_in_all(stage, &big.mobs(), 0.7)?;
        line.wait_for(stage, "The space is curved")?;
        let words = [
            kit.text(stage, CURVED, 40.0, blue, v(0.0, -1.1))?,
            kit.text(stage, FLAT_WORD, 40.0, orange, v(0.0, 2.4))?,
        ];
        fade_in_lagged(stage, &words, 1.0, 0.5)?;
        line.wait_for(stage, "Then wrap the answer back on")?;
        let back = kit.text(stage, WRAP_BACK, 40.0, Palette::ink(), v(3.0, 0.5))?;
        fade_in_all(stage, &[back], 0.4)?;
        let piece = lower_third(&kit, stage, THE_FLAT_PIECE, TANGENT_SPACE)?;
        play!(stage; fade_in(stage.arena_mut(), piece, ORIGIN, 1.0)?.rt(0.5));
        kit.hold(stage)?;
        // the badge goes to its corner and stays there for the rest of the video's chapters
        let mut gone = vec![piece, back];
        gone.extend(words);
        leave(stage, &gone, &[], 0.5)?;
        let moves = big.move_to(stage, 0.25, BADGE_CORNER, 1.0)?;
        stage.play_prepared(moves)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{CARD_W, LOWER_THIRD_TOP, SAFE, fact, probe, run_chapter};
    use super::*;

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "04_halfway").expect("chapter 04");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 04 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 04_2: five seconds; the computed halfway is 180, and the card asks
    /// about the headings the scene draws.
    #[test]
    fn the_quiz_asks_about_the_two_headings() {
        assert_eq!(ASK.seconds().unwrap(), 5);
        assert_eq!(fact("G-Q.04_2.answer").unwrap(), "180");
        assert_eq!(ASK.answer_index().unwrap(), 2);
        let (a, b) = (fact_f64("G-H.a").unwrap(), fact_f64("G-H.b").unwrap());
        assert_eq!((a, b), (170.0, -170.0));
        assert!(ASK.question.contains("170°") && ASK.question.contains(&format!("{MINUS}170°")));
        // the ruler's end label and the card use the same minus as live numbers
        assert_eq!(RULER_ENDS[0], format!("{MINUS}180"));
    }

    /// 04_3: the average of the numbers is zero, straight ahead, and it is
    /// 180 degrees from the honest halfway.
    #[test]
    fn the_average_points_the_wrong_way() {
        let (a, b) = (fact_f64("G-H.a").unwrap(), fact_f64("G-H.b").unwrap());
        assert_eq!((a + b) / 2.0, fact_f64("G-H.mean").unwrap());
        assert_eq!(model::halfway_deg(a, b).abs(), fact_f64("G-H.halfway").unwrap());
        assert!((fact_f64("G-H.off_by").unwrap() - 180.0).abs() < 1e-9);
        // both arrows point nearly straight back: each within ten degrees
        for h in [a, b] {
            let (_, dy) = dir(h);
            assert!(dy < -0.98, "heading {h} points back");
        }
        // and "ahead" is up
        assert_eq!(dir(0.0), (0.0, 1.0));
    }

    /// 04_4: measured on the flat piece at the first heading, the second is
    /// 20 degrees away ("−340 becomes +20"); half of that on from the first
    /// is 180.
    #[test]
    fn on_the_flat_piece_the_step_is_twenty_and_half_is_ten() {
        let (a, b) = (fact_f64("G-H.a").unwrap(), fact_f64("G-H.b").unwrap());
        assert_eq!(b - a, -340.0);
        let gap = model::wrap_deg(b - a);
        assert_eq!(gap, 20.0);
        assert_eq!(gap, fact_f64("G-H.gap").unwrap());
        assert_eq!(model::wrap_deg(a + gap / 2.0).abs(), 180.0);
        // the tangent ruler touches the circle at the first heading, at
        // right angles to the radius, with one degree an arc-degree long
        let touch = tangent_point(a, 0.0);
        assert_eq!(touch, on_circle(a, RADIUS));
        let step = tangent_point(a, 1.0);
        let (rx, ry) = (touch[0] - CENTRE[0], touch[1] - CENTRE[1]);
        let (sx, sy) = (step[0] - touch[0], step[1] - touch[1]);
        assert!((rx * sx + ry * sy).abs() < 1e-12, "along the tangent");
        assert!((sx.hypot(sy) - RADIUS * DEG).abs() < 1e-12);
        // the flat piece is not the circle: the +20 mark is a little off the
        // second heading's place round the circle, which is why the answer is
        // wrapped back on; for a step this small the difference is under a
        // tenth of the radius
        let flat20 = tangent_point(a, gap);
        let round20 = on_circle(a + gap, RADIUS);
        let err = (flat20[0] - round20[0]).hypot(flat20[1] - round20[1]);
        assert!(err > 0.01 * RADIUS && err < 0.1 * RADIUS, "{err}");
        // and the half, wrapped back, lands exactly on heading 180: the tick
        let landing = on_circle(a + gap / 2.0, RADIUS);
        let tick = on_circle(fact_f64("G-H.halfway").unwrap(), RADIUS);
        assert!((landing[0] - tick[0]).hypot(landing[1] - tick[1]) < 1e-9);
    }

    /// 04_3: the ruler is as long as the circle is round, so wrapped its two
    /// ends meet at the back (heading 180) in the seam; flat it is a strip
    /// under the circle.
    #[test]
    fn the_ruler_wraps_onto_the_circle_and_meets_itself_at_the_back() {
        let (lo, hi) = (ruler_point(-180.0, 1.0), ruler_point(180.0, 1.0));
        assert!((lo[0] - hi[0]).hypot(lo[1] - hi[1]) < 1e-9, "the seam closes");
        let back = on_circle(180.0, RADIUS);
        assert!((lo[0] - back[0]).hypot(lo[1] - back[1]) < 1e-9, "at the back");
        let (l0, h0) = (ruler_point(-180.0, 0.0), ruler_point(180.0, 0.0));
        assert!(
            ((h0[0] - l0[0]) - TAU * RADIUS).abs() < 1e-9,
            "as long as the circle is round"
        );
        assert_eq!(l0[1], h0[1]);
        // the flat ruler, with its end labels (about half a unit wide each
        // side of the end), fits the safe area and ends before the card
        assert!(
            l0[0] - 0.3 > -SAFE[0],
            "the left end label starts at {:.2}",
            l0[0] - 0.3
        );
        assert!(h0[0] + 0.3 < CARD_AT[0] - 0.5 * CARD_W * CARD_SCALE);
        // the zero of the ruler is straight ahead when wrapped
        let (zero, ahead) = (ruler_point(0.0, 1.0), on_circle(0.0, RADIUS));
        assert!((zero[0] - ahead[0]).hypot(zero[1] - ahead[1]) < 1e-12);
    }

    /// 04_4: the tangent ruler, a whole circle long, ends inside the safe
    /// area (its left end ran to the edge), and the labels round the circle
    /// stand apart: the two headings, the step and the half.
    #[test]
    fn the_flat_piece_and_its_labels_fit() {
        let (a, b) = (fact_f64("G-H.a").unwrap(), fact_f64("G-H.b").unwrap());
        let gap = fact_f64("G-H.gap").unwrap();
        let (left, right) = (tangent_point(a, -180.0), tangent_point(a, 180.0));
        assert!(left[0] > -SAFE[0] + 0.15 && right[0] < SAFE[0]);
        // "170°" and "–170°" are about 0.6 wide: their centres are well apart
        let (la, lb) = (label_spot(a, LABEL_A_SHIFT), label_spot(b, LABEL_B_SHIFT));
        assert!(
            lb[0] - la[0] > 1.2,
            "the headings' labels are {:.2} apart",
            lb[0] - la[0]
        );
        // "step 20°" (about 1.6 wide) stands clear of both labels and above
        // the line it measures
        let dot = tangent_point(a, gap);
        let step = v(dot[0] + STEP_SHIFT[0], dot[1] + STEP_SHIFT[1]);
        for l in [la, lb] {
            assert!(
                (step[0] - l[0]).abs() > 1.1 || (step[1] - l[1]).abs() > 0.5,
                "the step label stands on a heading's label"
            );
        }
        // the step label is above the tangent line there, outside the circle
        let line_y = tangent_point(a, gap + STEP_SHIFT[0] / RADIUS / DEG)[1];
        assert!(step[1] > line_y + 0.2);
        assert!((step[0] - CENTRE[0]).hypot(step[1] - CENTRE[1]) > RADIUS + 0.4);
    }

    /// 04_3: the strip is named under itself, clear of its end labels and
    /// above the lower third, inside the safe area.
    #[test]
    fn the_ruler_is_named_under_its_strip() {
        probe(|stage| {
            let kit = Kit::new()?;
            let p = flat_point(0.0);
            let name = kit.text(stage, THE_RULER, 26.0, Palette::ink(), v(p[0], p[1] - RULER_NAME_DROP))?;
            let zero = kit.text(stage, RULER_ENDS[1], 24.0, Palette::dim(), v(p[0], p[1] - 0.4))?;
            assert!(
                stage.get_top(name)[1] < stage.get_bottom(zero)[1] - 0.1,
                "the name touches the zero's label"
            );
            assert!(stage.get_bottom(name)[1] > LOWER_THIRD_TOP + 0.1);
            assert!(stage.get_left(name)[0] > -SAFE[0] && stage.get_right(name)[0] < SAFE[0]);
            Ok(())
        });
    }

    /// 04_4: the panel's comment is the storyboard's tag in the typewriter
    /// face and the facts' numbers, its lines stand inside the panel, and the
    /// panel once in place clears the ruler's right end.
    #[test]
    fn the_panel_says_the_convention_in_the_facts_numbers() {
        let code = code_lines().unwrap();
        assert_eq!(code[0], "d = wrap(b - a)");
        let said = code[1..]
            .iter()
            .map(|l| l.trim_start_matches("# "))
            .collect::<Vec<_>>()
            .join(" ")
            .replace('-', "\u{2212}")
            .replace("...", "\u{2026}");
        assert_eq!(
            said,
            "bring the difference into \u{2212}180\u{2026}180, the shorter way round: \u{2212}340 becomes +20"
        );
        assert!(
            crate::STORYBOARD.contains(&said),
            "the storyboard's tag is not {said:?}"
        );
        let a = fact_f64("G-H.a").unwrap();
        probe(|stage| {
            let kit = Kit::new()?;
            let refs: Vec<&str> = code.iter().map(String::as_str).collect();
            let panel = CodePanel::build(&kit, stage, &refs, CODE_SIZE, CODE_WIDTH)?;
            let right = stage.get_right(panel.group)[0];
            for &line in &panel.lines {
                assert!(
                    stage.get_right(line)[0] < right - 0.2,
                    "a code line reaches {:.2} of the panel's {right:.2}",
                    stage.get_right(line)[0]
                );
            }
            // in place: the panel's left edge, and its lower edge, against the
            // tangent ruler's right end
            let left = stage.get_left(panel.group)[0] - (panel.width + 0.45);
            let bottom = stage.get_bottom(panel.group)[1];
            let end = tangent_point(a, 180.0);
            assert!(
                left > end[0] + 0.1 || bottom > end[1] + 0.1,
                "the panel (left {left:.2}, bottom {bottom:.2}) meets the ruler's end {end:?}"
            );
            assert!(left + panel.width < SAFE[0] + 0.2);
            Ok(())
        });
    }

    /// The whole chapter constructs, headless.
    #[test]
    fn the_chapter_constructs_end_to_end() {
        run_chapter(&mut Halfway {
            kit: Kit::new().expect("the kit"),
        });
    }

    /// The robot's nose points along its heading.
    #[test]
    fn the_robot_faces_its_heading() {
        for h in [0.0, 90.0, 170.0, -170.0] {
            let [nose, ..] = robot_points(v(0.0, 0.0), h);
            let (dx, dy) = dir(h);
            assert!((nose[0] - 0.30 * dx).abs() < 1e-12 && (nose[1] - 0.30 * dy).abs() < 1e-12);
        }
    }
}
