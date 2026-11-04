//! Chapter 09, the centralizer of the horizontal flip s. Two rigid tiles run
//! "move g, then flip" against "flip, then move g" for every g, and a
//! scoreboard of rack thumbnails collects the verdicts. The reason is then
//! shown, not asserted: the flip's mirror line rides on a tile, the moves with
//! checks are exactly those that return the line, and undo-flip-redo (g s g⁻¹)
//! leaves every point of the carried line fixed, so it is the flip across the
//! carried line. The centralizer is gathered from the board and matched with
//! the subgroup the chapter 08 machine grew; the subgroup proof slides s
//! across a·b⁻¹; the center keeps all four mirror lines.
use crate::d4::apply;
use crate::d4_group::{self as d4g, ALL, E, Elem, R, R2, R2S, R3, R3S, RS, S};
use crate::tile_actor;
use crate::tile_model::{self, Matrix};
use explainer_kit::{Kit, Palette, Timed, ink_style, play, stroke_style, tween, v};
use fmn::prelude::*;

pub struct Centralizer {
    pub kit: Kit,
}

const CORNERS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

/// Return true precisely when the selected move commutes with horizontal reflection.
pub fn commutes_with_flip(turns: u8, flip: bool) -> bool {
    CORNERS
        .iter()
        .all(|&p| apply(apply(p, turns, flip), 0, true) == apply(apply(p, 0, true), turns, flip))
}

const LEFT_X: f64 = -3.4;
const RIGHT_X: f64 = 3.4;
const TILE_Y: f64 = 0.75;
const SCALE: f64 = 0.9;
const BOARD_Y: f64 = -1.7;
const MARK_Y: f64 = -2.95;
const THUMB_K: f64 = 0.8;

/// The order in which the two tiles test the moves: the quarter-turn and
/// the half-turn by narration, then the rest of the rack as a montage.
const ORDER: [Elem; 8] = [R, R2, E, R3, S, RS, R2S, R3S];

/// Tracker value t drives move `ORDER[t / 4]`: local 0→1 is the first
/// step, 1→2 the second, 2→4 rewinds both, so the next move starts from
/// the untouched tile with no jump.
fn compare_local(t: f64) -> (Elem, f64) {
    let t = t.clamp(0.0, 32.0);
    let k = ((t / 4.0).floor() as usize).min(ORDER.len() - 1);
    let local = t - 4.0 * k as f64;
    (ORDER[k], if local <= 2.0 { local } else { 4.0 - local })
}

fn start_of(g: Elem) -> f64 {
    4.0 * ORDER.iter().position(|&x| x == g).expect("every move is tested") as f64
}

fn move_then_flip(t: f64) -> Matrix {
    let (g, local) = compare_local(t);
    d4g::two_step(g, S, local)
}

fn flip_then_move(t: f64) -> Matrix {
    let (g, local) = compare_local(t);
    d4g::two_step(S, g, local)
}

/// The mirror-line demonstration: half-turn, quarter-turn, diagonal flip,
/// each performed (local 0→1) and then undone (1→2).
const CARRY: [Elem; 3] = [R2, R, RS];

fn carry_pose(t: f64) -> Matrix {
    let t = t.clamp(0.0, 6.0);
    let k = ((t / 2.0).floor() as usize).min(CARRY.len() - 1);
    let local = t - 2.0 * k as f64;
    d4g::motion(CARRY[k], if local <= 1.0 { local } else { 2.0 - local })
}

/// Undo the quarter-turn, flip, redo the quarter-turn: r s r⁻¹, with the
/// undo drawn as a quarter-turn backwards.
fn conjugate_by_r(t: f64) -> Matrix {
    let undo = t.clamp(0.0, 1.0);
    let flip = (t - 1.0).clamp(0.0, 1.0);
    let redo = (t - 2.0).clamp(0.0, 1.0);
    tile_model::compose(
        tile_model::turn(redo),
        tile_model::compose(tile_model::flip(flip), tile_model::turn(-undo)),
    )
}

/// The center demonstration: the half-turn, then the quarter-turn, each
/// performed and undone.
const CENTER_DEMO: [Elem; 2] = [R2, R];

fn center_pose(t: f64) -> Matrix {
    let t = t.clamp(0.0, 4.0);
    let k = ((t / 2.0).floor() as usize).min(CENTER_DEMO.len() - 1);
    let local = t - 2.0 * k as f64;
    d4g::motion(CENTER_DEMO[k], if local <= 1.0 { local } else { 2.0 - local })
}

fn board_at(g: Elem) -> (f64, f64) {
    (-5.25 + 1.5 * g.index() as f64, BOARD_Y)
}

/// Where corner `corner` of a tile drawn at `center` sits after `pose`.
fn corner_at(center: (f64, f64), scale: f64, pose: Matrix, corner: usize) -> (f64, f64) {
    let (x, y) = CORNERS[corner];
    let p = tile_model::apply(pose, [0.9 * f64::from(x), 0.9 * f64::from(y), 0.0]);
    (center.0 + scale * p[0], center.1 + scale * p[1])
}

fn verdict_mark(stage: &mut Stage<'_>, at: (f64, f64), agrees: bool) -> fmn::Result<Mob> {
    let (x, y) = at;
    let (color, strokes) = if agrees {
        (
            Palette::green(),
            [
                (v(x - 0.17, y + 0.01), v(x - 0.05, y - 0.12)),
                (v(x - 0.05, y - 0.12), v(x + 0.19, y + 0.16)),
            ],
        )
    } else {
        (
            Palette::gold(),
            [
                (v(x - 0.13, y - 0.13), v(x + 0.13, y + 0.13)),
                (v(x - 0.13, y + 0.13), v(x + 0.13, y - 0.13)),
            ],
        )
    };
    let mut parts = Vec::new();
    for (a, b) in strokes {
        parts.push(
            stage
                .arena_mut()
                .add(Line::new(a, b).style(stroke_style(color, 5.0)).build()?),
        );
    }
    d4g::group_of(stage, &parts)
}

fn ring(stage: &mut Stage<'_>, at: (f64, f64), color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(0.38)
            .style(stroke_style(color, 5.0))
            .build()
            .moved_to(v(at.0, at.1)),
    )
}

/// A dot fixed to the tile body at `body`, redrawn with the tile's pose.
fn riding_dot(
    stage: &mut Stage<'_>,
    progress: Mob,
    center: (f64, f64),
    scale: f64,
    body: (f64, f64),
    color: Srgb,
    pose: impl Fn(f64) -> Matrix + Copy + Send + Sync + 'static,
) -> fmn::Result<Mob> {
    let dot = stage.always_redraw(move |arena| {
        let matrix = pose(arena.tracker_value(progress).unwrap_or(0.0));
        let p = tile_model::project(tile_model::apply(matrix, [body.0, body.1, 0.0]), 6.0)
            .expect("dot stays in front of camera");
        arena.add(
            Circle::new()
                .radius(0.12)
                .style(ink_style(color))
                .build()
                .moved_to(v(center.0 + scale * p[0], center.1 + scale * p[1])),
        )
    });
    stage.add_to_scene(dot)?;
    Ok(dot)
}

/// A fixed, faint copy of a mirror line through `center` at `eighths`·45°.
fn reference_line(stage: &mut Stage<'_>, center: (f64, f64), scale: f64, eighths: u8, color: Srgb) -> fmn::Result<Mob> {
    let angle = std::f64::consts::FRAC_PI_4 * f64::from(eighths);
    let (dx, dy) = (1.45 * scale * angle.cos(), 1.45 * scale * angle.sin());
    let line = Line::new(v(center.0 - dx, center.1 - dy), v(center.0 + dx, center.1 + dy))
        .style(stroke_style(color, 3.0))
        .build()?;
    Ok(stage.arena_mut().add(line))
}

/// The row of eight rack thumbnails that collects verdicts.
struct Board {
    items: Vec<Mob>,
    thumbs: Vec<Mob>,
    marks: Vec<Mob>,
}

impl Board {
    fn new(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Self> {
        let mut items = Vec::new();
        let mut thumbs = Vec::new();
        for g in ALL {
            let (x, y) = board_at(g);
            let thumb = d4g::thumb(stage, kit, g, (x, y), THUMB_K)?;
            thumbs.push(thumb);
            items.push(thumb);
            items.push(kit.text(stage, g.name(), 24.0, Palette::ink(), v(x, y - 0.66))?);
        }
        Ok(Self {
            items,
            thumbs,
            marks: Vec::new(),
        })
    }

    /// Frame and mark `g` with the computed verdict.
    fn score(&mut self, stage: &mut Stage<'_>, g: Elem, run_time: f64) -> fmn::Result<()> {
        let agrees = d4g::commutes(g, S);
        // The group product and the corner-by-corner comparison must agree.
        assert_eq!(agrees, commutes_with_flip(g.turns, g.flip), "verdict for {}", g.name());
        let color = if agrees { Palette::green() } else { Palette::gold() };
        let (x, y) = board_at(g);
        let frame = d4g::slot_frame(stage, (x, y), THUMB_K, color)?;
        let mark = verdict_mark(stage, (x, MARK_Y), agrees)?;
        d4g::show(stage, &[frame, mark], run_time)?;
        self.marks.extend([frame, mark]);
        Ok(())
    }

    fn pulse(&self, stage: &mut Stage<'_>, moves: &[Elem]) -> fmn::Result<()> {
        let mut pulses: Vec<Box<dyn Animation>> = Vec::new();
        for &g in moves {
            let a = fmn::animation::indicate(stage.arena_mut(), self.thumbs[g.index()], 1.2, None)?.rt(0.8);
            pulses.push(stage.prepare(a)?);
        }
        let group = explainer_kit::lagged(stage, pulses, 0.2)?;
        play!(stage; group);
        Ok(())
    }
}

/// The pair of comparison tiles and their trackers.
struct Pair {
    left: Mob,
    right: Mob,
    parts: Vec<Mob>,
}

impl Pair {
    fn both(&self, stage: &mut Stage<'_>, value: f64, run_time: f64) -> fmn::Result<()> {
        play!(stage; tween(self.left, value, run_time)?, tween(self.right, value, run_time)?);
        Ok(())
    }
}

fn move_label(stage: &mut Stage<'_>, kit: &Kit, g: Elem) -> fmn::Result<Mob> {
    kit.tex(
        stage,
        &format!("g={}", g.tex()),
        40.0,
        Palette::ink(),
        &[],
        v(0.0, TILE_Y),
    )
}

fn verdict_text(stage: &mut Stage<'_>, kit: &Kit, g: Elem) -> fmn::Result<Mob> {
    let (text, color) = if d4g::commutes(g, S) {
        ("Same result", Palette::green())
    } else {
        ("Different results", Palette::gold())
    };
    kit.text(stage, text, 29.0, color, v(0.0, -0.55))
}

impl SceneConstruct for Centralizer {
    fn name(&self) -> &str {
        "09_centralizer"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 09_1: the two orders, side by side; prediction hold.
        kit.say(stage, "09_1")?;
        let title = kit.text(
            stage,
            "Which moves commute with the flip?",
            34.0,
            Palette::ink(),
            v(0.0, 3.35),
        )?;
        let l = kit.text(stage, "Move g, then flip s", 27.0, Palette::blue(), v(LEFT_X, 2.35))?;
        let r = kit.text(stage, "Flip s, then move g", 27.0, Palette::hero(), v(RIGHT_X, 2.35))?;
        d4g::show(stage, &[title], 0.4)?;
        let pair = {
            let left = stage.add_value_tracker(0.0);
            let right = stage.add_value_tracker(0.0);
            let mut parts = tile_actor::add_tile(stage, &kit, left, LEFT_X, TILE_Y, SCALE, move_then_flip)?;
            parts.extend(tile_actor::add_tile(
                stage,
                &kit,
                right,
                RIGHT_X,
                TILE_Y,
                SCALE,
                flip_then_move,
            )?);
            Pair { left, right, parts }
        };
        kit.until_sentence(stage, "09_1", 1)?;
        d4g::show(stage, &[l, r], 0.4)?;
        kit.until_sentence(stage, "09_1", 2)?;
        let mut board = Board::new(stage, &kit)?;
        d4g::show(stage, &board.items, 0.6)?;
        kit.hold(stage)?;
        stage.wait(2.5)?;

        // 09_2: g = r. Left tile first, then the right tile; the blue corner.
        kit.say(stage, "09_2")?;
        let mut label = move_label(stage, &kit, R)?;
        d4g::show(stage, &[label], 0.4)?;
        kit.until_sentence(stage, "09_2", 1)?;
        play!(stage; tween(pair.left, 1.0, 1.0)?);
        play!(stage; tween(pair.left, 2.0, 1.2)?);
        kit.until_sentence(stage, "09_2", 2)?;
        play!(stage; tween(pair.right, 1.0, 1.2)?);
        play!(stage; tween(pair.right, 2.0, 1.0)?);
        kit.until_sentence(stage, "09_2", 3)?;
        let rings = [
            ring(
                stage,
                corner_at((LEFT_X, TILE_Y), SCALE, move_then_flip(2.0), 0),
                Palette::yellow(),
            ),
            ring(
                stage,
                corner_at((RIGHT_X, TILE_Y), SCALE, flip_then_move(2.0), 0),
                Palette::yellow(),
            ),
        ];
        let mut rings_in: Vec<Box<dyn Animation>> = Vec::new();
        for m in rings {
            rings_in.push(stage.prepare(show_creation(m).rt(0.6))?);
        }
        stage.play_prepared(rings_in)?;
        let mut verdict = verdict_text(stage, &kit, R)?;
        d4g::show(stage, &[verdict], 0.4)?;
        kit.until_sentence(stage, "09_2", 4)?;
        board.score(stage, R, 0.5)?;
        d4g::hide(stage, &rings, 0.3)?;

        // 09_3: g = r², both tiles together.
        kit.say(stage, "09_3")?;
        let base = start_of(R2);
        pair.both(stage, base, 0.8)?; // rewind r
        let next = move_label(stage, &kit, R2)?;
        play!(stage; fmn::animation::replacement_transform(label, next).rt(0.5));
        label = next;
        d4g::hide(stage, &[verdict], 0.3)?;
        kit.until_sentence(stage, "09_3", 1)?;
        pair.both(stage, base + 1.0, 1.2)?;
        pair.both(stage, base + 2.0, 1.0)?;
        kit.until_sentence(stage, "09_3", 2)?;
        let mut corner_rings = Vec::new();
        for corner in 0..4 {
            let color = d4g::corner_color(corner);
            corner_rings.push(ring(
                stage,
                corner_at((LEFT_X, TILE_Y), SCALE, move_then_flip(base + 2.0), corner),
                color,
            ));
            corner_rings.push(ring(
                stage,
                corner_at((RIGHT_X, TILE_Y), SCALE, flip_then_move(base + 2.0), corner),
                color,
            ));
        }
        d4g::show(stage, &corner_rings, 0.5)?;
        verdict = verdict_text(stage, &kit, R2)?;
        d4g::show(stage, &[verdict], 0.4)?;
        kit.until_sentence(stage, "09_3", 3)?;
        board.score(stage, R2, 0.5)?;
        d4g::hide(stage, &corner_rings, 0.3)?;

        // 09_4: the scoreboard handles the rest, at montage speed.
        kit.say(stage, "09_4")?;
        d4g::hide(stage, &[verdict], 0.25)?;
        pair.both(stage, base + 4.0, 0.6)?;
        kit.until_sentence(stage, "09_4", 1)?;
        for &g in &ORDER[2..] {
            let start = start_of(g);
            let next = move_label(stage, &kit, g)?;
            play!(stage; fmn::animation::replacement_transform(label, next).rt(0.25));
            label = next;
            pair.both(stage, start + 1.0, 0.45)?;
            pair.both(stage, start + 2.0, 0.45)?;
            board.score(stage, g, 0.25)?;
            pair.both(stage, start + 4.0, 0.4)?;
        }

        // 09_5: read the board.
        kit.say(stage, "09_5")?;
        let checks: Vec<Elem> = ALL.into_iter().filter(|&g| d4g::commutes(g, S)).collect();
        let crosses: Vec<Elem> = ALL.into_iter().filter(|&g| !d4g::commutes(g, S)).collect();
        kit.until_sentence(stage, "09_5", 1)?;
        board.pulse(stage, &checks)?;
        kit.until_sentence(stage, "09_5", 3)?;
        board.pulse(stage, &crosses)?;
        kit.until_sentence(stage, "09_5", 4)?;
        let mut retire = pair.parts.clone();
        retire.extend([l, r, label]);
        d4g::hide(stage, &retire, 0.5)?;
        kit.hold(stage)?;
        stage.wait(1.5)?;

        // 09_9: the mirror line rides on the tile.
        kit.say(stage, "09_9")?;
        let demo_at = (-2.6, 0.8);
        let carry = stage.add_value_tracker(0.0);
        let mirror = reference_line(stage, demo_at, SCALE, 0, Palette::faint())?;
        d4g::show(stage, &[mirror], 0.3)?;
        let demo = tile_actor::add_tile(stage, &kit, carry, demo_at.0, demo_at.1, SCALE, carry_pose)?;
        let carried = d4g::carried_line(
            stage,
            carry,
            demo_at.0,
            demo_at.1,
            SCALE,
            0,
            Palette::hero(),
            carry_pose,
        )?;
        let mirror_name = kit.text(stage, "Mirror line of s", 26.0, Palette::hero(), v(2.6, 2.1))?;
        d4g::show(stage, &[mirror_name], 0.4)?;
        let notes = [
            ("Half-turn: lands on itself", Palette::green()),
            ("Quarter-turn: stands upright", Palette::gold()),
            ("Diagonal flip: stands upright", Palette::gold()),
        ];
        let mut note_mobs = Vec::new();
        for (step, (sentence, (text, color))) in [2_usize, 3, 4].into_iter().zip(notes).enumerate() {
            kit.until_sentence(stage, "09_9", sentence)?;
            let k = step as f64;
            play!(stage; tween(carry, 2.0 * k + 1.0, 1.4)?);
            let note = kit.text(stage, text, 26.0, color, v(2.6, 1.2 - 0.7 * k))?;
            d4g::show(stage, &[note], 0.3)?;
            note_mobs.push(note);
            stage.wait(0.5)?;
            play!(stage; tween(carry, 2.0 * k + 2.0, 0.8)?);
        }
        kit.until_sentence(stage, "09_9", 5)?;
        let mut board_lines = Vec::new();
        for g in ALL {
            let (x, y) = board_at(g);
            let eighths = g.carry_line(0);
            let color = if eighths == 0 {
                Palette::green()
            } else {
                Palette::gold()
            };
            board_lines.push(reference_line(stage, (x, y), 0.32, eighths, color)?);
        }
        d4g::show(stage, &board_lines, 0.6)?;

        // 09_10: undo the move, flip, redo it — the flip across the carried line.
        kit.say(stage, "09_10")?;
        let mut old = demo;
        old.extend([carried, mirror, mirror_name]);
        old.extend(note_mobs);
        d4g::hide(stage, &old, 0.4)?;
        let conj = stage.add_value_tracker(0.0);
        let horizontal = reference_line(stage, demo_at, SCALE, 0, Palette::faint())?;
        let carried_line = reference_line(stage, demo_at, SCALE, R.carry_line(0), Palette::hero())?;
        d4g::show(stage, &[horizontal, carried_line], 0.3)?;
        let conj_tile = tile_actor::add_tile(stage, &kit, conj, demo_at.0, demo_at.1, SCALE, conjugate_by_r)?;
        let riders = [
            riding_dot(
                stage,
                conj,
                demo_at,
                SCALE,
                (0.0, 0.62),
                Palette::hero(),
                conjugate_by_r,
            )?,
            riding_dot(
                stage,
                conj,
                demo_at,
                SCALE,
                (0.0, -0.62),
                Palette::hero(),
                conjugate_by_r,
            )?,
        ];
        let carried_name = kit.text(
            stage,
            "Carried line: where r puts the mirror",
            26.0,
            Palette::hero(),
            v(2.6, 2.1),
        )?;
        d4g::show(stage, &[carried_name], 0.4)?;
        kit.until_sentence(stage, "09_10", 1)?;
        let steps = [r"\text{undo }r", r"\text{flip }s", r"\text{redo }r"];
        let mut step_mobs = Vec::new();
        for (k, step) in steps.into_iter().enumerate() {
            let m = kit.tex(stage, step, 32.0, Palette::ink(), &[], v(2.6, 1.25 - 0.65 * k as f64))?;
            d4g::show(stage, &[m], 0.25)?;
            step_mobs.push(m);
            play!(stage; tween(conj, k as f64 + 1.0, 1.1)?);
        }
        kit.until_sentence(stage, "09_10", 2)?;
        // The riders are back exactly where they started: ring their places.
        let homes = [0.62, -0.62].map(|y| ring(stage, (demo_at.0, demo_at.1 + SCALE * y), Palette::hero()));
        let mut ring_in: Vec<Box<dyn Animation>> = Vec::new();
        for home in homes {
            ring_in.push(stage.prepare(show_creation(home).rt(0.6))?);
        }
        stage.play_prepared(ring_in)?;
        let outcome = kit.tex(stage, r"rsr^{-1}=r^2s", 34.0, Palette::hero(), &[], v(2.6, -0.75))?;
        d4g::show(stage, &[outcome], 0.4)?;
        kit.until_sentence(stage, "09_10", 3)?;
        let back = kit.tex(
            stage,
            r"gsg^{-1}=s\iff gs=sg",
            32.0,
            Palette::green(),
            &[],
            v(-2.6, -0.75),
        )?;
        d4g::show(stage, &[back], 0.4)?;
        kit.until_sentence(stage, "09_10", 4)?;
        let moved = kit.tex(stage, r"r^2s\neq s", 32.0, Palette::gold(), &[], v(2.6, -0.75))?;
        play!(stage; fmn::animation::replacement_transform(outcome, moved).rt(0.6));

        // 09_6: gather the checks; the same subgroup the machine grew.
        kit.say(stage, "09_6")?;
        let mut old = conj_tile;
        old.extend(riders);
        old.extend(homes);
        old.extend([horizontal, carried_line, carried_name, back, moved]);
        old.extend(step_mobs);
        d4g::hide(stage, &old, 0.4)?;
        let centralizer = d4g::centralizer(S);
        let mut gathered = Vec::new();
        let mut targets = Vec::new();
        for (k, &g) in centralizer.iter().enumerate() {
            let copy = d4g::thumb(stage, &kit, g, board_at(g), THUMB_K)?;
            gathered.push(copy);
            targets.push((copy, (-2.4 + 1.6 * k as f64, 1.0)));
        }
        d4g::show(stage, &gathered, 0.2)?;
        d4g::glide_all(stage, &targets, 1.1)?;
        let members: Vec<&str> = centralizer.iter().map(|g| g.name()).collect();
        let name = kit.text(
            stage,
            &format!("Centralizer of s: {}", members.join(", ")),
            30.0,
            Palette::green(),
            v(0.0, 2.25),
        )?;
        d4g::show(stage, &[name], 0.4)?;
        kit.until_sentence(stage, "09_6", 3)?;
        let grown = kit.text(
            stage,
            "The closure machine grew it from s and r²",
            28.0,
            Palette::blue(),
            v(0.0, -0.25),
        )?;
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(6.6)
                .height(1.5)
                .corner_radius(0.2)
                .style(stroke_style(Palette::blue(), 4.0))
                .build()?
                .moved_to(v(0.0, 1.0)),
        );
        d4g::show(stage, &[grown], 0.4)?;
        play!(stage; show_creation(frame).rt(0.8));

        // 09_7: any group. Slide s across a·b⁻¹.
        kit.say(stage, "09_7")?;
        let mut old = gathered;
        old.extend([name, grown, frame]);
        old.extend(board.items.iter().copied());
        old.extend(board.marks.iter().copied());
        old.extend(board_lines);
        d4g::hide(stage, &old, 0.5)?;
        let any_group = kit.text(stage, "In any group", 30.0, Palette::ink(), v(0.0, 2.45))?;
        d4g::show(stage, &[any_group], 0.3)?;
        let nonempty = kit.tex(
            stage,
            r"es=s=se\quad\Rightarrow\quad e\in C_G(s)",
            32.0,
            Palette::green(),
            &[],
            v(0.0, 1.6),
        )?;
        d4g::show(stage, &[nonempty], 0.4)?;
        kit.until_sentence(stage, "09_7", 2)?;
        d4g::hide(stage, &[nonempty], 0.3)?;
        let given = kit.tex(stage, r"as=sa,\qquad bs=sb", 34.0, Palette::ink(), &[], v(0.0, 1.6))?;
        d4g::show(stage, &[given], 0.4)?;
        kit.until_sentence(stage, "09_7", 3)?;
        let multiply = kit.tex(
            stage,
            r"b^{-1}(bs)b^{-1}=b^{-1}(sb)b^{-1}",
            34.0,
            Palette::blue(),
            &[],
            v(0.0, 0.85),
        )?;
        d4g::show(stage, &[multiply], 0.4)?;
        stage.wait(1.3)?;
        d4g::hide(stage, &[multiply], 0.3)?;
        let cancel = kit.tex(
            stage,
            r"(b^{-1}b)sb^{-1}=b^{-1}s(bb^{-1})",
            34.0,
            Palette::blue(),
            &[],
            v(0.0, 0.85),
        )?;
        d4g::show(stage, &[cancel], 0.4)?;
        stage.wait(1.3)?;
        d4g::hide(stage, &[cancel], 0.3)?;
        let inverse = kit.tex(
            stage,
            r"bs=sb\Rightarrow b^{-1}s=sb^{-1}",
            34.0,
            Palette::ink(),
            &[],
            v(0.0, 0.85),
        )?;
        d4g::show(stage, &[inverse], 0.4)?;
        kit.until_sentence(stage, "09_7", 4)?;
        let row_y = -0.35;
        let a = kit.tex(stage, "a", 52.0, Palette::blue(), &[], v(-1.3, row_y))?;
        let b_inv = kit.tex(stage, r"b^{-1}", 52.0, Palette::blue(), &[], v(0.0, row_y))?;
        let s = kit.tex(stage, "s", 52.0, Palette::hero(), &[], v(1.3, row_y))?;
        d4g::show(stage, &[a, b_inv, s], 0.4)?;
        kit.until_sentence(stage, "09_7", 5)?;
        // s hops over b⁻¹ (b⁻¹ commutes with s), then over a (a commutes with s).
        for (k, passed) in [b_inv, a].into_iter().enumerate() {
            let from_x = 1.3 - 1.3 * k as f64;
            let to_x = from_x - 1.3;
            d4g::glide(stage, s, (from_x, row_y + 0.75), 0.35)?;
            d4g::glide_all(stage, &[(s, (to_x, row_y + 0.75)), (passed, (from_x, row_y))], 0.6)?;
            d4g::glide(stage, s, (to_x, row_y), 0.35)?;
        }
        kit.until_sentence(stage, "09_7", 6)?;
        let chain = kit.tex(
            stage,
            r"(ab^{-1})s=a(b^{-1}s)=a(sb^{-1})=(as)b^{-1}=s(ab^{-1})",
            32.0,
            Palette::green(),
            &[],
            v(0.0, -1.55),
        )?;
        d4g::show(stage, &[chain], 0.5)?;
        let passes = kit.text(
            stage,
            "One-step test passed: the centralizer is a subgroup",
            27.0,
            Palette::green(),
            v(0.0, -2.5),
        )?;
        d4g::show(stage, &[passes], 0.4)?;

        // 09_8: the center keeps all four mirror lines.
        kit.say(stage, "09_8")?;
        kit.until_sentence(stage, "09_8", 2)?;
        d4g::hide(stage, &[any_group, given, inverse, a, b_inv, s, chain, passes], 0.5)?;
        let center_title = kit.text(
            stage,
            "Center: commutes with every move",
            30.0,
            Palette::ink(),
            v(0.0, 2.45),
        )?;
        d4g::show(stage, &[center_title], 0.4)?;
        kit.until_sentence(stage, "09_8", 3)?;
        let spin = stage.add_value_tracker(0.0);
        let center_at = (-2.6, 0.3);
        let line_colors = [Palette::hero(), Palette::teal(), Palette::purple(), Palette::yellow()];
        let mut fixed = Vec::new();
        for eighths in 0_u8..4 {
            fixed.push(reference_line(stage, center_at, SCALE, eighths, Palette::faint())?);
        }
        d4g::show(stage, &fixed, 0.3)?;
        let mut center_parts = tile_actor::add_tile(stage, &kit, spin, center_at.0, center_at.1, SCALE, center_pose)?;
        // Keep the opaque corner markers and their digits above the mirror lines.
        // The face remains below the lines, so the four axes stay visible inside.
        let corner_parts = center_parts[1..].to_vec();
        for (eighths, &color) in (0_u8..4).zip(&line_colors) {
            center_parts.push(d4g::carried_line(
                stage,
                spin,
                center_at.0,
                center_at.1,
                SCALE,
                eighths,
                color,
                center_pose,
            )?);
        }
        for &part in &corner_parts {
            stage.arena_mut().set_z_index(part, 1, true);
            stage.arena_mut().bring_to_front(part)?;
        }
        kit.until_sentence(stage, "09_8", 4)?;
        play!(stage; tween(spin, 1.0, 1.4)?);
        let keeps = kit.text(
            stage,
            "Half-turn: all four lines return",
            26.0,
            Palette::green(),
            v(2.8, 0.9),
        )?;
        d4g::show(stage, &[keeps], 0.3)?;
        play!(stage; tween(spin, 2.0, 0.8)?);
        play!(stage; tween(spin, 3.0, 1.2)?);
        let trades = kit.text(
            stage,
            "Quarter-turn: lines trade places",
            26.0,
            Palette::gold(),
            v(2.8, 0.2),
        )?;
        d4g::show(stage, &[trades], 0.3)?;
        play!(stage; tween(spin, 4.0, 0.8)?);
        kit.until_sentence(stage, "09_8", 5)?;
        let center: Vec<&str> = d4g::center().iter().map(|g| g.tex()).collect();
        let shared = kit.tex(
            stage,
            &format!(r"Z=\{{{}\}}", center.join(",")),
            36.0,
            Palette::green(),
            &[],
            v(2.8, -0.7),
        )?;
        d4g::show(stage, &[shared], 0.4)?;
        let mut wipe = center_parts;
        wipe.extend(fixed);
        wipe.extend([keeps, trades, shared, center_title, title]);
        kit.hold(stage)?;
        d4g::hide(stage, &wipe, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(a: Matrix, b: Matrix) -> bool {
        (0..3).all(|i| (0..3).all(|j| (a[i][j] - b[i][j]).abs() < 1e-9))
    }

    #[test]
    fn tiles_end_in_the_two_orders_and_agree_exactly_for_the_checked_moves() {
        for g in ORDER {
            let end = start_of(g) + 2.0;
            assert!(same(move_then_flip(end), S.mul(g).matrix()));
            assert!(same(flip_then_move(end), g.mul(S).matrix()));
            assert_eq!(same(move_then_flip(end), flip_then_move(end)), d4g::commutes(g, S));
            assert_eq!(d4g::commutes(g, S), commutes_with_flip(g.turns, g.flip));
            // Each rewind returns both tiles to the untouched tile.
            assert!(same(move_then_flip(start_of(g) + 4.0), tile_model::IDENTITY));
        }
    }

    #[test]
    fn quarter_turn_moves_the_blue_corner_as_chapter_05_showed() {
        let end = start_of(R) + 2.0;
        let left = tile_model::apply(move_then_flip(end), [1.0, 1.0, 0.0]);
        let right = tile_model::apply(flip_then_move(end), [1.0, 1.0, 0.0]);
        assert!((left[0] + 1.0).abs() < 1e-9 && (left[1] + 1.0).abs() < 1e-9);
        assert!((right[0] - 1.0).abs() < 1e-9 && (right[1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn checks_are_exactly_the_moves_that_return_the_mirror_line() {
        for g in ALL {
            assert_eq!(d4g::commutes(g, S), g.carry_line(0) == 0, "{}", g.name());
        }
        // Narrated carry demonstrations: half-turn lands, quarter-turn and
        // diagonal flip stand the line upright.
        assert_eq!(R2.carry_line(0), 0);
        assert_eq!(R.carry_line(0), 2);
        assert_eq!(RS.carry_line(0), 2);
        for (k, carried) in CARRY.iter().enumerate() {
            assert!(same(carry_pose(2.0 * k as f64 + 1.0), carried.matrix()));
            assert!(same(carry_pose(2.0 * k as f64 + 2.0), tile_model::IDENTITY));
        }
    }

    #[test]
    fn undo_flip_redo_is_the_flip_across_the_carried_line_and_fixes_it() {
        let end = conjugate_by_r(3.0);
        assert!(same(end, R.mul(S).mul(R.inv()).matrix()));
        assert_eq!(R.mul(S).mul(R.inv()), R2S);
        assert_eq!(R2S.mirror(), Some(R.carry_line(0)));
        for y in [0.62, -0.62] {
            let p = tile_model::apply(end, [0.0, y, 0.0]);
            assert!(p[0].abs() < 1e-9 && (p[1] - y).abs() < 1e-9);
        }
        for g in ALL {
            let conj = g.mul(S).mul(g.inv());
            assert_eq!(conj.mirror(), Some(g.carry_line(0)));
            assert_eq!(conj == S, d4g::commutes(g, S));
        }
    }

    #[test]
    fn centralizer_equals_the_subgroup_grown_from_s_and_the_half_turn() {
        let mut grown = d4g::closure_rounds(&[S, R2]).concat();
        grown.sort();
        let mut centralizer = d4g::centralizer(S);
        centralizer.sort();
        assert_eq!(centralizer, grown);
        assert_eq!(d4g::centralizer(S), vec![E, R2, S, R2S]);
    }

    #[test]
    fn center_is_what_keeps_all_four_mirror_lines_and_what_all_centralizers_share() {
        let keeps_all: Vec<Elem> = ALL
            .into_iter()
            .filter(|g| (0..4).all(|k| g.carry_line(k) == k))
            .collect();
        assert_eq!(keeps_all, d4g::center());
        let shared: Vec<Elem> = ALL
            .into_iter()
            .filter(|&g| ALL.iter().all(|&x| d4g::centralizer(x).contains(&g)))
            .collect();
        assert_eq!(shared, vec![E, R2]);
        assert!(same(center_pose(1.0), R2.matrix()) && same(center_pose(3.0), R.matrix()));
    }

    #[test]
    fn sliding_proof_holds_for_every_pair_in_every_centralizer() {
        for x in ALL {
            let c = d4g::centralizer(x);
            for &a in &c {
                for &b in &c {
                    assert!(d4g::commutes(b.inv(), x));
                    assert!(c.contains(&a.mul(b.inv())));
                }
            }
        }
    }
}
