//! Chapter 09, the air in a room: a tube of air cut into slices, each with
//! mass, one slice pushed so that its neighbours crowd on one side and thin
//! out on the other (09_1); the slices as a row of carts and springs and the
//! table that goes with them, grown from 8 by 8 to 40 by 40 (09_2); the
//! arrow test on the air: one slice gives a jagged push, one smooth hump
//! gives itself back, and so do two and three humps (09_3); the quiz, how
//! many round trips a second in a bathroom 2.5 m long (09_4); and the tones
//! of that bathroom, a pitch line sliding from 60 to 150 Hz while the room's
//! outline lights at every tone it crosses (09_5).
//!
//! Every number is a fact: the speed of sound `K-R0.sound_m_per_s`, the
//! slices `K-R1.slices`, the angle one slice gives `K-R1.lump_turn_deg`, the
//! bound `K-R1.error_bound_percent`, the bathroom's sides `K-R2.sides_m` and
//! its tones `K-R2.tones_to_150_hz`, the quiz answer `K-Q.09_4.answer`, and
//! the card's years `K-S.*`.

use explainer_kit::props::RowStrips;
use explainer_kit::{Kit, Palette, fade_all, play, v};
use fmn::prelude::*;

use super::carts;
use super::draw::{
    MatrixGrid, QuizCard, arrow, fact, fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail, flat, group, moves,
    numbers, own, panel, pause, pushes, say, show, squeezed, stretched, tween_linear, wait_until,
};
use super::sound;
use crate::model::carts::chain;
use crate::model::linalg::dot;

pub struct TheAirInARoom {
    pub kit: Kit,
}

// ------------------------------------------------------------ the tube (09_1)

const TUBE_X0: f64 = -5.8;
const TUBE_X1: f64 = 5.8;
const TUBE_Y: f64 = 0.7;
/// The slices drawn in 09_1: a block each, with a gap of air between.
const SLICES: usize = 24;
const PERIOD: f64 = (TUBE_X1 - TUBE_X0) / SLICES as f64;
const BLOCK_W: f64 = 0.30;
const BLOCK_H: f64 = 0.95;
/// The slice that is pushed, and how far (scene units; the gap is
/// `PERIOD - BLOCK_W`).
const PUSHED: usize = 11;
const PUSH_MAX: f64 = 0.15;
// the pushed slice has a neighbour on each side
const _: () = assert!(PUSHED >= 1 && PUSHED + 1 < SLICES);

/// The centre of slice `i` at rest.
fn slice_x(i: usize) -> f64 {
    TUBE_X0 + PERIOD * (i as f64 + 0.5)
}

/// A unit-wide rectangle's points, centred at `(cx, cy)`, made `w` wide.
fn scaled(template: &[Vec3], cx: f64, cy: f64, w: f64) -> Vec<Vec3> {
    template
        .iter()
        .map(|p| [cx + p[0] * w.max(0.004), cy + p[1], p[2]])
        .collect()
}

/// A vertex list as the run of points a mobject keeps (and `set_points`
/// wants): each pair of vertices with a handle at its midpoint, which is what
/// `polyline` builds. Raw vertices are refused when their count is even and
/// drawn as curves when it is odd.
fn corners(vertices: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(2 * vertices.len());
    for w in vertices.windows(2) {
        out.push(w[0]);
        out.push([
            (w[0][0] + w[1][0]) / 2.0,
            (w[0][1] + w[1][1]) / 2.0,
            (w[0][2] + w[1][2]) / 2.0,
        ]);
    }
    if let Some(&last) = vertices.last() {
        out.push(last);
    }
    out
}

/// How far a gap that is `ratio` of its rest width is from at rest, as a
/// colour: squeezed magenta below one, thinned-out teal above, opacity by
/// how far.
fn gap_paint(ratio: f64) -> (Srgb, f64) {
    if ratio < 1.0 {
        (squeezed(), (1.0 - ratio).clamp(0.0, 1.0))
    } else {
        (stretched(), (ratio - 1.0).clamp(0.0, 1.0))
    }
}

/// The moves of a row of slices as the arrow test draws them.
pub fn pushes_of(u: &[f64]) -> Vec<f64> {
    let n = u.len();
    let k = chain(n);
    k.mul_vec(u)
}

/// The angle in degrees between a move of the 40 slices and the push that
/// holds it.
pub fn turn_of(u: &[f64]) -> f64 {
    let f = pushes_of(u);
    let c = dot(u, &f) / (dot(u, u) * dot(&f, &f)).sqrt();
    c.clamp(-1.0, 1.0).acos().to_degrees()
}

// ----------------------------------------------------- the arrow test (09_3)

/// The slices of 09_3 (the fact `K-R1.slices`): 40 slices between two walls.
const AIR: usize = 40;
const CURVE_X0: f64 = -5.4;
const CURVE_X1: f64 = 5.4;
const CURVE_Y: f64 = 0.55;
const CURVE_H: f64 = 1.5;

/// The shape number `k` of the 40 slices: 0 is one slice alone (the middle
/// one), 1 to 3 are that many humps of a sine wave. (A blend of two shapes
/// is turned from its push by as much as 85 degrees and only comes right in
/// the last hair before the hump, so the picture swaps shapes and never
/// shows a blend.)
fn shape(k: usize) -> Vec<f64> {
    if k == 0 {
        let mut u = vec![0.0; AIR];
        u[AIR / 2] = 1.0;
        u
    } else {
        (1..=AIR)
            .map(|j| (k as f64 * std::f64::consts::PI * j as f64 / (AIR as f64 + 1.0)).sin())
            .collect()
    }
}

/// The shape a tracker value names (the nearest whole shape, 0 to 3).
fn which(m: f64) -> usize {
    m.round().clamp(0.0, 3.0) as usize
}

/// Both curves drawn to the same height: the points of the move and of the
/// push that holds it, including the two walls, at height `CURVE_H` for the
/// largest value of each.
fn curves(k: usize) -> (Vec<Vec3>, Vec<Vec3>) {
    let u = shape(k);
    let f = pushes_of(&u);
    let height = |w: &[f64]| -> Vec<Vec3> {
        let top = w.iter().fold(0.0f64, |s, x| s.max(x.abs())).max(1e-12);
        let mut pts = vec![v(CURVE_X0, CURVE_Y)];
        for (j, x) in w.iter().enumerate() {
            let at = CURVE_X0 + (CURVE_X1 - CURVE_X0) * (j as f64 + 1.0) / (AIR as f64 + 1.0);
            pts.push(v(at, CURVE_Y + CURVE_H * x / top));
        }
        pts.push(v(CURVE_X1, CURVE_Y));
        pts
    };
    (height(&u), height(&f))
}

// ------------------------------------------------------- the carts (09_2)

const CARTS: usize = 8;
const CART_X0: f64 = -5.6;
const CART_X1: f64 = 5.6;
const CART_Y: f64 = 2.55;
const CART_W: f64 = 0.62;
const CART_H: f64 = 0.46;

fn cart_x(i: usize) -> f64 {
    CART_X0 + (CART_X1 - CART_X0) * (i as f64 + 0.5) / CARTS as f64
}

/// The table of `n` slices as strings, a row each: 2 on the diagonal, −1 on
/// each side of it, 0 elsewhere.
fn table_entries(n: usize) -> Vec<Vec<String>> {
    let k = chain(n);
    (0..n)
        .map(|i| (0..n).map(|j| format!("{}", k.at(i, j) as i64)).collect())
        .collect()
}

// ----------------------------------------------------------- the quiz (09_4)

const CARD_AT: Vec3 = [0.0, -0.95, 0.0];
const CARD_SIZE: (f64, f64) = (8.8, 4.4);
const PULSE_TUBE_Y: f64 = 2.85;

// -------------------------------------------------------- the tones (09_5)

pub(crate) const PITCH_LO: f64 = 60.0;
pub(crate) const PITCH_HI: f64 = 150.0;
const PITCH_X0: f64 = -5.6;
const PITCH_X1: f64 = 5.6;
const PITCH_Y: f64 = -1.1;
/// How wide, in hertz, a tone's glow is on each side.
pub(crate) const GLOW_HZ: f64 = 1.3;
/// The five tones that fit one side of the room, with the side: the voice's
/// "near a hundred and seven" is the third.
const ONE_SIDE: [(&str, &str); 5] = [
    ("68.6", "length 2.5 m"),
    ("71.5", "height 2.4 m"),
    ("107.2", "width 1.6 m"),
    ("137.2", "length 2.5 m"),
    ("142.9", "height 2.4 m"),
];

fn pitch_x(hz: f64) -> f64 {
    PITCH_X0 + (PITCH_X1 - PITCH_X0) * (hz - PITCH_LO) / (PITCH_HI - PITCH_LO)
}

/// The room in plan, with its figure and the lines that lie along its sides.
const ROOM_AT: Vec3 = [-1.2, 1.6, 0.0];
const ROOM_W: f64 = 3.5;
const ROOM_H: f64 = 2.24;

impl SceneConstruct for TheAirInARoom {
    fn name(&self) -> &str {
        "09_the_air_in_a_room"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();

        // ---- 09_1: a tube of air cut into slices; one is pushed
        let line = say(&kit, stage, "09_1")?;
        let unit = Rectangle::new().width(1.0).height(BLOCK_H).build()?.points().to_vec();
        let outline = stage.arena_mut().add(
            Rectangle::new()
                .width(TUBE_X1 - TUBE_X0 + 0.3)
                .height(BLOCK_H + 0.3)
                .corner_radius(0.15)
                .style(faded_line(dim, 3.0, 1.0))
                .build()?
                .moved_to(v(0.0, TUBE_Y)),
        );
        let mut blocks = Vec::new();
        for i in 0..SLICES {
            let (c, o) = if i == PUSHED { (moves(), 1.0) } else { (dim, 0.55) };
            let r = Rectangle::new()
                .width(BLOCK_W)
                .height(BLOCK_H)
                .style(flat(c, o))
                .build()?
                .moved_to(v(slice_x(i), TUBE_Y));
            blocks.push(stage.arena_mut().add(r));
        }
        let mut gaps = Vec::new();
        for i in 0..SLICES - 1 {
            let r = Rectangle::new()
                .width(PERIOD - BLOCK_W)
                .height(BLOCK_H)
                .style(flat(squeezed(), 0.0))
                .build()?
                .moved_to(v(slice_x(i) + PERIOD / 2.0, TUBE_Y));
            let g = stage.arena_mut().add(r);
            stage.add_to_scene(g)?;
            gaps.push(g);
        }
        // the push: the pushed block moves, its right gap narrows (squeezed)
        // and its left gap widens (thinned out)
        let push = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        {
            let (blocks, gaps, unit) = (blocks.clone(), gaps.clone(), unit.clone());
            let mut last = f64::NAN;
            stage.add_updater(
                anchor,
                move |s, _| {
                    let d = s.tracker_value(push).unwrap_or(0.0);
                    if d.to_bits() == last.to_bits() {
                        return;
                    }
                    last = d;
                    let rest = PERIOD - BLOCK_W;
                    let _ = s.set_points(blocks[PUSHED], &scaled(&unit, slice_x(PUSHED) + d, TUBE_Y, BLOCK_W));
                    // the gap to its right: from the block's right edge to its neighbour's left
                    let right = rest - d;
                    let at = slice_x(PUSHED) + BLOCK_W / 2.0 + d + right / 2.0;
                    let _ = s.set_points(gaps[PUSHED], &scaled(&unit, at, TUBE_Y, right));
                    let (c, o) = gap_paint(right / rest);
                    s.set_fill(gaps[PUSHED], Some(c), Some(o), None, false);
                    // the gap to its left
                    let left = rest + d;
                    let at = slice_x(PUSHED - 1) + BLOCK_W / 2.0 + left / 2.0;
                    let _ = s.set_points(gaps[PUSHED - 1], &scaled(&unit, at, TUBE_Y, left));
                    let (c, o) = gap_paint(left / rest);
                    s.set_fill(gaps[PUSHED - 1], Some(c), Some(o), None, false);
                },
                true,
            )?;
        }
        stage.add_to_scene(anchor)?;
        let slice_tag = kit.text(stage, "one slice of air", 26.0, dim, v(slice_x(PUSHED), TUBE_Y - 1.15))?;
        let squeezed_tag = kit.text(
            stage,
            "squeezed",
            30.0,
            squeezed(),
            v(slice_x(PUSHED) + 1.15, TUBE_Y + 1.15),
        )?;
        let thinned_tag = kit.text(
            stage,
            "thinned out",
            30.0,
            stretched(),
            v(slice_x(PUSHED) - 1.45, TUBE_Y + 1.15),
        )?;
        fade_in_all(stage, &[outline], 0.7)?;
        line.wait_for(stage, "Cut the air")?;
        fade_in_lagged(stage, &blocks, 1.3, 0.06)?;
        line.wait_for(stage, "each with mass")?;
        fade_in_all(stage, &[slice_tag], 0.4)?;
        line.wait_for(stage, "squeezed air pushes back")?;
        fade_in_all(stage, &[squeezed_tag, thinned_tag], 0.4)?;
        play!(stage; tween_linear(push, PUSH_MAX, 0.9)?);
        line.wait_for(stage, "like a spring")?;
        play!(stage; tween_linear(push, 0.0, 1.0)?);
        kit.hold(stage)?;

        // ---- 09_2: so a tube of air is a row of carts and springs; its table
        let line = say(&kit, stage, "09_2")?;
        let mut leaving = blocks.clone();
        leaving.extend([outline, slice_tag, squeezed_tag, thinned_tag]);
        fade_out_all(stage, &leaving, 0.5)?;
        stage.remove_from_scene(anchor);
        let mut cart_mobs: Vec<Mob> = Vec::new();
        let mut springs: Vec<Mob> = Vec::new();
        for i in 0..CARTS {
            // as chapter 02's carts: a rounded box, outlined in ink
            let body = Rectangle::new()
                .width(CART_W)
                .height(CART_H)
                .corner_radius(0.06)
                .style(Style::default().fill(panel(), 1.0).stroke(ink, 2.5, 1.0))
                .build()?
                .moved_to(v(cart_x(i), CART_Y));
            cart_mobs.push(stage.arena_mut().add(body));
        }
        // a spring to each neighbour, and one to each wall
        let ends: Vec<f64> = std::iter::once(CART_X0 - 0.15)
            .chain((0..CARTS).map(cart_x))
            .chain(std::iter::once(CART_X1 + 0.15))
            .collect();
        for w in ends.windows(2) {
            let (xa, xb) = (w[0], w[1]);
            let inner_a = if xa < CART_X0 { xa } else { xa + CART_W / 2.0 };
            let inner_b = if xb > CART_X1 { xb } else { xb - CART_W / 2.0 };
            let s = polyline(carts::zigzag(inner_a, inner_b, CART_Y, 3, 0.1))
                .style(faded_line(ink, 4.0, 1.0))
                .build();
            springs.push(stage.arena_mut().add(s));
        }
        let mut row = cart_mobs.clone();
        row.extend(&springs);
        fade_in_all(stage, &row, 0.8)?;
        let entries = table_entries(CARTS);
        let table_at = v(-1.2, -0.5);
        let grid = MatrixGrid::build(&kit, stage, &entries, 30.0, ink, table_at)?;
        for (r, row) in entries.iter().enumerate() {
            for (c, e) in row.iter().enumerate() {
                if e == "0" {
                    stage.set_color(grid.cells[r][c], dim, None, true);
                }
            }
        }
        let same = kit.text(stage, "the same table as before,\nonly longer", 28.0, dim, v(4.6, -0.5))?;
        line.wait_for(stage, "a row of carts")?;
        fade_in_all(stage, &[grid.group, same], 0.8)?;
        kit.hold(stage)?;
        // grow to 40 by 40, as flat cells
        let n = fact("K-R1.slices")?
            .parse::<usize>()
            .map_err(|e| fail(format!("K-R1.slices: {e}")))?;
        let cell = 4.0 / n as f64;
        let big = chain(n);
        let colours: Vec<Vec<Srgb>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| match big.at(i, j) as i64 {
                        2 => ink,
                        -1 => dim,
                        _ => panel(),
                    })
                    .collect()
            })
            .collect();
        let strips = RowStrips::build(stage, v(table_at[0] - 2.0, table_at[1] + 2.0), (cell, cell), &colours)?;
        let forty = kit.text(
            stage,
            &format!("{n} slices: a {n} by {n} table"),
            28.0,
            ink,
            v(4.6, -0.5),
        )?;
        fade_out_all(stage, &[grid.group, same], 0.5)?;
        fade_in_all(stage, &[strips.group, forty], 0.9)?;
        kit.hold(stage)?;

        // ---- 09_3: the arrow test on the air
        let line = say(&kit, stage, "09_3")?;
        let mut leaving = row.clone();
        leaving.extend([strips.group, forty]);
        fade_out_all(stage, &leaving, 0.5)?;
        let shape_at = stage.add_value_tracker(0.0);
        let veil = stage.add_value_tracker(1.0);
        let base = polyline(vec![v(CURVE_X0, CURVE_Y), v(CURVE_X1, CURVE_Y)])
            .style(faded_line(Palette::faint(), 2.0, 1.0))
            .build();
        let base = stage.arena_mut().add(base);
        let (c0, f0) = curves(0);
        let move_curve = stage
            .arena_mut()
            .add(polyline(c0).style(faded_line(moves(), 9.0, 1.0)).build());
        let push_curve = stage
            .arena_mut()
            .add(polyline(f0).style(faded_line(pushes(), 4.0, 1.0)).build());
        let drawn = stage.arena_mut().add(VMobject::new());
        {
            let mut shown = usize::MAX;
            let mut seen = f64::NAN;
            stage.add_updater(
                drawn,
                move |s, _| {
                    let k = which(s.tracker_value(shape_at).unwrap_or(0.0));
                    let a = s.tracker_value(veil).unwrap_or(1.0);
                    if k != shown {
                        let (mv, ps) = curves(k);
                        let _ = s.set_points(move_curve, &corners(&mv));
                        let _ = s.set_points(push_curve, &corners(&ps));
                        // lined up: the move is its own push, and goes gold
                        let c = if turn_of(&shape(k)) < 0.05 { own() } else { moves() };
                        s.set_stroke(move_curve, Some(c), None, None, None, false);
                        shown = k;
                    }
                    if a.to_bits() != seen.to_bits() {
                        s.set_stroke(move_curve, None, None, Some(a), None, false);
                        s.set_stroke(push_curve, None, None, Some(a), None, false);
                        seen = a;
                    }
                },
                true,
            )?;
        }
        let angle = kit.live_value(
            stage,
            move |s| turn_of(&shape(which(s.tracker_value(shape_at).unwrap_or(0.0)))),
            0,
            false,
            54.0,
            ink,
            v(4.6, 2.9),
            RIGHT,
        )?;
        let degrees = kit.text(stage, "\u{b0}", 54.0, ink, v(4.85, 2.95))?;
        let angle_words = kit.text(stage, "the push is turned from the move by", 26.0, dim, v(3.2, 3.55))?;
        let blue_words = kit.text(
            stage,
            "moves of the slices",
            28.0,
            moves(),
            v(-4.1, CURVE_Y + CURVE_H + 0.6),
        )?;
        let orange_words = kit.text(
            stage,
            "the pushes that hold them",
            28.0,
            pushes(),
            v(-4.1, CURVE_Y + CURVE_H + 1.1),
        )?;
        let same_height = kit.text(stage, "both drawn to the same height", 24.0, dim, v(-3.6, -1.1))?;
        stage.add_to_scene(drawn)?;
        fade_in_all(
            stage,
            &[
                base,
                move_curve,
                push_curve,
                angle_words,
                degrees,
                blue_words,
                orange_words,
                same_height,
            ],
            0.7,
        )?;
        let humps = [
            kit.text(stage, "one hump: half of a sine wave", 34.0, own(), v(0.0, -2.0))?,
            kit.text(stage, "two humps", 34.0, own(), v(0.0, -2.0))?,
            kit.text(stage, "three humps", 34.0, own(), v(0.0, -2.0))?,
        ];
        let own_words = kit.text(
            stage,
            "an own pattern: it comes back as itself",
            28.0,
            own(),
            v(0.0, -2.7),
        )?;
        // the next shape: the curves fade, the shape changes while they are
        // gone, and they come back drawn from it
        line.wait_for(stage, "One smooth hump")?;
        play!(stage; tween_linear(veil, 0.0, 0.25)?);
        stage.set_tracker_value(shape_at, 1.0)?;
        play!(stage; tween_linear(veil, 1.0, 0.35)?);
        fade_in_all(stage, &[humps[0], own_words], 0.4)?;
        line.wait_for(stage, "two, three")?;
        play!(stage; tween_linear(veil, 0.0, 0.2)?);
        stage.set_tracker_value(shape_at, 2.0)?;
        fade_out_all(stage, &[humps[0]], 0.1)?;
        play!(stage; tween_linear(veil, 1.0, 0.3)?);
        fade_in_all(stage, &[humps[1]], 0.3)?;
        line.wait_for(stage, "three, each")?;
        play!(stage; tween_linear(veil, 0.0, 0.2)?);
        stage.set_tracker_value(shape_at, 3.0)?;
        fade_out_all(stage, &[humps[1]], 0.1)?;
        play!(stage; tween_linear(veil, 1.0, 0.3)?);
        // the note that says how good the 40-slice table is comes with the
        // third hump, which the voice reaches in the line's last second, and
        // it stands through the pulse of 09_4 (it used to come in after the
        // line and go 0.5 s later; with the third hump it was up 1.4 s, too
        // few for eight words and a number)
        let bound = kit.text(
            stage,
            &format!(
                "{} slices give the lowest tone to within {}%",
                fact("K-R1.slices")?,
                fact("K-R1.error_bound_percent")?
            ),
            26.0,
            dim,
            v(0.0, -3.4),
        )?;
        fade_in_all(stage, &[humps[2], bound], 0.3)?;
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "09_3", "bound", stage.scene().time().to_f64());
        kit.hold(stage)?;

        // ---- 09_4: a round trip is five metres; how many a second?
        let line = say(&kit, stage, "09_4")?;
        fade_out_all(
            stage,
            &[
                base,
                move_curve,
                push_curve,
                angle,
                degrees,
                angle_words,
                blue_words,
                orange_words,
                same_height,
                humps[2],
                own_words,
            ],
            0.5,
        )?;
        stage.remove_from_scene(drawn);
        // a pulse of pressed air runs down a tube, bounces off the wall and comes back
        let tube = stage.arena_mut().add(
            Rectangle::new()
                .width(TUBE_X1 - TUBE_X0 + 0.3)
                .height(0.8)
                .corner_radius(0.12)
                .style(faded_line(dim, 3.0, 1.0))
                .build()?
                .moved_to(v(0.0, PULSE_TUBE_Y)),
        );
        let pulse_unit = Rectangle::new().width(1.0).height(0.6).build()?.points().to_vec();
        let pulse = stage.arena_mut().add(
            Rectangle::new()
                .width(0.35)
                .height(0.6)
                .style(flat(squeezed(), 0.9))
                .build()?
                .moved_to(v(TUBE_X0 + 0.3, PULSE_TUBE_Y)),
        );
        let run = stage.add_value_tracker(0.0);
        let pulse_anchor = stage.arena_mut().add(VMobject::new());
        {
            let mut last = f64::NAN;
            stage.add_updater(
                pulse_anchor,
                move |s, _| {
                    let t = s.tracker_value(run).unwrap_or(0.0);
                    if t.to_bits() == last.to_bits() {
                        return;
                    }
                    last = t;
                    // out to the far wall and back
                    let span = TUBE_X1 - TUBE_X0 - 0.7;
                    let x = if t <= 1.0 {
                        TUBE_X0 + 0.35 + span * t
                    } else {
                        TUBE_X1 - 0.35 - span * (t - 1.0)
                    };
                    let _ = s.set_points(pulse, &scaled(&pulse_unit, x, PULSE_TUBE_Y, 0.35));
                },
                true,
            )?;
        }
        stage.add_to_scene(pulse_anchor)?;
        fade_in_all(stage, &[tube, pulse], 0.5)?;
        play!(stage; tween_linear(run, 2.0, 3.6)?);
        // the card: a room, the speed of sound, the round trip, the question
        let sides = numbers("K-R2.sides_m")?;
        let long = show(sides[0], 1)?;
        let round = show(2.0 * sides[0], 0)?;
        let card = QuizCard::build(&kit, stage, "09_4", CARD_AT, CARD_SIZE)?;
        let (centre, _, _) = card.content();
        let room_w = 1.7;
        let room_h = room_w * sides[1] / sides[0];
        let room_at = v(centre[0] - 2.5, centre[1] + 0.1);
        let room = stage.arena_mut().add(
            Rectangle::new()
                .width(room_w)
                .height(room_h)
                .style(faded_line(ink, 3.0, 1.0))
                .build()?
                .moved_to(room_at),
        );
        let there = arrow(
            stage,
            v(room_at[0] - 0.65, room_at[1] + 0.17),
            v(room_at[0] + 0.65, room_at[1] + 0.17),
            ink,
            4.0,
        )?;
        let back = arrow(
            stage,
            v(room_at[0] + 0.65, room_at[1] - 0.17),
            v(room_at[0] - 0.65, room_at[1] - 0.17),
            ink,
            4.0,
        )?;
        let length = kit.text(
            stage,
            &format!("{long} m"),
            28.0,
            ink,
            v(room_at[0], room_at[1] - room_h / 2.0 - 0.35),
        )?;
        let speed = kit.text_left(
            stage,
            &format!("sound: {} m a second", fact("K-R0.sound_m_per_s")?),
            30.0,
            ink,
            v(centre[0] - 0.9, centre[1] + 0.7),
        )?;
        let trip = kit.text_left(
            stage,
            &format!("a round trip: 2 \u{d7} {long} = {round} m"),
            30.0,
            ink,
            v(centre[0] - 0.9, centre[1] + 0.2),
        )?;
        let ask = kit.text_left(
            stage,
            "round trips a second?",
            30.0,
            ink,
            v(centre[0] - 0.9, centre[1] - 0.3),
        )?;
        let problem = [room, there, back, length, speed, trip, ask];
        line.wait_sentence(stage, 1)?;
        fade_out_all(stage, &[tube, pulse, bound], 0.4)?;
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "09_3", "bound_gone", stage.scene().time().to_f64());
        stage.remove_from_scene(pulse_anchor);
        card.show_frame(stage, &problem, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 09_5: about seventy; the room's own tones
        let line = say(&kit, stage, "09_5")?;
        let answer_text = format!(
            "{} \u{f7} {round} = {} round trips a second",
            fact("K-R0.sound_m_per_s")?,
            fact("K-Q.09_4.answer")?
        );
        // the answer lands under the question, where the card's problem stood
        let answer = kit.text_left(
            stage,
            &answer_text,
            28.0,
            own(),
            v(card.content().0[0] - 0.9, card.content().0[1] - 0.8),
        )?;
        card.land(stage, &[answer], 0.3)?;
        // the tones: nine, of which five fit one side
        let tones = numbers("K-R2.tones_to_150_hz")?;
        let one_side: Vec<f64> = ONE_SIDE
            .iter()
            .map(|(hz, _)| hz.parse::<f64>().unwrap_or(0.0))
            .collect();
        let pitch_line = stage.arena_mut().add(
            polyline(vec![v(PITCH_X0 - 0.2, PITCH_Y), v(PITCH_X1 + 0.2, PITCH_Y)])
                .style(faded_line(dim, 4.0, 1.0))
                .build(),
        );
        let mut ticks = Vec::new();
        let mut tags: Vec<Mob> = Vec::new();
        for (i, &hz) in tones.iter().enumerate() {
            let x = pitch_x(hz);
            let labelled = one_side.iter().any(|h| (h - hz).abs() < 0.05);
            let o = if labelled { 1.0 } else { 0.45 };
            let t = polyline(vec![v(x, PITCH_Y - 0.14), v(x, PITCH_Y + 0.14)])
                .style(faded_line(dim, 5.0, o))
                .build();
            ticks.push(stage.arena_mut().add(t));
            // the tone's number above its tick, on two levels so that
            // neighbours never touch
            let level = if i % 2 == 0 { 0.4 } else { 0.8 };
            let number = kit.text(
                stage,
                &show(hz, 1)?,
                24.0,
                if labelled { ink } else { dim },
                v(x, PITCH_Y + level),
            )?;
            tags.push(number);
        }
        for (k, (hz, side)) in ONE_SIDE.iter().enumerate() {
            let hz: f64 = hz.parse().unwrap_or(0.0);
            let x = pitch_x(hz);
            let y = if k % 2 == 0 { PITCH_Y - 0.55 } else { PITCH_Y - 1.0 };
            let tag = kit.text(stage, side, 24.0, ink, v(x, y))?;
            tags.push(tag);
        }
        let cap = |x: f64| x.clamp(PITCH_LO, PITCH_HI);
        let hz_now = stage.add_value_tracker(PITCH_LO);
        let rate = stage.add_value_tracker(0.0);
        let room_outline = stage.arena_mut().add(
            Rectangle::new()
                .width(ROOM_W)
                .height(ROOM_H)
                .style(faded_line(dim, 4.0, 1.0))
                .build()?
                .moved_to(v(ROOM_AT[0], ROOM_AT[1])),
        );
        let room_glow = stage.arena_mut().add(
            Rectangle::new()
                .width(ROOM_W + 0.12)
                .height(ROOM_H + 0.12)
                .style(faded_line(own(), 7.0, 0.0))
                .build()?
                .moved_to(v(ROOM_AT[0], ROOM_AT[1])),
        );
        let marker = stage.arena_mut().add(
            polyline(vec![
                v(pitch_x(PITCH_LO), PITCH_Y - 0.4),
                v(pitch_x(PITCH_LO), PITCH_Y + 0.4),
            ])
            .style(faded_line(moves(), 7.0, 1.0))
            .build(),
        );
        let lit: Vec<Mob> = ticks.clone();
        let tone_list = tones.clone();
        let hummer = stage.arena_mut().add(VMobject::new());
        {
            let mut passed = vec![false; tone_list.len()];
            let mut last_glow = f64::NAN;
            let mut last_hz = f64::NAN;
            stage.add_dt_updater(
                hummer,
                move |s, _, dt| {
                    let mut hz = s.tracker_value(hz_now).unwrap_or(PITCH_LO);
                    let r = s.tracker_value(rate).unwrap_or(0.0);
                    if dt > 0.0 && r > 0.0 {
                        hz = cap(hz + r * dt);
                        let _ = s.set_tracker_value(hz_now, hz);
                    }
                    if hz.to_bits() != last_hz.to_bits() {
                        let x = pitch_x(hz);
                        let _ = s.set_points(marker, &corners(&[v(x, PITCH_Y - 0.4), v(x, PITCH_Y + 0.4)]));
                        last_hz = hz;
                        for (i, &t) in tone_list.iter().enumerate() {
                            if !passed[i] && hz >= t {
                                passed[i] = true;
                                s.set_stroke(lit[i], Some(own()), None, None, None, false);
                            }
                        }
                    }
                    // the outline glows while the pitch is within a hertz or so of a tone
                    let near = tone_list
                        .iter()
                        .map(|t| (1.0 - (hz - t).abs() / GLOW_HZ).max(0.0))
                        .fold(0.0f64, f64::max);
                    if near.to_bits() != last_glow.to_bits() {
                        s.set_stroke(room_glow, None, None, Some(near), None, false);
                        last_glow = near;
                    }
                },
                true,
            )?;
        }
        stage.add_to_scene(hummer)?;
        // a figure humming in the room, and the sound it makes
        let fig = v(ROOM_AT[0] + 0.6, ROOM_AT[1] - 0.45);
        let mut figure: Vec<Mob> = Vec::new();
        figure.push(
            stage.arena_mut().add(
                Dot::new()
                    .point(v(fig[0], fig[1] + 0.62))
                    .radius(0.16)
                    .style(flat(ink, 1.0))
                    .build(),
            ),
        );
        for pts in [
            vec![v(fig[0], fig[1] + 0.45), v(fig[0], fig[1] - 0.2)],
            vec![
                v(fig[0] - 0.28, fig[1] + 0.12),
                v(fig[0], fig[1] + 0.3),
                v(fig[0] + 0.28, fig[1] + 0.12),
            ],
            vec![
                v(fig[0] - 0.22, fig[1] - 0.62),
                v(fig[0], fig[1] - 0.2),
                v(fig[0] + 0.22, fig[1] - 0.62),
            ],
        ] {
            figure.push(
                stage
                    .arena_mut()
                    .add(polyline(pts).style(faded_line(ink, 4.0, 1.0)).build()),
            );
        }
        for (k, r) in [0.3, 0.46, 0.62].iter().enumerate() {
            let a0 = -0.6f64;
            let pts: Vec<Vec3> = (0..=10)
                .map(|j| {
                    let a = a0 + 1.2 * j as f64 / 10.0;
                    v(fig[0] - 0.16 + r * a.cos(), fig[1] + 0.62 + r * a.sin())
                })
                .collect();
            figure.push(
                stage.arena_mut().add(
                    polyline(pts)
                        .style(faded_line(moves(), 3.0, 0.9 - 0.25 * k as f64))
                        .build(),
                ),
            );
        }
        let side_labels = [
            kit.text(
                stage,
                &format!("{} m", show(sides[0], 1)?),
                24.0,
                dim,
                v(ROOM_AT[0], ROOM_AT[1] - ROOM_H / 2.0 - 0.3),
            )?,
            kit.text(
                stage,
                &format!("{} m", show(sides[1], 1)?),
                24.0,
                dim,
                v(ROOM_AT[0] - ROOM_W / 2.0 - 0.45, ROOM_AT[1]),
            )?,
        ];
        let height_note = kit.text(
            stage,
            &format!("{} m high", show(sides[2], 1)?),
            24.0,
            dim,
            v(ROOM_AT[0] - 0.8, ROOM_AT[1] + 0.55),
        )?;
        let first = kit.text(
            stage,
            &format!(
                "{} \u{f7} {round} = {} Hz",
                fact("K-R0.sound_m_per_s")?,
                fact("K-R2.first_hz")?
            ),
            40.0,
            ink,
            v(-4.7, 3.4),
        )?;
        let hum_unit = kit.text(stage, "Hz", 30.0, dim, v(6.0, 2.0))?;
        let hum_words = kit.text(stage, "the hum is at", 26.0, dim, v(4.7, 2.6))?;
        line.wait_for(stage, "Sixty-eight point six hertz")?;
        let mut leaving = card.mobs();
        leaving.extend(problem);
        leaving.push(answer);
        fade_out_all(stage, &leaving, 0.4)?;
        // (a live number is in the scene as it is built; the end of the chapter fades it)
        let _ = kit.live_number(stage, hz_now, 0, false, 40.0, ink, v(5.4, 2.0), RIGHT)?;
        let mut stage_in = vec![pitch_line, room_outline, room_glow, marker, first, hum_unit, hum_words];
        stage_in.extend(&ticks);
        stage_in.extend(&tags);
        stage_in.extend(&side_labels);
        stage_in.push(height_note);
        fade_in_all(stage, &stage_in, 0.7)?;
        // `room_glow` is faded in at zero opacity and the updater lights it
        line.wait_for(stage, "But hum a slow slide")?;
        fade_in_all(stage, &figure, 0.4)?;
        let t_hum = line.at("But hum a slow slide")?;
        let t_back = line.at("the room hums back")?;
        // 60 Hz to the 107.2 Hz tone as the voice says the room hums back
        let to_107 = 107.2 - PITCH_LO;
        stage.set_tracker_value(rate, to_107 / (t_back - t_hum).max(0.5))?;
        line.wait_for(stage, "the room hums back")?;
        // the rest of the slide, to 150 Hz, takes the line's end and the 2 s listen window
        let now = stage.scene().time().to_f64();
        let left = (line.end() + 2.0 - now).max(1.0);
        stage.set_tracker_value(rate, (PITCH_HI - 107.2) / left)?;
        line.wait_end(stage)?;
        // a small card: Helmholtz's word for it, and Hilbert's
        let helmholtz = kit.text(
            stage,
            "\u{201e}Ein Luftraum \u{2026} hat gewisse Eigent\u{f6}ne\u{201c}",
            28.0,
            ink,
            v(-3.2, -2.75),
        )?;
        let who = kit.text(
            stage,
            &format!("Helmholtz, {}", fact("K-S.helmholtz_year")?),
            24.0,
            dim,
            v(-3.2, -3.2),
        )?;
        let gloss = kit.text(
            stage,
            "an enclosed volume of air has tones of its own",
            22.0,
            dim,
            v(-3.2, -3.6),
        )?;
        let hilbert = kit.text(
            stage,
            &format!("Eigenwerte: Hilbert, {}", fact("K-S.hilbert_year")?),
            28.0,
            ink,
            v(3.4, -2.75),
        )?;
        let years = kit.text(
            stage,
            &format!("the same word, {} years apart", fact("K-S.eigen_gap_years")?),
            24.0,
            dim,
            v(3.4, -3.2),
        )?;
        let backing = stage.arena_mut().add(
            Rectangle::new()
                .width(13.2)
                .height(1.45)
                .corner_radius(0.12)
                .style(Style::default().fill(panel(), 0.95).stroke(dim, 2.0, 1.0))
                .build()?
                .moved_to(v(0.0, -3.1)),
        );
        let card_group = group(stage, &[backing, helmholtz, who, gloss, hilbert, years])?;
        // it comes in as the listen window opens and is read through it
        fade_in_all(stage, &[card_group], 0.6)?;
        // the window opens; the hum is heard sliding on from where the line
        // is to 150 Hz, loud at each tone it crosses (the outline's glow),
        // and the pitch line runs the window's whole length to match it
        kit.hold(stage)?;
        let from_hz = stage.tracker_value(hz_now).unwrap_or(PITCH_LO);
        let wav = sound::room_hum(from_hz, PITCH_HI, &tones, GLOW_HZ)?;
        kit.sonify(stage, sound::ROOM_S, &format!("{}:09_5", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        stage.set_tracker_value(rate, ((PITCH_HI - from_hz) / sound::ROOM_S).max(0.0))?;
        wait_until(stage, start + sound::ROOM_S)?;
        stage.set_tracker_value(rate, 0.0)?;
        pause(stage, 1.5)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, items, testing};
    use super::*;
    use crate::model::rooms::{BATHROOM, SOUND_M_PER_S};

    /// 09_3: one slice gives a push turned 35.3 degrees from it (the fact),
    /// and one, two or three humps of a sine wave come back as themselves,
    /// to the last decimal: the angle is zero.
    #[test]
    fn one_slice_turns_and_a_smooth_hump_comes_back_as_itself() {
        let lump = turn_of(&shape(0));
        assert_eq!(show(lump, 1).unwrap(), fact("K-R1.lump_turn_deg").unwrap());
        // the storyboard's "35 degrees" and the live number's zero decimals
        assert_eq!(show(lump, 0).unwrap(), "35");
        for k in 1..=3 {
            let u = shape(k);
            assert!(turn_of(&u) < 1e-3, "{k} humps: {}", turn_of(&u));
            // the push is a fixed multiple of the move: its own eigenvalue
            let f = pushes_of(&u);
            let lambda = 2.0 - 2.0 * (k as f64 * std::f64::consts::PI / (AIR as f64 + 1.0)).cos();
            for (a, b) in u.iter().zip(&f) {
                assert!((lambda * a - b).abs() < 1e-9, "{k}");
            }
        }
        // a blend of the spike and the hump is turned far more than either:
        // 80 degrees half way, which is why the picture swaps and never blends
        let blend: Vec<f64> = shape(0).iter().zip(shape(1)).map(|(a, b)| 0.5 * a + 0.5 * b).collect();
        assert!(turn_of(&blend) > 70.0, "{}", turn_of(&blend));
        assert_eq!((which(0.2), which(0.7), which(2.9), which(7.0)), (0, 1, 3, 3));
        assert_eq!(AIR.to_string(), fact("K-R1.slices").unwrap());
        // both curves reach the same height, and start and end on the walls
        let (m, p) = curves(0);
        let top = |c: &[Vec3]| c.iter().fold(0.0f64, |s, q| s.max((q[1] - CURVE_Y).abs()));
        assert!((top(&m) - CURVE_H).abs() < 1e-9 && (top(&p) - CURVE_H).abs() < 1e-9);
        assert_eq!(m.len(), AIR + 2);
        // what `set_points` is given is a quad path: an odd run, anchors at its even places
        let quad = corners(&m);
        assert_eq!(quad.len(), 2 * m.len() - 1);
        assert_eq!(quad[0], m[0]);
        assert_eq!(quad[2], m[1]);
        assert_eq!(quad[quad.len() - 1], m[m.len() - 1]);
        assert_eq!(m[0][1], CURVE_Y);
        assert_eq!(m[AIR + 1][1], CURVE_Y);
    }

    /// 09_1: the pushed slice and the gaps beside it fit between the walls;
    /// a gap never closes or flips, and the squeezed side is the side the
    /// slice moves toward.
    #[test]
    fn the_pushed_slice_squeezes_one_gap_and_thins_the_other() {
        let rest = PERIOD - BLOCK_W;
        assert!(PUSH_MAX < rest, "a push never closes the gap");
        let (right, left) = (gap_paint((rest - PUSH_MAX) / rest), gap_paint((rest + PUSH_MAX) / rest));
        assert_eq!(right.0, squeezed());
        assert_eq!(left.0, stretched());
        assert!(right.1 > 0.5 && left.1 > 0.5);
        assert_eq!(gap_paint(1.0).1, 0.0);
        assert!((slice_x(0) - (TUBE_X0 + PERIOD / 2.0)).abs() < 1e-12);
        assert!((slice_x(SLICES - 1) - (TUBE_X1 - PERIOD / 2.0)).abs() < 1e-12);
    }

    /// 09_2: eight carts have a table with 2 down the diagonal, −1 on each
    /// side and zeros elsewhere; the long table is the same pattern.
    #[test]
    fn the_table_of_the_tube_is_two_down_the_diagonal_and_minus_one_beside_it() {
        let t = table_entries(CARTS);
        assert_eq!(t.len(), CARTS);
        for (i, row) in t.iter().enumerate() {
            for (j, e) in row.iter().enumerate() {
                let want = match i.abs_diff(j) {
                    0 => "2",
                    1 => "-1",
                    _ => "0",
                };
                assert_eq!(e, want, "({i}, {j})");
            }
        }
        assert!(cart_x(0) > CART_X0 && cart_x(CARTS - 1) < CART_X1);
        // a spring is a lead, `2 x coils` teeth and a lead
        assert_eq!(carts::zigzag(0.0, 1.0, 0.0, 3, 0.1).len(), 2 + 2 * 3 + 2);
    }

    /// 09_4 and 09_5: a bathroom 2.5 m long has a round trip of 5 m, so
    /// 343 / 5 = 68.6 round trips a second (the quiz and its first tone);
    /// the five tones the card labels are exactly those of the room's
    /// tones that fit one side, and every one of the nine is a mode.
    #[test]
    fn the_quiz_and_the_pitch_line_say_what_the_model_says() {
        let sides = numbers("K-R2.sides_m").unwrap();
        assert_eq!(sides[0], 2.5);
        assert_eq!(
            show(SOUND_M_PER_S / (2.0 * sides[0]), 1).unwrap(),
            fact("K-Q.09_4.answer").unwrap()
        );
        assert_eq!(fact("K-Q.09_4.answer").unwrap(), fact("K-R2.first_hz").unwrap());
        let tones = numbers("K-R2.tones_to_150_hz").unwrap();
        assert_eq!(tones.len(), 9);
        assert!(tones[0] > PITCH_LO && tones[8] < PITCH_HI);
        // every labelled tone is a tone of the list, on a side it names
        let by_side: [(&str, usize); 3] = [("length 2.5 m", 0), ("width 1.6 m", 1), ("height 2.4 m", 2)];
        for (hz, side) in ONE_SIDE {
            let want: f64 = hz.parse().unwrap();
            assert!(tones.iter().any(|t| (t - want).abs() < 0.05), "{hz}");
            let axis = by_side.iter().find(|(s, _)| *s == side).unwrap().1;
            // a mode that is humps along that side only: n halves of c / 2L
            let n = (1..=3u32)
                .find(|n| {
                    let mut idx = [0u32; 3];
                    idx[axis] = *n;
                    (BATHROOM.mode_hz(idx) - want).abs() < 0.05
                })
                .unwrap_or(0);
            assert!(n >= 1, "{hz} Hz is not a mode along its labelled side ({side})");
        }
        assert_eq!(items("K-R2.voice_hz").unwrap(), ["107.2", "137.2"]);
        // the pitch line keeps the tones in order and inside the frame
        assert!(pitch_x(PITCH_LO) < pitch_x(PITCH_HI));
        assert!(pitch_x(tones[0]) > -6.8 && pitch_x(tones[8]) < 6.8);
    }

    /// The chapter, constructed with the layout lint on, and the section 5
    /// audit of its quiz on a scratch-voice run.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("09_the_air_in_a_room").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "one slice of air",
            "squeezed",
            "thinned out",
            "40 slices: a 40 by 40 table",
            "40 slices give the lowest tone to within 0.03%",
            "sound: 343 m a second",
            "a round trip: 2 \u{d7} 2.5 = 5 m",
            "round trips a second?",
            "343 \u{f7} 5 = 68.6 round trips a second",
            "343 \u{f7} 5 = 68.6 Hz",
            "length 2.5 m",
            "height 2.4 m",
            "width 1.6 m",
            "146.0",
            "Helmholtz, 1863",
            "Eigenwerte: Hilbert, 1904",
            "the same word, 41 years apart",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never sets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 110.0, "{} s", ran.seconds);
    }

    #[test]
    fn the_quiz_is_asked_and_counted_down_as_section_five_asks() {
        let a = testing::audit_quiz("09_4").unwrap();
        assert!((a.window.1 - a.window.0 - a.seconds).abs() <= 0.15, "{a:?}");
        assert_eq!(a.seconds, 6.0);
        assert!(a.card_shown <= a.question_ends, "{a:?}");
        assert!(a.ring_armed <= a.question_ends, "{a:?}");
        assert!(a.window.0 >= a.question_ends - 1e-9, "{a:?}");
        assert!(a.spoken_inside.is_empty(), "{a:?}");
        assert!(a.answer_lands >= a.window.1 - 1e-9, "{a:?}");
        if let Some((next, at)) = &a.next_line {
            assert!(*at >= a.window.1 - 0.05, "{next} starts inside the countdown");
        }
    }

    /// 09_3: the note that says how good the 40-slice table is (within
    /// 0.03%) comes in with the third hump, while the line is still being
    /// said, and stands through the pulse of 09_4 (the dense read, P5). It
    /// used to come in after the line and be gone half a second later; keyed
    /// to the third hump alone it was up 1.4 s on the real narration (26.3 to
    /// 27.7 s of the f63a276 draft, "three, each" being the line's last
    /// second), too few for eight words and a number.
    ///
    /// The scratch voice this test runs on places "three, each" by its share
    /// of the sentence's characters, about as late as the real one does.
    #[test]
    fn the_tables_error_note_stands_for_the_end_of_the_line() {
        let voiced = testing::run_voiced("09_the_air_in_a_room").unwrap();
        let events = super::super::draw::quiz_log::take(&voiced.tag, "09_3");
        let at = events.iter().find(|e| e.0 == "bound").expect("the note was shown").1;
        let (_, start, len) = voiced.lines.iter().find(|l| l.0 == "09_3").cloned().unwrap();
        let next = voiced.lines.iter().find(|l| l.0 == "09_4").unwrap().1;
        assert!(at >= start, "the note is up before its line starts");
        assert!(at < start + len, "the note comes in at {at:.2}, after its line");
        assert!(next - at >= 1.0, "the note stands {:.2} s", next - at);
        // and it is not taken out with the line's end: it goes with the
        // pulse's tube, 4 s or more into 09_4
        let gone = events.iter().find(|e| e.0 == "bound_gone").expect("the note went").1;
        assert!(gone - at >= 4.0, "the note stands {:.2} s in all", gone - at);
        assert!(
            gone >= next + 3.0,
            "the note goes at {gone:.2}, 09_4 starts at {next:.2}"
        );
    }
}
