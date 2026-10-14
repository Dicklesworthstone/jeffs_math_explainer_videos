//! What chapters 09 to 16 add to the central picture of `draw.rs` (the
//! plane of dots carried along their arrows):
//!
//! - the pure geometry of following a rule, which the scenes draw and the
//!   tests check against the model: where every start is after time `t`, the
//!   smooth path of one point, its path in straight steps, the arrow
//!   standing on a place;
//! - [`drive_rule`], which drives a `draw::Flow` from a rule whose four
//!   entries are trackers (09_3 slides the rule from the quarter turn to a
//!   spiral, and the arrows and trails re-form from the numbers);
//! - [`LiveMatrix`], a matrix whose entries are live numbers read from
//!   trackers each frame, with drawn brackets;
//! - [`precisely_tex`], the "precisely:" strip whose statement is
//!   mathematics.
#![allow(dead_code, reason = "the pieces land before the chapters that use each one")]

use explainer_kit::{Kit, Palette, Timed, tween};
use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{Flow, FlowSpec, arrow_points, corners, panel, result, tween_linear};
use crate::model::{Mat, expm_t};

// ------------------------------------------------------------ fades to batch

/// A fade-in of `m` over `rt` seconds, prepared, to play with others in
/// `play_prepared`. (Written as two statements because the constructor and
/// `prepare` each borrow the stage: `stage.prepare(fade_in(stage.arena_mut(),
/// …))` does not compile.)
pub fn fade_in_anim(stage: &mut Stage<'_>, m: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// A fade-out of `m` over `rt` seconds, prepared. Its updaters are cleared
/// first, so a live readout fades instead of being redrawn at full opacity.
pub fn fade_out_anim(stage: &mut Stage<'_>, m: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    stage.clear_updaters(m, true);
    let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// A fade-out of `m` that drifts by `shift` as it goes, prepared.
pub fn fade_out_drifting(stage: &mut Stage<'_>, m: Mob, shift: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_out(stage.arena_mut(), m, shift, 1.0)?;
    stage.prepare(a.rt(rt))
}

// -------------------------------------------------------------- the geometry

/// The field's grid: `cols` by `rows` starts `step` apart, centred on the
/// origin by half a step, so none of them sits on the centre (the one dot
/// that does is drawn on purpose).
pub fn grid_starts(cols: usize, rows: usize, step: f64) -> Vec<[f64; 2]> {
    let (cx, cy) = ((cols as f64 - 1.0) / 2.0, (rows as f64 - 1.0) / 2.0);
    (0..rows)
        .flat_map(|j| (0..cols).map(move |i| [(i as f64 - cx) * step, (j as f64 - cy) * step]))
        .collect()
}

/// The standard field of chapters 09 and 14: ten by eight starts, half a
/// unit apart (80 dots).
pub const FIELD_COLS: usize = 10;
pub const FIELD_ROWS: usize = 8;
pub const FIELD_STEP: f64 = 0.5;

pub fn field_starts() -> Vec<[f64; 2]> {
    grid_starts(FIELD_COLS, FIELD_ROWS, FIELD_STEP)
}

/// A rule's entries (row by row) as a matrix.
pub fn rule_of(entries: [f64; 4]) -> Mat {
    Mat::from_rows(&[&[entries[0], entries[1]], &[entries[2], entries[3]]])
}

/// A 2 by 2 matrix's entries, row by row.
pub fn entries_of(m: &Mat) -> [f64; 4] {
    [m.get(0, 0), m.get(0, 1), m.get(1, 0), m.get(1, 1)]
}

/// Where each start is after following `rule` for time `t`.
pub fn placed(rule: &Mat, t: f64, starts: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let e = expm_t(rule, t);
    starts
        .iter()
        .map(|s| {
            let p = e.apply(s);
            [p[0], p[1]]
        })
        .collect()
}

/// One point's smooth path: where it is at times 0, t/n, …, t.
pub fn path(rule: &Mat, t: f64, start: [f64; 2], n: usize) -> Vec<[f64; 2]> {
    let step = expm_t(rule, t / n as f64);
    let mut here = start;
    let mut out = vec![here];
    for _ in 0..n {
        let next = step.apply(&here);
        here = [next[0], next[1]];
        out.push(here);
    }
    out
}

/// One point's path in `n` straight steps, each along the arrow it stands
/// on: (I + (t/n) A)^k x for k = 0..n.
pub fn tiny_path(rule: &Mat, t: f64, start: [f64; 2], n: usize) -> Vec<[f64; 2]> {
    let step = Mat::identity(2).add(&rule.scale(t / n as f64));
    let mut here = start;
    let mut out = vec![here];
    for _ in 0..n {
        let next = step.apply(&here);
        here = [next[0], next[1]];
        out.push(here);
    }
    out
}

/// The arrow standing on place `at`: from there to where the rule's velocity
/// there, times `gain`, reaches (plane units).
pub fn velocity_ends(rule: &Mat, at: [f64; 2], gain: f64) -> ([f64; 2], [f64; 2]) {
    let v = rule.apply(&at);
    (at, [at[0] + gain * v[0], at[1] + gain * v[1]])
}

// ------------------------------------------------------------ a moving rule

/// The trackers a rule-driven flow reads: the time followed and the rule's
/// four entries, row by row.
#[derive(Clone, Copy, Debug)]
pub struct RuleTrackers {
    pub time: Mob,
    pub entries: [Mob; 4],
}

fn read(s: &Arena, m: Mob) -> f64 {
    s.tracker_value(m).unwrap_or(0.0)
}

impl RuleTrackers {
    /// Trackers at time 0 of `rule`.
    pub fn new(stage: &mut Stage<'_>, rule: &Mat) -> Self {
        let e = entries_of(rule);
        Self {
            time: stage.add_value_tracker(0.0),
            entries: [
                stage.add_value_tracker(e[0]),
                stage.add_value_tracker(e[1]),
                stage.add_value_tracker(e[2]),
                stage.add_value_tracker(e[3]),
            ],
        }
    }

    /// The rule as the trackers hold it now.
    pub fn rule(&self, s: &Arena) -> Mat {
        rule_of(self.entries.map(|m| read(s, m)))
    }

    /// Follow the rule on to time `t` over `rt` seconds, at constant speed.
    pub fn run_to(&self, t: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        tween_linear(self.time, t, rt)
    }

    /// The tweens that carry the rule's four entries to `to`'s.
    pub fn slide_to(&self, to: &Mat, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
        self.entries
            .iter()
            .zip(entries_of(to))
            .map(|(&m, x)| tween(m, x, rt))
            .collect()
    }

    /// One entry of e^{tA} as it is now, for a live readout.
    pub fn exp_entry(&self, i: usize, j: usize) -> impl Fn(&Arena) -> f64 + 'static {
        let me = *self;
        move |s| expm_t(&me.rule(s), read(s, me.time)).get(i, j)
    }

    /// One entry of the rule as it is now, for a live readout.
    pub fn rule_entry(&self, i: usize, j: usize) -> impl Fn(&Arena) -> f64 + 'static {
        let m = self.entries[2 * i + j];
        move |s| read(s, m)
    }
}

/// How many straight pieces a driven trail is drawn from.
const TRAIL_PIECES: usize = 24;

/// Drive `flow` (built from `spec`, with a `velocity`) from the trackers:
/// every dot sits at e^{tA} applied to its start, each arrow is the rule's
/// velocity at that place, each trail is the way there, and all of it is
/// rewritten by one updater, only when a tracker has moved. The flow's own
/// `drive` follows a rule that never changes; this is for a rule whose
/// entries slide. Returns the updater's anchor (add it to the scene; clear
/// its updaters before the flow fades).
pub fn drive_rule(stage: &mut Stage<'_>, flow: &Flow, spec: &FlowSpec, trackers: RuleTrackers) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let plane = spec.plane;
    let starts = spec.starts.clone();
    let (dots, arrows, trails) = (flow.dots.clone(), flow.arrows.clone(), flow.trails.clone());
    let trail_of = spec.trails.clone();
    let velocity = spec.velocity.map(|vel| (vel.scale, vel.head));
    let template: Vec<Vec3> = Dot::new().radius(spec.dot_radius).build().points().to_vec();
    let levels = flow.levels;
    let mut last: Option<([f64; 5], [f64; 3])> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = read(s, trackers.time);
            let e = trackers.entries.map(|m| read(s, m));
            let lv = levels.map(|m| read(s, m));
            let key = [t, e[0], e[1], e[2], e[3]];
            if last.is_some_and(|(k, l)| k == key && l == lv) {
                return;
            }
            let moved = last.is_none_or(|(k, _)| k != key);
            last = Some((key, lv));
            if moved {
                let rule = rule_of(e);
                let here = placed(&rule, t, &starts);
                for (&d, q) in dots.iter().zip(&here) {
                    let c = plane.p(q[0], q[1]);
                    let pts: Vec<Vec3> = template.iter().map(|p| [p[0] + c[0], p[1] + c[1], p[2]]).collect();
                    let _ = s.set_points(d, &pts);
                }
                if let Some((gain, head)) = velocity {
                    for (&a, q) in arrows.iter().zip(&here) {
                        let (from, to) = velocity_ends(&rule, *q, gain);
                        let _ = s.set_points(
                            a,
                            &corners(&arrow_points(plane.p(from[0], from[1]), plane.p(to[0], to[1]), head)),
                        );
                    }
                }
                for (&tr, &i) in trails.iter().zip(&trail_of) {
                    let mut way: Vec<Vec3> = path(&rule, t, starts[i], TRAIL_PIECES)
                        .into_iter()
                        .map(|q| plane.p(q[0], q[1]))
                        .collect();
                    if t.abs() < 1e-3 {
                        // no time has passed: no length to draw
                        way.truncate(1);
                        way.push([way[0][0] + 1e-4, way[0][1], 0.0]);
                    }
                    let _ = s.set_points(tr, &corners(&way));
                }
            }
            for &d in &dots {
                s.set_opacity(d, lv[0], false);
            }
            // arrows and trails are strokes: their fill stays at zero
            // (`set_opacity` would fill an open trail into a petal)
            for &a in &arrows {
                s.set_stroke(a, None, None, Some(lv[1]), None, false);
            }
            for &tr in &trails {
                s.set_stroke(tr, None, None, Some(lv[2]), None, false);
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

// -------------------------------------------------------- piece by piece

/// An opacity tracker for each of a set of mobjects, written by one updater:
/// the cells of a grid come in one by one with ordinary tweens, however they
/// are grouped (a fade on a child of a group would fight its parent).
pub struct Levels {
    pub anchor: Mob,
    pub trackers: Vec<Mob>,
}

impl Levels {
    /// Trackers at `initial` for each of `mobs` (the updater is on the
    /// scene; the mobjects themselves must be, or be in a family that is).
    pub fn build(stage: &mut Stage<'_>, mobs: &[Mob], initial: f64) -> fmn::Result<Self> {
        Self::build_with_strokes(stage, mobs, &[], initial)
    }

    /// As [`Levels::build`], with `strokes` (some of `mobs`) the open lines
    /// of a drawing, brackets and the like: `set_opacity` writes the fill
    /// alpha as well, and the fill of an open bracket is the solid bar
    /// between its path and its chord (11's matrix A was drawn with bars
    /// for brackets), so those pieces take their level in the stroke alone.
    pub fn build_with_strokes(stage: &mut Stage<'_>, mobs: &[Mob], strokes: &[Mob], initial: f64) -> fmn::Result<Self> {
        let trackers: Vec<Mob> = mobs.iter().map(|_| stage.add_value_tracker(initial)).collect();
        let anchor = stage.arena_mut().add(VMobject::new());
        let (pieces, levels) = (mobs.to_vec(), trackers.clone());
        let line: Vec<bool> = mobs.iter().map(|m| strokes.contains(m)).collect();
        let mut last = vec![f64::NAN; pieces.len()];
        stage.add_updater(
            anchor,
            move |s, _| {
                for (i, (&m, &t)) in pieces.iter().zip(&levels).enumerate() {
                    let l = read(s, t);
                    if l.to_bits() != last[i].to_bits() {
                        if line[i] {
                            s.set_stroke(m, None, None, Some(l), None, true);
                        } else {
                            s.set_opacity(m, l, true);
                        }
                        last[i] = l;
                    }
                }
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { anchor, trackers })
    }

    /// Bring piece `i` to `to` (0 to 1) over `rt` seconds.
    pub fn tween(&self, i: usize, to: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        tween(self.trackers[i], to, rt)
    }
}

// ------------------------------------------------------ the exact version

/// A piece of a "precisely:" statement: words in the text face, or
/// mathematics in TeX.
#[derive(Clone, Copy, Debug)]
pub enum Part<'a> {
    Words(&'a str),
    Math(&'a str),
}

/// A "precisely:" band at the foot of the frame whose statement mixes words
/// and mathematics (the kit's `precisely` sets plain text only): the label in
/// the hero role, then the parts left to right, on the same card as the
/// kit's lower-thirds. A statement wider than the band is scaled to fit.
pub fn precisely_parts(kit: &Kit, stage: &mut Stage<'_>, parts: &[Part<'_>]) -> fmn::Result<Mob> {
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let left = -w / 2.0 + 0.25;
    let tag = kit.text_left(stage, "precisely:", 26.0, result(), [left, 0.0, 0.0])?;
    let mut mobs = Vec::with_capacity(parts.len());
    for part in parts {
        mobs.push(match part {
            Part::Words(s) => kit.text(stage, s, 28.0, Palette::ink(), ORIGIN)?,
            Part::Math(s) => kit.tex(stage, s, 34.0, Palette::ink(), &[], ORIGIN)?,
        });
    }
    let gap = 0.14;
    let indent = left + stage.get_width(tag) + 0.25;
    let room = w + left - indent - 0.25;
    let natural: f64 = mobs.iter().map(|&m| stage.get_width(m)).sum::<f64>() + gap * (mobs.len() as f64 - 1.0);
    let fit = (room / natural).min(1.0);
    let mut tall: f64 = 0.36;
    for &m in &mobs {
        if fit < 1.0 {
            stage.scale(m, fit);
        }
        tall = tall.max(stage.get_height(m));
    }
    let h = 0.3 + tall;
    let y = -FRAME_Y_RADIUS + 0.2 + h / 2.0;
    stage.move_to(tag, [left, y, 0.0], LEFT);
    let mut x = indent;
    for &m in &mobs {
        let width = stage.get_width(m);
        stage.move_to(m, [x + width / 2.0, y, 0.0], ORIGIN);
        x += width + gap * fit;
    }
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to([0.0, y, 0.0]);
    let back = stage.arena_mut().add(back);
    let group = stage.arena_mut().add(VMobject::new());
    stage.attach(group, back)?;
    stage.attach(group, tag)?;
    for &m in &mobs {
        stage.attach(group, m)?;
    }
    Ok(group)
}

/// A "precisely:" band whose statement is all mathematics.
pub fn precisely_tex(kit: &Kit, stage: &mut Stage<'_>, src: &str) -> fmn::Result<Mob> {
    precisely_parts(kit, stage, &[Part::Math(src)])
}

// ------------------------------------------------------- live matrices

/// The two bracket strokes of a matrix `w` wide and `h` tall centred at
/// `at`, each a polyline of four points with ticks `tick` long.
pub fn bracket_points(at: Vec3, w: f64, h: f64, tick: f64) -> ([Vec3; 4], [Vec3; 4]) {
    let (l, r) = (at[0] - w / 2.0, at[0] + w / 2.0);
    let (t, b) = (at[1] + h / 2.0, at[1] - h / 2.0);
    (
        [[l + tick, t, 0.0], [l, t, 0.0], [l, b, 0.0], [l + tick, b, 0.0]],
        [[r - tick, t, 0.0], [r, t, 0.0], [r, b, 0.0], [r - tick, b, 0.0]],
    )
}

/// The centres of a `rows` by `cols` grid of cells `dx` wide and `dy` tall
/// about `at`, row by row.
pub fn cell_centres(at: Vec3, rows: usize, cols: usize, dx: f64, dy: f64) -> Vec<Vec3> {
    let (cx, cy) = ((cols as f64 - 1.0) / 2.0, (rows as f64 - 1.0) / 2.0);
    (0..rows)
        .flat_map(|i| (0..cols).map(move |j| [at[0] + (j as f64 - cx) * dx, at[1] - (i as f64 - cy) * dy, 0.0]))
        .collect()
}

/// A matrix whose entries are live numbers, with drawn brackets: its
/// brackets (one group) and one readout per cell, row by row.
pub struct LiveMatrix {
    pub brackets: Mob,
    pub cells: Vec<Mob>,
    pub centres: Vec<Vec3>,
}

impl LiveMatrix {
    /// The grid centred at `at`: `value(i, j)` is the closure that reads
    /// entry (i, j) from the arena each frame; numbers have `places`
    /// decimals and sit with their right edges on a common line, so the
    /// digits never wobble as a sign comes and goes. Brackets in `color`,
    /// detached. The kit's live readouts put themselves on the scene when
    /// they are built, so build the matrix when it should appear and bring
    /// its brackets in with it.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        at: Vec3,
        rows: usize,
        cols: usize,
        (dx, dy): (f64, f64),
        size: f64,
        places: usize,
        color: Srgb,
        value: impl Fn(usize, usize) -> Box<dyn Fn(&Arena) -> f64>,
    ) -> fmn::Result<Self> {
        let centres = cell_centres(at, rows, cols, dx, dy);
        let (w, h) = (cols as f64 * dx + 0.3, rows as f64 * dy + 0.1);
        let (left, right) = bracket_points(at, w, h, 0.18);
        let style = Style::default().fill(color, 0.0).stroke(color, 4.0, 1.0);
        let l = stage.arena_mut().add(polyline(left.to_vec()).style(style).build());
        let r = stage.arena_mut().add(polyline(right.to_vec()).style(style).build());
        let brackets = stage.arena_mut().add(VMobject::new());
        stage.attach(brackets, l)?;
        stage.attach(brackets, r)?;
        let mut cells = Vec::with_capacity(rows * cols);
        for i in 0..rows {
            for j in 0..cols {
                let c = centres[i * cols + j];
                let m = kit.live_value(
                    stage,
                    value(i, j),
                    places,
                    false,
                    size,
                    color,
                    [c[0] + 0.5 * dx - 0.25, c[1], 0.0],
                    RIGHT,
                )?;
                cells.push(m);
            }
        }
        Ok(Self {
            brackets,
            cells,
            centres,
        })
    }

    /// The brackets and every readout.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.brackets];
        out.extend(&self.cells);
        out
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use explainer_kit::narration::wait_frames;
    use explainer_kit::{Palette, v};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{Plane, Velocity, map2, rule};
    use super::*;
    use crate::model::quarter_turn;

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    #[test]
    fn the_grid_is_offset_by_half_a_step_so_no_dot_sits_on_the_centre() {
        let g = field_starts();
        assert_eq!(g.len(), 80);
        assert!(g.iter().all(|p| p[0] != 0.0 && p[1] != 0.0));
        let near = g.iter().map(|p| p[0].hypot(p[1])).fold(f64::INFINITY, f64::min);
        assert!((near - 0.25_f64.hypot(0.25)).abs() < 1e-12);
        let (x_max, y_max) = g.iter().fold((0.0_f64, 0.0_f64), |m, p| (m.0.max(p[0]), m.1.max(p[1])));
        assert_eq!((x_max, y_max), (2.25, 1.75));
        for p in &g {
            assert!(g.contains(&[-p[0], -p[1]]), "{p:?} has no mirror");
        }
        // an odd grid would put one on the centre: that is why ours is even
        assert!(grid_starts(5, 5, 0.5).contains(&[0.0, 0.0]));
    }

    #[test]
    fn brackets_and_cells_are_laid_out_row_by_row() {
        let (l, r) = bracket_points([1.0, 2.0, 0.0], 4.0, 2.0, 0.25);
        assert_eq!(l[1], [-1.0, 3.0, 0.0], "the top left corner");
        assert_eq!(l[2], [-1.0, 1.0, 0.0], "the bottom left corner");
        assert_eq!(r[1], [3.0, 3.0, 0.0]);
        assert_eq!((l[0][0], r[0][0]), (-0.75, 2.75), "the ticks point inward");
        let c = cell_centres([0.0, 0.0, 0.0], 2, 2, 2.0, 1.0);
        assert_eq!(
            c,
            vec![[-1.0, 0.5, 0.0], [1.0, 0.5, 0.0], [-1.0, -0.5, 0.0], [1.0, -0.5, 0.0]]
        );
    }

    #[test]
    fn the_smooth_path_ends_at_the_exponential_and_the_quarter_turn_makes_a_circle() {
        let j = quarter_turn();
        let way = path(&j, PI, [1.0, 0.0], 40);
        assert_eq!(way.len(), 41);
        let end = way[40];
        assert!((end[0] + 1.0).abs() < 1e-9 && end[1].abs() < 1e-9, "{end:?}");
        for p in &way {
            assert!((p[0].hypot(p[1]) - 1.0).abs() < 1e-9, "stays on the unit circle");
        }
        assert!(
            way[20][0].abs() < 1e-9 && (way[20][1] - 1.0).abs() < 1e-9,
            "halfway is the quarter turn"
        );
    }

    /// 06_4 and 09_2: the straight-step paths end where the facts say, and
    /// more steps hug the circle more tightly.
    #[test]
    fn tiny_steps_end_where_the_facts_say() {
        let j = quarter_turn();
        for n in [4_usize, 16, 100, 1000] {
            let end = *tiny_path(&j, PI, [1.0, 0.0], n).last().unwrap();
            let want = super::super::draw::numbers(&format!("F-N.turn.steps.n{n}")).unwrap();
            assert!(
                (end[0] - want[0]).abs() < 1e-4 && (end[1] - want[1]).abs() < 1e-4,
                "n = {n}"
            );
        }
        let radius = |n| {
            tiny_path(&j, PI, [1.0, 0.0], n)
                .last()
                .map(|p| p[0].hypot(p[1]))
                .unwrap()
        };
        assert!(radius(4) > radius(16) && radius(16) > radius(100) && radius(100) > 1.0);
    }

    #[test]
    fn the_arrow_on_a_dot_is_the_rule_applied_to_its_place() {
        let j = quarter_turn();
        let (from, to) = velocity_ends(&j, [2.0, 1.0], 0.5);
        assert_eq!(from, [2.0, 1.0]);
        assert_eq!(to, [2.0 - 0.5, 1.0 + 0.5 * 2.0]);
        let spiral = crate::facts::spiral_rule();
        assert_eq!(velocity_ends(&spiral, [0.0, 0.0], 1.0).1, [0.0, 0.0]);
        // the arrow is at right angles to the place under the quarter turn
        for p in [[1.0, 0.0], [0.3, -2.0], [-4.5, 0.25]] {
            let (_, tip) = velocity_ends(&j, p, 1.0);
            let push = [tip[0] - p[0], tip[1] - p[1]];
            assert!((push[0] * p[0] + push[1] * p[1]).abs() < 1e-12);
        }
    }

    /// The driver: every dot goes where e^{tA} sends it as the time tracker
    /// moves, the entries sliding to a spiral send them where the spiral
    /// does, each trail ends on its dot, and nothing is added per frame.
    #[test]
    fn the_driver_places_every_dot_and_never_grows_the_scene() {
        run(|stage| {
            let j = quarter_turn();
            let plane = Plane {
                centre: v(-2.9, 0.1),
                unit: 1.3,
            };
            let starts = field_starts();
            let spec = FlowSpec {
                plane,
                starts: starts.clone(),
                dot_radius: 0.05,
                dot_color: Palette::ink(),
                dot_level: 1.0,
                velocity: Some(Velocity {
                    rule: map2(&j),
                    scale: 0.3,
                    color: rule(),
                    width: 3.0,
                    head: 0.13,
                }),
                trails: (0..starts.len()).step_by(2).collect(),
                trail_color: Palette::hero(),
                trail_width: 3.0,
            };
            let flow = Flow::build(stage, spec.clone())?;
            stage.add_to_scene(flow.group)?;
            let trackers = RuleTrackers::new(stage, &j);
            drive_rule(stage, &flow, &spec, trackers)?;
            // the layers: dots lit, arrows and trails lit too
            for lv in flow.levels {
                stage.set_tracker_value(lv, 1.0)?;
            }
            let count = |stage: &Stage<'_>| stage.roots().iter().map(|&r| stage.family(r).len()).sum::<usize>();
            let before = count(stage);
            stage.set_tracker_value(trackers.time, FRAC_PI_2)?;
            wait_frames(stage, 2)?;
            for (i, s) in starts.iter().enumerate() {
                // a quarter turn takes (x, y) to (−y, x)
                let want = plane.p(-s[1], s[0]);
                let got = stage.get_center(flow.dots[i]);
                assert!(
                    (got[0] - want[0]).abs() < 1e-6 && (got[1] - want[1]).abs() < 1e-6,
                    "dot {i}: {got:?} against {want:?}"
                );
            }
            // the entries slide to the spiral: the dots follow the new rule
            let spiral = crate::facts::spiral_rule();
            for (m, x) in trackers.entries.iter().zip(entries_of(&spiral)) {
                stage.set_tracker_value(*m, x)?;
            }
            wait_frames(stage, 2)?;
            let want = placed(&spiral, FRAC_PI_2, &starts);
            for (i, w) in want.iter().enumerate() {
                let (got, w) = (stage.get_center(flow.dots[i]), plane.p(w[0], w[1]));
                assert!((got[0] - w[0]).abs() < 1e-6 && (got[1] - w[1]).abs() < 1e-6, "dot {i}");
            }
            // each trail ends on its dot
            for (&tr, &i) in flow.trails.iter().zip(&spec.trails) {
                let pts = stage.get_points(tr).expect("a trail has points");
                let last = pts[pts.len() - 1];
                let dot = stage.get_center(flow.dots[i]);
                assert!(
                    (last[0] - dot[0]).abs() < 1e-6 && (last[1] - dot[1]).abs() < 1e-6,
                    "trail {i} ends at {last:?}, its dot is at {dot:?}"
                );
            }
            // the live readouts read the same matrix
            let e = expm_t(&spiral, FRAC_PI_2).get(0, 0);
            assert!((trackers.exp_entry(0, 0)(stage.arena()) - e).abs() < 1e-12);
            assert_eq!(trackers.rule_entry(1, 0)(stage.arena()), 1.0);
            assert_eq!(count(stage), before, "following a rule adds nothing to the scene");
            Ok(())
        });
    }

    /// Pieces of a group come in one at a time from their trackers.
    #[test]
    fn levels_show_and_hide_pieces_one_by_one() {
        run(|stage| {
            let a = stage
                .arena_mut()
                .add(Dot::new().point([0.0, 0.0, 0.0]).radius(0.2).build());
            let b = stage
                .arena_mut()
                .add(Dot::new().point([2.0, 0.0, 0.0]).radius(0.2).build());
            stage.add_to_scene(a)?;
            stage.add_to_scene(b)?;
            let levels = Levels::build(stage, &[a, b], 0.0)?;
            wait_frames(stage, 2)?;
            let seen = |s: &Stage<'_>, m: Mob| explainer_kit::layout::visible_box(s.arena(), m).is_some();
            assert!(!seen(stage, a) && !seen(stage, b), "both start hidden");
            stage.set_tracker_value(levels.trackers[1], 1.0)?;
            wait_frames(stage, 2)?;
            assert!(!seen(stage, a) && seen(stage, b), "only the second came in");
            Ok(())
        });
    }

    /// A live matrix is four readouts inside two brackets that enclose them.
    #[test]
    fn a_live_matrix_has_its_cells_inside_its_brackets() {
        run(|stage| {
            let kit = Kit::new()?;
            let t = stage.add_value_tracker(2.5);
            let m = LiveMatrix::build(
                &kit,
                stage,
                [1.0, -0.5, 0.0],
                2,
                2,
                (1.5, 0.8),
                44.0,
                1,
                Palette::teal(),
                |i, j| {
                    let k = (2 * i + j) as f64;
                    Box::new(move |s: &Arena| read(s, t) * k)
                },
            )?;
            stage.add_to_scene(m.brackets)?;
            wait_frames(stage, 2)?;
            assert_eq!(m.cells.len(), 4);
            let (w, h) = (stage.get_width(m.brackets), stage.get_height(m.brackets));
            // (points are stored as 32-bit floats)
            assert!((w - 3.3).abs() < 1e-5, "brackets {w} wide");
            assert!((h - 1.7).abs() < 1e-5, "brackets {h} tall");
            let (lo, hi) = (stage.get_left(m.brackets)[0], stage.get_right(m.brackets)[0]);
            for &c in &m.cells {
                let (l, r) = (stage.get_left(c)[0], stage.get_right(c)[0]);
                assert!(lo < l && r < hi, "a readout sits between the brackets");
            }
            Ok(())
        });
    }
}
