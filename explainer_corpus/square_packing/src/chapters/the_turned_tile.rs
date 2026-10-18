//! Chapter 01, the turned tile: five equal tiles and the smallest square box
//! that holds them. Four fill a box two wide and the fifth has nowhere to go;
//! the obvious box is three wide, with room for four more (01_1 to 01_3). A
//! question with a four-second think (01_4). The middle tile turns an eighth
//! of a turn without hitting anything, its corners on the circle through the
//! inner corners (01_5); the squeeze, every frame a packing, stops when the
//! inner corners touch the middles of the turned tile's sides (01_6, 01_7);
//! the gap's diagonal is one tile, so the gap is 1/√2 and the box
//! 2 + 1/√2 ≈ 2.71 (01_8, 01_9); one step further the corners bite in, in
//! red (01_10); a picture that fits is not a proof that nothing smaller does
//! (01_11).
//!
//! From 01_5 on, every tile and the box are redrawn each frame from
//! `model::corners_and_middle` at the frame's box width and turn (one
//! updater, two trackers), so the picture is the model's packing at every
//! frame, and the model's tests check those frames: `turning` and `squeezed`
//! never overlap. Only 01_10 goes past the stop, and it says so in red.

use explainer_kit::{BACKGROUND, Kit, fade_all, hex, tween, v};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, common, corner, dim, dot, fact, fact_int, fade_in_lagged, fail, faint, flat, fly, grow, ink,
    line_style, path, pause, polygon, reshape, retrace, ring, say, swap, together, touch, turned, warn,
};
use crate::model::{self, MIDDLE, TILES};

pub struct TheTurnedTile {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 11] = [
    "01_1", "01_2", "01_3", "01_4", "01_5", "01_6", "01_7", "01_8", "01_9", "01_10", "01_11",
];

// the chapter's fixed words
pub const QUESTION: &str = "How small can the box be?";
pub const TURNED_BY: &str = "turned";
pub const DEGREE: &str = "°";
pub const WIDTH: &str = "box width";
pub const THINK: &str = "think";
pub const OVERLAP: &str = "overlap";
pub const FITS: &str = "fits";
pub const SMALLER: &str = "smaller?";
pub const PROOF: &str = "needs a proof";
pub const PRECISELY: &str = "precisely:";
pub const SOURCE: &str = "this packing: F. Göbel,";
pub const GAP_TIMES: &str = r"\text{gap} \times \sqrt{2} = 1";
pub const GAP_IS: &str = r"\text{gap} = \frac{1}{\sqrt{2}}";
pub const ONE: &str = "1";
pub const ONE_OVER_ROOT: &str = r"\frac{1}{\sqrt{2}}";

// ---------------------------------------------------------------- layout

/// The box, centred left of middle; a tile is 1.45 scene units wide, so the
/// box of three spans 4.35 of the frame's 8 units of height.
pub const BOARD: Board = Board {
    at: [-2.3, -0.35, 0.0],
    k: 1.45,
};
/// The middle of the panel to the right of the box.
pub const PANEL_X: f64 = 3.7;
/// The think ring: where, and how big.
pub const RING_AT: Vec3 = [3.7, 0.35, 0.0];
pub const RING_R: f64 = 0.55;
/// The think window (storyboard 01_4: listen 4 s).
pub const THINK_S: f64 = 4.0;
/// How far 01_10 squeezes past the stop (tiles): far enough that the
/// overlaps are seen. A drawing distance: no number on screen is typed from
/// it (the width readout shows the frame's width as it is).
pub const OVERSHOOT: f64 = 0.15;
/// Where the five tiles wait in 01_1: a row about the frame's middle, this
/// many tiles from one centre to the next.
pub const ROW_Y: f64 = -0.35;
pub const ROW_PITCH: f64 = 1.3;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    full: u32,
    side3: u32,
    room: u32,
    degrees: u32,
    reach: f64,
    stop: f64,
    side_2dp: String,
    side_5dp: String,
    year: u32,
}

fn want(key: &str, model: u32) -> fmn::Result<u32> {
    let got = fact_int(key)?;
    if got == model {
        Ok(got)
    } else {
        Err(fail(format!("{key} is {got} but the model has {model}")))
    }
}

fn reads() -> fmn::Result<Reads> {
    let tiles = want("tiles", TILES)?;
    let full = want("grid.full", model::full_grid(tiles))?;
    want("grid.full.holds", full * full)?;
    let side3 = want("grid.side", model::grid_side(tiles))?;
    let room = want("grid.room", side3 * side3 - tiles)?;
    let reach2 = model::turn_reach2();
    if fact("turn.reach2")? != reach2.to_string() || fact("turn.inner2")? != reach2.to_string() {
        return Err(fail("the circle of reach is not the facts'".into()));
    }
    let exact = model::turned_side();
    if fact("turned.side")? != exact.to_string() {
        return Err(fail(format!("turned.side is not the model's {exact}")));
    }
    let side_2dp = fact("turned.side.2dp")?;
    let side_5dp = fact("turned.side.5dp")?;
    if side_2dp != exact.decimals(2) || side_5dp != exact.decimals(5) {
        return Err(fail("the side's decimals are not the model's".into()));
    }
    if fact("turned.check")? != "fits" {
        return Err(fail("the five turned tiles do not fit".into()));
    }
    Ok(Reads {
        full,
        side3,
        room,
        degrees: fact_int("turned.degrees")?,
        reach: reach2.to_f64().sqrt(),
        stop: exact.to_f64(),
        side_2dp,
        side_5dp,
        year: fact_int("source.gobel")?,
    })
}

/// A tile drawn alone, its centre at the scene point `at` (straight).
fn lone_tile(stage: &mut Stage<'_>, at: Vec3, c: Srgb) -> Mob {
    let b = Board { at, k: BOARD.k };
    let pts = b.tile(1.0, &model::Tile::straight(0.5, 0.5));
    polygon(stage, pts, flat(c, 1.0))
}

/// One cell's outline, centred at the box point `q` of the box `side` wide.
fn cell(stage: &mut Stage<'_>, side: f64, q: (f64, f64), c: Srgb) -> Mob {
    let b = Board {
        at: BOARD.p(side, q),
        k: BOARD.k,
    };
    path(stage, b.outline(1.0), line_style(c, 3.0))
}

/// The tiles, the box and the overlap patches as the model has them, for a
/// box `side` wide and a middle tile turned `degrees`, every frame either
/// tracker moves (one updater on an anchor in the scene).
struct Layout {
    side: Mob,
    degrees: Mob,
    anchor: Mob,
}

impl Layout {
    fn install(
        stage: &mut Stage<'_>,
        tiles: [Mob; 5],
        outline: Mob,
        patches: [Mob; 4],
        side: f64,
    ) -> fmn::Result<Self> {
        let side_t = stage.add_value_tracker(side);
        let degrees_t = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = (
                    s.tracker_value(side_t).unwrap_or(side),
                    s.tracker_value(degrees_t).unwrap_or(0.0),
                );
                if now.0.to_bits() == last.0.to_bits() && now.1.to_bits() == last.1.to_bits() {
                    return;
                }
                last = now;
                let (w, a) = (now.0, now.1.to_radians());
                let p = model::corners_and_middle(w, a.cos(), a.sin());
                for (m, t) in tiles.iter().zip(&p.tiles) {
                    reshape(s, *m, BOARD.tile(w, t));
                }
                retrace(s, outline, BOARD.outline(w));
                let middle = p.tiles[MIDDLE].corners();
                for (k, &m) in patches.iter().enumerate() {
                    let bite = common(&p.tiles[k].corners(), &middle);
                    let pts: Vec<Vec3> = if bite.len() >= 3 {
                        bite.iter().map(|&q| BOARD.p(w, q)).collect()
                    } else {
                        vec![BOARD.at; 3]
                    };
                    reshape(s, m, pts);
                }
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self {
            side: side_t,
            degrees: degrees_t,
            anchor,
        })
    }
}

/// A formula with what `TransformMatchingTex` pairs its glyphs by.
struct Formula {
    mob: Mob,
    spans: SpanMapData,
}

fn formula(kit: &Kit, stage: &mut Stage<'_>, src: &str, at: Vec3) -> fmn::Result<Formula> {
    let (mob, spans) = kit.tex_spans(stage, src, 46.0, ink(), &[], at)?;
    Ok(Formula { mob, spans })
}

/// Morph `from` (on screen) into `to` (detached), `rt` seconds.
fn morph(stage: &mut Stage<'_>, from: &Formula, to: &Formula, rt: f64) -> fmn::Result<()> {
    let m = TransformMatchingTex::new(stage.arena_mut(), from.mob, to.mob, &from.spans, &to.spans)?;
    let a = stage.prepare(explainer_kit::Timed::rt(m, rt))?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

/// The box-of-three tween of a tracker to `value`, `rt` seconds, played.
fn drive(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(tween(tracker, value, rt)?)?;
    Ok(())
}

impl SceneConstruct for TheTurnedTile {
    fn name(&self) -> &str {
        "01_the_turned_tile"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let (two, three) = (f64::from(r.full), f64::from(r.side3));

        // ---------------------------------------------------------- 01_1
        // five tiles in a row, the gold one last: the fifth
        let n = TILES as usize;
        let tiles: [Mob; 5] = std::array::from_fn(|i| {
            let x = BOARD.k * ROW_PITCH * (i as f64 - (n as f64 - 1.0) / 2.0);
            let c = if i == MIDDLE { turned() } else { corner() };
            lone_tile(stage, v(x, ROW_Y), c)
        });
        let question = kit.text(stage, QUESTION, 44.0, ink(), v(0.0, 3.35))?;
        let line = say(&kit, stage, LINES[0])?;
        fade_in_lagged(stage, &tiles, 1.8, 0.3)?;
        line.wait_for(stage, "How small")?;
        swap(stage, &[], &[question], 0.6)?;

        // ---------------------------------------------------------- 01_2
        // four fill the box two wide; the fifth waits outside it
        let line = say(&kit, stage, LINES[1])?;
        let outline = path(stage, BOARD.outline(two), line_style(ink(), 4.0));
        swap(stage, &[], &[outline], 0.5)?;
        let cells = [(0.5, 0.5), (1.5, 0.5), (0.5, 1.5), (1.5, 1.5)];
        let mut moves = Vec::new();
        for (k, &q) in cells.iter().enumerate() {
            moves.push(fly(tiles[k], BOARD.p(two, q), 1.2)?);
        }
        moves.push(fly(tiles[MIDDLE], BOARD.p(two, (two + 1.0, two / 2.0)), 1.2)?);
        together(stage, moves)?;
        line.wait_for(stage, "The fifth")?;
        together(stage, vec![grow(tiles[MIDDLE], 1.1, 0.3)?])?;
        together(stage, vec![grow(tiles[MIDDLE], 1.0 / 1.1, 0.3)?])?;

        // ---------------------------------------------------------- 01_3
        // the box of three: corners and middle; room for four more
        let line = say(&kit, stage, LINES[2])?;
        let straight = model::corners_and_middle(three, 1.0, 0.0);
        let mut moves = vec![grow(outline, three / two, 1.2)?];
        for (k, t) in straight.tiles.iter().enumerate() {
            moves.push(fly(tiles[k], BOARD.p(three, t.centre), 1.2)?);
        }
        together(stage, moves)?;
        let empty: Vec<Mob> = [(1.5, 0.5), (0.5, 1.5), (2.5, 1.5), (1.5, 2.5)]
            .iter()
            .map(|&q| cell(stage, three, q, faint()))
            .collect();
        if empty.len() != r.room as usize {
            return Err(fail(format!(
                "{} empty cells drawn, grid.room is {}",
                empty.len(),
                r.room
            )));
        }
        line.wait_for(stage, "with room")?;
        fade_in_lagged(stage, &empty, 1.2, 0.4)?;

        // ---------------------------------------------------------- 01_4
        // the question, and four seconds to think
        let line = say(&kit, stage, LINES[3])?;
        swap(stage, &empty, &[], 0.5)?;
        line.wait_for(stage, "Could turning")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        let think = kit.text(stage, THINK, 30.0, dim(), v(RING_AT[0], RING_AT[1] - RING_R - 0.4))?;
        swap(stage, &[], &[track, arc, think], 0.4)?;
        together(stage, vec![grow(tiles[MIDDLE], 1.12, 0.35)?])?;
        together(stage, vec![grow(tiles[MIDDLE], 1.0 / 1.12, 0.35)?])?;
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
        kit.listen(stage, THINK_S, LINES[3])?;
        swap(stage, &[track, arc, think], &[], 0.4)?;

        // ---------------------------------------------------------- 01_5
        // the turn, inside the circle through the inner corners
        let patches: [Mob; 4] = std::array::from_fn(|_| polygon(stage, vec![BOARD.at; 3], flat(warn(), 0.9)));
        let layout = Layout::install(stage, tiles, outline, patches, three)?;
        let line = say(&kit, stage, LINES[4])?;
        let reach = ring(stage, BOARD.at, BOARD.k * r.reach, faint(), 4.0);
        let inner = model::corners_and_middle(three, 1.0, 0.0);
        let marks: Vec<Mob> = (0..4)
            .map(|k| dot(stage, BOARD.p(three, inner.inner_corner(k)), 0.06, dim()))
            .collect();
        let label = kit.text_left(stage, TURNED_BY, 34.0, dim(), v(PANEL_X - 1.6, 1.6))?;
        let degree_at = v(PANEL_X + 0.6, 1.6);
        let readout = kit.live_number(stage, layout.degrees, 0, false, 44.0, turned(), degree_at, RIGHT)?;
        let degree = kit.text_left(
            stage,
            DEGREE,
            44.0,
            turned(),
            v(degree_at[0] + 0.06, degree_at[1] + 0.1),
        )?;
        let mut ins = vec![reach, label, degree];
        ins.extend(&marks);
        swap(stage, &[], &ins, 0.6)?;
        line.wait_for(stage, "Its corners")?;
        let rt = line.left(stage, 2.5) - 0.3;
        drive(stage, layout.degrees, f64::from(r.degrees), rt)?;

        // ---------------------------------------------------------- 01_6
        // the squeeze, with the box of three as a ghost
        let line = say(&kit, stage, LINES[5])?;
        let ghost = path(stage, BOARD.outline(three), line_style(faint(), 3.0));
        let mut outs = vec![reach, label, degree, readout];
        outs.extend(&marks);
        swap(stage, &outs, &[ghost], 0.5)?;
        let width_y = BOARD.at[1] - BOARD.k * three / 2.0 - 0.5;
        let width_label = kit.text_left(stage, WIDTH, 32.0, dim(), v(BOARD.at[0] - 2.1, width_y))?;
        swap(stage, &[], &[width_label], 0.3)?;
        kit.live_number(
            stage,
            layout.side,
            2,
            false,
            40.0,
            ink(),
            v(BOARD.at[0] + 0.3, width_y),
            LEFT,
        )?;
        let rt = line.left(stage, 3.0) - 0.3;
        drive(stage, layout.side, r.stop, rt)?;

        // ---------------------------------------------------------- 01_7
        // the four touches, at the middles of the turned tile's sides
        let line = say(&kit, stage, LINES[6])?;
        let packed = model::squeezed(r.stop);
        let dots: Vec<Mob> = (0..4)
            .map(|k| dot(stage, BOARD.p(r.stop, packed.inner_corner(k)), 0.075, touch()))
            .collect();
        line.wait_for(stage, "touches")?;
        fade_in_lagged(stage, &dots, 1.0, 0.3)?;

        // ---------------------------------------------------------- 01_8
        // the square gap, and its diagonal of one tile
        let line = say(&kit, stage, LINES[7])?;
        let corners: Vec<Vec3> = [0, 1, 3, 2, 0]
            .iter()
            .map(|&k| BOARD.p(r.stop, packed.inner_corner(k)))
            .collect();
        let dark = hex(BACKGROUND);
        let gap = path(stage, corners, line_style(dark, 4.0));
        swap(stage, &[], &[gap], 0.8)?;
        line.wait_for(stage, "opposite")?;
        let (a, b) = (
            BOARD.p(r.stop, packed.inner_corner(0)),
            BOARD.p(r.stop, packed.inner_corner(3)),
        );
        let diagonal = path(stage, vec![a, b], line_style(dark, 5.0));
        let one = kit.tex(stage, ONE, 40.0, dark, &[], v(BOARD.at[0] - 0.22, BOARD.at[1] + 0.22))?;
        swap(stage, &[], &[diagonal, one], 0.8)?;

        // ---------------------------------------------------------- 01_9
        // gap × √2 = 1, so gap = 1/√2, and the box is 2 + 1/√2 ≈ 2.71
        let line = say(&kit, stage, LINES[8])?;
        let f_at = v(PANEL_X, 1.4);
        let f1 = formula(&kit, stage, GAP_TIMES, f_at)?;
        let w = stage.prepare(explainer_kit::Timed::rt(write(stage.arena(), f1.mob), 1.4))?;
        stage.play_prepared(vec![w])?;
        line.wait_for(stage, "So the gap")?;
        let f2 = formula(&kit, stage, GAP_IS, f_at)?;
        morph(stage, &f1, &f2, 1.0)?;
        // the gap's width, measured along the floor between the two lower
        // corner tiles, where nothing else is drawn
        let floor_y = 0.12;
        let (l, rr) = (
            BOARD.p(r.stop, (1.0, floor_y)),
            BOARD.p(r.stop, (r.stop - 1.0, floor_y)),
        );
        let measure = path(stage, vec![l, rr], line_style(dim(), 3.0));
        let measure_label = kit.tex(
            stage,
            ONE_OVER_ROOT,
            26.0,
            dim(),
            &[],
            BOARD.p(r.stop, (r.stop / 2.0, 0.4)),
        )?;
        swap(stage, &[], &[measure, measure_label], 0.6)?;
        line.wait_for(stage, "and the box")?;
        let box_is = format!(r"\text{{box}} = 2 + \frac{{1}}{{\sqrt{{2}}}} \approx {}", r.side_2dp);
        let f3 = formula(&kit, stage, &box_is, f_at)?;
        morph(stage, &f2, &f3, 1.0)?;
        let precisely = kit.text_left(stage, PRECISELY, 28.0, turned(), v(PANEL_X - 2.3, 0.3))?;
        let exact_src = format!(r"2 + \tfrac{{\sqrt{{2}}}}{{2}} = {}\ldots", r.side_5dp);
        let exact = kit.tex_left(stage, &exact_src, 30.0, dim(), &[], v(PANEL_X - 0.6, 0.3))?;
        swap(stage, &[], &[precisely, exact], 0.6)?;

        // ---------------------------------------------------------- 01_10
        // one step further: the corners bite in, in red; then back
        let line = say(&kit, stage, LINES[9])?;
        let mut outs = vec![gap, diagonal, one, measure, measure_label];
        outs.extend(&dots);
        swap(stage, &outs, &[], 0.4)?;
        for &m in &patches {
            stage.add_to_scene(m)?;
        }
        let bite = kit.text(stage, OVERLAP, 40.0, warn(), v(PANEL_X, -1.0))?;
        let rt = (line.left(stage, 1.6) * 0.5).max(0.8);
        drive(stage, layout.side, r.stop - OVERSHOOT, rt)?;
        swap(stage, &[], &[bite], 0.3)?;
        pause(stage, 0.8)?;
        drive(stage, layout.side, r.stop, 0.9)?;
        swap(stage, &[bite], &[], 0.3)?;

        // ---------------------------------------------------------- 01_11
        // it fits; that nothing smaller fits needs a proof
        let line = say(&kit, stage, LINES[10])?;
        let f3_out = vec![f3.mob, precisely, exact, ghost];
        swap(stage, &f3_out, &[], 0.5)?;
        let tick_at = v(PANEL_X - 1.2, 1.2);
        let tick = path(
            stage,
            vec![
                v(tick_at[0] - 0.22, tick_at[1]),
                v(tick_at[0] - 0.05, tick_at[1] - 0.18),
                v(tick_at[0] + 0.25, tick_at[1] + 0.22),
            ],
            line_style(touch(), 7.0),
        );
        let fits = kit.text_left(
            stage,
            &format!("{FITS}: {}", r.side_2dp),
            40.0,
            ink(),
            v(tick_at[0] + 0.5, tick_at[1]),
        )?;
        swap(stage, &[], &[tick, fits], 0.6)?;
        line.wait_for(stage, "nothing smaller")?;
        let ask_at = v(PANEL_X - 1.2, 0.0);
        let mark = kit.text(stage, "?", 54.0, warn(), ask_at)?;
        let smaller = kit.text_left(stage, SMALLER, 40.0, ink(), v(ask_at[0] + 0.5, ask_at[1]))?;
        swap(stage, &[], &[mark, smaller], 0.6)?;
        line.wait_for(stage, "needs a proof")?;
        let proof = kit.text_left(stage, PROOF, 36.0, dim(), v(ask_at[0] + 0.5, ask_at[1] - 0.7))?;
        let source = kit.text_left(
            stage,
            &format!("{SOURCE} {}", r.year),
            24.0,
            dim(),
            v(-FRAME_X_RADIUS + 0.45, -3.6),
        )?;
        swap(stage, &[], &[proof, source], 0.6)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        stage.clear_updaters(layout.anchor, true);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The box of three and the panel stay in the frame and apart; the gap's
    /// label fits in the strip under the turned tile.
    #[test]
    fn the_layout_fits_the_frame() {
        let half = BOARD.k * 3.0 / 2.0;
        assert!(BOARD.at[0] - half > -FRAME_X_RADIUS + 0.3);
        assert!(BOARD.at[1] + half < 3.35 - 0.4, "the question clears the box");
        assert!(
            BOARD.at[1] - half - 0.5 > -FRAME_Y_RADIUS + 0.3,
            "the width readout is in the frame"
        );
        assert!(BOARD.at[0] + half < PANEL_X - 2.4, "the panel clears the box");
        // the row of five
        let last = BOARD.k * ROW_PITCH * 2.0 + BOARD.k / 2.0;
        assert!(last < FRAME_X_RADIUS - 0.3);
        // under the turned tile at the stop, the strip between the lower corner
        // tiles is clear up to the tile's bottom corner
        let stop = model::turned_side().to_f64();
        let p = model::squeezed(stop);
        let lowest = p.tiles[MIDDLE]
            .corners()
            .iter()
            .map(|c| c.1)
            .fold(f64::INFINITY, f64::min);
        assert!(lowest > 0.6, "{lowest}");
    }

    #[test]
    fn the_overshoot_overlaps_and_the_stop_does_not() {
        let stop = model::turned_side().to_f64();
        assert!(model::squeezed(stop - OVERSHOOT).check(1e-12).overlapping.len() == 4);
        assert!(model::squeezed(stop).check(1e-12).fits());
    }

    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, LINES.to_vec());
    }
}
