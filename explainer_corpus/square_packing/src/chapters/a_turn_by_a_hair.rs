//! Chapter 03, a turn by a hair: what holds the turned tile, once the pins
//! have missed it (03_1). With the box, the four corner tiles and the
//! middle tile's centre held, the corners it touches lie on the circle
//! inside it (03_2); a question and four seconds to think (03_3); any turn
//! moves the sides off that circle's touching points, so every corner sinks
//! in (03_4), seen in a lens 44 times larger with the same geometry, and a
//! turn twice as far sinks almost four times deeper (03_5): the bite is
//! ½(1 − cos turn), the square of the turn, tiny but never zero (03_6). In
//! the box of three the corners sat on the outer circle instead and the turn
//! was free; held here it is blocked either way (03_7). That is one motion
//! of one arrangement, not a proof about every arrangement (03_8).
//!
//! Every frame is `model::hair` at the tracker's turn: the board at true
//! scale (where the bite is under a pixel, drawn in red all the same) and
//! the lens, which clips the same tiles to a window `facts::shown::LENS_HALF`
//! of a tile around the lower right corner's held point and draws it
//! `facts::shown::LENS_MAG` times the board's scale, with a scale bar. The
//! readouts are computed from the model each frame. Layers keep the red over
//! the tiles and the marks over both (chapter 02's lesson).

use explainer_kit::{Kit, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    arc_points, common, corner, dim, dot, fact, fact_int, fade_to, fail, faint, flat, ink, line_style, path, polygon,
    redot, reshape, retrace, ring, say, stand, swap, together, touch, turned, warn,
};
use super::the_turned_tile::{BOARD, DEGREE, PRECISELY, RING_AT, RING_R, THINK_S};
use crate::facts::shown;
use crate::model::{self, MIDDLE, Point, Surd};

pub struct ATurnByAHair {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6", "03_7", "03_8"];

// the chapter's fixed words
pub const HELD: &str = "held fixed:";
pub const HELD_WHAT: &str = "the box, the corner tiles, the centre";
pub const HALF: &str = r"\tfrac{1}{2}";
pub const TURN: &str = "turn";
pub const BITE: &str = "bite";
pub const TILE: &str = "tile";
pub const TIMES: &str = "×";
pub const DEEPER: &str = "deeper";
pub const FORMULA: &str = r"\text{bite} = \tfrac{1}{2}\,(1 - \cos\,\text{turn}) \approx \tfrac{1}{4}\,\text{turn}^2";
pub const RADIANS: &str = "turn in radians, about the held centre";
pub const BOX_OF: &str = "box of";
pub const OUTER: &str = "corners on the outer circle";
pub const BLOCKED: &str = "blocked: this one turn, all else held";
pub const SMALLER: &str = "smaller box? a proof about all arrangements";

// ---------------------------------------------------------------- layout

/// The left edge of the panel's text.
pub const TEXT_X: f64 = 1.7;
/// The lens's centre (scene units).
pub const LENS_AT: Vec3 = [3.6, -0.45, 0.0];
/// The small box of three in 03_7.
pub const MINI_AT: Vec3 = [2.7, 2.6, 0.0];
pub const MINI_K: f64 = 0.42;
/// Radii of the held corners' dots and of the dots on the sides' middles.
pub const HELD_R: f64 = 0.06;
pub const TOUCH_R: f64 = 0.045;
/// Draw layers (z index; the tiles and the lens's fills are at 0).
pub const LAYER_OVERLAPS: i32 = 1;
pub const LAYER_MARKS: i32 = 2;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    stop: f64,
    /// The exact small turn and twice it, in radians.
    one: f64,
    two: f64,
    mag: u32,
    half: f64,
    bar: f64,
    bar_label: String,
    ratio_2dp: String,
    three: u32,
    reach: f64,
}

fn reads() -> fmn::Result<Reads> {
    let turn = model::small_turn();
    if fact("hair.turn")? != format!("{},{}", turn.0, turn.1) {
        return Err(fail("hair.turn is not the model's small turn".into()));
    }
    let angle = |t: Point<Surd>| t.1.to_f64().atan2(t.0.to_f64());
    let (one, two) = (angle(turn), angle(model::compose(turn, turn)));
    for (key, value) in [("hair.degrees.1dp", one), ("hair.degrees2.1dp", two)] {
        if fact(key)? != format!("{:.1}", value.to_degrees()) {
            return Err(fail(format!("{key} is not the drawn turn")));
        }
    }
    for (key, want) in [
        ("hair.inner2", "1/4"),
        ("hair.inradius2", "1/4"),
        ("hair.check", "overlap:0-4,1-4,2-4,3-4"),
        ("hair.check.minus", "overlap:0-4,1-4,2-4,3-4"),
        ("hair.roomy", "free|free"),
        ("hair.lens.fits", "yes"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    let mag = fact_int("hair.lens.mag")?;
    if mag != shown::LENS_MAG {
        return Err(fail("hair.lens.mag is not the lens drawn".into()));
    }
    let bar = Surd::ratio(shown::LENS_BAR.0, shown::LENS_BAR.1);
    Ok(Reads {
        stop: model::turned_side().to_f64(),
        one,
        two,
        mag,
        half: Surd::ratio(shown::LENS_HALF.0, shown::LENS_HALF.1).to_f64(),
        bar: bar.to_f64(),
        bar_label: bar.decimals(2),
        ratio_2dp: fact("hair.ratio.2dp")?,
        three: fact_int("grid.side")?,
        reach: model::turn_reach2().to_f64().sqrt(),
    })
}

/// The lens: the window `half` a tile around `held`, drawn `scale` scene
/// units a tile (the board's scale times the magnification) about
/// [`LENS_AT`].
#[derive(Clone, Copy)]
struct Lens {
    held: Point<f64>,
    half: f64,
    scale: f64,
}

impl Lens {
    fn at(&self, q: Point<f64>) -> Vec3 {
        v(
            LENS_AT[0] + self.scale * (q.0 - self.held.0),
            LENS_AT[1] + self.scale * (q.1 - self.held.1),
        )
    }

    /// The window, counter-clockwise (tiles).
    fn window(&self) -> Vec<Point<f64>> {
        let ((x, y), w) = (self.held, self.half);
        vec![(x - w, y - w), (x + w, y - w), (x + w, y + w), (x - w, y + w)]
    }

    /// The lens's half-width on screen.
    fn radius(&self) -> f64 {
        self.half * self.scale
    }

    /// A clipped region on screen, or nothing.
    fn region(&self, pts: &[Point<f64>]) -> Vec<Vec3> {
        if pts.len() >= 3 {
            pts.iter().map(|&q| self.at(q)).collect()
        } else {
            vec![LENS_AT; 3]
        }
    }
}

/// The parts every frame redraws.
#[derive(Clone, Copy)]
struct Parts {
    tiles: [Mob; 5],
    patches: [Mob; 4],
    touches: [Mob; 4],
    /// The lens's corner tile, middle tile and their overlap.
    lens: [Mob; 3],
}

/// Redraw a frame: the middle tile turned `delta` radians past contact.
fn draw(s: &mut fmn::mobject::Stage, parts: &Parts, lens: &Lens, stop: f64, delta: f64) {
    let p = model::hair(delta);
    for (m, t) in parts.tiles.iter().zip(&p.tiles) {
        reshape(s, *m, BOARD.tile(stop, t));
    }
    let middle = p.tiles[MIDDLE].corners();
    for (k, &m) in parts.patches.iter().enumerate() {
        let bite = common(&p.tiles[k].corners(), &middle);
        let pts = if bite.len() >= 3 {
            bite.iter().map(|&q| BOARD.p(stop, q)).collect()
        } else {
            vec![BOARD.at; 3]
        };
        reshape(s, m, pts);
    }
    for (m, q) in parts.touches.iter().zip(p.tiles[MIDDLE].midpoints()) {
        redot(s, *m, BOARD.p(stop, q), TOUCH_R);
    }
    let window = lens.window();
    let blue = common(&window, &p.tiles[shown::LENS_CORNER].corners());
    let gold = common(&window, &middle);
    let red = common(&blue, &middle);
    reshape(s, parts.lens[0], lens.region(&blue));
    reshape(s, parts.lens[1], lens.region(&gold));
    reshape(s, parts.lens[2], lens.region(&red));
}

/// The turn, drawn as it goes (one updater on an anchor in the scene).
struct Motion {
    tracker: Mob,
    anchor: Mob,
}

impl Motion {
    fn install(stage: &mut Stage<'_>, parts: Parts, lens: Lens, stop: f64) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = s.tracker_value(tracker).unwrap_or(0.0);
                if now.to_bits() == last.to_bits() {
                    return;
                }
                last = now;
                draw(s, &parts, &lens, stop, now);
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { tracker, anchor })
    }
}

/// Tween `tracker` to `value` over `rt` seconds, played.
fn drive(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(tween(tracker, value, rt)?)?;
    Ok(())
}

/// Bring parts already in the scene to `opacity` together (fills only: an
/// opacity change would fill a stroke).
fn reveal(stage: &mut Stage<'_>, mobs: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    let moves = mobs
        .iter()
        .map(|&(m, o)| fade_to(m, o, rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

impl SceneConstruct for ATurnByAHair {
    fn name(&self) -> &str {
        "03_a_turn_by_a_hair"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let stop = r.stop;
        let packed = model::hair(0.0);
        let centre = packed.tiles[MIDDLE].centre;
        let held_lens = packed.inner_corner(shown::LENS_CORNER);
        let lens = Lens {
            held: held_lens,
            half: r.half,
            scale: f64::from(r.mag) * BOARD.k,
        };

        // filled parts enter the scene now, unseen, layered (tiles and the
        // lens's fills at 0, overlaps above, dots on top); strokes stay
        // detached and fade in when their beat comes
        let hidden = || vec![BOARD.at; 3];
        let tiles: [Mob; 5] = std::array::from_fn(|i| {
            let c = if i == MIDDLE { turned() } else { corner() };
            polygon(stage, hidden(), flat(c, 1.0))
        });
        let patches: [Mob; 4] = std::array::from_fn(|_| polygon(stage, hidden(), flat(warn(), 0.9)));
        let touches: [Mob; 4] = std::array::from_fn(|_| dot(stage, BOARD.at, TOUCH_R, ink()));
        let lens_fills = [
            polygon(stage, hidden(), flat(corner(), 1.0)),
            polygon(stage, hidden(), flat(turned(), 1.0)),
            polygon(stage, hidden(), flat(warn(), 0.9)),
        ];
        let held: [Mob; 4] =
            std::array::from_fn(|k| dot(stage, BOARD.p(stop, packed.inner_corner(k)), HELD_R, touch()));
        let centre_dot = dot(stage, BOARD.p(stop, centre), HELD_R, ink());
        let lens_held = dot(stage, LENS_AT, HELD_R * 1.4, touch());
        for &m in &patches {
            stage.set_z_index(m, LAYER_OVERLAPS, true);
        }
        stage.set_z_index(lens_fills[2], LAYER_OVERLAPS, true);
        for &m in touches.iter().chain(&held).chain(&[centre_dot, lens_held]) {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        let mut filled: Vec<Mob> = tiles.to_vec();
        filled.extend(lens_fills);
        filled.extend(patches);
        filled.extend(touches);
        filled.extend(held);
        filled.extend([centre_dot, lens_held]);
        for &m in &filled {
            stage.add_to_scene(m)?;
            stage.set_opacity(m, 0.0, true);
        }
        let parts = Parts {
            tiles,
            patches,
            touches,
            lens: lens_fills,
        };
        let turn = Motion::install(stage, parts, lens, stop)?;
        let outline = path(stage, BOARD.outline(stop), line_style(ink(), 4.0));
        stage.set_z_index(outline, LAYER_MARKS, true);

        // ---------------------------------------------------------- 03_1
        // the turned packing; what is held
        let line = say(&kit, stage, LINES[0])?;
        let mut arrive: Vec<(Mob, f64)> = tiles.iter().map(|&m| (m, 1.0)).collect();
        arrive.extend(patches.iter().map(|&m| (m, 0.9)));
        reveal(stage, &arrive, 0.8)?;
        swap(stage, &[], &[outline], 0.5)?;
        line.wait_for(stage, "Fix the box")?;
        let held_word = kit.text_left(stage, HELD, 26.0, dim(), v(TEXT_X, 3.1))?;
        let held_what = kit.text_left(stage, HELD_WHAT, 24.0, dim(), v(TEXT_X, 2.65))?;
        reveal(stage, &[(centre_dot, 1.0)], 0.5)?;
        swap(stage, &[], &[held_word, held_what], 0.5)?;

        // ---------------------------------------------------------- 03_2
        // the held corners, half a tile out, on the circle inside the tile
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, "half a tile")?;
        let corner_ll = packed.inner_corner(0);
        let radius = path(
            stage,
            vec![BOARD.p(stop, centre), BOARD.p(stop, corner_ll)],
            line_style(dim(), 3.0),
        );
        let mid = ((centre.0 + corner_ll.0) / 2.0, (centre.1 + corner_ll.1) / 2.0);
        let half_label = kit.tex(stage, HALF, 30.0, dim(), &[], {
            let at = BOARD.p(stop, mid);
            v(at[0] - 0.22, at[1] + 0.18)
        })?;
        swap(stage, &[], &[radius, half_label], 0.6)?;
        line.wait_for(stage, "on the circle")?;
        // the circle inside the middle tile: its radius is how deep the
        // tile's own centre lies inside it, half a tile
        let inner_r = BOARD.k * model::depth_inside(&packed.tiles[MIDDLE], centre);
        let circle = ring(stage, BOARD.p(stop, centre), inner_r, dim(), 3.0);
        stage.set_z_index(circle, LAYER_MARKS, true);
        swap(stage, &[], &[circle], 0.6)?;
        reveal(stage, &held.map(|m| (m, 1.0)), 0.6)?;

        // ---------------------------------------------------------- 03_3
        // the question, and four seconds to think
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, "Do the corners")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        kit.hold(stage)?;
        let mut elapsed = 0.0;
        stage.add_dt_updater(
            arc,
            move |s: &mut fmn::mobject::Stage, m, dt| {
                elapsed += dt;
                retrace(s, m, arc_points(RING_AT, RING_R, 1.0 - elapsed / THINK_S));
            },
            false,
        )?;
        kit.listen(stage, THINK_S, LINES[2])?;

        // ---------------------------------------------------------- 03_4
        // the turn: the sides leave the circle's touching points, the
        // corners sink in
        let line = say(&kit, stage, LINES[3])?;
        swap(stage, &[track, arc, radius, half_label, held_word, held_what], &[], 0.4)?;
        let turn_label = kit.text_left(stage, TURN, 30.0, dim(), v(TEXT_X, 1.55))?;
        let degree_at = v(TEXT_X + 1.7, 1.55);
        let degree = kit.text_left(
            stage,
            DEGREE,
            36.0,
            turned(),
            v(degree_at[0] + 0.06, degree_at[1] + 0.1),
        )?;
        swap(stage, &[], &[turn_label, degree], 0.3)?;
        let delta_t = turn.tracker;
        kit.live_value(
            stage,
            move |s: &fmn::mobject::Stage| s.tracker_value(delta_t).unwrap_or(0.0).to_degrees(),
            1,
            false,
            36.0,
            turned(),
            degree_at,
            RIGHT,
        )?;
        reveal(stage, &touches.map(|m| (m, 1.0)), 0.4)?;
        let rt = line.left(stage, 2.0) - 0.3;
        drive(stage, turn.tracker, r.one, rt)?;

        // ---------------------------------------------------------- 03_5
        // one corner magnified, the same geometry; twice the turn
        let line = say(&kit, stage, LINES[4])?;
        let rl = lens.radius();
        let (l, rt_edge, b, t) = (LENS_AT[0] - rl, LENS_AT[0] + rl, LENS_AT[1] - rl, LENS_AT[1] + rl);
        let frame = path(
            stage,
            vec![v(l, b), v(rt_edge, b), v(rt_edge, t), v(l, t), v(l, b)],
            line_style(ink(), 3.0),
        );
        let callout = path(
            stage,
            vec![BOARD.p(stop, held_lens), v(l, LENS_AT[1])],
            line_style(dim(), 2.0),
        );
        let mag = kit.text(
            stage,
            &format!("{TIMES}{}", r.mag),
            32.0,
            ink(),
            v(rt_edge - 0.35, t + 0.3),
        )?;
        let bar_y = b - 0.3;
        let bar = path(
            stage,
            vec![v(l, bar_y), v(l + r.bar * lens.scale, bar_y)],
            line_style(ink(), 4.0),
        );
        let bar_label = kit.text_left(
            stage,
            &format!("{} {TILE}", r.bar_label),
            24.0,
            dim(),
            v(l + r.bar * lens.scale + 0.2, bar_y),
        )?;
        for &m in &[frame, callout, bar] {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        swap(stage, &[], &[frame, callout, mag, bar, bar_label], 0.5)?;
        reveal(
            stage,
            &[
                (lens_fills[0], 1.0),
                (lens_fills[1], 1.0),
                (lens_fills[2], 0.9),
                (lens_held, 1.0),
            ],
            0.5,
        )?;
        let bite_y = bar_y - 0.55;
        let bite_label = kit.text_left(stage, BITE, 30.0, dim(), v(TEXT_X, bite_y))?;
        let tile_word = kit.text_left(stage, TILE, 30.0, dim(), v(TEXT_X + 2.65, bite_y))?;
        swap(stage, &[], &[bite_label, tile_word], 0.3)?;
        let held_point = held_lens;
        kit.live_value(
            stage,
            move |s: &fmn::mobject::Stage| {
                let p = model::hair(s.tracker_value(delta_t).unwrap_or(0.0));
                model::depth_inside(&p.tiles[MIDDLE], held_point).max(0.0)
            },
            4,
            false,
            34.0,
            ink(),
            v(TEXT_X + 2.5, bite_y),
            RIGHT,
        )?;
        line.wait_for(stage, "Turn twice")?;
        let rt = (line.left(stage, 1.6) - 0.6).max(1.6);
        drive(stage, turn.tracker, r.two, rt)?;
        let deeper = kit.text_left(
            stage,
            &format!("{TIMES}{} {DEEPER}", r.ratio_2dp),
            30.0,
            ink(),
            v(TEXT_X, bite_y - 0.55),
        )?;
        swap(stage, &[], &[deeper], 0.4)?;

        // ---------------------------------------------------------- 03_6
        // the square of the turn
        let line = say(&kit, stage, LINES[5])?;
        // the formula a row higher, and the caption on its own line under
        // "precisely:", from the panel's left edge, so it ends well inside
        // the safe area (beside the label it ran to x 7.05)
        let formula = kit.tex(stage, FORMULA, 32.0, ink(), &[], v(3.6, 3.05))?;
        let precise = kit.text_left(stage, PRECISELY, 24.0, turned(), v(TEXT_X, 2.55))?;
        let radians = kit.text_left(stage, RADIANS, 24.0, dim(), v(TEXT_X, 2.25))?;
        swap(stage, &[], &[formula, precise, radians], 0.6)?;
        line.wait_for(stage, "never zero")?;

        // ---------------------------------------------------------- 03_7
        // the box of three, free; here, blocked either way
        let line = say(&kit, stage, LINES[6])?;
        let mini = super::draw::Board { at: MINI_AT, k: MINI_K };
        let three = f64::from(r.three);
        let free = model::turning(std::f64::consts::FRAC_PI_6);
        let mut mini_parts: Vec<Mob> = free
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let c = if i == MIDDLE { turned() } else { corner() };
                polygon(stage, mini.tile(three, t), flat(c, 1.0))
            })
            .collect();
        mini_parts.push(path(stage, mini.outline(three), line_style(ink(), 2.0)));
        mini_parts.push(ring(
            stage,
            mini.p(three, (three / 2.0, three / 2.0)),
            r.reach * MINI_K,
            dim(),
            2.0,
        ));
        for k in 0..4 {
            mini_parts.push(dot(stage, mini.p(three, free.inner_corner(k)), 0.035, touch()));
        }
        let box_of = kit.text_left(stage, &format!("{BOX_OF} {}:", r.three), 24.0, dim(), v(3.6, 2.85))?;
        let outer = kit.text_left(stage, OUTER, 22.0, dim(), v(3.6, 2.45))?;
        mini_parts.extend([box_of, outer]);
        // the ×3.99 comparison belongs to the doubled turn: it goes before
        // the control and the return move the live bite elsewhere
        swap(stage, &[formula, precise, radians, deeper], &mini_parts, 0.6)?;
        line.wait_for(stage, "Held here")?;
        drive(stage, turn.tracker, -r.one, 1.2)?;
        drive(stage, turn.tracker, 0.0, 0.7)?;

        // ---------------------------------------------------------- 03_8
        // one motion blocked; every arrangement needs a proof
        let line = say(&kit, stage, LINES[7])?;
        let blocked = kit.text_left(stage, BLOCKED, 24.0, ink(), v(TEXT_X, 2.85))?;
        swap(stage, &mini_parts, &[blocked], 0.5)?;
        line.wait_for(stage, "Whether any")?;
        let smaller = kit.text_left(stage, SMALLER, 24.0, dim(), v(TEXT_X, 2.4))?;
        swap(stage, &[], &[smaller], 0.5)?;

        // the line and its breath, and the closing reveal stands before the
        // wipe
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        stage.clear_updaters(turn.anchor, true);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("03_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
    }

    fn lens() -> Lens {
        let packed = model::hair(0.0);
        Lens {
            held: packed.inner_corner(shown::LENS_CORNER),
            half: Surd::ratio(shown::LENS_HALF.0, shown::LENS_HALF.1).to_f64(),
            scale: f64::from(shown::LENS_MAG) * BOARD.k,
        }
    }

    /// The lens, its bar and the readouts under it stay in the frame and
    /// clear of the board; the small box of three clears the lens.
    #[test]
    fn the_layout_fits_the_frame() {
        let l = lens();
        let stop = model::turned_side().to_f64();
        let board_right = BOARD.p(stop, (stop, 0.0))[0];
        assert!(LENS_AT[0] - l.radius() > board_right + 1.0, "the lens clears the board");
        assert!(
            LENS_AT[0] + l.radius() < FRAME_X_RADIUS - 0.3,
            "the lens is in the frame"
        );
        let bite_y = LENS_AT[1] - l.radius() - 0.3 - 0.55;
        assert!(bite_y - 0.55 > -FRAME_Y_RADIUS + 0.2, "the readouts are in the frame");
        let mini_bottom = MINI_AT[1] - MINI_K * model::turn_box() / 2.0;
        assert!(
            mini_bottom > LENS_AT[1] + l.radius() + 0.6,
            "the box of three clears the lens"
        );
    }

    /// The lens is honest: its scale is the board's times the stated
    /// magnification, and the overlap it shows, at both turns drawn, is the
    /// model's overlap at that corner, entirely inside the window, with the
    /// depth the facts give.
    #[test]
    fn the_lens_shows_the_true_overlap() {
        let l = lens();
        assert!((l.scale / BOARD.k - f64::from(shown::LENS_MAG)).abs() < 1e-12);
        let one = model::small_turn();
        for turn in [one, model::compose(one, one)] {
            let delta = turn.1.to_f64().atan2(turn.0.to_f64());
            let p = model::hair(delta);
            let full = common(&p.tiles[shown::LENS_CORNER].corners(), &p.tiles[MIDDLE].corners());
            let shown_part = common(
                &common(&l.window(), &p.tiles[shown::LENS_CORNER].corners()),
                &p.tiles[MIDDLE].corners(),
            );
            assert_eq!(full.len(), 3);
            assert_eq!(shown_part.len(), 3, "the whole triangle is in the window");
            let area = |pts: &[Point<f64>]| {
                (0..pts.len())
                    .map(|i| {
                        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                        a.0 * b.1 - b.0 * a.1
                    })
                    .sum::<f64>()
                    / 2.0
            };
            assert!((area(&full) - area(&shown_part)).abs() < 1e-15);
            let exact = model::held_corner_depth(turn).to_f64();
            assert!((model::depth_inside(&p.tiles[MIDDLE], l.held) - exact).abs() < 1e-12);
        }
    }
}
