//! Chapter 06, the platform: poses work the same way on a floor (06_1); hold
//! one velocity for six seconds and where does it end up (06_2); a half circle
//! (06_3); velocity times time is the flat piece's answer, and the pose is
//! that answer wrapped round a centre (06_4).

use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Ask, BADGE_CORNER, Badge, Choice, QuizCard, QuizLook, fact_f64, fade_in_all, fade_in_lagged, faded_line, flat,
    leave, line_style, panel, say, tween_linear, value,
};
use crate::model::{self, DEG, planar_centre, planar_exp};
use crate::palette;

pub struct ThePlatform {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const THE_PLATFORM: &str = "the platform:";
const PLATFORM_NAME: &str = "KUKA KMR iiwa, Mecanum wheels";
const POSE: &str = "pose";
const VELOCITY: &str = "velocity";
const X_WORD: &str = "x";
const Y_WORD: &str = "y";
const HEADING: &str = "heading";
const FORWARD: &str = "forward";
const SIDEWAYS: &str = "sideways";
const SPIN: &str = "spin";
const METRES: &str = "m";
const PER_SECOND: &str = "m/s";
const DEG_PER_SECOND: &str = "°/s";
const DEGREE: &str = "°";
const OUR_NUMBERS: &str = "Our numbers, not KUKA's";
const EVERY_STEADY_MOTION: &str =
    "on a floor, every steady motion is a turn about a point, or with no spin a straight slide.";
const WHEELS: &str = "# wheel speeds: Lynch and Park eq. 13.10";
const HALF_CIRCLE_A: &str = "a half circle";
const HALF_CIRCLE_B: &str = "m long is";
const HALF_CIRCLE_C: &str = "÷ π =";
const HALF_CIRCLE_D: &str = "m across";
const RADIUS: &str = "radius";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 4] = ["06_1", "06_2", "06_3", "06_4"];

/// 06_2's quiz: where does the platform end up after six seconds? The keys
/// are what `G-Q.06_2.answer` prints.
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: "after six seconds,\nwhere is it?",
    choices: &[
        Choice {
            key: "ahead",
            label: "three meters ahead",
        },
        Choice {
            key: "beside",
            label: "beside where it started",
        },
        Choice {
            key: "back",
            label: "back where it started",
        },
    ],
};

/// The platform's own drawing, in metres: its body and wheels. (No chassis
/// dimension is shown or claimed: storyboard decision 3.)
const BODY: (f64, f64) = (0.40, 0.28);
const WHEEL: (f64, f64) = (0.10, 0.045);
const WHEEL_AT: (f64, f64) = (0.25, 0.31);
/// Scene units a metre is drawn as, close up and on the floor.
const CLOSE: f64 = 3.0;
const FLOOR: f64 = 1.15;
/// Where the floor's origin and the close-up's centre are on the screen.
const FLOOR_ORIGIN: [f64; 2] = [-5.0, -0.5];
const CLOSE_ORIGIN: [f64; 2] = [-3.6, 0.5];
/// An arrow's length on the platform: metres of drawing per metre a second.
const ARROW_PER_SPEED: f64 = 0.9;
/// Where the line "a half circle 3 m long is 6 ÷ π = 1.91 m across" is
/// centred (it is about 5.7 wide: at x = 4.2 it ran off the safe area).
const ACROSS_AT: [f64; 2] = [3.9, -1.2];
/// Where 06_2's quiz card stands.
const CARD_AT: [f64; 2] = [4.35, 0.3];
const CARD_SCALE: f64 = 0.92;
/// Where the slider of 06_4 ends, degrees a second: the quiz's third choice
/// (`G-P.spin60.end`).
const SLIDER_TOP_DEG: f64 = 60.0;

/// The diagonal each wheel's rollers lie along (degrees from the wheel's
/// long axis), front left, front right, rear left, rear right: a Mecanum
/// set's rollers make an X.
fn roller_angles() -> [f64; 4] {
    [45.0, -45.0, -45.0, 45.0]
}

/// Metres to the screen at `zoom` (0 close up, 1 on the floor): the scale
/// and where the origin is.
fn floor_map(zoom: f64) -> (f64, [f64; 2]) {
    let z = zoom.clamp(0.0, 1.0);
    let lerp = |a: f64, b: f64| a + (b - a) * z;
    (
        lerp(CLOSE, FLOOR),
        [
            lerp(CLOSE_ORIGIN[0], FLOOR_ORIGIN[0]),
            lerp(CLOSE_ORIGIN[1], FLOOR_ORIGIN[1]),
        ],
    )
}

/// The platform's velocity (forward m/s, spin rad/s) and how long it is held
/// (s), from the facts the video states.
fn drive() -> fmn::Result<(f64, f64, f64)> {
    Ok((fact_f64("G-P.v")?, fact_f64("G-P.spin_deg")? * DEG, fact_f64("G-P.t")?))
}

/// An arrow as shaft and head, in screen units.
fn arrow_parts(from: Vec3, to: Vec3, color: Srgb, width: f64, opacity: f64) -> Vec<VMobject> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let head = (0.28 * len).min(0.28);
    let base = v(to[0] - head * ux, to[1] - head * uy);
    vec![
        polyline(vec![from, base])
            .style(faded_line(color, width, opacity))
            .build(),
        Polygon::new([
            to,
            v(base[0] - 0.5 * head * uy, base[1] + 0.5 * head * ux),
            v(base[0] + 0.5 * head * uy, base[1] - 0.5 * head * ux),
        ])
        .style(flat(color, opacity))
        .build(),
    ]
}

/// The platform at metres pose `(x, y, h)` (heading `h` in radians from the
/// +x axis), `k` scene units a metre, origin at `o`: body, four wheels with
/// their rollers, and the velocity's arrows at `arrows` opacity (the sideways
/// one at `side`).
fn platform_parts(k: f64, o: [f64; 2], pose: (f64, f64, f64), arrows: f64, side: f64) -> Vec<VMobject> {
    let (x, y, h) = pose;
    let (c, s) = (h.cos(), h.sin());
    // a point in the platform's own frame (forward, left) on the screen
    let at = |f: f64, l: f64| v(o[0] + k * (x + c * f - s * l), o[1] + k * (y + s * f + c * l));
    let mut out = Vec::new();
    // the body: a rectangle with its corners cut
    let (bx, by, cut) = (BODY.0, BODY.1, 0.07);
    let body = Polygon::new([
        at(bx - cut, by),
        at(-bx + cut, by),
        at(-bx, by - cut),
        at(-bx, -by + cut),
        at(-bx + cut, -by),
        at(bx - cut, -by),
        at(bx, -by + cut),
        at(bx, by - cut),
    ])
    .style(Style::default().fill(panel(), 1.0).stroke(palette::steel(), 3.0, 1.0))
    .build();
    out.push(body);
    // the wheels, each with three rollers on its diagonal
    let angles = roller_angles();
    for (i, (sf, sl)) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
        .into_iter()
        .enumerate()
    {
        let (cf, cl) = (sf * WHEEL_AT.0, sl * WHEEL_AT.1);
        out.push(
            Polygon::new([
                at(cf + WHEEL.0, cl + WHEEL.1),
                at(cf - WHEEL.0, cl + WHEEL.1),
                at(cf - WHEEL.0, cl - WHEEL.1),
                at(cf + WHEEL.0, cl - WHEEL.1),
            ])
            .style(
                Style::default()
                    .fill(palette::steel(), 0.25)
                    .stroke(palette::steel(), 2.0, 1.0),
            )
            .build(),
        );
        let a = angles[i] * DEG;
        for step in [-0.055, 0.0, 0.055] {
            let (px, py) = (cf + step, cl);
            let half = WHEEL.1 * 0.95;
            out.push(
                polyline(vec![
                    at(px - half * a.cos(), py - half * a.sin()),
                    at(px + half * a.cos(), py + half * a.sin()),
                ])
                .style(line_style(palette::steel(), 2.0))
                .build(),
            );
        }
    }
    let orange = palette::flat();
    if arrows > 0.01 {
        // forward: along the platform's x, as long as its speed
        out.extend(arrow_parts(
            at(0.0, 0.0),
            at(ARROW_PER_SPEED * 0.5, 0.0),
            orange,
            5.0,
            arrows,
        ));
        // spin: a curl round its centre
        let curl: Vec<Vec3> = (0..=16)
            .map(|j| {
                let a = 0.6 + 4.2 * f64::from(j) / 16.0;
                at(0.17 * a.cos(), 0.17 * a.sin())
            })
            .collect();
        let n = curl.len();
        out.push(polyline(curl.clone()).style(faded_line(orange, 4.0, arrows)).build());
        out.extend(arrow_parts(curl[n - 2], curl[n - 1], orange, 4.0, arrows));
    }
    if side > 0.01 {
        out.extend(arrow_parts(
            at(0.0, 0.0),
            at(0.0, ARROW_PER_SPEED * 0.5),
            orange,
            5.0,
            side,
        ));
    }
    out
}

/// The heading a readout prints, in whole degrees from a heading in radians.
/// Rounded here: the readout prints no places, and a number of no places is
/// truncated, so a heading of 179.99999999999997 would print 179 at the end
/// of a half circle. (Never −0.)
fn shown_heading(h: f64) -> f64 {
    model::wrap_deg(h / DEG).round() + 0.0
}

/// The platform's path of `seconds`, held at `spin` rad/s and `speed` m/s,
/// as `n + 1` points in metres.
fn path_points(speed: f64, spin: f64, seconds: f64, n: usize) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|k| {
            let (x, y, _) = planar_exp(speed, 0.0, spin, seconds * k as f64 / n as f64);
            (x, y)
        })
        .collect()
}

/// The length of a polyline of metre points.
#[cfg(test)]
fn length(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
        .sum()
}

impl SceneConstruct for ThePlatform {
    fn name(&self) -> &str {
        "06_the_platform"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (orange, teal) = (palette::flat(), palette::place());
        let (speed, spin, seconds) = drive()?;
        let (zoom, clock, arrows, sideways) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let badge = Badge::build(stage, BADGE_CORNER, 1.0)?;

        // the platform, redrawn each frame from the zoom and the clock: its
        // pose is the model's `planar_exp` of the velocity held for the clock
        let platform = stage.always_redraw(move |s| {
            let (k, o) = floor_map(value(s, zoom));
            let pose = planar_exp(speed, 0.0, spin, value(s, clock));
            let parts = platform_parts(k, o, pose, value(s, arrows), value(s, sideways));
            s.add(v_group(parts))
        });

        // 06_1: a floor, a platform that can drive forward, slide sideways and
        // spin; three numbers for its pose, three for its velocity
        let line = say(&kit, stage, BEATS[0])?;
        stage.add_to_scene(platform)?;
        let name = lower_third(&kit, stage, THE_PLATFORM, PLATFORM_NAME)?;
        play!(stage; fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.6));
        line.wait_for(stage, "Its wheels carry slanted rollers")?;
        play!(stage; tween(arrows, 1.0, 0.6)?);
        line.wait_for(stage, "slide sideways")?;
        play!(stage; tween(sideways, 1.0, 0.6)?);
        line.wait_for(stage, "Three numbers for its pose")?;
        // pose and velocity, three numbers each
        let pose_rows = [(X_WORD, METRES, 1.9), (Y_WORD, METRES, 1.25), (HEADING, DEGREE, 0.6)];
        let velocity_rows = [
            (FORWARD, PER_SECOND, -0.6, speed),
            (SIDEWAYS, PER_SECOND, -1.25, 0.0),
            (SPIN, DEG_PER_SECOND, -1.9, spin / DEG),
        ];
        let mut readout = Vec::new();
        let pose_title = kit.text(stage, POSE, 30.0, Palette::dim(), v(2.6, 2.55))?;
        let velocity_title = kit.text(stage, VELOCITY, 30.0, Palette::dim(), v(2.9, 0.0))?;
        readout.extend([pose_title, velocity_title]);
        for (words, unit, y) in pose_rows {
            readout.push(kit.text_left(stage, words, 28.0, palette::turn(), v(1.9, y))?);
            readout.push(kit.text_left(stage, unit, 26.0, Palette::dim(), v(6.3, y))?);
        }
        for (words, unit, y, amount) in velocity_rows {
            readout.push(kit.text_left(stage, words, 28.0, orange, v(1.9, y))?);
            readout.push(kit.text(stage, &format!("{amount:.1}"), 34.0, Palette::ink(), v(5.5, y))?);
            readout.push(kit.text_left(stage, unit, 26.0, Palette::dim(), v(6.0, y))?);
        }
        fade_in_lagged(stage, &readout, 1.2, 0.3)?;
        // the live pose numbers sit between the words and the units
        let pose_x = kit.live_value(
            stage,
            move |s| planar_exp(speed, 0.0, spin, value(s, clock)).0,
            2,
            false,
            34.0,
            Palette::ink(),
            v(5.55, 1.9),
            RIGHT,
        )?;
        let pose_y = kit.live_value(
            stage,
            move |s| planar_exp(speed, 0.0, spin, value(s, clock)).1,
            2,
            false,
            34.0,
            Palette::ink(),
            v(5.55, 1.25),
            RIGHT,
        )?;
        let pose_h = kit.live_value(
            stage,
            move |s| shown_heading(planar_exp(speed, 0.0, spin, value(s, clock)).2),
            0,
            false,
            34.0,
            Palette::ink(),
            v(5.55, 0.6),
            RIGHT,
        )?;

        // 06_2: hold one velocity for six seconds: where does it end up?
        leave(stage, &[name], &[], 0.4)?;
        let mut hide = readout.clone();
        hide.extend([pose_x, pose_y, pose_h]);
        leave(stage, &hide, &[sideways, arrows], 0.5)?;
        play!(stage; tween(zoom, 1.0, 1.0)?);
        // the floor, in metres
        let origin = floor_map(1.0).1;
        let floor_lines = {
            let mut lines = Vec::new();
            for m in -1..=3 {
                let (a, b) = (
                    v(origin[0] + FLOOR * f64::from(m), origin[1] - FLOOR),
                    v(origin[0] + FLOOR * f64::from(m), origin[1] + 3.0 * FLOOR),
                );
                lines.push(
                    stage
                        .arena_mut()
                        .add(polyline(vec![a, b]).style(line_style(Palette::faint(), 1.5)).build()),
                );
            }
            for m in -1..=3 {
                let (a, b) = (
                    v(origin[0] - FLOOR, origin[1] + FLOOR * f64::from(m)),
                    v(origin[0] + 3.2 * FLOOR, origin[1] + FLOOR * f64::from(m)),
                );
                lines.push(
                    stage
                        .arena_mut()
                        .add(polyline(vec![a, b]).style(line_style(Palette::faint(), 1.5)).build()),
                );
            }
            lines
        };
        let mut marks = Vec::new();
        for m in 1..=3 {
            marks.push(kit.text(
                stage,
                &format!("{m} {METRES}"),
                22.0,
                Palette::dim(),
                v(origin[0] + FLOOR * f64::from(m), origin[1] - FLOOR - 0.28),
            )?);
        }
        let mut floor = floor_lines;
        floor.extend(marks);
        stage.scene_mut().bring_to_front(&[platform])?;
        let line = say(&kit, stage, BEATS[1])?;
        fade_in_all(stage, &floor, 0.6)?;
        stage.scene_mut().bring_to_front(&[platform])?;
        play!(stage; tween(arrows, 1.0, 0.5)?);
        // the two numbers the line says, where their arrows are: the pose
        // and velocity rows of 06_1 have left to make room for the card
        let (here, k_floor) = (floor_map(1.0).1, floor_map(1.0).0);
        line.wait_for(stage, "Forward at half a meter")?;
        let forward = kit.text_left(
            stage,
            &format!("{speed:.1} {PER_SECOND}"),
            26.0,
            orange,
            v(here[0] + k_floor * (ARROW_PER_SPEED * 0.5) + 0.12, here[1]),
        )?;
        fade_in_all(stage, &[forward], 0.3)?;
        line.wait_for(stage, "spinning at thirty degrees")?;
        let spinning = kit.text(
            stage,
            &format!("{:.0} {DEG_PER_SECOND}", spin / DEG),
            26.0,
            orange,
            v(here[0], here[1] + 0.8),
        )?;
        fade_in_all(stage, &[spinning], 0.3)?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 06_3: beside where it started: a half circle, just under two
        // metres to the left, facing backward
        let line = say(&kit, stage, BEATS[2])?;
        let mut gone = card.mobs();
        gone.extend([forward, spinning]);
        leave(stage, &gone, &[], 0.4)?;
        let trail = stage.always_redraw(move |s| {
            let t = value(s, clock).max(1e-6);
            let (k, o) = floor_map(value(s, zoom));
            let n = ((t * 12.0).ceil() as usize).max(2);
            let pts: Vec<Vec3> = path_points(speed, spin, t, n)
                .into_iter()
                .map(|(x, y)| v(o[0] + k * x, o[1] + k * y))
                .collect();
            s.add(polyline(pts).style(line_style(teal, 5.0)).build())
        });
        stage.add_to_scene(trail)?;
        let live: Vec<Mob> = {
            let rows = [(X_WORD, METRES, 1.9), (Y_WORD, METRES, 1.25), (HEADING, DEGREE, 0.6)];
            let mut out = Vec::new();
            for (words, unit, y) in rows {
                out.push(kit.text_left(stage, words, 28.0, palette::turn(), v(1.9, y))?);
                out.push(kit.text_left(stage, unit, 26.0, Palette::dim(), v(6.3, y))?);
            }
            out
        };
        fade_in_all(stage, &live, 0.4)?;
        let (px, py, ph) = (
            kit.live_value(
                stage,
                move |s| planar_exp(speed, 0.0, spin, value(s, clock)).0,
                2,
                false,
                34.0,
                Palette::ink(),
                v(5.55, 1.9),
                RIGHT,
            )?,
            kit.live_value(
                stage,
                move |s| planar_exp(speed, 0.0, spin, value(s, clock)).1,
                2,
                false,
                34.0,
                Palette::ink(),
                v(5.55, 1.25),
                RIGHT,
            )?,
            kit.live_value(
                stage,
                move |s| shown_heading(planar_exp(speed, 0.0, spin, value(s, clock)).2).abs(),
                0,
                false,
                34.0,
                Palette::ink(),
                v(5.55, 0.6),
                RIGHT,
            )?,
        );
        play!(stage; tween_linear(clock, seconds, seconds)?);
        let across = fact_f64("G-P.arc_length")?;
        let end = fact_list_pair("G-P.end")?;
        let across_words = kit.text(
            stage,
            &format!(
                "{HALF_CIRCLE_A} {across:.0} {HALF_CIRCLE_B} {:.0} {HALF_CIRCLE_C} {:.2} {HALF_CIRCLE_D}",
                2.0 * across,
                end.1
            ),
            28.0,
            Palette::ink(),
            v(ACROSS_AT[0], ACROSS_AT[1]),
        )?;
        line.wait_for(stage, "It drove a half circle")?;
        fade_in_all(stage, &[across_words], 0.5)?;

        // 06_4: the flat piece's answer is three metres straight ahead; wrap
        // it on and the same three metres bend round a fixed point
        let line = say(&kit, stage, BEATS[3])?;
        let mut away = live.clone();
        away.extend([px, py, ph, across_words, trail]);
        leave(stage, &away, &[arrows], 0.5)?;
        play!(stage; tween(clock, 0.0, 0.6)?);
        let slider = stage.add_value_tracker(0.0);
        let answer = stage.always_redraw(move |s| {
            let spin_now = value(s, slider) * DEG;
            let (k, o) = floor_map(1.0);
            let pts: Vec<Vec3> = path_points(speed, spin_now, seconds, 90)
                .into_iter()
                .map(|(x, y)| v(o[0] + k * x, o[1] + k * y))
                .collect();
            s.add(polyline(pts).style(line_style(orange, 6.0)).build())
        });
        let centre_dot = stage.always_redraw(move |s| {
            let spin_now = value(s, slider) * DEG;
            let (k, o) = floor_map(1.0);
            let r = if spin_now > 0.05 {
                let (cx, cy) = planar_centre(speed, 0.0, spin_now);
                v(o[0] + k * cx, o[1] + k * cy)
            } else {
                v(-20.0, -20.0)
            };
            s.add(
                Dot::new()
                    .point(r)
                    .radius(0.1)
                    .style(flat(palette::screw(), 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(answer)?;
        stage.add_to_scene(centre_dot)?;
        line.wait_for(stage, "you predict three meters straight ahead")?;
        play!(stage; fade_in(stage.arena_mut(), badge.body, ORIGIN, 1.0)?.rt(0.4));
        line.wait_for(stage, "Wrap it onto the curved space")?;
        play!(stage;
            tween(slider, spin / DEG, 1.6)?,
            fade_in(stage.arena_mut(), badge.arrow, ORIGIN, 1.0)?.rt(0.6),
        );
        let radius = fact_f64("G-P.radius")?;
        let r_word = kit.text(
            stage,
            &format!("{RADIUS} {radius:.2} {METRES}"),
            28.0,
            palette::screw(),
            v(4.4, 2.2),
        )?;
        let tag = CodePanel::build(
            &kit,
            stage,
            &[
                "radius = speed / spin",
                &format!("       = {speed:.1} / {spin:.3} = {r:.2} m", r = speed / spin),
                "# spin in radians a second",
                WHEELS,
            ],
            24.0,
            5.8,
        )?;
        fade_in_all(stage, &[r_word], 0.4)?;
        play!(stage; tag.slide_in(0.8)?);
        // the slider: the same three metres, as the spin runs from 0 to 60
        play!(stage; tween(slider, 0.0, 1.0)?);
        play!(stage; tween(slider, SLIDER_TOP_DEG, 3.0)?);
        let note = lower_third(
            &kit,
            stage,
            "precisely:",
            &format!("{EVERY_STEADY_MOTION} {OUR_NUMBERS}"),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5));
        kit.finish(stage)
    }
}

/// A fact that is two comma-separated numbers.
fn fact_list_pair(key: &str) -> fmn::Result<(f64, f64)> {
    let list = super::draw::fact_list(key)?;
    match list.as_slice() {
        [a, b] => Ok((*a, *b)),
        _ => Err(super::draw::fail(format!("fact {key} is not a pair"))),
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::super::draw::{SAFE, fact, probe};
    use super::*;

    /// 06_2: the two numbers the line says stand by their arrows, right of the
    /// platform and left of the quiz card, above the floor's metre marks.
    #[test]
    fn the_velocity_numbers_stand_by_their_arrows() {
        probe(|stage| {
            let kit = Kit::new()?;
            let (speed, spin, _) = drive()?;
            let (k, o) = floor_map(1.0);
            let forward = kit.text_left(
                stage,
                &format!("{speed:.1} {PER_SECOND}"),
                26.0,
                Palette::ink(),
                v(o[0] + k * (ARROW_PER_SPEED * 0.5) + 0.12, o[1]),
            )?;
            let spinning = kit.text(
                stage,
                &format!("{:.0} {DEG_PER_SECOND}", spin / DEG),
                26.0,
                Palette::ink(),
                v(o[0], o[1] + 0.8),
            )?;
            let card_left = CARD_AT[0] - 0.5 * super::super::draw::CARD_W * CARD_SCALE;
            assert!(stage.get_right(forward)[0] < card_left - 0.5);
            // the arrow's tip is at the label's left edge, not under it
            assert!(stage.get_left(forward)[0] > o[0] + k * ARROW_PER_SPEED * 0.5);
            // the spin number is above the body (0.28 m half width)
            assert!(stage.get_bottom(spinning)[1] > o[1] + k * BODY.1);
            // and both are above the metre marks
            assert!(stage.get_bottom(forward)[1] > o[1] - k - 0.28 + 0.3);
            Ok(())
        });
    }

    /// 06_3: the half circle's line (about 5.7 wide) ends inside the safe
    /// area (it ran to the frame's edge) and starts clear of the floor.
    #[test]
    fn the_half_circle_line_fits_the_frame() {
        probe(|stage| {
            let kit = Kit::new()?;
            let across = fact_f64("G-P.arc_length")?;
            let end = fact_list_pair("G-P.end")?;
            let words = format!(
                "{HALF_CIRCLE_A} {across:.0} {HALF_CIRCLE_B} {:.0} {HALF_CIRCLE_C} {:.2} {HALF_CIRCLE_D}",
                2.0 * across,
                end.1
            );
            let line = kit.text(stage, &words, 28.0, Palette::ink(), v(ACROSS_AT[0], ACROSS_AT[1]))?;
            let (left, right) = (stage.get_left(line)[0], stage.get_right(line)[0]);
            assert!(right < SAFE[0], "the line ends at {right:.2}");
            let (k, o) = floor_map(1.0);
            assert!(
                left > o[0] + k * 3.2 + 0.3,
                "the line starts at {left:.2}, the floor ends"
            );
            Ok(())
        });
    }

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "06_the_platform").expect("chapter 06");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 06 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 06_3: the heading readout (whole degrees, which truncates) reads the
    /// heading the facts give, from the first frame to the half circle's
    /// end: thirty degrees a second, so 180 after six seconds, never 179 (and
    /// never −0).
    #[test]
    fn the_heading_readout_reads_the_headings_of_the_drive() {
        let (speed, spin, seconds) = drive().unwrap();
        for k in 0..=24 {
            let t = seconds * f64::from(k) / 24.0;
            let h = planar_exp(speed, 0.0, spin, t).2;
            let shown = shown_heading(h).abs();
            let want = (30.0 * t).round();
            assert_eq!(shown, want, "t = {t}: {shown} against {want}");
            assert!(
                shown_heading(h).is_sign_positive() || shown_heading(h) < 0.0,
                "never −0"
            );
        }
        assert_eq!(shown_heading(planar_exp(speed, 0.0, spin, seconds).2).abs(), 180.0);
    }

    /// 06_2: six seconds; the computed answer is "beside where it started".
    #[test]
    fn the_quiz_asks_where_it_ends_up() {
        assert_eq!(ASK.seconds().unwrap(), 6);
        assert_eq!(fact("G-Q.06_2.answer").unwrap(), "beside");
        assert_eq!(ASK.answer_index().unwrap(), 1);
    }

    /// 06_3: half a metre a second at thirty degrees a second for six
    /// seconds is a half circle three metres long: it ends 6 / π = 1.91 m to
    /// the left of its start, facing backward.
    #[test]
    fn six_seconds_is_a_half_circle() {
        let (speed, spin, seconds) = drive().unwrap();
        assert_eq!((speed, seconds), (0.5, 6.0));
        let (x, y, h) = planar_exp(speed, 0.0, spin, seconds);
        assert!(x.abs() < 1e-12);
        assert!((y - 6.0 / PI).abs() < 1e-12);
        assert!((model::wrap_deg(h / DEG).abs() - 180.0).abs() < 1e-9, "facing back");
        let end = fact_list_pair("G-P.end").unwrap();
        assert!((end.0 - x).abs() < 1e-4 && (end.1 - y).abs() < 1e-4);
        assert!((fact_f64("G-P.arc_length").unwrap() - speed * seconds).abs() < 1e-9);
        assert!(format!("{:.2}", y) == "1.91");
    }

    /// 06_4: the flat piece says three metres straight ahead; wrapped, the
    /// same three metres go round a fixed centre at radius speed / spin; and
    /// along the whole slider the length stays three metres, the end running
    /// through (3, 0), (0, 1.91) and (0, 0).
    #[test]
    fn the_same_three_metres_bend_round_a_centre() {
        let (speed, spin, seconds) = drive().unwrap();
        let naive = fact_list_pair("G-P.naive").unwrap();
        assert_eq!(naive, (speed * seconds, 0.0));
        let radius = fact_f64("G-P.radius").unwrap();
        assert!((radius - speed / spin).abs() < 1e-4);
        assert_eq!(format!("{:.2}", speed / spin), "0.95");
        let (cx, cy) = planar_centre(speed, 0.0, spin);
        assert!(cx.abs() < 1e-12 && (cy - radius).abs() < 1e-3);
        // every point of the arc is the radius from the centre
        for (x, y) in path_points(speed, spin, seconds, 24) {
            assert!(((x - cx).hypot(y - cy) - speed / spin).abs() < 1e-9);
        }
        // the slider keeps the length
        for deg in [0.0, 15.0, 30.0, 45.0, 60.0] {
            let pts = path_points(speed, deg * DEG, seconds, 2000);
            assert!((length(&pts) - speed * seconds).abs() < 2e-3, "{deg} degrees a second");
        }
        // and ends where the quiz's three choices are
        let end = |deg: f64| {
            let (x, y, _) = planar_exp(speed, 0.0, deg * DEG, seconds);
            (x, y)
        };
        let zero = fact_list_pair("G-P.spin0.end").unwrap();
        let sixty = fact_list_pair("G-P.spin60.end").unwrap();
        assert!((end(0.0).0 - zero.0).abs() < 1e-4 && end(0.0).1.abs() < 1e-9);
        assert!(end(60.0).0.abs() < 1e-9 && end(60.0).1.abs() < 1e-9);
        assert!(sixty.0.abs() < 1e-4 && sixty.1.abs() < 1e-4);
        assert_eq!(fact("G-X5.floor").unwrap(), "45/45");
    }

    /// 06_1: a Mecanum set's rollers make an X (opposite corners alike), and
    /// the drawing fits the floor it is drawn on.
    #[test]
    fn the_wheels_are_an_x_and_the_floor_fits() {
        let a = roller_angles();
        assert_eq!(a[0], a[3]);
        assert_eq!(a[1], a[2]);
        assert_eq!(a[0], -a[1]);
        // the floor's metre marks and the platform stay left of the card
        let (k, o) = floor_map(1.0);
        assert!(o[0] + k * 3.2 < 1.7, "the floor ends before the quiz card");
        // close up, the platform's body is bigger on the screen than on the floor
        assert!(floor_map(0.0).0 > floor_map(1.0).0);
    }
}
