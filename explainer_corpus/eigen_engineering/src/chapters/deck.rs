//! The footbridge's deck, from above (storyboard 01 and 14_6): a long
//! strip bowing from side to side under walkers, each walker moving along
//! it with a rhythm of their own, carried by the deck's sway. Everything is
//! rewritten by one updater from a clock that counts the scene's own time,
//! so a sway is as fast on screen as the number beside it says.
//!
//! The pure functions are the picture's geometry and are tested; the
//! builders only place what they say.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::f64::consts::{PI, TAU};

use explainer_kit::{Palette, v};
use fmn::prelude::*;

use super::draw::{corners, faded_line, flat};

/// Where the deck sits and how big it is (scene units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeckSpec {
    pub centre: Vec3,
    pub length: f64,
    pub width: f64,
}

/// How many points an edge is drawn from.
const EDGE_POINTS: usize = 48;

/// The first sideways mode of a deck held at both ends: a bow, `x` the
/// fraction along the deck, 0 to 1.
pub fn bow(x: f64) -> f64 {
    (PI * x).sin()
}

/// The deck's centre line at fraction `x` along it, `amp` scene units of
/// sway at midspan, at time `t` seconds, swinging `hz` times a second.
pub fn centre_line(spec: &DeckSpec, amp: f64, hz: f64, t: f64, x: f64) -> Vec3 {
    v(
        spec.centre[0] - spec.length / 2.0 + spec.length * x,
        spec.centre[1] + amp * bow(x) * (TAU * hz * t).cos(),
    )
}

/// One edge of the deck, `side` +1 or -1, as `EDGE_POINTS` points.
pub fn edge(spec: &DeckSpec, amp: f64, hz: f64, t: f64, side: f64) -> Vec<Vec3> {
    (0..EDGE_POINTS)
        .map(|k| {
            let x = k as f64 / (EDGE_POINTS - 1) as f64;
            let c = centre_line(spec, amp, hz, t, x);
            v(c[0], c[1] + side * spec.width / 2.0)
        })
        .collect()
}

/// Walker `i` of `n` at time `t`: how far along the deck (a fraction, 0 to
/// 1, wrapping) and how far across it (-1 to 1 of the half width). Walkers
/// start spread along the deck, half walking each way, in lanes of their
/// own, at speeds that differ a little: none of them in step.
pub fn walker(i: usize, n: usize, t: f64) -> (f64, f64) {
    // a low-discrepancy spread: golden ratio steps along and across
    let phi = 0.618_033_988_749_895;
    let start = (i as f64 * phi).fract();
    let lane = ((i as f64 * phi * 2.0 + 0.3).fract() - 0.5) * 1.6;
    let direction = if i.is_multiple_of(2) { 1.0 } else { -1.0 };
    let speed = 0.05 + 0.012 * ((i * 7 % n.max(1)) as f64 / n.max(1) as f64);
    let along = (start + direction * speed * t).rem_euclid(1.0);
    // a little personal sway across, at a rate and phase of their own
    let sway = 0.12 * (TAU * (0.9 + 0.04 * (i % 5) as f64) * t + i as f64).sin();
    (along, (lane + sway).clamp(-0.92, 0.92))
}

/// The deck's walkers in scene units at time `t`, carried by the deck's
/// own sway.
pub fn walker_place(spec: &DeckSpec, amp: f64, hz: f64, t: f64, i: usize, n: usize) -> Vec3 {
    let (along, across) = walker(i, n, t);
    let c = centre_line(spec, amp, hz, t, along);
    v(c[0], c[1] + across * (spec.width / 2.0 - 0.1))
}

/// The deck from above: two edges and a crowd of walkers.
pub struct Deck {
    pub spec: DeckSpec,
    /// The scene's own clock, seconds: a tracker advanced every frame by
    /// the scene's dt (added to the scene with the deck).
    pub clock: Mob,
    /// The sway at midspan, scene units; and how many times a second the
    /// deck swings; and how much of the edges and of the walkers shows, 0
    /// to 1 (the parts are in the scene from [`Deck::start`], at opacity 0:
    /// they are shown by these, never faded, or the updater would undo the
    /// fade the next frame).
    pub sway: Mob,
    pub swings: Mob,
    pub edges_on: Mob,
    pub walkers_on: Mob,
    pub edges: [Mob; 2],
    pub walkers: Vec<Mob>,
    anchor: Mob,
}

impl Deck {
    /// Build the deck, detached, with `n` walkers. The clock starts at 0
    /// and runs from the moment [`Deck::start`] puts it in the scene.
    pub fn build(stage: &mut Stage<'_>, spec: DeckSpec, n: usize) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let at0 = |side: f64| edge(&spec, 0.0, 1.0, 0.0, side);
        let edges = [
            stage
                .arena_mut()
                .add(polyline(at0(1.0)).style(faded_line(ink, 5.0, 0.0)).build()),
            stage
                .arena_mut()
                .add(polyline(at0(-1.0)).style(faded_line(ink, 5.0, 0.0)).build()),
        ];
        let mut walkers = Vec::new();
        for i in 0..n {
            let p = walker_place(&spec, 0.0, 1.0, 0.0, i, n);
            let d = Dot::new()
                .point(p)
                .radius(0.07)
                .style(flat(Palette::dim(), 0.0))
                .build();
            walkers.push(stage.arena_mut().add(d));
        }
        let clock = stage.add_value_tracker(0.0);
        let sway = stage.add_value_tracker(0.0);
        let swings = stage.add_value_tracker(1.0);
        let walkers_on = stage.add_value_tracker(0.0);
        let edges_on = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (edges_m, walkers_m) = (edges, walkers.clone());
        let mut t = 0.0_f64;
        let mut last = [f64::NAN; 5];
        stage.add_dt_updater(
            anchor,
            move |s, _, dt| {
                if dt > 0.0 {
                    t += dt;
                    let _ = s.set_tracker_value(clock, t);
                }
                let amp = s.tracker_value(sway).unwrap_or(0.0);
                let hz = s.tracker_value(swings).unwrap_or(1.0);
                let on = s.tracker_value(walkers_on).unwrap_or(0.0);
                let edge_on = s.tracker_value(edges_on).unwrap_or(0.0);
                let now = [t, amp, hz, on, edge_on];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                for (side, &m) in [1.0, -1.0].iter().zip(&edges_m) {
                    let _ = s.set_points(m, &corners(&edge(&spec, amp, hz, t, *side)));
                    s.set_stroke(m, None, None, Some(edge_on), None, false);
                }
                for (i, &d) in walkers_m.iter().enumerate() {
                    s.move_to(d, walker_place(&spec, amp, hz, t, i, n), ORIGIN);
                    s.set_fill(d, None, Some(on), None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            spec,
            clock,
            sway,
            swings,
            edges_on,
            walkers_on,
            edges,
            walkers,
            anchor,
        })
    }

    /// Put the updater (and so the clock), the edges and the walkers in the
    /// scene, at opacity 0 until [`Deck::edges_on`] and [`Deck::walkers_on`]
    /// rise: a mobject the updater drives that is never added is never
    /// drawn.
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        for &m in self.edges.iter().chain(&self.walkers) {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// The deck's own parts (edges and walkers), to fade with it.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.edges.to_vec();
        all.extend(&self.walkers);
        all
    }
}

/// A tracker that follows `0.5 (1 - cos)` of `clock` between `lo` and `hi`
/// with a period of `period` seconds, from the moment it is added: the
/// slow breathing of a picture ("sagging and springing back"). Returns the
/// anchor to clear with `clear_updaters`.
pub fn oscillate(stage: &mut Stage<'_>, tracker: Mob, clock: Mob, period: f64, lo: f64, hi: f64) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut start: Option<f64> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = s.tracker_value(clock).unwrap_or(0.0);
            let t0 = *start.get_or_insert(t);
            let phase = 0.5 * (1.0 - (TAU * (t - t0) / period).cos());
            let _ = s.set_tracker_value(tracker, lo + (hi - lo) * phase);
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> DeckSpec {
        DeckSpec {
            centre: v(0.0, 0.5),
            length: 11.0,
            width: 1.0,
        }
    }

    /// The deck is held at both ends: no sway there, and the most at
    /// midspan; at a quarter period it swings through straight.
    #[test]
    fn the_deck_bows_from_fixed_ends_and_swings_through_straight() {
        let s = spec();
        let (a, m, b) = (
            centre_line(&s, 0.4, 1.0, 0.0, 0.0),
            centre_line(&s, 0.4, 1.0, 0.0, 0.5),
            centre_line(&s, 0.4, 1.0, 0.0, 1.0),
        );
        assert!((a[1] - 0.5).abs() < 1e-12 && (b[1] - 0.5).abs() < 1e-12, "{a:?} {b:?}");
        assert!((m[1] - 0.9).abs() < 1e-12, "{m:?}");
        assert_eq!((a[0], b[0]), (-5.5, 5.5));
        let q = centre_line(&s, 0.4, 1.0, 0.25, 0.5);
        assert!((q[1] - 0.5).abs() < 1e-9, "straight a quarter swing in: {q:?}");
        // the swing rate is the one asked for: back at the start after 1/hz
        let again = centre_line(&s, 0.4, 0.95, 1.0 / 0.95, 0.5);
        assert!((again[1] - m[1]).abs() < 1e-9);
    }

    #[test]
    fn the_edges_are_a_width_apart_along_the_whole_deck() {
        let s = spec();
        let (up, down) = (edge(&s, 0.3, 1.0, 0.13, 1.0), edge(&s, 0.3, 1.0, 0.13, -1.0));
        assert_eq!(up.len(), EDGE_POINTS);
        for (a, b) in up.iter().zip(&down) {
            assert!((a[1] - b[1] - s.width).abs() < 1e-12);
            assert_eq!(a[0], b[0]);
        }
    }

    /// Walkers stay on the deck, wrap round at its ends, walk both ways,
    /// and no two are in step (their across-sway phases all differ).
    #[test]
    fn walkers_stay_on_the_deck_walk_both_ways_and_are_not_in_step() {
        let n = 24;
        let (mut left, mut right) = (0, 0);
        for i in 0..n {
            for k in 0..200 {
                let t = f64::from(k) * 0.37;
                let (along, across) = walker(i, n, t);
                assert!((0.0..1.0).contains(&along) && across.abs() <= 0.92, "{i} {t}");
            }
            let (a0, _) = walker(i, n, 0.0);
            let (a1, _) = walker(i, n, 0.5);
            let step = (a1 - a0 + 0.5).rem_euclid(1.0) - 0.5;
            if step > 0.0 {
                right += 1;
            } else {
                left += 1;
            }
        }
        assert_eq!((left, right), (12, 12));
        // distinct personal sways: no two walkers' across positions are equal at t = 3
        let mut across: Vec<f64> = (0..n).map(|i| walker(i, n, 3.0).1).collect();
        across.sort_by(f64::total_cmp);
        assert!(across.windows(2).all(|w| (w[1] - w[0]).abs() > 1e-6));
    }

    /// Carried by the deck: a walker's place is on the deck's centre line
    /// at its fraction along it, within half a width.
    #[test]
    fn a_walker_is_carried_by_the_deck() {
        let s = spec();
        for i in 0..24 {
            let p = walker_place(&s, 0.4, 0.95, 2.0, i, 24);
            let (along, _) = walker(i, 24, 2.0);
            let c = centre_line(&s, 0.4, 0.95, 2.0, along);
            assert!((p[0] - c[0]).abs() < 1e-12);
            assert!((p[1] - c[1]).abs() <= s.width / 2.0 + 1e-12);
        }
    }

    /// The deck really swings: on a bare stage (no render), once it is
    /// given a sway its edges are an odd run bowed off straight at the
    /// middle, the clock counts the scene's seconds, and walkers move.
    #[test]
    fn the_deck_really_swings_and_the_clock_counts_seconds() {
        crate::chapters::carts::probe::run_scene(|stage| {
            let deck = Deck::build(stage, spec(), 6)?;
            deck.start(stage)?;
            // what the updater drives is on the scene, or it is never drawn
            let on_scene = stage.scene().mobjects().to_vec();
            assert!(deck.edges.iter().chain(&deck.walkers).all(|m| on_scene.contains(m)));
            stage.set_tracker_value(deck.sway, 0.4)?;
            stage.set_tracker_value(deck.swings, 1.0)?;
            stage.set_tracker_value(deck.walkers_on, 1.0)?;
            stage.set_tracker_value(deck.edges_on, 1.0)?;
            stage.wait(0.3)?;
            let t = stage.tracker_value(deck.clock).unwrap_or(0.0);
            assert!((t - 0.3).abs() < 0.11, "the clock reads {t} s after waiting 0.3 s");
            let edge_pts = stage.get_points(deck.edges[0]).expect("an edge has points");
            assert_eq!(edge_pts.len(), 2 * EDGE_POINTS - 1);
            let mid = edge_pts[edge_pts.len() / 2];
            let straight = spec().centre[1] + spec().width / 2.0;
            assert!((mid[1] - straight).abs() > 0.03, "the deck did not bow: {mid:?}");
            // its ends are held
            let (a, b) = (edge_pts[0], edge_pts[edge_pts.len() - 1]);
            assert!((a[1] - straight).abs() < 1e-6 && (b[1] - straight).abs() < 1e-6);
            // walkers have moved from where they began
            let first = stage.get_center(deck.walkers[0]);
            stage.wait(0.5)?;
            let later = stage.get_center(deck.walkers[0]);
            assert!((first[0] - later[0]).abs() + (first[1] - later[1]).abs() > 1e-4);
            Ok(())
        });
    }
}
