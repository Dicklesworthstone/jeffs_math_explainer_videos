//! Chapter 02, four pins: why five straight tiles need a box three wide.
//! In a box narrower than three, marks at a third and two thirds cut the
//! floor into pieces shorter than a tile (02_1, 02_2), so a straight tile's
//! shadow always holds a mark strictly inside (02_3); with the wall marked
//! too, every straight tile holds one of four pins strictly inside (02_4);
//! tiles cannot share a pin, so at most four fit (02_5, 02_6, with the think
//! window); at exactly three the pins sit on edges and five fit (02_7); the
//! turned tile holds no pin, so the argument cannot reach it (02_8).
//!
//! The proof is written in `model.rs` (§ four pins) and each of its steps is
//! a checked fact. Every frame here is redrawn from the model: the marks and
//! pins from `model::marks` and `model::pins`, the halo on a caught mark or
//! pin from `model::marks_inside` and `model::pins_inside`, the tiles from
//! `model::wander`, `corners_and_middle`, `turning` and `squeezed`. The
//! frames where tiles overlap (02_6, 02_7 below three) show the overlap in
//! red and are never called a packing.
//!
//! What draws over what is set by layer, not by when a part arrives: every
//! addition to the scene (an animation's too) re-sorts it by z index and
//! then by order of addition, so a tile revealed late would otherwise cover
//! the pins. The tiles sit at 0, the red overlaps above them, the marks,
//! pins and outline above those, and the halos, thin rings, on top: in the
//! box of 2.71 each shared pin lies inside its red overlap, and a ring
//! leaves the red in view.

use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    arc_points, common, corner, dim, dot, fact, fact_int, fade_to, fail, faint, flat, ink, line_style, path, polygon,
    redot, reshape, retrace, ring, say, stand, swap, together, turned, warn,
};
use super::the_turned_tile::{BOARD, PANEL_X, PRECISELY, RING_AT, RING_R, THINK_S};
use crate::model::{self, MIDDLE, Packing, Surd, Tile};

pub struct FourPins {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["02_1", "02_2", "02_3", "02_4", "02_5", "02_6", "02_7", "02_8"];

// the chapter's fixed words
pub const MARK_LO: &str = r"\tfrac{1}{3}";
pub const MARK_HI: &str = r"\tfrac{2}{3}";
pub const BOX: &str = "box";
pub const PIECE: &str = "each piece";
pub const UNDER_ONE: &str = r"< 1";
pub const AT_MOST: &str = "at most";
pub const STRAIGHT: &str = "straight tiles";
pub const NARROWER: &str = "for every box narrower than";
pub const NO_ROOM: &str = "below one tile wide, none fit";
pub const NO_PIN: &str = "turned tile: no pin inside";

// ---------------------------------------------------------------- layout

/// Half a mark's tick, across the side it marks (tiles).
pub const TICK: f64 = 0.12;
/// A pin's radius and its halo's (scene units), and the halo's opacity.
pub const PIN_R: f64 = 0.055;
pub const HALO_R: f64 = 0.12;
pub const HALO_O: f64 = 0.95;
/// A halo's ring width (Reference stroke units).
pub const HALO_W: f64 = 4.0;
/// Draw layers (z index; the tiles and the shadow are at 0).
pub const LAYER_OVERLAPS: i32 = 1;
pub const LAYER_MARKS: i32 = 2;
pub const LAYER_HALOS: i32 = 3;
/// The shadow's bar under the floor (tiles below the floor line).
pub const SHADOW: (f64, f64) = (-0.32, -0.2);
/// How far 02_2 widens the box towards three and back, as a share of the
/// way (a drawing distance: the readouts show the widths as they are).
pub const SWEEP: f64 = 0.85;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    stop: f64,
    side_2dp: String,
    count: u32,
    threshold: u32,
    pieces: usize,
}

fn reads() -> fmn::Result<Reads> {
    let stop = model::turned_side();
    let side_2dp = fact("turned.side.2dp")?;
    if side_2dp != stop.decimals(2) {
        return Err(fail("turned.side.2dp is not the model's".into()));
    }
    let count = fact_int("pins.count")?;
    if count as usize != model::pins(stop).len() {
        return Err(fail(format!("pins.count is {count}")));
    }
    let threshold = fact_int("pins.threshold")?;
    if fact("pins.threshold")? != model::pin_threshold().to_string() {
        return Err(fail("pins.threshold is not the model's".into()));
    }
    for (key, want) in [
        ("pins.lemma", "yes"),
        ("pins.straight_lemma", "yes"),
        ("pins.lemma.at3", "no"),
        ("pins.lemma.half", "no room"),
        ("straight.check", "fits"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    if !fact("pins.owners.turned")?.ends_with("|none") {
        return Err(fail("the turned tile holds a pin".into()));
    }
    Ok(Reads {
        stop: stop.to_f64(),
        side_2dp,
        count,
        threshold,
        pieces: model::marks(Surd::int(1)).len() + 1,
    })
}

/// What one frame of the chapter shows.
struct Shown {
    side: f64,
    /// The tiles, in the order of the mobjects the motion was given.
    tiles: Vec<Tile<f64>>,
    /// Tile 0's shadow on the floor, its caught mark haloed (02_3).
    shadow: bool,
    /// The pins strictly inside a tile haloed (from 02_4).
    pins: bool,
}

impl Shown {
    fn of(p: Packing<f64>) -> Self {
        Self {
            side: p.side,
            tiles: p.tiles,
            shadow: false,
            pins: true,
        }
    }
}

/// The board's parts every frame redraws, in the scene from the start.
#[derive(Clone, Copy)]
struct Parts {
    outline: Mob,
    shadow: Mob,
    floor: [Mob; 2],
    wall: [Mob; 2],
    floor_halos: [Mob; 2],
    pins: [Mob; 4],
    halos: [Mob; 4],
    patches: [Mob; 4],
}

/// The shadow's bar under the floor, for the shadow [a, a + 1].
fn bar(side: f64, a: f64) -> Vec<Vec3> {
    let (lo, hi) = SHADOW;
    [(a, lo), (a + 1.0, lo), (a + 1.0, hi), (a, hi)]
        .iter()
        .map(|&q| BOARD.p(side, q))
        .collect()
}

/// A mark's tick, unseen until its frame draws it. Detached.
fn tick(stage: &mut Stage<'_>) -> Mob {
    path(stage, vec![BOARD.at, BOARD.at], line_style(Palette::purple(), 5.0))
}

/// A yellow halo for a caught mark or pin: a ring, so what is under it
/// stays in view. Detached; shown and hidden by its stroke's opacity.
fn halo(stage: &mut Stage<'_>) -> Mob {
    ring(stage, BOARD.at, HALO_R, Palette::yellow(), HALO_W)
}

/// Redraw everything a frame shows.
fn draw(s: &mut fmn::mobject::Stage, parts: &Parts, tiles: &[Mob], shown: &Shown) {
    let w = shown.side;
    let hidden = || vec![BOARD.at; 3];
    retrace(s, parts.outline, BOARD.outline(w));
    for (i, &m) in model::marks(w).iter().enumerate() {
        retrace(s, parts.floor[i], vec![BOARD.p(w, (m, -TICK)), BOARD.p(w, (m, TICK))]);
        retrace(s, parts.wall[i], vec![BOARD.p(w, (-TICK, m)), BOARD.p(w, (TICK, m))]);
        redot(s, parts.floor_halos[i], BOARD.p(w, (m, 0.0)), HALO_R);
    }
    for (m, t) in tiles.iter().zip(&shown.tiles) {
        reshape(s, *m, BOARD.tile(w, t));
    }
    let inside = match (shown.shadow, shown.tiles.first()) {
        (true, Some(t)) => {
            let a = t.centre.0 - 0.5;
            reshape(s, parts.shadow, bar(w, a));
            model::marks_inside(a, w)
        }
        _ => {
            reshape(s, parts.shadow, hidden());
            Vec::new()
        }
    };
    for (i, &h) in parts.floor_halos.iter().enumerate() {
        s.set_stroke(
            h,
            None,
            None,
            Some(if inside.contains(&i) { HALO_O } else { 0.0 }),
            None,
            true,
        );
    }
    let caught: Vec<usize> = if shown.pins {
        shown.tiles.iter().flat_map(|t| model::pins_inside(t, w)).collect()
    } else {
        Vec::new()
    };
    for (k, &p) in model::pins(w).iter().enumerate() {
        redot(s, parts.pins[k], BOARD.p(w, p), PIN_R);
        redot(s, parts.halos[k], BOARD.p(w, p), HALO_R);
        let o = if caught.contains(&k) { HALO_O } else { 0.0 };
        s.set_stroke(parts.halos[k], None, None, Some(o), None, true);
    }
    let middle = (shown.tiles.len() == 5).then(|| shown.tiles[MIDDLE].corners());
    for (k, &m) in parts.patches.iter().enumerate() {
        let bite = match &middle {
            Some(mid) => common(&shown.tiles[k].corners(), mid),
            None => Vec::new(),
        };
        let pts = if bite.len() >= 3 {
            bite.iter().map(|&q| BOARD.p(w, q)).collect()
        } else {
            hidden()
        };
        reshape(s, m, pts);
    }
}

/// One motion of the chapter: every frame its tracker moves, the box, its
/// marks and pins, the given tiles and the halos are redrawn from
/// `frame(tracker)` (one updater on an anchor in the scene).
struct Motion {
    tracker: Mob,
    anchor: Mob,
}

impl Motion {
    fn install(
        stage: &mut Stage<'_>,
        parts: Parts,
        tiles: Vec<Mob>,
        start: f64,
        frame: Box<dyn Fn(f64) -> Shown>,
    ) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(start);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = s.tracker_value(tracker).unwrap_or(start);
                if now.to_bits() == last.to_bits() {
                    return;
                }
                last = now;
                draw(s, &parts, &tiles, &frame(now));
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { tracker, anchor })
    }

    /// Stop redrawing; the next motion takes over from the same frame.
    fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// Tween `tracker` to `value` over `rt` seconds, played.
fn drive(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(tween(tracker, value, rt)?)?;
    Ok(())
}

/// Bring board parts already in the scene to `opacity` together.
fn reveal(stage: &mut Stage<'_>, mobs: &[Mob], opacity: f64, rt: f64) -> fmn::Result<()> {
    let moves = mobs
        .iter()
        .map(|&m| fade_to(m, opacity, rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

impl SceneConstruct for FourPins {
    fn name(&self) -> &str {
        "02_four_pins"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let stop = r.stop;
        let three = f64::from(r.threshold);

        // the board's filled parts enter the scene now, unseen, and are
        // revealed by opacity; the outline and the ticks are strokes, which an
        // opacity change would fill, so they stay detached and fade in when
        // their beat comes. Their layers are set first (module doc).
        let hidden = || vec![BOARD.at; 3];
        let outline = path(stage, BOARD.outline(stop), line_style(ink(), 4.0));
        let shadow = polygon(stage, hidden(), flat(Palette::purple(), 0.8));
        let probe = polygon(stage, hidden(), flat(corner(), 1.0));
        let corners: [Mob; 4] = std::array::from_fn(|_| polygon(stage, hidden(), flat(corner(), 1.0)));
        let mid_straight = polygon(stage, hidden(), flat(corner(), 1.0));
        let mid_turned = polygon(stage, hidden(), flat(turned(), 1.0));
        let patches: [Mob; 4] = std::array::from_fn(|_| polygon(stage, hidden(), flat(warn(), 0.9)));
        let floor = [tick(stage), tick(stage)];
        let wall = [tick(stage), tick(stage)];
        let pins: [Mob; 4] = std::array::from_fn(|_| dot(stage, BOARD.at, PIN_R, Palette::purple()));
        let floor_halos = [halo(stage), halo(stage)];
        let halos: [Mob; 4] = std::array::from_fn(|_| halo(stage));
        let parts = Parts {
            outline,
            shadow,
            floor,
            wall,
            floor_halos,
            pins,
            halos,
            patches,
        };
        for &m in &patches {
            stage.set_z_index(m, LAYER_OVERLAPS, true);
        }
        for &m in [outline].iter().chain(&floor).chain(&wall).chain(&pins) {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        for &m in floor_halos.iter().chain(&halos) {
            stage.set_z_index(m, LAYER_HALOS, true);
        }
        let mut board: Vec<Mob> = vec![shadow, probe];
        board.extend(corners);
        board.extend([mid_straight, mid_turned]);
        board.extend(patches);
        board.extend(pins);
        board.extend(floor_halos);
        board.extend(halos);
        for &m in &board {
            stage.add_to_scene(m)?;
            stage.set_opacity(m, 0.0, true);
        }

        // ---------------------------------------------------------- 02_1
        // the box narrower than three; marks a third and two thirds along
        let width = Motion::install(
            stage,
            parts,
            Vec::new(),
            stop,
            Box::new(|w| Shown {
                side: w,
                tiles: Vec::new(),
                shadow: false,
                pins: false,
            }),
        )?;
        let line = say(&kit, stage, LINES[0])?;
        swap(stage, &[], &[outline], 0.6)?;
        line.wait_for(stage, "mark its floor")?;
        let [lo, hi] = model::marks(stop);
        let label_y = -0.62;
        let lo_label = kit.tex(stage, MARK_LO, 30.0, dim(), &[], BOARD.p(stop, (lo, label_y)))?;
        let hi_label = kit.tex(stage, MARK_HI, 30.0, dim(), &[], BOARD.p(stop, (hi, label_y)))?;
        swap(stage, &[], &[floor[0], floor[1], lo_label, hi_label], 0.6)?;

        // ---------------------------------------------------------- 02_2
        // three pieces, each shorter than a tile, in any box narrower than three
        let line = say(&kit, stage, LINES[1])?;
        swap(stage, &[lo_label, hi_label], &[], 0.4)?;
        let (label_x, value_x) = (PANEL_X - 2.0, PANEL_X + 1.0);
        let box_label = kit.text_left(stage, BOX, 34.0, dim(), v(label_x, 2.2))?;
        let piece_label = kit.text_left(stage, PIECE, 34.0, dim(), v(label_x, 1.5))?;
        let under_one = kit.tex_left(stage, UNDER_ONE, 38.0, ink(), &[], v(value_x + 0.2, 1.5))?;
        swap(stage, &[], &[box_label, piece_label, under_one], 0.4)?;
        let side_t = width.tracker;
        let box_value = kit.live_number(stage, side_t, 2, false, 40.0, ink(), v(value_x, 2.2), RIGHT)?;
        let piece_value = kit.live_value(
            stage,
            move |s: &fmn::mobject::Stage| model::longest_piece(s.tracker_value(side_t).unwrap_or(stop)),
            2,
            false,
            40.0,
            ink(),
            v(value_x, 1.5),
            RIGHT,
        )?;
        let wide = stop + (three - stop) * SWEEP;
        let rt = ((line.left(stage, 3.0) - 0.4) / 2.0).max(1.2);
        drive(stage, side_t, wide, rt)?;
        drive(stage, side_t, stop, rt)?;

        // ---------------------------------------------------------- 02_3
        // a straight tile's shadow on the floor always holds a mark inside
        width.stop(stage);
        let slide = Motion::install(
            stage,
            parts,
            vec![probe],
            0.0,
            Box::new(move |a| Shown {
                side: stop,
                tiles: vec![Tile::straight(a + 0.5, 0.5)],
                shadow: true,
                pins: false,
            }),
        )?;
        let line = say(&kit, stage, LINES[2])?;
        swap(
            stage,
            &[box_label, piece_label, under_one, box_value, piece_value],
            &[],
            0.4,
        )?;
        reveal(stage, &[probe, shadow], 1.0, 0.5)?;
        let rt = ((line.left(stage, 3.0) - 0.3) / 2.0).max(1.4);
        drive(stage, slide.tracker, stop - 1.0, rt)?;
        drive(stage, slide.tracker, 0.0, rt)?;

        // ---------------------------------------------------------- 02_4
        // the wall marked too: every straight tile holds a pin strictly inside
        slide.stop(stage);
        let roam = Motion::install(
            stage,
            parts,
            vec![probe],
            0.0,
            Box::new(move |t| {
                let (a, b) = model::wander(t, stop);
                Shown {
                    side: stop,
                    tiles: vec![Tile::straight(a + 0.5, b + 0.5)],
                    shadow: false,
                    pins: true,
                }
            }),
        )?;
        let line = say(&kit, stage, LINES[3])?;
        reveal(stage, &[shadow], 0.0, 0.3)?;
        // the wall's marks and the pins where they cross arrive together,
        // and the tile wanders for the rest of the line
        reveal(stage, &pins, 1.0, 0.6)?;
        swap(stage, &[], &wall, 0.4)?;
        let rt = line.left(stage, 3.0) - 0.2;
        drive(stage, roam.tracker, 1.0, rt)?;

        // ---------------------------------------------------------- 02_5
        // four tiles, four pins, and the question
        roam.stop(stage);
        let four = Motion::install(
            stage,
            parts,
            corners.to_vec(),
            stop,
            Box::new(|w| {
                let mut shown = Shown::of(model::corners_and_middle(w, 1.0, 0.0));
                shown.tiles.truncate(4);
                shown
            }),
        )?;
        let line = say(&kit, stage, LINES[4])?;
        reveal(stage, &[probe], 0.0, 0.4)?;
        let mut moves = Vec::new();
        for &c in &corners {
            moves.push(fade_to(c, 1.0, 0.6)?);
        }
        let g = {
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for m in moves {
                anims.push(stage.prepare(m)?);
            }
            AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, 0.4)?.with_run_time(1.4)
        };
        stage.play(g)?;
        line.wait_for(stage, "So how many")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        // the window opens when the line and its breath are done; the ring
        // drains with the frames from there
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
        kit.listen(stage, THINK_S, LINES[4])?;

        // ---------------------------------------------------------- 02_6
        // a fifth needs a fifth pin: at most four
        four.stop(stage);
        let pigeons = Motion::install(
            stage,
            parts,
            vec![corners[0], corners[1], corners[2], corners[3], mid_straight],
            stop,
            Box::new(|w| Shown::of(model::corners_and_middle(w, 1.0, 0.0))),
        )?;
        let line = say(&kit, stage, LINES[5])?;
        swap(stage, &[track, arc], &[], 0.4)?;
        // the fifth tile and its overlaps arrive together
        let mut arrive = vec![fade_to(mid_straight, 1.0, 0.6)?];
        for &m in &patches {
            arrive.push(fade_to(m, 0.9, 0.6)?);
        }
        together(stage, arrive)?;
        let most = kit.text_left(
            stage,
            &format!("{AT_MOST} {} {STRAIGHT}", r.count),
            40.0,
            ink(),
            v(PANEL_X - 2.0, 1.6),
        )?;
        swap(stage, &[], &[most], 0.5)?;
        line.wait_for(stage, "So five")?;
        let precise = kit.text_left(stage, PRECISELY, 26.0, turned(), v(PANEL_X - 2.0, 0.7))?;
        let narrower = kit.text_left(
            stage,
            &format!("{NARROWER} {},", r.threshold),
            26.0,
            dim(),
            v(PANEL_X - 2.0, 0.3),
        )?;
        let no_room = kit.text_left(stage, NO_ROOM, 26.0, dim(), v(PANEL_X - 2.0, -0.1))?;
        swap(stage, &[], &[precise, narrower, no_room], 0.5)?;

        // ---------------------------------------------------------- 02_7
        // at exactly three the pins reach the edges, and five fit
        let line = say(&kit, stage, LINES[6])?;
        swap(stage, &[most, precise, narrower, no_room], &[], 0.4)?;
        let rt = (line.left(stage, 2.5) * 0.45).max(2.0);
        drive(stage, pigeons.tracker, three, rt)?;
        let whole = format!(r"{} \div {} = 1", r.threshold, r.pieces);
        let whole = kit.tex(stage, &whole, 44.0, ink(), &[], v(PANEL_X, 1.6))?;
        swap(stage, &[], &[whole], 0.5)?;

        // ---------------------------------------------------------- 02_8
        // the turned tile holds no pin: the argument cannot reach it
        pigeons.stop(stage);
        let turn = Motion::install(
            stage,
            parts,
            vec![corners[0], corners[1], corners[2], corners[3], mid_turned],
            0.0,
            Box::new(|degrees: f64| Shown::of(model::turning(degrees.to_radians()))),
        )?;
        let line = say(&kit, stage, LINES[7])?;
        swap(stage, &[whole], &[], 0.4)?;
        line.wait_for(stage, "In the turned packing")?;
        // the same pose, blue to gold: the straight middle tile of the box of
        // three is the turn's first frame
        together(
            stage,
            vec![fade_to(mid_straight, 0.0, 0.4)?, fade_to(mid_turned, 1.0, 0.4)?],
        )?;
        let degrees = fact_int("turned.degrees")?;
        drive(stage, turn.tracker, f64::from(degrees), 1.6)?;
        turn.stop(stage);
        let squeeze = Motion::install(
            stage,
            parts,
            vec![corners[0], corners[1], corners[2], corners[3], mid_turned],
            three,
            Box::new(|w| Shown::of(model::squeezed(w))),
        )?;
        drive(stage, squeeze.tracker, stop, 2.0)?;
        let no_pin = kit.text_left(stage, NO_PIN, 34.0, turned(), v(PANEL_X - 2.0, 1.6))?;
        let width_note = kit.text_left(
            stage,
            &format!("{BOX} {}", r.side_2dp),
            34.0,
            dim(),
            v(PANEL_X - 2.0, 1.0),
        )?;
        swap(stage, &[], &[no_pin, width_note], 0.5)?;

        // the line and its breath, and the closing reveal stands before the
        // wipe (it comes after the squeeze, near the line's end)
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        squeeze.stop(stage);
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
            .filter(|id| id.starts_with("02_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
    }

    /// The marks' labels and the shadow's bar sit under the floor, clear of
    /// the box and inside the frame; the readouts and notes sit in the panel.
    #[test]
    fn the_layout_fits_the_frame() {
        let stop = model::turned_side().to_f64();
        let floor = BOARD.p(stop, (0.0, 0.0))[1];
        let (bar_lo, bar_hi) = (BOARD.p(stop, (0.0, SHADOW.0))[1], BOARD.p(stop, (0.0, SHADOW.1))[1]);
        assert!(bar_hi < BOARD.p(stop, (0.0, -TICK))[1], "the bar clears the ticks");
        assert!(bar_lo > -FRAME_Y_RADIUS + 0.3, "the bar is in the frame");
        assert!(
            floor - BOARD.p(stop, (0.0, -0.62))[1] < 1.2,
            "the labels stay near the floor"
        );
        let right = BOARD.p(model::turn_box(), (model::turn_box(), 0.0))[0];
        assert!(right < PANEL_X - 2.1, "the widest box clears the panel");
    }

    /// In 02_6 each shared pin lies inside its red overlap, and the overlap
    /// is wider than the pin's dot, so the red stays in view around it.
    #[test]
    fn each_shared_pin_sits_in_its_red_overlap() {
        let stop = model::turned_side().to_f64();
        let p = model::corners_and_middle(stop, 1.0, 0.0);
        let middle = p.tiles[MIDDLE].corners();
        for (k, pin) in model::pins(stop).iter().enumerate() {
            let bite = common(&p.tiles[k].corners(), &middle);
            assert!(bite.len() >= 3, "corner tile {k} overlaps the fifth");
            let xs = bite.iter().map(|q| q.0);
            let (lo, hi) = (
                xs.clone().fold(f64::INFINITY, f64::min),
                xs.fold(f64::NEG_INFINITY, f64::max),
            );
            assert!(lo < pin.0 && pin.0 < hi, "pin {k} is inside its overlap");
            assert!((hi - lo) * BOARD.k > 3.0 * PIN_R, "overlap {k} is wider than the pin");
        }
    }

    /// Every frame the chapter draws in 02_7 below three overlaps, and the
    /// frame at three fits with every pin on an edge.
    #[test]
    fn the_widening_overlaps_until_three() {
        let stop = model::turned_side().to_f64();
        for k in 0..100 {
            let w = stop + (3.0 - stop) * f64::from(k) / 100.0;
            let p = model::corners_and_middle(w, 1.0, 0.0);
            assert_eq!(p.check(1e-12).overlapping.len(), 4, "{w}");
        }
        let at3 = model::corners_and_middle(3.0, 1.0, 0.0);
        assert!(at3.check(1e-12).fits());
        assert!(at3.tiles.iter().all(|t| model::pins_inside(t, 3.0).is_empty()));
    }
}
