//! Chapter 15, the doorway: the leftover of the loop matters most for
//! rotations. Flip the quarter turn J across its diagonal and every sign
//! changes: a matrix like that, used as a rule, is a spin, and following a
//! spin gives a rotation; three dimensions have three, one for each pair of
//! axes (15_1). Your turn: tip a cube a little about x, then about y, undo
//! each; are you back where you began (15_2)? No: repeat that loop and the
//! cube creeps round the third axis; the commutator of two spins is a third
//! spin (15_3). Add two spins or take their commutator and you still have a
//! spin: a Lie algebra; follow one rotation by another or undo one and you
//! still have a rotation: a Lie group; and the exponential takes each spin
//! to a rotation and reaches every rotation (15_4).
//!
//! Every number is a fact: the spins `F-L.bracket.*` and `F-L.Lz`, the
//! rotation at t = 1 `F-L.exp.Lz.t1`, the counts `F-L.spins.*`, the loop
//! `F-L.loop.*` and its creep `F-L.creep.*`, the quiz answer
//! `F-Q.15_2.answer`.

use super::draw::lower_third;
use explainer_kit::{Kit, Palette, fade_all, play, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, QuizCard, arrow, fact, fade_in_all, fade_out_all, faded_line, fit_width, music_hold, result, rule, say,
    tween_linear, whole,
};
use super::flow::{fade_in_anim, precisely_tex};
use crate::facts::{cube_loop, shown, spin_axes};
use crate::model::{Mat, angle_axis, expm_t, spins};

// ------------------------------------------------------------------ choices

/// The cube's size on screen (scene units for a half-side of one).
const CUBE_SCALE: f64 = 1.15;
/// Where the cube sits when it has the left of the frame.
const CUBE_AT: Vec3 = [-4.0, -0.4, 0.0];
/// The view: a turn about the vertical, then a tilt back (radians). The
/// "from along z" view of 15_3 is the tilt of a quarter turn.
pub const VIEW_YAW: f64 = 0.62;
pub const VIEW_TILT: f64 = 1.05;
const TOP_YAW: f64 = 0.0;
const TOP_TILT: f64 = std::f64::consts::FRAC_PI_2;

// ------------------------------------------------------------- the geometry

/// The cube's eight corners (half-side one), indexed by their bits.
pub fn corners() -> [[f64; 3]; 8] {
    let mut c = [[0.0; 3]; 8];
    for (i, p) in c.iter_mut().enumerate() {
        *p = [
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        ];
    }
    c
}

/// The twelve edges: pairs of corners one bit apart.
pub fn edges() -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for a in 0..8_usize {
        for bit in [1_usize, 2, 4] {
            if a & bit == 0 {
                out.push((a, a | bit));
            }
        }
    }
    out
}

/// The corners of the cube's top face (z = +1), the face painted gold, in
/// order round it.
pub fn top_face() -> [usize; 4] {
    [4, 5, 7, 6]
}

/// A point of space on screen: turned `yaw` about the vertical, then seen
/// from `tilt` above the horizon (a quarter turn looks straight down).
pub fn project(yaw: f64, tilt: f64, p: [f64; 3]) -> [f64; 2] {
    let (sy, cy) = yaw.sin_cos();
    let (x, y) = (cy * p[0] - sy * p[1], sy * p[0] + cy * p[1]);
    [x, y * tilt.sin() + p[2] * tilt.cos()]
}

/// The skew matrix as `hat` of the unit vector for axis 0 (x), 1 (y), 2 (z).
fn spin(axis: usize) -> Mat {
    spin_axes()[axis].clone()
}

/// The pose after tipping about x by `tip`, then about y, then undoing the
/// x tip, then the y tip, `loops` times over, with `loops` fractional: the
/// loop in progress is played stage by stage (the later motion on the left,
/// as the facts multiply it).
pub fn creep_pose(tip: f64, loops: f64) -> Mat {
    let (lx, ly) = (spin(0), spin(1));
    let whole_loops = loops.floor().max(0.0);
    let mut pose = Mat::identity(3);
    let one = cube_loop(tip);
    for _ in 0..whole_loops as u32 {
        pose = one.mul(&pose);
    }
    let f = (loops - whole_loops) * 4.0;
    let stage = (f.floor() as usize).min(3);
    let g = f - stage as f64;
    let moves: [(&Mat, f64); 4] = [(&lx, 1.0), (&ly, 1.0), (&lx, -1.0), (&ly, -1.0)];
    for (i, (m, sign)) in moves.iter().enumerate() {
        let t = if i < stage {
            *sign * tip
        } else if i == stage {
            *sign * tip * g
        } else {
            break;
        };
        pose = expm_t(m, t).mul(&pose);
    }
    pose
}

/// How far a rotation has turned, in degrees (0 to 180).
pub fn degrees(r: &Mat) -> f64 {
    angle_axis(r).0.to_degrees()
}

// ---------------------------------------------------------------- the cube

/// A wire cube with one gold face, redrawn from `pose(arena)` whenever the
/// pose changes.
pub struct Cube {
    pub edges: Vec<Mob>,
    pub face: Mob,
    pub anchor: Mob,
}

impl Cube {
    pub fn build(
        stage: &mut Stage<'_>,
        at: Vec3,
        scale: f64,
        (yaw, tilt): (f64, f64),
        pose: impl Fn(&Arena) -> Mat + 'static,
    ) -> fmn::Result<Self> {
        let place = move |m: &Mat, p: [f64; 3]| {
            let q = m.apply(&p);
            let s = project(yaw, tilt, [q[0], q[1], q[2]]);
            v(at[0] + scale * s[0], at[1] + scale * s[1])
        };
        let corner = corners();
        let id = Mat::identity(3);
        let mut edge_mobs = Vec::new();
        for (a, b) in edges() {
            let m = polyline(vec![place(&id, corner[a]), place(&id, corner[b])])
                .style(faded_line(Palette::ink(), 3.5, 1.0))
                .build();
            edge_mobs.push(stage.arena_mut().add(m));
        }
        let face_pts: Vec<Vec3> = top_face().iter().map(|&i| place(&id, corner[i])).collect();
        let face = Polygon::new(face_pts)
            .style(Style::default().fill(result(), 0.5).stroke(result(), 3.5, 1.0))
            .build();
        let face = stage.arena_mut().add(face);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (edge_ids, edge_pairs) = (edge_mobs.clone(), edges());
        let mut last: Option<Vec<u64>> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let m = pose(s);
                let key: Vec<u64> = m.entries().iter().map(|x| x.to_bits()).collect();
                if last.as_ref() == Some(&key) {
                    return;
                }
                last = Some(key);
                for (&mob, &(a, b)) in edge_ids.iter().zip(&edge_pairs) {
                    let _ = s.set_points(
                        mob,
                        polyline(vec![place(&m, corner[a]), place(&m, corner[b])])
                            .build()
                            .points(),
                    );
                }
                let quad: Vec<Vec3> = top_face().iter().map(|&i| place(&m, corner[i])).collect();
                let _ = s.set_points(face, Polygon::new(quad).build().points());
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self {
            edges: edge_mobs,
            face,
            anchor,
        })
    }

    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = self.edges.clone();
        out.push(self.face);
        out
    }
}

/// Rows of a model matrix as whole numbers.
fn whole_rows(m: &Mat) -> Vec<Vec<String>> {
    (0..m.n())
        .map(|i| m.row(i).iter().map(|&x| whole(x)).collect())
        .collect()
}

/// Where 15_4's row of three spins is centred (x), its row's height, its type
/// size and the room left between neighbouring brackets.
pub const ALGEBRA_X: f64 = -3.9;
const ALGEBRA_Y: f64 = 0.5;
const ALGEBRA_PT: f64 = 26.0;
const ALGEBRA_GAP: f64 = 0.4;

/// The three spins' grids, built and set side by side about `ALGEBRA_X`
/// (detached): their groups, left to right.
fn algebra_row(kit: &Kit, stage: &mut Stage<'_>, mats: &[Mat]) -> fmn::Result<Vec<Mob>> {
    let mut groups = Vec::new();
    let mut widths = Vec::new();
    for m in mats {
        let g = MatrixGrid::build(kit, stage, &whole_rows(m), ALGEBRA_PT, rule(), v(ALGEBRA_X, ALGEBRA_Y))?;
        widths.push(g.size.0);
        groups.push(g.group);
    }
    let total: f64 = widths.iter().sum::<f64>() + ALGEBRA_GAP * (widths.len() as f64 - 1.0);
    let mut x = ALGEBRA_X - total / 2.0;
    for (&g, &w) in groups.iter().zip(&widths) {
        stage.move_to(g, v(x + w / 2.0, ALGEBRA_Y), ORIGIN);
        x += w + ALGEBRA_GAP;
    }
    Ok(groups)
}

/// 15_2: where the quiz card stands, and its size.
const QUIZ_AT: Vec3 = [3.6, 0.3, 0.0];
const QUIZ_SIZE: (f64, f64) = (6.2, 3.9);
/// The card's lines, from the middle of its content: the moves, the question
/// the voice asks, and where the answer lands (the card stated the set-up and
/// not its question: the dense read of the scored draft).
const MOVES_DY: f64 = 0.6;
const QUESTION_DY: f64 = 0.0;
const ANSWER_DY: f64 = -0.85;

/// 15_2's card's lines (detached, to be shown with the card or landed): the
/// four moves in one line, fitted to `room`; the question; and the answer.
fn quiz_lines(kit: &Kit, stage: &mut Stage<'_>, middle: Vec3, room: f64) -> fmn::Result<[Mob; 3]> {
    let ink = Palette::ink();
    let moves = kit.text(
        stage,
        "tip about x, then about y, then undo each",
        30.0,
        ink,
        v(middle[0], middle[1] + MOVES_DY),
    )?;
    fit_width(stage, moves, room);
    let question = kit.text(
        stage,
        "back where you began?",
        40.0,
        ink,
        v(middle[0], middle[1] + QUESTION_DY),
    )?;
    fit_width(stage, question, room);
    let answer = kit.text(
        stage,
        "no",
        64.0,
        super::draw::warn(),
        v(middle[0], middle[1] + ANSWER_DY),
    )?;
    Ok([moves, question, answer])
}

// -------------------------------------------------------------- the scene

pub struct TheDoorway {
    pub kit: Kit,
}

impl SceneConstruct for TheDoorway {
    fn name(&self) -> &str {
        "15_the_doorway"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());
        let grids3: Vec<Mat> = spin_axes().to_vec();
        let axis_names = ["x", "y", "z"];
        let pair_names = ["yz", "zx", "xy"];

        // ---- 15_1: flip J across its diagonal: a spin; three of them
        let line = say(&kit, stage, "15_1")?;
        let j_rows = super::draw::entries("F-J.power.k1", None)?;
        let j_grid = MatrixGrid::build(&kit, stage, &j_rows, 60.0, teal, v(-4.4, 1.5))?;
        let j_name = kit.tex(stage, "J", 56.0, teal, &[], v(-4.4, 2.9))?;
        fade_in_all(stage, &[j_grid.group, j_name], 0.6)?;
        line.wait_for(stage, "Flip it across its diagonal")?;
        // J flipped across its diagonal is minus J: every sign changes
        let flipped = [
            [
                crate::model::quarter_turn().get(0, 0),
                crate::model::quarter_turn().get(1, 0),
            ],
            [
                crate::model::quarter_turn().get(0, 1),
                crate::model::quarter_turn().get(1, 1),
            ],
        ];
        let flip_rows: Vec<Vec<String>> = flipped.iter().map(|r| r.iter().map(|&x| whole(x)).collect()).collect();
        let flip_grid = MatrixGrid::build(&kit, stage, &flip_rows, 60.0, teal, v(0.0, 1.5))?;
        let flip_name = kit.tex(stage, r"J^{T}=-J", 56.0, ink, &[("J", teal)], v(0.0, 2.9))?;
        let to = arrow(stage, v(-2.7, 1.5), v(-1.7, 1.5), ink, 4.0)?;
        fade_in_all(stage, &[to, flip_grid.group, flip_name], 0.7)?;
        let skew = precisely_tex(&kit, stage, r"\text{skew: }A^{T}=-A")?;
        fade_in_all(stage, &[skew], 0.5)?;
        line.wait_for(stage, "call it a spin")?;
        // one spin at a time: the cube turns about its axis
        fade_out_all(
            stage,
            &[j_grid.group, j_name, to, flip_grid.group, flip_name, skew],
            0.5,
        )?;
        let which = stage.add_value_tracker(0.0);
        let turn = stage.add_value_tracker(0.0);
        let spins_list = grids3.clone();
        let cube = Cube::build(stage, CUBE_AT, CUBE_SCALE, (VIEW_YAW, VIEW_TILT), move |s| {
            let k = s.tracker_value(which).unwrap_or(0.0).round().clamp(0.0, 2.0) as usize;
            expm_t(&spins_list[k], s.tracker_value(turn).unwrap_or(0.0))
        })?;
        fade_in_all(stage, &cube.mobs(), 0.6)?;
        line.wait_for(stage, "Follow a spin, and you get a rotation")?;
        let grid_at = v(3.2, 0.9);
        let mut grids = Vec::new();
        let mut names = Vec::new();
        for (k, m) in grids3.iter().enumerate() {
            grids.push(MatrixGrid::build(&kit, stage, &whole_rows(m), 44.0, teal, grid_at)?);
            names.push(kit.text(
                stage,
                &format!("the {}-spin: turns {}", axis_names[k], pair_names[k]),
                30.0,
                Palette::dim(),
                v(grid_at[0], grid_at[1] + 1.9),
            )?);
        }
        #[cfg(test)]
        for (g, key) in grids.iter().zip(["15_1:spin_x", "15_1:spin_y", "15_1:spin_z"]) {
            super::draw::watch_dwell(&kit, stage, g.group, key)?;
        }
        let mut shown_now = vec![grids[0].group, names[0]];
        fade_in_all(stage, &shown_now, 0.5)?;
        // the exact version comes in with the first spin, not after all
        // three (it was up about a second before the next line)
        let exact = lower_third(
            &kit,
            stage,
            "precisely:",
            &format!(
                "every rotation comes from a spin; a full turn and no turn are the same rotation. \
                 Four axes would make {} pairs",
                spins(4)
            ),
        )?;
        fade_in_all(stage, &[exact], 0.5)?;
        play!(stage; tween_linear(turn, 1.2, 1.4)?);
        line.wait_for(stage, "Three dimensions have three")?;
        for k in 1..3 {
            // the one spin's grid is gone before the next one comes: the two
            // cross-faded in place, a few frames of digits over digits (the
            // dense read of the scored draft)
            fade_out_all(stage, &shown_now, 0.25)?;
            stage.set_tracker_value(which, k as f64)?;
            stage.set_tracker_value(turn, 0.0)?;
            shown_now = vec![grids[k].group, names[k]];
            let mut next: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &shown_now {
                next.push(fade_in_anim(stage, m, 0.35)?);
            }
            next.push(stage.prepare(tween_linear(turn, 1.2, 1.4)?)?);
            stage.play_prepared(next)?;
        }
        kit.hold(stage)?;

        // ---- 15_2: your turn: tip about x, about y, undo each
        let line = say(&kit, stage, "15_2")?;
        let mut off = shown_now.clone();
        off.push(exact);
        fade_out_all(stage, &off, 0.5)?;
        stage.set_tracker_value(turn, 0.0)?;
        let card = QuizCard::build(&kit, stage, "15_2", QUIZ_AT, QUIZ_SIZE)?;
        let (middle, room, _) = card.content();
        let [moves, question, answer] = quiz_lines(&kit, stage, middle, room)?;
        line.wait_last(stage)?;
        card.show(stage, &[moves, question], 0.5)?;
        card.count(&kit, stage)?;
        card.land(stage, &[answer], 0.4)?;

        // ---- 15_3: the loop creeps round the third axis
        let line = say(&kit, stage, "15_3")?;
        let mut leave = card.mobs();
        leave.extend([moves, question, answer]);
        let mut cube_old = cube.mobs();
        cube_old.push(cube.anchor);
        stage.clear_updaters(cube.anchor, true);
        leave.extend(&cube_old);
        // the answer stays fully up until "No." has been said (the finished
        // read puts that at 0.82 s; the card went at once)
        line.wait_first_sentence(stage)?;
        fade_out_all(stage, &leave, 0.5)?;
        let tip = shown::CREEP_TIP;
        let loops = stage.add_value_tracker(0.0);
        let top_at = [-2.6, 0.2, 0.0];
        // the start, kept as a ghost, and the cube seen from along z
        let ghost = Polygon::new(
            top_face()
                .iter()
                .map(|&i| {
                    let p = corners()[i];
                    let s = project(TOP_YAW, TOP_TILT, p);
                    v(
                        top_at[0] + CUBE_SCALE * 1.15 * s[0],
                        top_at[1] + CUBE_SCALE * 1.15 * s[1],
                    )
                })
                .collect::<Vec<_>>(),
        )
        .style(
            Style::default()
                .fill(Palette::dim(), 0.0)
                .stroke(Palette::dim(), 3.0, 0.8),
        )
        .build();
        let ghost = stage.arena_mut().add(ghost);
        let creeping = Cube::build(stage, top_at, CUBE_SCALE * 1.15, (TOP_YAW, TOP_TILT), move |s| {
            creep_pose(tip, s.tracker_value(loops).unwrap_or(0.0))
        })?;
        let ghost_name = kit.text(
            stage,
            "where it began",
            26.0,
            Palette::dim(),
            v(top_at[0], top_at[1] - 2.3),
        )?;
        let mut ins = creeping.mobs();
        ins.extend([ghost, ghost_name]);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "Repeat that loop")?;
        let angle_label = kit.text(stage, "the cube has turned:", 30.0, Palette::dim(), v(3.5, 2.2))?;
        let angle = kit.live_value(
            stage,
            move |s| degrees(&creep_pose(tip, s.tracker_value(loops).unwrap_or(0.0))),
            1,
            false,
            56.0,
            gold,
            v(3.0, 1.3),
            LEFT,
        )?;
        let deg = kit.text(stage, "degrees", 30.0, Palette::dim(), v(5.7, 1.3))?;
        fade_in_all(stage, &[angle_label, deg], 0.4)?;
        let n_loops = f64::from(shown::CREEP_LOOPS);
        // (6.5 s while the card went at once; the card now waits 0.8 s for its
        // sentence and the beat keeps its length)
        play!(stage; tween_linear(loops, n_loops, 5.7)?);
        line.wait_for(stage, "The commutator of two spins is a third spin")?;
        let bracket = kit.tex(
            stage,
            r"[\,L_x,\,L_y\,]=L_z",
            52.0,
            ink,
            &[("L_x", teal), ("L_y", teal), ("L_z", teal)],
            v(3.4, -0.4),
        )?;
        let backwards = kit.text(
            stage,
            &format!("this loop runs it backwards ({})", fact("F-L.creep.direction")?),
            26.0,
            Palette::dim(),
            v(3.4, -1.3),
        )?;
        fade_in_all(stage, &[bracket, backwards], 0.6)?;
        kit.hold(stage)?;

        // ---- 15_4: a Lie algebra, a Lie group, and exp between them
        let line = say(&kit, stage, "15_4")?;
        let mut away = creeping.mobs();
        away.extend([ghost, ghost_name, angle_label, angle, deg, bracket, backwards]);
        stage.clear_updaters(creeping.anchor, true);
        fade_out_all(stage, &away, 0.5)?;
        // left: the three spins, to be added and bracketed, side by side with
        // a gap between neighbours' brackets (a pitch of 1.75 left them
        // touching, one long matrix)
        let algebra = algebra_row(&kit, stage, &grids3)?;
        let left_title = kit.text(stage, "spins: add them, bracket them", 28.0, ink, v(ALGEBRA_X, 2.5))?;
        let left_name = kit.text(stage, "a Lie algebra, so(3)", 30.0, teal, v(ALGEBRA_X, -1.1))?;
        // right: the cube, followed by one rotation after another
        let tour = stage.add_value_tracker(0.0);
        let group_cube = Cube::build(stage, [3.6, 0.3, 0.0], CUBE_SCALE, (VIEW_YAW, VIEW_TILT), move |s| {
            let u = s.tracker_value(tour).unwrap_or(0.0);
            expm_t(&spin(2), u).mul(&expm_t(&spin(0), 0.6 * u))
        })?;
        let right_title = kit.text(
            stage,
            "rotations: follow one by another, undo one",
            28.0,
            ink,
            v(3.6, 2.5),
        )?;
        let right_name = kit.text(stage, "a Lie group, SO(3)", 30.0, gold, v(3.6, -1.9))?;
        let from_to = arrow(stage, v(-0.9, 0.5), v(0.9, 0.5), gold, 5.0)?;
        let exp_name = kit.tex(stage, r"\exp", 48.0, gold, &[], v(0.0, 1.1))?;
        let mut first = algebra.clone();
        first.extend([left_title]);
        line.wait_for(stage, "Add two spins")?;
        fade_in_all(stage, &first, 0.6)?;
        line.wait_for(stage, "A set closed like that is a Lie algebra")?;
        fade_in_all(stage, &[left_name], 0.5)?;
        line.wait_for(stage, "Follow one rotation by another")?;
        let mut second = group_cube.mobs();
        second.push(right_title);
        fade_in_all(stage, &second, 0.6)?;
        play!(stage; tween_linear(tour, 2.4, 3.0)?);
        line.wait_for(stage, "That is a Lie group")?;
        fade_in_all(stage, &[right_name], 0.5)?;
        line.wait_for(stage, "The exponential takes each spin to a rotation")?;
        fade_in_all(stage, &[from_to, exp_name], 0.7)?;
        music_hold(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use explainer_kit::stats::Data;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{fact, matrix, quiz_log, testing};
    use super::*;
    use crate::model::{expm, hat, quarter_turn, rotation_defect};

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

    /// 15_1 (the dense read of J's first half): each spin's grid is gone
    /// before the next one comes, not cross-faded over it (two matrices'
    /// digits stood over one another for a few frames).
    #[test]
    fn one_spins_grid_is_gone_before_the_next_comes() {
        let voiced = testing::run_voiced("15_the_doorway").unwrap();
        let at = |key: &str, kind: &str| {
            quiz_log::take(&voiced.tag, key)
                .iter()
                .find(|(k, _)| *k == kind)
                .map(|e| e.1)
        };
        let keys = ["15_1:spin_x", "15_1:spin_y", "15_1:spin_z"];
        for pair in keys.windows(2) {
            let gone = at(pair[0], "down").unwrap_or_else(|| panic!("{} never goes", pair[0]));
            let came = at(pair[1], "up").unwrap_or_else(|| panic!("{} never comes", pair[1]));
            assert!(
                came - gone >= 0.2,
                "{} is up at {came:.2} s, {} is gone at {gone:.2} s",
                pair[1],
                pair[0]
            );
        }
    }

    /// 15_2 (the dense read of the scored draft): the card states the
    /// question the voice asks, not only the set-up; its three lines are in
    /// the card's content, stacked clear of one another, and the question
    /// is the line's own words.
    #[test]
    fn the_card_asks_the_question_the_voice_asks() {
        // the voice's last sentence of 15_2
        let said = crate::SCRIPT
            .lines()
            .find_map(|l| l.strip_prefix("15_2\t"))
            .expect("15_2 is in the script");
        assert!(said.ends_with("Are you back where you began?"), "{said}");
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, "15_2", QUIZ_AT, QUIZ_SIZE)?;
            let (middle, room, height) = card.content();
            let [moves, question, answer] = quiz_lines(&kit, stage, middle, room)?;
            let top = middle[1] + height / 2.0;
            assert!(
                stage.get_top(moves)[1] <= top + 0.01,
                "the moves poke out of the content"
            );
            assert!(stage.get_width(moves) <= room + 1e-6 && stage.get_width(question) <= room + 1e-6);
            assert!(
                stage.get_bottom(moves)[1] > stage.get_top(question)[1] + 0.1,
                "the moves {:.2} over the question {:.2}",
                stage.get_bottom(moves)[1],
                stage.get_top(question)[1]
            );
            assert!(
                stage.get_bottom(question)[1] > stage.get_top(answer)[1] + 0.1,
                "the question {:.2} over the answer {:.2}",
                stage.get_bottom(question)[1],
                stage.get_top(answer)[1]
            );
            // the answer stays in the card
            let card_bottom = QUIZ_AT[1] - QUIZ_SIZE.1 / 2.0;
            assert!(stage.get_bottom(answer)[1] > card_bottom + 0.2);
            Ok(())
        });
    }

    fn skew(m: &Mat) -> bool {
        m.add(&m.transpose()).max_abs() < 1e-12
    }

    fn random_spin(data: &mut Data) -> Mat {
        hat([
            data.uniform(-2.0, 2.0),
            data.uniform(-2.0, 2.0),
            data.uniform(-2.0, 2.0),
        ])
    }

    fn is_rotation(r: &Mat) -> bool {
        let (defect, det) = rotation_defect(r);
        defect < 1e-11 && (det - 1.0).abs() < 1e-11
    }

    /// 15_1: J flipped across its diagonal is minus J (every sign changes),
    /// the three spins are skew, their brackets are each other (the facts'),
    /// and following a spin gives a rotation.
    #[test]
    fn a_skew_matrix_is_a_spin_and_following_it_is_a_rotation() {
        let j = quarter_turn();
        assert!(j.transpose().add(&j).max_abs() == 0.0, "J flipped is minus J");
        assert!(skew(&j));
        let [lx, ly, lz] = spin_axes();
        for l in [&lx, &ly, &lz] {
            assert!(skew(l));
            for t in [0.0, 0.3, 1.2, 3.0] {
                assert!(is_rotation(&expm_t(l, t)), "t = {t}");
            }
        }
        // z-spin: the quarter turn with a fixed third axis; its rotation is the fact's
        let want = matrix("F-L.exp.Lz.t1").unwrap();
        let got = expm_t(&lz, 1.0);
        for (i, row) in want.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                assert!((got.get(i, j) - x).abs() < 5e-7);
            }
        }
        // each spin's bracket is the next one
        assert_eq!(lx.commutator(&ly), lz);
        assert_eq!(ly.commutator(&lz), lx);
        assert_eq!(lz.commutator(&lx), ly);
        assert_eq!(fact("F-L.bracket.xy_is_z").unwrap(), "true");
        // three dimensions have three, one for each pair of axes; four have six
        let pairs = |n: usize| (0..n).flat_map(|a| (a + 1..n).map(move |b| (a, b))).count();
        assert_eq!((pairs(3), pairs(4)), (3, 6));
        assert_eq!((spins(3), spins(4)), (3, 6));
        assert_eq!(fact("F-V.15_1.spins").unwrap(), "3,6");
        // a full turn and no turn are the same rotation
        assert!(expm_t(&lz, TAU).sub(&Mat::identity(3)).max_abs() < 1e-12);
        assert_eq!(fact("F-L.full_turn_is_identity").unwrap(), "true");
    }

    /// 15_4, the always sentences, at their edges: the sum of two spins and
    /// the commutator of two spins are spins; a rotation followed by another
    /// and the undoing of one are rotations; and the exponential reaches
    /// every rotation (the spin it comes from is found from its angle and
    /// axis, for angles under a half turn).
    #[test]
    fn spins_are_closed_rotations_are_closed_and_exp_reaches_them_all() {
        let mut data = Data::new(20_261_015);
        for _ in 0..200 {
            let (a, b) = (random_spin(&mut data), random_spin(&mut data));
            assert!(skew(&a.add(&b)), "a sum of spins is a spin");
            assert!(skew(&a.commutator(&b)), "a commutator of spins is a spin");
            let (ra, rb) = (expm(&a), expm(&b));
            assert!(is_rotation(&ra.mul(&rb)), "one rotation after another");
            assert!(is_rotation(&ra.transpose()), "undoing one");
            assert!(ra.transpose().mul(&ra).sub(&Mat::identity(3)).max_abs() < 1e-11);
            // the exponential reaches ra: take its angle and axis back to a spin
            let (angle, axis) = angle_axis(&ra);
            let back = hat([angle * axis[0], angle * axis[1], angle * axis[2]]);
            let need = a.entries().iter().fold(0.0_f64, |m, x| m.max(x.abs()));
            let length = (a.get(2, 1).powi(2) + a.get(0, 2).powi(2) + a.get(1, 0).powi(2)).sqrt();
            if length < std::f64::consts::PI - 0.05 && need > 0.0 {
                assert!(
                    expm(&back).sub(&ra).max_abs() < 1e-9,
                    "the log of a rotation exponentiates back"
                );
            }
        }
        // the edges: no spin at all gives no turn; a spin of length two pi gives none either
        assert!(expm(&Mat::zeros(3)).sub(&Mat::identity(3)).max_abs() < 1e-15);
        assert!(expm(&hat([0.0, 0.0, TAU])).sub(&Mat::identity(3)).max_abs() < 1e-12);
    }

    /// 15_3: the loop's pose at whole loops is the facts' loop to that power,
    /// it is continuous through the stages, one loop is 2.28 degrees, twenty
    /// are 45.7 degrees about minus z, and the live angle only climbs.
    #[test]
    fn the_cube_creeps_round_the_third_axis_backwards() {
        let tip = shown::CREEP_TIP;
        let mut power = Mat::identity(3);
        for k in 0..=shown::CREEP_LOOPS {
            assert!(
                creep_pose(tip, f64::from(k)).sub(&power).max_abs() < 1e-12,
                "after {k} loops"
            );
            power = cube_loop(tip).mul(&power);
        }
        let mut last = creep_pose(tip, 0.0);
        let mut angles = Vec::new();
        for i in 1..=(4 * 3) {
            let now = creep_pose(tip, f64::from(i) / 4.0);
            assert!(now.sub(&last).max_abs() < 0.25, "a stage at a time: step {i}");
            last = now;
        }
        for k in 0..=shown::CREEP_LOOPS {
            angles.push(degrees(&creep_pose(tip, f64::from(k))));
        }
        assert!(angles.windows(2).all(|w| w[1] >= w[0] - 1e-9), "{angles:?}");
        let end = creep_pose(tip, f64::from(shown::CREEP_LOOPS));
        let (angle, axis) = angle_axis(&end);
        assert_eq!(
            format!("{:.1}", angle.to_degrees()),
            fact("F-L.creep.total.degrees").unwrap()
        );
        assert!(axis[2] < -0.9, "backwards about z: {axis:?}");
        assert_eq!(fact("F-L.creep.direction").unwrap(), "backwards about z");
        assert_eq!(
            format!("{:.2}", degrees(&cube_loop(tip))),
            fact("F-L.creep.one.degrees").unwrap()
        );
        assert_eq!(fact("F-Q.15_2.answer").unwrap(), "no: off by 2.28 degrees");
        // for short times the loop leaves minus the z-spin
        assert_eq!(fact("F-L.loop.is_minus_z_spin").unwrap(), "true");
    }

    #[test]
    fn the_cube_is_eight_corners_twelve_edges_and_a_gold_top() {
        let c = corners();
        assert_eq!(c.len(), 8);
        let e = edges();
        assert_eq!(e.len(), 12);
        for (a, b) in e {
            let differ = (0..3).filter(|&k| c[a][k] != c[b][k]).count();
            assert_eq!(differ, 1, "an edge joins corners one coordinate apart");
        }
        assert!(top_face().iter().all(|&i| c[i][2] == 1.0));
        // from along z the cube is the square (x, y): the quarter-tilt view
        for p in c {
            let s = project(TOP_YAW, TOP_TILT, p);
            assert!((s[0] - p[0]).abs() < 1e-12 && (s[1] - p[1]).abs() < 1e-12);
        }
        // from the side view the vertical stays vertical
        let up = project(VIEW_YAW, VIEW_TILT, [0.0, 0.0, 1.0]);
        assert!(up[0].abs() < 1e-12 && up[1] > 0.0);
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("15_the_doorway").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "J^{T}=-J",
            "the z-spin: turns xy",
            "tip about x, then about y,",
            "the cube has turned:",
            "[\\,L_x,\\,L_y\\,]=L_z",
            "a Lie algebra, so(3)",
            "a Lie group, SO(3)",
            "\\exp",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
