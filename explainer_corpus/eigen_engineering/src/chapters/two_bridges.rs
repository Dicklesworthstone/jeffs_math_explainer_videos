//! Chapter 08, two bridges: both are told from their primary sources, and
//! both are the same story. Tacoma Narrows, November 1940: a suspension
//! bridge four months old twists itself apart in a wind of about forty
//! miles an hour (08_1). The textbook guess, the wind puffing in time with
//! the bridge, fails on the numbers: puffs about once a second, a twist
//! once every five (08_2). What grew was one twisting mode, and the wind's
//! push followed its motion and gave more than it took back (08_3). So what
//! was pushing London's footbridge (08_4)? Its sideways modes sit in the
//! range where walkers push, and the more the deck moved, the harder they
//! pushed (08_5). Neither bridge's trouble was strength: each had a mode
//! and a push that grew with it, which engineers call negative damping
//! (08_7).
//!
//! Every number is a fact: the dates `K-B1.opened` and `K-B1.failed`, the
//! wind `K-B1.wind_mph`, the rates `K-B1.vortex_hz` and `K-B1.twist_period_s`,
//! the footbridge's measured modes `K-B2.lateral_hz`, the walker's
//! `K-B2.walker_lateral_hz`, the range `K-B2.risk_below_hz`, Arup's test
//! `K-B2.test_steady` and `K-B2.test_added`. The push is drawn as a push
//! that FOLLOWS the motion (it points the way the deck is already going and
//! is largest as it passes level), never as one in time with the bridge
//! (R1 R3): neither "resonance" nor "synchrony" is shown as the cause.

use std::f64::consts::{PI, TAU};

use explainer_kit::animatic::wrap;
use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::deck::{Deck, DeckSpec, scene_clock};
use super::draw::{
    arrow_points, corners, fact, fade_in_all, fade_out_all, faded_line, fail, flat, group, items, numbers, own, panel,
    predict_hold, pushes, say, tween_all, whole,
};
use super::the_bridge_that_swayed::{date_text, question_card};

// ------------------------------------------------------------------ choices

/// The deck of 08_1 and 08_3, seen end on: where it sits, half its width,
/// how high its cables hang above it, and the push arrows' longest reach.
const TACOMA_AT: Vec3 = [-0.4, 0.5, 0.0];
const TACOMA_HALF: f64 = 3.2;
const HANGER: f64 = 2.0;
/// The largest twist drawn, radians (a picture's size, not a measurement).
const AMP_MAX: f64 = 0.55;
/// How far a push arrow reaches at the largest twist, scene units.
const PUSH_REACH: f64 = 0.95;
/// The footbridge's plan view (08_5): the three spans' height on screen,
/// the deck's drawn width, the most a span swings across and the longest
/// push arrow.
const PLAN_Y: f64 = 2.45;
const PLAN_WIDTH: f64 = 0.42;
const PLAN_AMP: f64 = 0.42;
const PLAN_PUSH: f64 = 0.62;
/// Where along a span the push arrows stand (a fraction of its length).
const ARROW_AT: [f64; 3] = [0.25, 0.5, 0.75];
/// How many points a span's edge is drawn from.
const SPAN_POINTS: usize = 40;
/// The number line of 08_5: its ends, its height and the rate it reaches
/// (swings a second).
const LINE_X: (f64, f64) = (-5.6, 5.0);
const LINE_Y: f64 = -0.5;
const LINE_MAX_HZ: f64 = 1.5;

// ------------------------------------------------------------- the geometry

/// The two ends of a deck seen end on, twisted by `angle` radians about its
/// centre: left end, right end.
pub fn twisted_ends(centre: Vec3, half_w: f64, angle: f64) -> [Vec3; 2] {
    let (c, s) = (angle.cos(), angle.sin());
    [
        v(centre[0] - half_w * c, centre[1] - half_w * s),
        v(centre[0] + half_w * c, centre[1] + half_w * s),
    ]
}

/// The twist at time `t`: `amp` radians, swinging `hz` times a second.
pub fn twist_angle(amp: f64, hz: f64, t: f64) -> f64 {
    amp * (TAU * hz * t).sin()
}

/// The push that follows the motion, as a share of its largest: the
/// twist's rate of turning over its largest. +1 as the deck passes level
/// going up on the right, 0 at the end of a swing.
pub fn follow_share(hz: f64, t: f64) -> f64 {
    (TAU * hz * t).cos()
}

/// How bright a puff lamp is `t` seconds into a rhythm of `hz` a second:
/// full at each puff and dying away; dark until the first has come.
pub fn puff(t: f64, hz: f64) -> f64 {
    if t * hz < 1.0 {
        0.0
    } else {
        (-6.0 * (t * hz).rem_euclid(1.0)).exp()
    }
}

/// How many have happened `t` seconds in, at `hz` a second.
pub fn happened(t: f64, hz: f64) -> f64 {
    (t * hz).floor().max(0.0)
}

/// One sideways mode of a span of the footbridge: how many half waves it
/// has along the span, how many swings a second, and its share of the
/// span's swing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mode {
    pub half_waves: u32,
    pub hz: f64,
    pub share: f64,
}

/// A span in the plan view: where it starts and ends along the screen and
/// the modes it swings in.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub x0: f64,
    pub x1: f64,
    pub modes: Vec<Mode>,
}

/// How far the span is across from its rest line at the fraction `u` along
/// it, `t` seconds in, as a share of its largest swing.
pub fn span_shift(modes: &[Mode], u: f64, t: f64) -> f64 {
    let total: f64 = modes.iter().map(|m| m.share).sum();
    modes
        .iter()
        .map(|m| m.share * (f64::from(m.half_waves) * PI * u).sin() * (TAU * m.hz * t).cos())
        .sum::<f64>()
        / total.max(1e-12)
}

/// Which way, and how fast as a share of the largest it can be, that point
/// is going: the rate of change of [`span_shift`] (a mode counts for as
/// many swings a second as it has), over its largest.
pub fn span_speed(modes: &[Mode], u: f64, t: f64) -> f64 {
    let total: f64 = modes.iter().map(|m| m.share * m.hz).sum();
    modes
        .iter()
        .map(|m| -m.share * m.hz * (f64::from(m.half_waves) * PI * u).sin() * (TAU * m.hz * t).sin())
        .sum::<f64>()
        / total.max(1e-12)
}

/// The three spans as the measured modes give them (storyboard 08_5):
/// south 0.80 Hz, centre 0.48 and 0.95 Hz, north 1.03 Hz. The facts list the
/// four in rising order. The spans are drawn equal: a plan view, not to
/// scale.
pub fn spans_from(lateral: &[f64]) -> Option<Vec<Span>> {
    if lateral.len() != 4 {
        return None;
    }
    let (left, right) = (-5.6, 5.2);
    let gap = 0.15;
    let w = (right - left - 2.0 * gap) / 3.0;
    let at = |k: usize| (left + (w + gap) * k as f64, left + (w + gap) * k as f64 + w);
    let one = |hz: f64| {
        vec![Mode {
            half_waves: 1,
            hz,
            share: 1.0,
        }]
    };
    Some(vec![
        Span {
            x0: at(0).0,
            x1: at(0).1,
            modes: one(lateral[1]),
        },
        Span {
            x0: at(1).0,
            x1: at(1).1,
            modes: vec![
                Mode {
                    half_waves: 1,
                    hz: lateral[0],
                    share: 1.0,
                },
                Mode {
                    half_waves: 2,
                    hz: lateral[2],
                    share: 0.5,
                },
            ],
        },
        Span {
            x0: at(2).0,
            x1: at(2).1,
            modes: one(lateral[3]),
        },
    ])
}

/// Where a rate sits on the number line.
pub fn hz_x(hz: f64) -> f64 {
    LINE_X.0 + (LINE_X.1 - LINE_X.0) * hz / LINE_MAX_HZ
}

/// A swing that dies away (`rate` negative) or grows (`rate` positive),
/// at `u` of the way along the plot, `cycles` swings in all; the largest
/// value is 1 (at the start when dying, at the end when growing).
pub fn swing_value(rate: f64, cycles: f64, u: f64) -> f64 {
    let envelope = if rate < 0.0 {
        (rate * u).exp()
    } else {
        (rate * (u - 1.0)).exp()
    };
    envelope * (TAU * cycles * u).sin()
}

// ------------------------------------------------------------------ props

/// A tracker value as an updater reads it.
fn value(s: &Arena, m: Mob) -> f64 {
    s.tracker_value(m).unwrap_or(0.0)
}

/// Where a deck seen end on sits and how big it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwistSpec {
    pub centre: Vec3,
    pub half_w: f64,
    pub hanger: f64,
}

/// A deck seen end on, twisting about its centre line at `hz` swings a
/// second, its cables above, and the wind's push at each end drawn as an
/// arrow that points the way the end is going and is longest as the deck
/// passes level. The twist's size is `amp` (radians), a tracker.
pub struct TwistDeck {
    /// The twist's size, radians; how much of the deck shows; how much of
    /// the push arrows shows (all 0 to 1 but the first).
    pub amp: Mob,
    pub shown: Mob,
    pub push_on: Mob,
    pub deck: Mob,
    pub hangers: [Mob; 2],
    pub pins: [Mob; 2],
    pub push: [Mob; 2],
    anchor: Mob,
}

impl TwistDeck {
    /// Build the picture, detached; `clock` is a tracker counting the
    /// scene's seconds.
    pub fn build(stage: &mut Stage<'_>, spec: TwistSpec, hz: f64, colour: Srgb, clock: Mob) -> fmn::Result<Self> {
        let dim = Palette::dim();
        let ends = twisted_ends(spec.centre, spec.half_w, 0.0);
        let cable = [
            v(spec.centre[0] - spec.half_w, spec.centre[1] + spec.hanger),
            v(spec.centre[0] + spec.half_w, spec.centre[1] + spec.hanger),
        ];
        let deck = stage
            .arena_mut()
            .add(polyline(ends.to_vec()).style(faded_line(colour, 14.0, 0.0)).build());
        let mut hangers = Vec::new();
        let mut pins = Vec::new();
        let mut push = Vec::new();
        for k in 0..2 {
            hangers.push(
                stage.arena_mut().add(
                    polyline(vec![ends[k], cable[k]])
                        .style(faded_line(dim, 3.0, 0.0))
                        .build(),
                ),
            );
            pins.push(
                stage
                    .arena_mut()
                    .add(Dot::new().point(cable[k]).radius(0.08).style(flat(dim, 0.0)).build()),
            );
            push.push(
                stage.arena_mut().add(
                    polyline(arrow_points(ends[k], v(ends[k][0] + 1e-3, ends[k][1]), 0.2))
                        .style(faded_line(pushes(), 6.0, 0.0))
                        .build(),
                ),
            );
        }
        let (amp, shown, push_on) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let (hangers_m, pins_m, push_m) = ([hangers[0], hangers[1]], [pins[0], pins[1]], [push[0], push[1]]);
        let mut last = [f64::NAN; 4];
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = [value(s, clock), value(s, amp), value(s, shown), value(s, push_on)];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let [t, a, on, push_on_v] = now;
                let ends = twisted_ends(spec.centre, spec.half_w, twist_angle(a, hz, t));
                let _ = s.set_points(deck, &corners(&ends));
                s.set_stroke(deck, None, None, Some(on), None, false);
                // the push follows the motion: up where the end is going up,
                // longest as the deck passes level, as long as the twist is big
                let share = follow_share(hz, t) * (a / AMP_MAX);
                for k in 0..2 {
                    let _ = s.set_points(hangers_m[k], &corners(&[ends[k], cable[k]]));
                    s.set_stroke(hangers_m[k], None, None, Some(0.8 * on), None, false);
                    s.set_fill(pins_m[k], None, Some(on), None, false);
                    let sign = if k == 0 { -1.0 } else { 1.0 };
                    let tip = v(ends[k][0], ends[k][1] + sign * PUSH_REACH * share);
                    let _ = s.set_points(push_m[k], &corners(&arrow_points(ends[k], tip, 0.2)));
                    s.set_stroke(push_m[k], None, None, Some(push_on_v), None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            amp,
            shown,
            push_on,
            deck,
            hangers: [hangers[0], hangers[1]],
            pins: [pins[0], pins[1]],
            push: [push[0], push[1]],
            anchor,
        })
    }

    /// Put the updater and every part in the scene, at opacity 0 until the
    /// trackers rise (a part the updater drives is shown by its tracker,
    /// never faded).
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        for &m in [self.deck]
            .iter()
            .chain(&self.hangers)
            .chain(&self.pins)
            .chain(&self.push)
        {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// The footbridge's spans in plan view: each span's two edges, lit, swinging
/// in its measured modes, with the walkers' push at three places along it
/// drawn as an arrow that points the way that place is going.
pub struct PlanSpans {
    /// How large the swing is, 0 to 1; how much of the spans shows; how
    /// much of the push arrows shows.
    pub grow: Mob,
    pub shown: Mob,
    pub push_on: Mob,
    pub edges: Vec<[Mob; 2]>,
    pub push: Vec<Vec<Mob>>,
    anchor: Mob,
}

impl PlanSpans {
    pub fn build(stage: &mut Stage<'_>, spans: Vec<Span>, y: f64, colour: Srgb, clock: Mob) -> fmn::Result<Self> {
        let mut edges = Vec::new();
        let mut push = Vec::new();
        for sp in &spans {
            let rest = |side: f64| -> Vec<Vec3> {
                (0..SPAN_POINTS)
                    .map(|k| {
                        let u = k as f64 / (SPAN_POINTS - 1) as f64;
                        v(sp.x0 + (sp.x1 - sp.x0) * u, y + side * PLAN_WIDTH / 2.0)
                    })
                    .collect()
            };
            let pair = [1.0, -1.0].map(|side| {
                stage
                    .arena_mut()
                    .add(polyline(rest(side)).style(faded_line(colour, 6.0, 0.0)).build())
            });
            edges.push(pair);
            let arrows: Vec<Mob> = ARROW_AT
                .iter()
                .map(|&u| {
                    let x = sp.x0 + (sp.x1 - sp.x0) * u;
                    stage.arena_mut().add(
                        polyline(arrow_points(v(x, y), v(x + 1e-3, y), 0.18))
                            .style(faded_line(pushes(), 5.0, 0.0))
                            .build(),
                    )
                })
                .collect();
            push.push(arrows);
        }
        let (grow, shown, push_on) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let (edges_m, push_m) = (edges.clone(), push.clone());
        let mut last = [f64::NAN; 4];
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = [value(s, clock), value(s, grow), value(s, shown), value(s, push_on)];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let [t, g, on, push_on_v] = now;
                for (i, sp) in spans.iter().enumerate() {
                    for (e, side) in [1.0, -1.0].iter().enumerate() {
                        let pts: Vec<Vec3> = (0..SPAN_POINTS)
                            .map(|k| {
                                let u = k as f64 / (SPAN_POINTS - 1) as f64;
                                v(
                                    sp.x0 + (sp.x1 - sp.x0) * u,
                                    y + side * PLAN_WIDTH / 2.0 + PLAN_AMP * g * span_shift(&sp.modes, u, t),
                                )
                            })
                            .collect();
                        let _ = s.set_points(edges_m[i][e], &corners(&pts));
                        s.set_stroke(edges_m[i][e], None, None, Some(on), None, false);
                    }
                    for (k, &u) in ARROW_AT.iter().enumerate() {
                        let x = sp.x0 + (sp.x1 - sp.x0) * u;
                        let base = v(x, y + PLAN_AMP * g * span_shift(&sp.modes, u, t));
                        let tip = v(x, base[1] + PLAN_PUSH * g * span_speed(&sp.modes, u, t));
                        let _ = s.set_points(push_m[i][k], &corners(&arrow_points(base, tip, 0.18)));
                        s.set_stroke(push_m[i][k], None, None, Some(push_on_v), None, false);
                    }
                }
            },
            true,
        )?;
        Ok(Self {
            grow,
            shown,
            push_on,
            edges,
            push,
            anchor,
        })
    }

    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        for m in self.edges.iter().flatten().chain(self.push.iter().flatten()) {
            stage.add_to_scene(*m)?;
        }
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// Where a swing plot sits and what it plots: its left edge and width, the
/// height of its middle line and of its largest swing, the envelope's rate
/// (negative dies away, positive grows) and how many cycles it has.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotSpec {
    pub left: f64,
    pub width: f64,
    pub mid_y: f64,
    pub height: f64,
    pub rate: f64,
    pub cycles: f64,
}

/// A swing drawn left to right as far as `reach` (0 to 1), in `colour`:
/// the same swing dying away or growing, as the spec says.
pub struct SwingPlot {
    pub reach: Mob,
    pub line: Mob,
    anchor: Mob,
}

impl SwingPlot {
    pub fn build(stage: &mut Stage<'_>, spec: PlotSpec, colour: Srgb) -> fmn::Result<Self> {
        let PlotSpec {
            left,
            width,
            mid_y,
            height,
            rate,
            cycles,
        } = spec;
        let points_to = move |r: f64| -> Vec<Vec3> {
            let n = ((r * 120.0).ceil() as usize).max(2);
            (0..n)
                .map(|k| {
                    let u = r * k as f64 / (n - 1) as f64;
                    v(left + width * u, mid_y + height * swing_value(rate, cycles, u))
                })
                .collect()
        };
        let line = stage
            .arena_mut()
            .add(polyline(points_to(1.0)).style(faded_line(colour, 5.0, 0.0)).build());
        let reach = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let r = value(s, reach);
                if r.to_bits() == last.to_bits() {
                    return;
                }
                last = r;
                let _ = s.set_points(line, &corners(&points_to(r.max(0.02))));
                s.set_stroke(line, None, None, Some(if r > 0.0 { 1.0 } else { 0.0 }), None, false);
            },
            true,
        )?;
        Ok(Self { reach, line, anchor })
    }

    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        stage.add_to_scene(self.line)?;
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// A tracker that runs from `from` to `to` over `seconds` of the scene's
/// own clock, starting now: a ramp the beats do not have to wait for.
/// Returns its anchor.
fn ramp(stage: &mut Stage<'_>, tracker: Mob, clock: Mob, from: f64, to: f64, seconds: f64) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let t0 = stage.tracker_value(clock).unwrap_or(0.0);
    let mut done = false;
    stage.add_updater(
        anchor,
        move |s, _| {
            if done {
                return;
            }
            let share = ((value(s, clock) - t0) / seconds).clamp(0.0, 1.0);
            let _ = s.set_tracker_value(tracker, from + (to - from) * share);
            done = share >= 1.0;
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// A quote on a card, wrapped to `chars` a line, with its source beneath,
/// edged in `edge`, centred at `at`, detached.
fn quote_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    quote: &str,
    source: &str,
    chars: usize,
    edge: Srgb,
    at: Vec3,
) -> fmn::Result<Mob> {
    let lines = wrap(&format!("\"{quote}\""), chars, 9);
    let pitch = 0.46;
    let height = pitch * lines.len() as f64 + 1.0;
    let top = at[1] + height / 2.0 - 0.45;
    let mut parts = Vec::new();
    let mut widest = 0.0_f64;
    for (k, l) in lines.iter().enumerate() {
        let t = kit.text(stage, l, 26.0, Palette::ink(), v(at[0], top - pitch * k as f64))?;
        widest = widest.max(stage.get_width(t));
        parts.push(t);
    }
    let by = kit.text(
        stage,
        &format!("\u{2013} {source}"),
        22.0,
        Palette::dim(),
        v(at[0], top - pitch * lines.len() as f64 - 0.15),
    )?;
    widest = widest.max(stage.get_width(by));
    let back = Rectangle::new()
        .width(widest + 0.7)
        .height(height)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.96).stroke(edge, 2.5, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let mut all = vec![back];
    all.extend(parts);
    all.push(by);
    group(stage, &all)
}

// -------------------------------------------------------------- the scene

pub struct TwoBridges {
    pub kit: Kit,
}

impl SceneConstruct for TwoBridges {
    fn name(&self) -> &str {
        "08_two_bridges"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim, gold, orange) = (Palette::ink(), Palette::dim(), own(), pushes());
        let twist_hz = numbers("K-B1.twist_hz")?[0];
        let (clock, clock_anchor) = scene_clock(stage)?;

        // ---- 08_1: a bridge four months old twists itself apart
        let line = say(&kit, stage, "08_1")?;
        let tacoma = TwistDeck::build(
            stage,
            TwistSpec {
                centre: TACOMA_AT,
                half_w: TACOMA_HALF,
                hanger: HANGER,
            },
            twist_hz,
            ink,
            clock,
        )?;
        tacoma.start(stage)?;
        let failed = date_text(fact("K-B1.failed")?).ok_or_else(|| fail("K-B1.failed is not a date".into()))?;
        let opened = date_text(fact("K-B1.opened")?).ok_or_else(|| fail("K-B1.opened is not a date".into()))?;
        let failed_label = kit.text_left(stage, &failed, 40.0, ink, v(-6.7, 3.4))?;
        let opened_label = kit.text_left(stage, &format!("opened {opened}"), 30.0, dim, v(-6.7, 2.7))?;
        let wind_label = kit.text_left(
            stage,
            &format!("{} mph, measured that morning", fact("K-B1.wind_mph")?),
            30.0,
            ink,
            v(1.6, 3.4),
        )?;
        let grow = ramp(stage, tacoma.amp, clock, 0.04, AMP_MAX, 9.5)?;
        line.wait_for(stage, "Tacoma Narrows")?;
        tween_all(stage, &[(tacoma.shown, 1.0)], 0.7)?;
        fade_in_all(stage, &[failed_label], 0.5)?;
        line.wait_for(stage, "A suspension bridge four months old")?;
        fade_in_all(stage, &[opened_label], 0.5)?;
        line.wait_for(stage, "in a wind of about forty miles an hour")?;
        fade_in_all(stage, &[wind_label], 0.5)?;

        // ---- 08_2: the wind did not puff in time with the bridge
        let line = say(&kit, stage, "08_2")?;
        stage.clear_updaters(grow, true);
        tween_all(stage, &[(tacoma.shown, 0.0)], 0.5)?;
        tacoma.stop(stage);
        fade_out_all(stage, &[failed_label, opened_label, wind_label], 0.5)?;
        // the wind: a lamp that flashes at each puff; the bridge: a small
        // deck that twists, each on the scene's own clock, with a count
        let vortex = numbers("K-B1.vortex_hz")?[0];
        let period = numbers("K-B1.twist_period_s")?[0];
        let lamp_at = v(-3.4, 1.4);
        let lamp = stage
            .arena_mut()
            .add(Dot::new().point(lamp_at).radius(0.55).style(flat(orange, 0.0)).build());
        let lamp_on = stage.add_value_tracker(0.0);
        let t0 = stage.tracker_value(clock).unwrap_or(0.0);
        let lamp_anchor = stage.arena_mut().add(VMobject::new());
        {
            let mut last = [f64::NAN; 2];
            stage.add_updater(
                lamp_anchor,
                move |s, _| {
                    let now = [value(s, clock), value(s, lamp_on)];
                    if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                        return;
                    }
                    last = now;
                    let level = 0.12 + 0.88 * puff(now[0] - t0, vortex);
                    s.set_fill(lamp, None, Some(now[1] * level), None, false);
                },
                true,
            )?;
            stage.add_to_scene(lamp_anchor)?;
            stage.add_to_scene(lamp)?;
        }
        let small = TwistDeck::build(
            stage,
            TwistSpec {
                centre: v(3.4, 1.3),
                half_w: 1.5,
                hanger: 1.1,
            },
            1.0 / period,
            ink,
            clock,
        )?;
        small.start(stage)?;
        stage.set_tracker_value(small.amp, 0.45)?;
        let wind_words = kit.text(stage, "the wind's puffs:", 28.0, dim, v(-3.4, 0.1))?;
        let wind_rate = kit.text(
            stage,
            &format!("about {} a second (estimate)", whole(vortex)),
            30.0,
            ink,
            v(-3.4, -0.45),
        )?;
        let twist_words = kit.text(stage, "the twist:", 28.0, dim, v(3.4, 0.1))?;
        let twist_rate = kit.text(
            stage,
            &format!("1 every {} seconds", whole(period)),
            30.0,
            ink,
            v(3.4, -0.45),
        )?;
        let source = lower_third(
            &kit,
            stage,
            "source:",
            "Billah and Scanlan, American Journal of Physics, 1991",
        )?;
        line.wait_for(stage, "You might guess")?;
        tween_all(stage, &[(lamp_on, 1.0), (small.shown, 1.0)], 0.6)?;
        fade_in_all(stage, &[source], 0.5)?;
        line.wait_for(stage, "Its puffs would come about once a second")?;
        fade_in_all(stage, &[wind_words, wind_rate], 0.5)?;
        let puffs = kit.live_value(
            stage,
            move |s| happened(value(s, clock) - t0, vortex),
            0,
            false,
            72.0,
            orange,
            v(-3.4, -1.55),
            ORIGIN,
        )?;
        line.wait_for(stage, "and the bridge twisted once every five")?;
        fade_in_all(stage, &[twist_words, twist_rate], 0.5)?;
        let twists = kit.live_value(
            stage,
            move |s| happened(value(s, clock) - t0, 1.0 / period),
            0,
            false,
            72.0,
            ink,
            v(3.4, -1.55),
            ORIGIN,
        )?;

        // ---- 08_3: one twisting mode grew, and the push followed its motion
        let line = say(&kit, stage, "08_3")?;
        tween_all(stage, &[(small.shown, 0.0), (lamp_on, 0.0)], 0.4)?;
        small.stop(stage);
        stage.clear_updaters(lamp_anchor, true);
        fade_out_all(
            stage,
            &[wind_words, wind_rate, twist_words, twist_rate, puffs, twists, source],
            0.5,
        )?;
        let big = TwistDeck::build(
            stage,
            TwistSpec {
                centre: v(-3.1, 0.7),
                half_w: 2.8,
                hanger: 1.8,
            },
            twist_hz,
            gold,
            clock,
        )?;
        big.start(stage)?;
        let growth = ramp(stage, big.amp, clock, 0.05, AMP_MAX, 8.0)?;
        let board = quote_card(
            &kit,
            stage,
            "The Tacoma Narrows Bridge was well designed and built to resist safely all static forces, \
             including wind, usually considered in the design of similar structures.",
            "the board of engineers, 1941",
            34,
            dim,
            v(3.7, 0.9),
        )?;
        let flutter = lower_third(&kit, stage, "names to search:", "aeroelastic flutter")?;
        line.wait_for(stage, "What grew was one twisting mode")?;
        tween_all(stage, &[(big.shown, 1.0)], 0.6)?;
        fade_in_all(stage, &[flutter], 0.5)?;
        line.wait_for(stage, "As the deck twisted")?;
        tween_all(stage, &[(big.push_on, 1.0)], 0.6)?;
        fade_in_all(stage, &[board], 0.6)?;
        line.wait_for(stage, "and gave more than it took back")?;

        // ---- 08_4: so what was pushing London's footbridge?
        let line = say(&kit, stage, "08_4")?;
        stage.clear_updaters(growth, true);
        tween_all(stage, &[(big.shown, 0.0), (big.push_on, 0.0)], 0.5)?;
        big.stop(stage);
        fade_out_all(stage, &[board, flutter], 0.5)?;
        let footbridge = Deck::build(
            stage,
            DeckSpec {
                centre: [0.0, -0.3, 0.0],
                length: 11.0,
                width: 1.0,
            },
            22,
        )?;
        footbridge.start(stage)?;
        let ask = question_card(
            &kit,
            stage,
            &["what was pushing", "London's footbridge?"],
            40.0,
            gold,
            v(0.0, 2.2),
        )?;
        line.wait_for(stage, "So what was pushing")?;
        tween_all(stage, &[(footbridge.edges_on, 1.0), (footbridge.walkers_on, 1.0)], 0.6)?;
        fade_in_all(stage, &[ask], 0.6)?;
        predict_hold(&kit, stage)?;

        // ---- 08_5: its sideways modes sit where walkers push
        let line = say(&kit, stage, "08_5")?;
        tween_all(stage, &[(footbridge.edges_on, 0.0), (footbridge.walkers_on, 0.0)], 0.5)?;
        footbridge.stop(stage);
        fade_out_all(stage, &[ask], 0.4)?;
        let lateral = numbers("K-B2.lateral_hz")?;
        let hz_text = items("K-B2.lateral_hz")?;
        let spans = spans_from(&lateral).ok_or_else(|| fail("K-B2.lateral_hz is not four rates".into()))?;
        let plan = PlanSpans::build(stage, spans.clone(), PLAN_Y, gold, clock)?;
        plan.start(stage)?;
        let names = [
            format!("south {} Hz", hz_text[1]),
            format!("centre {} and {} Hz", hz_text[0], hz_text[2]),
            format!("north {} Hz", hz_text[3]),
        ];
        let mut labels = Vec::new();
        for (sp, name) in spans.iter().zip(&names) {
            labels.push(kit.text(stage, name, 26.0, ink, v((sp.x0 + sp.x1) / 2.0, PLAN_Y - 0.95))?);
        }
        let not_to_scale = kit.text(stage, "plan view, not to scale", 22.0, dim, v(0.0, PLAN_Y + 0.85))?;
        // the number line: rates a second, the band where walkers push, the
        // four measured modes and the walker's own
        let axis = stage.arena_mut().add(
            polyline(vec![v(hz_x(0.0), LINE_Y), v(hz_x(LINE_MAX_HZ), LINE_Y)])
                .style(faded_line(dim, 3.0, 1.0))
                .build(),
        );
        let risk = numbers("K-B2.risk_below_hz")?[0];
        let band = Rectangle::new()
            .width(hz_x(risk) - hz_x(0.0))
            .height(0.55)
            .style(flat(gold, 0.16))
            .build()?
            .moved_to(v((hz_x(0.0) + hz_x(risk)) / 2.0, LINE_Y));
        let band = stage.arena_mut().add(band);
        let band_label = kit.text(
            stage,
            &format!("walkers can push here: below about {} a second", whole_or(risk)),
            24.0,
            gold,
            v((hz_x(0.0) + hz_x(risk)) / 2.0, LINE_Y + 0.62),
        )?;
        let mut line_parts = vec![axis, band, band_label];
        for tick in [0.0, 0.5, 1.0, 1.5] {
            let t = stage.arena_mut().add(
                polyline(vec![v(hz_x(tick), LINE_Y - 0.12), v(hz_x(tick), LINE_Y + 0.12)])
                    .style(faded_line(dim, 3.0, 1.0))
                    .build(),
            );
            let words = if tick.fract() == 0.0 {
                whole(tick)
            } else {
                format!("{tick:.1}")
            };
            line_parts.push(t);
            line_parts.push(kit.text(stage, &words, 22.0, dim, v(hz_x(tick), LINE_Y - 0.48))?);
        }
        for &hz in &lateral {
            let dot = stage.arena_mut().add(
                Dot::new()
                    .point(v(hz_x(hz), LINE_Y))
                    .radius(0.1)
                    .style(flat(gold, 1.0))
                    .build(),
            );
            line_parts.push(dot);
        }
        let walker_hz = numbers("K-B2.walker_lateral_hz")?[0];
        let walker_mark = stage.arena_mut().add(
            polyline(vec![v(hz_x(walker_hz), LINE_Y - 0.3), v(hz_x(walker_hz), LINE_Y + 0.3)])
                .style(faded_line(orange, 5.0, 1.0))
                .build(),
        );
        let walker_label = kit.text(
            stage,
            &format!("a walker's sway: {} a second", whole(walker_hz)),
            24.0,
            orange,
            v(hz_x(walker_hz), LINE_Y - 1.15),
        )?;
        let axis_caption = kit.text(
            stage,
            "sideways swings a second",
            22.0,
            dim,
            v(hz_x(LINE_MAX_HZ) - 1.2, LINE_Y - 0.85),
        )?;
        line_parts.extend([walker_mark, walker_label, axis_caption]);
        let dallard = lower_third(
            &kit,
            stage,
            "Dallard et al.:",
            "possible on any bridge with a sideways frequency below about 1.3 Hz, given enough walkers; \
             names to search: synchronous lateral excitation",
        )?;
        let arup = lower_third(
            &kit,
            stage,
            "Arup's test, in their words:",
            &format!(
                "\"the north span is stable with {} people but with just {} more people the movements \
                 increase suddenly\"",
                fact("K-B2.test_steady")?,
                fact("K-B2.test_added")?
            ),
        )?;
        let swing_grow = ramp(stage, plan.grow, clock, 0.12, 1.0, 9.5)?;
        line.wait_for(stage, "Its sideways modes")?;
        let mut first = labels.clone();
        first.push(not_to_scale);
        tween_all(stage, &[(plan.shown, 1.0)], 0.7)?;
        fade_in_all(stage, &first, 0.5)?;
        fade_in_all(stage, &line_parts, 0.7)?;
        fade_in_all(stage, &[dallard], 0.5)?;
        line.wait_for(stage, "And the more the deck moved")?;
        tween_all(stage, &[(plan.push_on, 1.0)], 0.6)?;
        fade_out_all(stage, &[dallard], 0.4)?;
        fade_in_all(stage, &[arup], 0.5)?;
        line.wait_for(stage, "to keep their balance")?;

        // ---- 08_7: neither trouble was strength: a mode and a push that grew with it
        let line = say(&kit, stage, "08_7")?;
        stage.clear_updaters(swing_grow, true);
        tween_all(stage, &[(plan.shown, 0.0), (plan.push_on, 0.0)], 0.5)?;
        plan.stop(stage);
        let mut gone = first.clone();
        gone.extend(&line_parts);
        gone.push(arup);
        fade_out_all(stage, &gone, 0.5)?;
        let tacoma_gold = TwistDeck::build(
            stage,
            TwistSpec {
                centre: v(-3.5, 1.7),
                half_w: 2.2,
                hanger: 1.3,
            },
            twist_hz,
            gold,
            clock,
        )?;
        tacoma_gold.start(stage)?;
        let single = vec![Span {
            x0: 1.1,
            x1: 5.9,
            modes: vec![Mode {
                half_waves: 1,
                hz: lateral[0],
                share: 1.0,
            }],
        }];
        let london = PlanSpans::build(stage, single, 1.7, gold, clock)?;
        london.start(stage)?;
        let names = [
            kit.text(stage, "Tacoma Narrows", 28.0, dim, v(-3.5, 3.55))?,
            kit.text(stage, "London's footbridge", 28.0, dim, v(3.5, 3.55))?,
        ];
        let plot = |left: f64, rate: f64| PlotSpec {
            left,
            width: 5.2,
            mid_y: -0.9,
            height: 0.6,
            rate,
            cycles: 6.0,
        };
        let drained = SwingPlot::build(stage, plot(-6.2, -2.4), gold)?;
        let fed = SwingPlot::build(stage, plot(1.0, 2.4), gold)?;
        drained.start(stage)?;
        fed.start(stage)?;
        let captions = [
            kit.text(stage, "damping: energy drained", 28.0, ink, v(-3.6, -2.0))?,
            kit.text(stage, "negative damping: energy fed in", 28.0, ink, v(3.6, -2.0))?,
        ];
        let both_grow = [
            ramp(stage, tacoma_gold.amp, clock, 0.05, AMP_MAX, 8.0)?,
            ramp(stage, london.grow, clock, 0.1, 1.0, 8.0)?,
        ];
        line.wait_for(stage, "Neither bridge's trouble was strength")?;
        tween_all(stage, &[(tacoma_gold.shown, 1.0), (london.shown, 1.0)], 0.7)?;
        fade_in_all(stage, &names, 0.5)?;
        line.wait_for(stage, "Each had a mode")?;
        tween_all(stage, &[(tacoma_gold.push_on, 1.0), (london.push_on, 1.0)], 0.6)?;
        line.wait_for(stage, "Damping drains a swing")?;
        fade_in_all(stage, &[captions[0]], 0.4)?;
        tween_all(stage, &[(drained.reach, 1.0)], 2.2)?;
        line.wait_for(stage, "so engineers call this negative damping")?;
        fade_in_all(stage, &[captions[1]], 0.4)?;
        tween_all(stage, &[(fed.reach, 1.0)], 2.2)?;
        kit.hold(stage)?;
        for a in both_grow {
            stage.clear_updaters(a, true);
        }
        tacoma_gold.stop(stage);
        london.stop(stage);
        drained.stop(stage);
        fed.stop(stage);
        stage.clear_updaters(clock_anchor, true);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// A rate as it reads on screen: `1.3`, and `1` for a whole one.
fn whole_or(x: f64) -> String {
    if x.fract() == 0.0 { whole(x) } else { format!("{x:.1}") }
}

#[cfg(test)]
mod tests {
    use super::super::carts::probe::run_scene;
    use super::super::draw::testing;
    use super::*;

    /// A deck seen end on keeps its width as it twists, lies level at no
    /// twist and stands on end at a quarter turn.
    #[test]
    fn a_twisted_deck_keeps_its_width() {
        let c = v(1.0, 2.0);
        for k in 0..20 {
            let a = f64::from(k) * 0.07 - 0.7;
            let [l, r] = twisted_ends(c, 3.0, a);
            assert!(((r[0] - l[0]).hypot(r[1] - l[1]) - 6.0).abs() < 1e-12, "{a}");
            assert!(((l[0] + r[0]) / 2.0 - c[0]).abs() < 1e-12, "centred");
        }
        let [l, r] = twisted_ends(c, 3.0, 0.0);
        assert_eq!((l, r), (v(-2.0, 2.0), v(4.0, 2.0)));
        let [l, r] = twisted_ends(c, 3.0, PI / 2.0);
        assert!((l[1] - (2.0 - 3.0)).abs() < 1e-12 && (r[1] - 5.0).abs() < 1e-12);
    }

    /// 08_3, the claim the picture makes: the push FOLLOWS the motion. The
    /// arrow at the right end is up exactly when that end is going up (its
    /// share has the sign of the end's speed), and it is largest as the deck
    /// passes level and zero at the end of a swing.
    #[test]
    fn the_push_points_the_way_the_end_is_already_going_and_is_largest_passing_level() {
        let (amp, hz) = (0.5, 0.2);
        let h = 1e-5;
        for k in 0..100 {
            let t = f64::from(k) * 0.093;
            let y = |t: f64| twisted_ends(v(0.0, 0.0), 3.0, twist_angle(amp, hz, t))[1][1];
            let speed = (y(t + h) - y(t - h)) / (2.0 * h);
            let share = follow_share(hz, t);
            if speed.abs() > 1e-3 {
                assert_eq!(speed.signum(), share.signum(), "t = {t}");
            }
        }
        // level (angle 0) at t = 0: the largest push; the end of a swing is a quarter period on
        assert_eq!(follow_share(hz, 0.0), 1.0);
        assert!(twist_angle(amp, hz, 0.0).abs() < 1e-12);
        assert!(follow_share(hz, 0.25 / hz).abs() < 1e-12);
        assert!((twist_angle(amp, hz, 0.25 / hz) - amp).abs() < 1e-12);
        // and the push does work on the deck (it is in phase with its speed): its average
        // product with the angle's rate is positive over a period
        let n = 2000;
        let work: f64 = (0..n)
            .map(|k| {
                let t = f64::from(k) / f64::from(n) / hz;
                follow_share(hz, t) * (TAU * hz * amp * (TAU * hz * t).cos())
            })
            .sum::<f64>()
            / f64::from(n);
        assert!(work > 0.0, "{work}");
    }

    /// 08_2: the wind's puffs and the bridge's twists are two clocks, one
    /// a second and one every five, and they are not in step: over ten
    /// seconds ten puffs and two twists, and a puff comes when no twist does.
    #[test]
    fn puffs_and_twists_are_two_clocks_out_of_step() {
        let (vortex, period) = (
            numbers("K-B1.vortex_hz").unwrap()[0],
            numbers("K-B1.twist_period_s").unwrap()[0],
        );
        assert_eq!((vortex, period), (1.0, 5.0));
        assert_eq!(happened(10.0, vortex), 10.0);
        assert_eq!(happened(10.0, 1.0 / period), 2.0);
        assert_eq!(happened(0.5, vortex), 0.0, "none before the first");
        // the lamp is dark before the first puff, full at it and fades after it
        assert_eq!(puff(0.5, vortex), 0.0);
        assert!((puff(1.0, vortex) - 1.0).abs() < 1e-12);
        assert!(puff(1.5, vortex) < 0.1 && puff(1.5, vortex) > 0.0);
        // 4 of every 5 puffs come with no twist finishing
        let together = (1..=10).filter(|k| f64::from(*k) % period == 0.0).count();
        assert_eq!(together, 2);
        // and the twist is 0.2 a second, as the fact says
        assert_eq!(numbers("K-B1.twist_hz").unwrap(), [0.2]);
        assert!((numbers("K-B1.twist_hz").unwrap()[0] - 1.0 / period).abs() < 1e-12);
    }

    /// 08_5: the spans are the facts' measured modes (south 0.80, centre
    /// 0.48 and 0.95, north 1.03), each end held and the first mode one
    /// bow; every mode, and the walker's own rate, is under the range at
    /// which walkers push.
    #[test]
    fn the_spans_swing_at_the_measured_rates_and_every_one_is_in_the_range() {
        let lateral = numbers("K-B2.lateral_hz").unwrap();
        assert_eq!(lateral, [0.48, 0.80, 0.95, 1.03]);
        let spans = spans_from(&lateral).unwrap();
        assert_eq!(spans.len(), 3);
        let hz: Vec<Vec<f64>> = spans.iter().map(|s| s.modes.iter().map(|m| m.hz).collect()).collect();
        assert_eq!(hz, vec![vec![0.80], vec![0.48, 0.95], vec![1.03]]);
        assert!(spans_from(&lateral[..3]).is_none());
        for sp in &spans {
            for t in [0.0, 0.3, 1.1] {
                assert!(span_shift(&sp.modes, 0.0, t).abs() < 1e-12, "held at its start");
                assert!(span_shift(&sp.modes, 1.0, t).abs() < 1e-9, "and at its end");
            }
        }
        // the first mode is one bow (largest in the middle); the centre's second is an S
        let centre = &spans[1].modes;
        assert!(span_shift(&centre[..1], 0.5, 0.0) > span_shift(&centre[..1], 0.25, 0.0));
        assert!(
            span_shift(&centre[1..], 0.5, 0.0).abs() < 1e-9,
            "an S is level in the middle"
        );
        assert!((span_shift(&centre[1..], 0.25, 0.0) + span_shift(&centre[1..], 0.75, 0.0)).abs() < 1e-9);
        // never more than the span's largest swing
        for k in 0..200 {
            let (u, t) = (f64::from(k % 20) / 19.0, f64::from(k) * 0.05);
            assert!(span_shift(centre, u, t).abs() <= 1.0 + 1e-12);
            assert!(span_speed(centre, u, t).abs() <= 1.0 + 1e-12);
        }
        let risk = numbers("K-B2.risk_below_hz").unwrap()[0];
        let walker = numbers("K-B2.walker_lateral_hz").unwrap()[0];
        assert_eq!(risk, 1.3);
        assert!(lateral.iter().all(|&f| f < risk) && walker < risk);
        // the number line puts a rate where it belongs
        assert_eq!(hz_x(0.0), LINE_X.0);
        assert_eq!(hz_x(LINE_MAX_HZ), LINE_X.1);
        assert!(hz_x(0.95) < hz_x(1.03) && hz_x(1.03) < hz_x(risk));
    }

    /// The plan's push at a place follows that place's motion: it points the
    /// way it is going (checked against the numeric rate of change, for the
    /// centre span's two modes at different rates), and is zero where the
    /// span is held.
    #[test]
    fn the_walkers_push_follows_the_span_and_is_nothing_at_the_piers() {
        let modes = spans_from(&[0.48, 0.8, 0.95, 1.03]).unwrap()[1].modes.clone();
        let h = 1e-5;
        for (k, u) in [0.25, 0.5, 0.7].iter().enumerate() {
            for j in 0..60 {
                let t = f64::from(j) * 0.11 + 0.01 * k as f64;
                let speed = (span_shift(&modes, *u, t + h) - span_shift(&modes, *u, t - h)) / (2.0 * h);
                if speed.abs() > 1e-3 {
                    assert_eq!(speed.signum(), span_speed(&modes, *u, t).signum(), "u = {u}, t = {t}");
                }
            }
        }
        assert!(span_speed(&modes, 0.0, 0.3).abs() < 1e-12 && span_speed(&modes, 1.0, 0.3).abs() < 1e-9);
    }

    /// 08_7: the same swing dying away and growing: the largest value is
    /// at the start for one and at the end for the other, and they cross
    /// zero at the same places.
    #[test]
    fn a_swing_dies_away_or_grows_by_the_same_cycles() {
        let peak = |rate: f64| {
            (0..2000)
                .map(|k| {
                    (
                        f64::from(k) / 1999.0,
                        swing_value(rate, 6.0, f64::from(k) / 1999.0).abs(),
                    )
                })
                .fold((0.0, 0.0), |m, p| if p.1 > m.1 { p } else { m })
        };
        let (dying_at, dying) = peak(-2.4);
        let (growing_at, growing) = peak(2.4);
        assert!(dying_at < 0.05 && growing_at > 0.95, "{dying_at} {growing_at}");
        assert!(dying <= 1.0 + 1e-9 && growing <= 1.0 + 1e-9 && dying > 0.9 && growing > 0.9);
        // the same zero crossings: a quarter cycle in, both are positive, and
        // three quarters in, both are negative
        for k in 0..6 {
            let u = (f64::from(k) + 0.25) / 6.0;
            assert!(
                swing_value(-2.4, 6.0, u) > 0.0 && swing_value(2.4, 6.0, u) > 0.0,
                "u = {u}"
            );
            let u = (f64::from(k) + 0.75) / 6.0;
            assert!(
                swing_value(-2.4, 6.0, u) < 0.0 && swing_value(2.4, 6.0, u) < 0.0,
                "u = {u}"
            );
        }
    }

    /// The board's sentence is the source's, whole: the card quotes it
    /// verbatim (whitespace aside) from the fact sheet's S6.
    #[test]
    fn the_quotes_are_in_the_fact_sheet_verbatim() {
        let sheet = include_str!("../../docs/fact-sheet.md");
        let squeeze = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let sheet = squeeze(sheet);
        for quote in [
            "The Tacoma Narrows Bridge was well designed and built to resist safely all static forces, including \
             wind, usually considered in the design of similar structures.",
            "Billah and Scanlan",
            "American Journal of Physics",
        ] {
            assert!(sheet.contains(&squeeze(quote)), "not in the fact sheet: {quote:?}");
        }
        // the dates and the wind the labels print
        assert_eq!(
            date_text(fact("K-B1.failed").unwrap()).as_deref(),
            Some("7 November 1940")
        );
        assert_eq!(date_text(fact("K-B1.opened").unwrap()).as_deref(), Some("1 July 1940"));
        assert_eq!(fact("K-B1.wind_mph").unwrap(), "42");
        assert_eq!(whole_or(1.3), "1.3");
        assert_eq!(whole_or(1.0), "1");
    }

    /// The props really follow their trackers, on a bare stage: the twisting
    /// deck's right end is where the model puts it and its push arrow is
    /// as long as the share says; the plan's edges bow and its arrows
    /// point; every run written is an odd quad path; and what the updaters
    /// drive is on the scene.
    #[test]
    fn the_props_really_follow_their_trackers() {
        run_scene(|stage| {
            let (clock, _) = scene_clock(stage)?;
            let spec = TwistSpec {
                centre: v(0.0, 0.0),
                half_w: 2.0,
                hanger: 1.0,
            };
            let hz = 0.25;
            let deck = TwistDeck::build(stage, spec, hz, Palette::ink(), clock)?;
            deck.start(stage)?;
            let on_scene = stage.scene().mobjects().to_vec();
            assert!(on_scene.contains(&deck.deck) && deck.push.iter().all(|m| on_scene.contains(m)));
            stage.set_tracker_value(deck.amp, 0.4)?;
            stage.set_tracker_value(deck.shown, 1.0)?;
            stage.set_tracker_value(deck.push_on, 1.0)?;
            stage.wait(0.5)?;
            let t = stage.tracker_value(clock).unwrap_or(0.0);
            assert!(t > 0.3 && t < 0.8, "the clock reads {t}");
            let want = twisted_ends(spec.centre, spec.half_w, twist_angle(0.4, hz, t));
            let pts = stage.get_points(deck.deck).expect("the deck has points");
            assert_eq!(pts.len() % 2, 1);
            let right = pts[pts.len() - 1];
            // the clock moves a frame between the read and the draw: compare to a tenth of a second's worth
            let slack = 0.4 * TAU * hz * 0.1 * spec.half_w;
            assert!(
                (right[1] - want[1][1]).abs() < slack + 1e-4,
                "{right:?} against {:?}",
                want[1]
            );
            // the push arrow at the right end: upward while the end rises
            let arrow = stage.get_points(deck.push[1]).expect("the arrow has points");
            let length = arrow[2][1] - right[1];
            assert!(length > 0.0 && length <= PUSH_REACH + 1e-6, "{length}");
            // the plan
            let spans = spans_from(&[0.48, 0.8, 0.95, 1.03]).unwrap();
            let plan = PlanSpans::build(stage, spans, 0.0, own(), clock)?;
            plan.start(stage)?;
            stage.set_tracker_value(plan.grow, 1.0)?;
            stage.set_tracker_value(plan.shown, 1.0)?;
            stage.set_tracker_value(plan.push_on, 1.0)?;
            stage.wait(0.4)?;
            let edge = stage.get_points(plan.edges[0][0]).expect("an edge has points");
            assert_eq!(edge.len(), 2 * SPAN_POINTS - 1);
            let mid = edge[edge.len() / 2];
            assert!(
                (mid[1] - PLAN_WIDTH / 2.0).abs() > 0.01,
                "the span did not swing: {mid:?}"
            );
            let held = edge[0];
            assert!(
                (held[1] - PLAN_WIDTH / 2.0).abs() < 1e-5,
                "the pier end moved: {held:?}"
            );
            let arrow = stage.get_points(plan.push[0][1]).expect("an arrow has points");
            assert_eq!(arrow.len() % 2, 1);
            // the swing plot draws as far as it is told
            let spec_plot = PlotSpec {
                left: 0.0,
                width: 4.0,
                mid_y: 0.0,
                height: 0.5,
                rate: 2.0,
                cycles: 5.0,
            };
            let plot = SwingPlot::build(stage, spec_plot, own())?;
            plot.start(stage)?;
            stage.set_tracker_value(plot.reach, 0.5)?;
            stage.wait(0.2)?;
            let curve = stage.get_points(plot.line).expect("the plot has points");
            assert_eq!(curve.len() % 2, 1);
            let end = curve[curve.len() - 1][0];
            assert!((end - 2.0).abs() < 1e-4, "half way along 4 units is 2: {end}");
            Ok(())
        });
    }

    /// The chapter runs on its storyboard with the layout lint clean, types
    /// the strings it should, stays inside the chapter budget on the
    /// scratch voice, and its one plain prediction pause has no countdown.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("08_two_bridges").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        // a lower-third wraps its text over lines, so read the strings joined
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        let joined = words.join(" ");
        for want in [
            "7 November 1940",
            "opened 1 July 1940",
            "42 mph, measured that morning",
            "the wind's puffs:",
            "about 1 a second (estimate)",
            "the twist:",
            "1 every 5 seconds",
            "Billah and Scanlan, American Journal of Physics, 1991",
            "aeroelastic flutter",
            "The Tacoma Narrows Bridge was well",
            "the board of engineers, 1941",
            "what was pushing",
            "London's footbridge?",
            "south 0.80 Hz",
            "centre 0.48 and 0.95 Hz",
            "north 1.03 Hz",
            "a walker's sway: 1 a second",
            "Dallard et al.:",
            "synchronous lateral excitation",
            "Arup's test, in their words:",
            "the north span is stable with 156 people but with just 10 more people the movements increase suddenly",
            "damping: energy drained",
            "negative damping: energy fed in",
        ] {
            assert!(joined.contains(want), "the chapter never typesets {want:?}: {words:?}");
        }
        // neither "resonance" nor "synchrony" is shown as the cause
        assert!(
            !words
                .iter()
                .any(|w| w.to_lowercase().contains("resonance") || w.to_lowercase().contains("synchrony")),
            "{words:?}"
        );
        let s = testing::run_voiced("08_two_bridges").unwrap().ran.seconds;
        eprintln!("08_two_bridges: {s:.1} s on the scratch voice");
        assert!(s > 45.0 && s < 85.0, "{s} s");
    }
}
