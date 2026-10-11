//! Chapter 06, three customers: 05's progress square grows a third side and
//! becomes a cube, one side per customer, ten steps on each (06_1); the
//! three "two inside" beams are drawn last; then orders drawn uniformly from
//! all of them thread the cube as glowing trails while the count climbs,
//! and no trail ever enters a beam (06_2). The orders, their count and the
//! draw are `model::bakery_count` (fact C-F4), the call the facts pin.
//!
//! The cube is drawn on the retained route, orthographically: every shape
//! keeps its points in lattice coordinates (a, b, c = the steps Blue, Green
//! and Purple have taken), and one updater turns, tilts, scales and places
//! them from trackers, so the camera's moves are tracker tweens. (The camera
//! route samples every stroke pixel in 3-D, far too slow for 200 trails.)

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, play, thousands, tween, v};
use explainer_score::rng::{SplitMix64, seed_for};
use fmn::prelude::*;

use super::draw::{Lattice, NAMES, badge, customer, group, order_gold, room_red};
use crate::model::{Cmp, OrderCount, bakery_count, bakery_program};

/// 05_7's last frame, which 06 opens on (ChartreuseLotus's square).
const SQUARE: Lattice = Lattice {
    center: [-2.4, -0.1, 0.0],
    side: 5.4,
    steps: 7,
};

/// How many orders thread the cube (E-C2: about 200 polylines).
const TRAILS: usize = 200;
/// Trails start this far apart on the reveal tracker, and each takes
/// `TRAIL_SPAN` of it to draw.
const TRAIL_LAG: f64 = 0.02;
const TRAIL_SPAN: f64 = 1.0;
const TRAIL_OPACITY: f64 = 0.34;

/// Where the view settles (radians): turned about Purple's side, then
/// tilted back so Purple's side stands up; then an orbit of `TURN` about
/// Purple's side that stays clear of the face-on views (spin 0 and −π/2,
/// where two sides line up and the cube flattens).
const SPIN: f64 = -0.4;
const TILT: f64 = 1.18;
const TURN: f64 = -0.75;
/// The settled cube: scene units per step, and where its centre sits.
const SCALE: f64 = 0.37;
const CENTER: (f64, f64) = (0.25, -0.25);

/// The view, all trackers: spin about the vertical (Purple's) axis, tilt
/// back, scene units per step, the cube centre's place, and how far the
/// trails have drawn.
#[derive(Clone, Copy)]
struct View {
    spin: Mob,
    tilt: Mob,
    scale: Mob,
    cx: Mob,
    cy: Mob,
    reveal: Mob,
}

/// The view's values in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    spin: f64,
    tilt: f64,
    scale: f64,
    cx: f64,
    cy: f64,
    reveal: f64,
}

impl Pose {
    /// Lattice point `p` on screen, the cube's centre at `mid` on each axis.
    fn project(&self, p: [f64; 3], mid: f64) -> Vec3 {
        let (x, y, z) = (p[0] - mid, p[1] - mid, p[2] - mid);
        let (s, c) = self.spin.sin_cos();
        let (x, y) = (x * c - y * s, x * s + y * c);
        let (st, ct) = self.tilt.sin_cos();
        let y = y * ct + z * st;
        v(self.cx + self.scale * x, self.cy + self.scale * y)
    }

    /// How much of trail `i` is drawn, 0 to 1.
    fn drawn(&self, i: usize) -> f64 {
        ((self.reveal - i as f64 * TRAIL_LAG) / TRAIL_SPAN).clamp(0.0, 1.0)
    }
}

/// One shape of the cube, in lattice coordinates.
struct Shape {
    mob: Mob,
    verts: Vec<[f64; 3]>,
    closed: bool,
    /// The trail's place in the reveal order (trails only).
    trail: Option<usize>,
    /// A tracker its opacity follows (0 hides it), with its full stroke and
    /// fill opacities.
    fade: Option<(Mob, f64, f64)>,
}

/// The cube's shapes and the view they are drawn from.
struct Cube {
    steps: usize,
    view: View,
    shapes: Vec<Shape>,
}

impl Cube {
    fn new(stage: &mut Stage<'_>, steps: usize) -> Self {
        let view = View {
            spin: stage.add_value_tracker(0.0),
            tilt: stage.add_value_tracker(0.0),
            scale: stage.add_value_tracker(SQUARE.side / steps as f64),
            cx: stage.add_value_tracker(SQUARE.center[0]),
            cy: stage.add_value_tracker(SQUARE.center[1]),
            reveal: stage.add_value_tracker(0.0),
        };
        Self {
            steps,
            view,
            shapes: Vec::new(),
        }
    }

    /// A shape through `verts`, drawn in `style` (detached; the updater
    /// places it).
    fn add(
        &mut self,
        stage: &mut Stage<'_>,
        verts: Vec<[f64; 3]>,
        closed: bool,
        style: Style,
        fade: Option<(Mob, f64, f64)>,
    ) -> Mob {
        let flat: Vec<Vec3> = verts.iter().map(|p| v(p[0], p[1])).collect();
        let vm = if closed {
            Polygon::new(flat).style(style).build()
        } else {
            polyline(flat).style(style).build()
        };
        let mob = stage.arena_mut().add(vm);
        self.shapes.push(Shape {
            mob,
            verts,
            closed,
            trail: None,
            fade,
        });
        mob
    }

    fn line(&mut self, stage: &mut Stage<'_>, a: [f64; 3], b: [f64; 3], style: Style, fade: Option<(Mob, f64)>) -> Mob {
        self.add(stage, vec![a, b], false, style, fade.map(|(t, o)| (t, o, 0.0)))
    }

    /// Draw every shape from the view from now on: one updater on `anchor`,
    /// redrawing only when a tracker has moved.
    fn install(self, stage: &mut Stage<'_>, anchor: Mob) -> fmn::Result<View> {
        let Self { steps, view, shapes } = self;
        let mid = steps as f64 / 2.0;
        let len = steps as f64 * 3.0;
        let mut last: Option<(Pose, Vec<f64>)> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let val = |s: &fmn::mobject::Stage, m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let pose = Pose {
                    spin: val(s, view.spin),
                    tilt: val(s, view.tilt),
                    scale: val(s, view.scale),
                    cx: val(s, view.cx),
                    cy: val(s, view.cy),
                    reveal: val(s, view.reveal),
                };
                let fades: Vec<f64> = shapes
                    .iter()
                    .map(|sh| sh.fade.map_or(1.0, |(t, _, _)| val(s, t)))
                    .collect();
                if last.as_ref() == Some(&(pose, fades.clone())) {
                    return;
                }
                for (sh, &a) in shapes.iter().zip(&fades) {
                    let mut pts: Vec<Vec3> = sh.verts.iter().map(|&p| pose.project(p, mid)).collect();
                    let mut alpha = a;
                    if let Some(i) = sh.trail {
                        let f = pose.drawn(i);
                        alpha = if f > 0.0 { 1.0 } else { 0.0 };
                        let m = f * len;
                        let full = (m.floor() as usize).min(pts.len() - 2);
                        let t = m - full as f64;
                        let (p, q) = (pts[full], pts[full + 1]);
                        pts.truncate(full + 1);
                        pts.push(v(p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])));
                    }
                    let vm = if sh.closed {
                        Polygon::new(pts).build()
                    } else {
                        polyline(pts).build()
                    };
                    if s.set_points(sh.mob, vm.points()).is_err() {
                        continue;
                    }
                    if sh.trail.is_some() {
                        s.set_stroke(sh.mob, None, None, Some(TRAIL_OPACITY * alpha), None, false);
                    } else if let Some((_, stroke, fill)) = sh.fade {
                        s.set_stroke(sh.mob, None, None, Some(stroke * alpha), None, false);
                        s.set_fill(sh.mob, None, Some(fill * alpha), None, false);
                    }
                }
                last = Some((pose, fades));
            },
            true,
        )?;
        Ok(view)
    }
}

/// A stroke and no fill.
fn stroke(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// Tween a tracker at a constant rate (an orbit).
fn steady(tracker: Mob, value: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

pub struct ThreeCustomers {
    pub kit: Kit,
}

impl SceneConstruct for ThreeCustomers {
    fn name(&self) -> &str {
        "06_three_customers"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let counted = bakery_count(3, true, Cmp::Lex);
        let total = counted.total();
        let steps = bakery_program(3, 0, true, Cmp::Lex).len();
        let n = steps as f64;
        let inside = n - 1.0;

        // the opening: 05's square as 05_7 left it, the both-inside point red
        let old_grid = stage.arena_mut().add(SQUARE.grid()?);
        let old_mark = stage
            .arena_mut()
            .add(SQUARE.mark(SQUARE.steps - 1, SQUARE.steps - 1, room_red()));
        play!(stage;
            fade_in(stage.arena_mut(), old_grid, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), old_mark, ORIGIN, 1.0)?.rt(0.6),
        );

        // the cube, in lattice coordinates: the floor's grid, its twelve
        // edges, the customers' sides, the beams, the trails
        let mut cube = Cube::new(stage, steps);
        let rise = stage.add_value_tracker(0.0);
        let beams_on = stage.add_value_tracker(0.0);
        let mut world = Vec::new();
        for k in 1..steps {
            let f = k as f64;
            world.push(cube.line(
                stage,
                [f, 0.0, 0.0],
                [f, n, 0.0],
                stroke(Palette::faint(), 1.5, 1.0),
                None,
            ));
            world.push(cube.line(
                stage,
                [0.0, f, 0.0],
                [n, f, 0.0],
                stroke(Palette::faint(), 1.5, 1.0),
                None,
            ));
        }
        let corner = |m: usize| [n * (m & 1) as f64, n * (m >> 1 & 1) as f64, n * (m >> 2) as f64];
        for a in 1..8usize {
            for bit in [1usize, 2, 4] {
                if a & bit != 0 {
                    continue;
                }
                let (p, q) = (corner(a), corner(a | bit));
                world.push(if p[2] == 0.0 && q[2] == 0.0 {
                    cube.line(stage, p, q, stroke(Palette::dim(), 2.5, 1.0), None)
                } else {
                    cube.line(stage, p, q, stroke(Palette::dim(), 2.0, 0.7), Some((rise, 0.7)))
                });
            }
        }
        // each customer's own side, from the corner where nobody has moved
        for i in 0..3 {
            let mut end = [0.0; 3];
            end[i] = n;
            let fade = (i == 2).then_some((rise, 1.0));
            world.push(cube.line(stage, [0.0; 3], end, stroke(customer(i + 1), 5.0, 1.0), fade));
        }
        // the both-inside point, then the three beams where two customers
        // are one step from the end, whatever the third has done
        let r = 0.23;
        let mark = cube.add(
            stage,
            vec![
                [inside - r, inside - r, 0.0],
                [inside + r, inside - r, 0.0],
                [inside + r, inside + r, 0.0],
                [inside - r, inside + r, 0.0],
            ],
            true,
            Style::default().fill(room_red(), 0.35).stroke(room_red(), 2.5, 1.0),
            None,
        );
        world.push(mark);
        let mut beams = Vec::new();
        for (i, j) in [(0, 1), (0, 2), (1, 2)] {
            for face in beam_faces(i, j, inside, n, r) {
                beams.push(cube.add(
                    stage,
                    face,
                    true,
                    Style::default().fill(room_red(), 0.0).stroke(room_red(), 1.5, 0.0),
                    Some((beams_on, 0.55, 0.13)),
                ));
            }
        }
        let trails = trails(&kit, stage, &mut cube, &counted);
        // the anchor carries the updater and draws nothing
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let view = cube.install(stage, anchor)?;

        // 06_1: a third customer; the square regrids to ten steps a side
        // and its third side rises as the view tilts
        let legend = legend(&kit, stage)?;
        kit.say(stage, "06_1")?;
        let mut show: Vec<Box<dyn Animation>> = Vec::new();
        for (k, &m) in legend.iter().enumerate() {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.1), 1.0)?.rt(0.6 + 0.35 * k as f64);
            show.push(stage.prepare(a)?);
        }
        stage.play_prepared(show)?;
        let ten = Lattice { steps, ..SQUARE };
        let grid_ten = stage.arena_mut().add(ten.grid()?);
        let mark_ten = stage.arena_mut().add(ten.mark(steps - 1, steps - 1, room_red()));
        play!(stage;
            Transform::new(old_grid, grid_ten).rt(1.2),
            Transform::new(old_mark, mark_ten).rt(1.2),
        );
        // the drawn cube takes over, on exactly those lines
        stage.remove_from_scene(old_grid);
        stage.remove_from_scene(old_mark);
        for &m in &world {
            stage.add_to_scene(m)?;
        }
        play!(stage;
            tween(view.tilt, TILT, 2.6)?,
            tween(view.spin, SPIN, 2.6)?,
            tween(view.scale, SCALE, 2.6)?,
            tween(view.cx, CENTER.0, 2.6)?,
            tween(view.cy, CENTER.1, 2.6)?,
            tween(rise, 1.0, 1.2)?,
        );
        // the both-inside point becomes the first beam; the others follow
        for &b in &beams {
            stage.add_to_scene(b)?;
        }
        stage.play(tween(beams_on, 1.0, 1.2)?)?;
        stage.remove_from_scene(mark);
        let key = kit.text(stage, "red: two in the room at once", 22.0, room_red(), v(4.95, 0.9))?;
        play!(stage; fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.6));
        let q = predict_pause(&kit, stage, v(4.95, 2.1))?;

        // 06_2: the count, and the orders themselves as trails drawn
        // uniformly from all of them
        let count = stage.add_value_tracker(0.0);
        kit.say(stage, "06_2")?;
        for &t in &trails {
            stage.add_to_scene(t)?;
        }
        // the beams stay drawn last, over every trail
        stage.scene_mut().bring_to_front(&beams)?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        let readout = kit.live_count(stage, count, 52.0, order_gold(), v(0.0, 3.3))?;
        let unit = kit.text(stage, "orders", 28.0, Palette::dim(), v(0.0, 2.75))?;
        stage.add_to_scene(unit)?;
        let full = TRAILS as f64 * TRAIL_LAG + TRAIL_SPAN;
        let climb = count
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(3.0),
                rate_func: Some(fmn::core::rate::rush_into),
                ..AnimateArgs::default()
            })?
            .set_value(total.orders as f64)?;
        play!(stage; climb, tween(view.reveal, full, 5.0)?);
        // and the beams stay empty: not one order passes through them
        let none = kit.text(
            stage,
            &format!("{} put two in the room", thousands(total.two_inside)),
            26.0,
            room_red(),
            v(4.95, 0.4),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), none, v(0.0, -0.1), 1.0)?.rt(0.6),
            tween(beams_on, 1.8, 0.4)?,
        );
        stage.play(tween(beams_on, 1.0, 0.6)?)?;
        let note = lower_third(
            &kit,
            stage,
            "trails:",
            &format!(
                "drawn uniformly at random from all {} orders, not as a real machine runs them",
                thousands(total.orders)
            ),
        )?;
        play!(stage;
            steady(view.spin, SPIN + TURN, 7.0)?,
            fade_in(stage.arena_mut(), note, v(0.0, 0.15), 1.0)?.rt(0.8),
        );
        kit.hold(stage)?;
        stage.clear_updaters(readout, true);
        stage.clear_updaters(anchor, true);
        explainer_kit::fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// The six faces of the beam where customers `i` and `j` (0-based) are both
/// in the room (step `inside` of `n`): a box `2r` thick around that line,
/// spanning the third customer's whole side.
fn beam_faces(i: usize, j: usize, inside: f64, n: f64, r: f64) -> Vec<Vec<[f64; 3]>> {
    let k = 3 - i - j;
    let at = |c: [usize; 3]| {
        let mut p = [0.0; 3];
        p[i] = inside + if c[0] == 1 { r } else { -r };
        p[j] = inside + if c[1] == 1 { r } else { -r };
        p[k] = if c[2] == 1 { n } else { 0.0 };
        p
    };
    let quad = |q: [[usize; 3]; 4]| q.into_iter().map(at).collect::<Vec<_>>();
    vec![
        quad([[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]),
        quad([[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]]),
        quad([[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]]),
        quad([[0, 1, 0], [1, 1, 0], [1, 1, 1], [0, 1, 1]]),
        quad([[0, 0, 0], [0, 1, 0], [0, 1, 1], [0, 0, 1]]),
        quad([[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]]),
    ]
}

/// Each customer's badge and name in her colour (Purple is new here).
fn legend(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 3]> {
    let mut out = Vec::new();
    for k in 1..=3 {
        let y = 3.3 - 0.5 * (k - 1) as f64;
        let b = badge(kit, stage, k, 0.17, v(-6.55, y))?;
        let t = kit.text_left(
            stage,
            &format!("{}'s steps", NAMES[k - 1]),
            26.0,
            customer(k),
            v(-6.25, y),
        )?;
        out.push(group(stage, &[b, t])?);
    }
    Ok([out[0], out[1], out[2]])
}

/// [`TRAILS`] orders drawn uniformly from all of them (seeded, so every
/// render draws the same ones), as gold staircases through the cube.
fn trails(kit: &Kit, stage: &mut Stage<'_>, cube: &mut Cube, counted: &OrderCount) -> Vec<Mob> {
    let mut rng = SplitMix64::new(seed_for("bakery_algorithm", &kit.chapter, "trails", 0));
    let mut random = move || rng.next_u64();
    let mut out = Vec::with_capacity(TRAILS);
    for k in 0..TRAILS {
        let order = counted.sample(&mut random);
        let mut at = [0.0; 3];
        let mut verts = vec![at];
        for &i in &order.moves {
            at[i] += 1.0;
            verts.push(at);
        }
        let m = cube.add(stage, verts, false, stroke(order_gold(), 2.2, 0.0), None);
        if let Some(sh) = cube.shapes.last_mut() {
            sh.trail = Some(k);
        }
        out.push(m);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_starts_on_05s_square() {
        // untilted, unturned, at the square's scale and place, the cube's
        // floor lies exactly on 05's square regridded to ten steps
        let steps = bakery_program(3, 0, true, Cmp::Lex).len();
        assert_eq!(steps, 10, "three customers take 3N + 1 = 10 steps a visit");
        let pose = Pose {
            spin: 0.0,
            tilt: 0.0,
            scale: SQUARE.side / steps as f64,
            cx: SQUARE.center[0],
            cy: SQUARE.center[1],
            reveal: 0.0,
        };
        let ten = Lattice { steps, ..SQUARE };
        let mid = steps as f64 / 2.0;
        for (a, b) in [(0.0, 0.0), (10.0, 0.0), (3.0, 7.0), (9.0, 9.0)] {
            for c in [0.0, 4.0, 10.0] {
                let p = pose.project([a, b, c], mid);
                let q = ten.p(a, b);
                assert!((p[0] - q[0]).abs() < 1e-12 && (p[1] - q[1]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn tilted_the_third_side_stands_up_and_no_trail_enters_a_beam() {
        let pose = Pose {
            spin: SPIN,
            tilt: TILT,
            scale: SCALE,
            cx: CENTER.0,
            cy: CENTER.1,
            reveal: 0.0,
        };
        let up = pose.project([0.0, 0.0, 10.0], 5.0)[1] - pose.project([0.0, 0.0, 0.0], 5.0)[1];
        assert!(up > 3.0, "Purple's side rises {up}");
        // the whole cube fits the frame, below the counter, at every angle
        // of the orbit
        for k in 0..=10 {
            let turned = Pose {
                spin: SPIN + TURN * f64::from(k) / 10.0,
                ..pose
            };
            for m in 0..8usize {
                let p = [
                    10.0 * (m & 1) as f64,
                    10.0 * (m >> 1 & 1) as f64,
                    10.0 * (m >> 2) as f64,
                ];
                let s = turned.project(p, 5.0);
                assert!(
                    s[0] > -5.3 && s[0] < 3.3 && s[1] > -3.1 && s[1] < 2.5,
                    "corner {p:?} at {s:?}"
                );
            }
        }
        // what 06_2 says, on the drawn orders and on all of them
        let c = bakery_count(3, true, Cmp::Lex);
        let mut rng = SplitMix64::new(seed_for("bakery_algorithm", "06_three_customers", "trails", 0));
        let mut random = move || rng.next_u64();
        for _ in 0..TRAILS {
            let o = c.sample(&mut random);
            let mut at = [0usize; 3];
            for &i in &o.moves {
                at[i] += 1;
                assert!(at.iter().filter(|&&s| s == 9).count() < 2, "a trail through a beam");
            }
        }
        assert!(
            c.total().orders > 2_000_000_000 && c.total().orders < 3_000_000_000,
            "over two billion"
        );
        assert_eq!(c.total().two_inside, 0, "not one");
    }
}
