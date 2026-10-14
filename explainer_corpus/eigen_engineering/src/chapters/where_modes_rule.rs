//! Chapter 11, where modes rule: a hall's sound is told by statistics, a
//! reverberation time from Sabine's rule (11_1); in a small room single modes
//! rule, and the audible bass is where the count is small (11_3); the quiz,
//! which of three rooms has several modes on exactly one tone, and the cube
//! that does (11_3 and 11_4); and a room that is not a box has no formula,
//! so its air is cut into cells, the table is built and its eigenvectors
//! asked for (11_5).
//!
//! Every number is a fact or the room model's own: the hall's sides, volume,
//! reverberation time and absorption are `K-H1.*` and `K-H4.*`, the small
//! room's `K-R4.*`, the three rooms of the quiz `K-R5.*`, the cube's first
//! tone `K-R3.*`, the attic's cells and modes `K-N1.*`, and the cited years
//! `K-S.*`. The modes drawn are the model's (`model::rooms`), counted in
//! whole numbers.

use explainer_kit::props::{RowStrips, lower_third, names_to_search};
use explainer_kit::{Kit, Palette, fade_all, play, thousands, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, QuizCard, corners, fact, fade_in_all, fade_in_lagged, fade_out_all, faded_line, ground, items,
    music_hold, numbers, own, panel, pause, say, show, squeezed, stretched, tween_linear, whole,
};
use crate::model::rooms::{CUBE, NARROW, Room, STUDY, attic_section};

pub struct WhereModesRule {
    pub kit: Kit,
}

// ---- 11_1: the hall, the crowd and the decay
/// Where the hall's plan sits (its lower-left corner) and how wide it is.
const HALL_LEFT: f64 = -6.4;
const HALL_BOTTOM: f64 = 0.15;
const HALL_W: f64 = 5.6;
/// The decay plot's frame.
const PLOT_LEFT: f64 = 1.2;
const PLOT_BOTTOM: f64 = 0.35;
const PLOT_W: f64 = 5.4;
const PLOT_H: f64 = 2.5;
/// How much longer the empty hall rings than the full one. Drawn, never
/// said or written: only the direction (more people, a shorter decay) is the
/// claim.
const EMPTY_STRETCH: f64 = 1.45;
const _: () = assert!(EMPTY_STRETCH > 1.0);
/// Rays in the hall, how many metres of each are drawn behind its head and
/// how fast the heads run (metres of path a second of the picture).
const RAYS: usize = 7;
const RAY_TRAIL_M: f64 = 9.0;
const RAY_SPEED: f64 = 14.0;
/// Segments in the trail of a ray (it has this many and one vertices).
const RAY_STEPS: usize = 12;
/// The seats' block, in rows and columns.
const SEAT_ROWS: usize = 7;
const SEAT_COLS: usize = 26;

// ---- 11_3 and 11_4: a small room's tones
/// The frequency line of 11_3 runs from 0 to this many hertz, from x0 to x1.
const LINE_MAX_HZ: f64 = 220.0;
const LINE_X: (f64, f64) = (-6.0, 6.0);
const LINE_Y: f64 = -0.2;
const PEAK_H: f64 = 1.5;
/// The quiz card, and the scale of the three rooms drawn on it (units a
/// metre).
const CARD_AT: Vec3 = [0.0, 0.35, 0.0];
const CARD_SIZE: (f64, f64) = (11.6, 5.5);
const ROOM_SCALE: f64 = 0.5;
/// How far the back of an outlined room sits behind its front, as a share of
/// its depth, right and up.
const OBLIQUE: (f64, f64) = (0.42, 0.3);
/// The three number lines of 11_4: where they run (0 to 200 Hz) and their
/// baselines.
const ROW_X: (f64, f64) = (-1.7, 6.4);
const ROW_Y: [f64; 3] = [1.75, 0.45, -0.85];
/// The tone the lines end at, in hertz, and how far under the lowest line
/// the scale's numbers stand.
const ROW_MAX_HZ: f64 = 200.0;
const SCALE_DROP: f64 = 0.4;
/// One stacked tick's height.
const TICK_H: f64 = 0.15;

// ---- 11_5: the attic
const CELL: f64 = 0.3;
const ATTIC_LEFT: f64 = -6.5;
const ATTIC_BOTTOM: f64 = -1.0;
const MOTES: usize = 300;
/// How far (in cells) a mote swings at the most, and the first mode's
/// period on screen (seconds).
const SWING: f64 = 0.4;
const BASE_PERIOD_S: f64 = 2.4;

// ---- the chapter's fixed words (every number in them comes from a fact)
const RULE_BIGGER: &str = "the bigger the room, the longer a sound lingers";
const RULE_SOAKS: &str = "and the more it soaks up, the shorter";
const SOUND_LEVEL: &str = "sound level";
const TIME: &str = "time";
const ABSORPTION: &str = "of absorption";
const CROSSOVER: &str = "crossover";
const A_MODEL: &str = "a model";
const MODES_BELOW: &str = "modes below";
const HZ: &str = "Hz";
const NARROW_NAME: &str = "a long narrow room";
const ORDINARY_NAME: &str = "an ordinary room";
const CUBE_NAME: &str = "a perfect cube";
const THE_CUBE: &str = "the cube";
const SHARE_A_TONE: &str = "share a tone";
const ROOMS_SOURCES: &str = "sources:";
const SHOEBOXES: &str = "three of the most admired concert halls are shoeboxes (Beranek); in a small room the trouble is \
                         sides that match or are in simple ratios (Walker, BBC 1993)";
const SIDE_VIEW: &str = "a side view: the room's width would add modes of its own";
const PROGRAMMERS: &str = "for programmers:";
const CELLS_TEXT: &str = "one unknown per cell; each pair of touching cells adds 1 to both diagonal entries and \
                          \u{2013}1 between them; a wall adds nothing; skip the first eigenvalue, 0";
const FRANKENSIM: &str = "FrankenSim's solver:";
const TABLE_CORNER: &str = "a corner of the table";
const MODE: &str = "mode";

/// `n` folded into `[0, len]`, as a ray bouncing between two parallel walls.
fn bounce(u: f64, len: f64) -> f64 {
    let m = u.rem_euclid(2.0 * len);
    if m <= len { m } else { 2.0 * len - m }
}

/// A fact that is a whole number, grouped in thousands for the screen.
fn grouped(key: &str) -> fmn::Result<String> {
    let n: u64 = fact(key)?
        .parse()
        .map_err(|e| super::draw::fail(format!("fact {key} is not a whole number: {e}")))?;
    Ok(thousands(n))
}

/// A straight segment as one flat polyline, detached.
fn seg(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(vec![a, b]).style(faded_line(color, width, 1.0)).build())
}

/// An open or closed outline through `points`, detached.
fn path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(faded_line(color, width, 1.0)).build())
}

/// The hall seen from above: its length and width in metres drawn at a
/// scale of units a metre.
struct Hall {
    left: f64,
    bottom: f64,
    scale: f64,
    length_m: f64,
    width_m: f64,
}

impl Hall {
    /// The hall of `K-H1.sides_m`, `HALL_W` units long.
    fn build() -> fmn::Result<Self> {
        let sides = numbers("K-H1.sides_m")?;
        Ok(Self {
            left: HALL_LEFT,
            bottom: HALL_BOTTOM,
            scale: HALL_W / sides[0],
            length_m: sides[0],
            width_m: sides[1],
        })
    }

    /// The screen point of the plan's point `(x, y)`, metres.
    fn at(&self, x: f64, y: f64) -> Vec3 {
        v(self.left + self.scale * x, self.bottom + self.scale * y)
    }

    fn height(&self) -> f64 {
        self.scale * self.width_m
    }

    /// The point a ray starting at the stage's edge, leaving at `angle`, has
    /// reached after `s` metres.
    fn ray_at(&self, angle: f64, s: f64) -> Vec3 {
        let (x0, y0) = (self.stage_edge_m(), self.width_m / 2.0);
        self.at(
            bounce(x0 + s * angle.cos(), self.length_m),
            bounce(y0 + s * angle.sin(), self.width_m),
        )
    }

    /// Where the stage ends and the seats begin, metres along the hall.
    fn stage_edge_m(&self) -> f64 {
        self.length_m * 0.14
    }
}

/// The angle of ray `i` of `RAYS`, spread either side of straight down the
/// hall.
fn ray_angle(i: usize) -> f64 {
    let spread = 1.25;
    -spread / 2.0 + spread * (i as f64 + 0.5) / RAYS as f64
}

/// The rays: one polyline each, a trail of the last `RAY_TRAIL_M` metres,
/// rewritten every frame by one updater on one anchor (the heads travel at
/// `RAY_SPEED` metres a second of the picture). Returns the anchor.
fn rays(stage: &mut Stage<'_>, hall: &Hall) -> fmn::Result<(Vec<Mob>, Mob)> {
    let mut trails = Vec::new();
    for i in 0..RAYS {
        let a = hall.ray_at(ray_angle(i), 0.0);
        // built with every vertex the updater will write (a speck at the
        // stage's edge): `set_points` does not take a run of another size,
        // and a ray built as two vertices and rewritten as thirteen never
        // appeared
        let speck: Vec<Vec3> = (0..=RAY_STEPS).map(|k| v(a[0] + 1e-4 * k as f64, a[1])).collect();
        trails.push(path(stage, speck, Palette::ink(), 3.0));
    }
    let anchor = stage.arena_mut().add(VMobject::new());
    let (length, width, left, bottom, scale) = (hall.length_m, hall.width_m, hall.left, hall.bottom, hall.scale);
    let edge = hall.stage_edge_m();
    let heads = trails.clone();
    let mut travelled = 0.0_f64;
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            if dt <= 0.0 {
                return;
            }
            travelled += RAY_SPEED * dt;
            for (i, &m) in heads.iter().enumerate() {
                let angle = ray_angle(i);
                let pts: Vec<Vec3> = (0..=RAY_STEPS)
                    .map(|k| {
                        let along = (travelled - RAY_TRAIL_M * k as f64 / RAY_STEPS as f64).max(0.0);
                        let x = bounce(edge + along * angle.cos(), length);
                        let y = bounce(width / 2.0 + along * angle.sin(), width);
                        v(left + scale * x, bottom + scale * y)
                    })
                    .collect();
                let _ = s.set_points(m, &corners(&pts));
            }
        },
        false,
    )?;
    Ok((trails, anchor))
}

/// The names of the attic's modes with mode `playing` (0 for the first) in
/// gold and the others in ink: which one is moving is what the names say.
fn light_the_playing(stage: &mut Stage<'_>, names: &[Mob], playing: usize) {
    for (k, &m) in names.iter().enumerate() {
        let colour = if k == playing { own() } else { Palette::ink() };
        stage.set_color(m, colour, None, true);
    }
}

/// How far from the empty seat's grey to the page's white a seat with a
/// person in it is drawn.
const SEAT_TINT: f64 = 0.5;

/// The colour of a seat with someone in it.
fn lit_seat() -> Srgb {
    super::arrows::mix(Palette::faint(), Palette::ink(), SEAT_TINT)
}

/// The pieces of the hall picture of 11_1 that move together: the people who
/// arrive and the decay that shortens, both from one tracker.
struct Crowd {
    arrive: Mob,
    seats: Mob,
    curve: Mob,
    anchor: Mob,
}

impl Crowd {
    fn build(stage: &mut Stage<'_>, hall: &Hall, curve: Mob, t_full: f64, t_max: f64) -> fmn::Result<Self> {
        let faint = Palette::faint();
        // a seat with someone in it is a tint, not the page's whitest white:
        // the full block of 182 was the brightest object of the video, and
        // the rays that pass over it would not show on it
        let lit = lit_seat();
        let seat = move |on: bool| if on { lit } else { faint };
        let seats_w = hall.length_m * 0.80 * hall.scale;
        let seats_h = hall.width_m * 0.84 * hall.scale;
        let top_left = v(
            hall.at(hall.length_m * 0.17, 0.0)[0],
            hall.at(0.0, hall.width_m * 0.92)[1],
        );
        let colors = vec![vec![seat(false); SEAT_COLS]; SEAT_ROWS];
        let strips = RowStrips::build(
            stage,
            top_left,
            (seats_w / SEAT_COLS as f64, seats_h / SEAT_ROWS as f64),
            &colors,
        )?;
        let seats = strips.group;
        let arrive = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        // a fixed shuffle: which seat fills at which share
        let n = SEAT_ROWS * SEAT_COLS;
        let rank: Vec<usize> = (0..n).map(|i| (i * 7919) % n).collect();
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut Arena, _| {
                let a = s.tracker_value(arrive).unwrap_or(0.0);
                if a.to_bits() == last.to_bits() {
                    return;
                }
                last = a;
                let t = t_full * (EMPTY_STRETCH - (EMPTY_STRETCH - 1.0) * a);
                let end = v(PLOT_LEFT + PLOT_W * t / t_max, PLOT_BOTTOM);
                let _ = s.set_points(curve, &corners(&[v(PLOT_LEFT, PLOT_BOTTOM + PLOT_H), end]));
                let lit = (a * n as f64).round() as usize;
                let colors: Vec<Vec<Srgb>> = (0..SEAT_ROWS)
                    .map(|r| (0..SEAT_COLS).map(|c| seat(rank[r * SEAT_COLS + c] < lit)).collect())
                    .collect();
                strips.recolor(s, &colors);
            },
            true,
        )?;
        Ok(Self {
            arrive,
            seats,
            curve,
            anchor,
        })
    }
}

/// A room's outline as a box seen a little from the right and above: its
/// front (length by height) and its back, and the four edges between, one
/// group, with its lower-left front corner at `at`. Returns the group and
/// its width and height on screen.
fn room_outline(stage: &mut Stage<'_>, room: &Room, at: Vec3, color: Srgb) -> fmn::Result<(Mob, f64, f64)> {
    let [lx, ly, lz] = room.sides_m();
    let (w, h) = (lx * ROOM_SCALE, lz * ROOM_SCALE);
    let (dx, dy) = (ly * ROOM_SCALE * OBLIQUE.0, ly * ROOM_SCALE * OBLIQUE.1);
    let p = |x: f64, y: f64| v(at[0] + x, at[1] + y);
    let front = path(
        stage,
        vec![p(0.0, 0.0), p(w, 0.0), p(w, h), p(0.0, h), p(0.0, 0.0)],
        color,
        4.0,
    );
    let back = path(
        stage,
        vec![p(dx, dy), p(w + dx, dy), p(w + dx, h + dy), p(dx, h + dy), p(dx, dy)],
        color,
        3.0,
    );
    let mut parts = vec![front, back];
    for (x, y) in [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)] {
        parts.push(seg(stage, p(x, y), p(x + dx, y + dy), color, 3.0));
    }
    let group = super::draw::group(stage, &parts)?;
    Ok((group, w + dx, h + dy))
}

/// The rooms' tones below 200 Hz as stacked ticks on one number line:
/// ticks at equal frequency (to the bit, decided from whole numbers by the
/// model: modes with the same sorted sides' squares) stand on one another.
/// Returns each tick and its height in the stack, tick by tick in frequency
/// order.
fn stacked_ticks(room: &Room, hertz: u64) -> Vec<(f64, usize)> {
    let modes = room.modes_below(hertz);
    let mut out: Vec<(f64, usize)> = Vec::new();
    for (_, hz) in modes {
        let height = out.iter().filter(|(f, _)| (f - hz).abs() < 1e-9).count();
        out.push((hz, height));
    }
    out
}

/// The cell-by-cell geometry of the attic: the cells of air as (row, col),
/// the eigenvectors of its table, the corner of the table and the modes.
struct Attic {
    cells: Vec<(usize, usize)>,
    /// The boundary of the room, in cells.
    outline: Vec<[f64; 2]>,
    /// `modes[k]`: the pressure in each cell for mode `k + 1` (the first,
    /// 0, is all the air at one pressure and is skipped), largest entry 1.
    modes: Vec<Vec<f64>>,
    hz: Vec<f64>,
}

impl Attic {
    fn new() -> Self {
        let (section, eigen) = attic_section();
        let cells = section.cells();
        let mut outline = vec![[0.0, 0.0], [section.cols as f64, 0.0]];
        // down the ceiling from the right wall to the left
        let ceiling = |col: usize| (0..section.rows).filter(|&r| section.air[r][col]).count() as f64;
        for col in (0..section.cols).rev() {
            let h = ceiling(col);
            outline.push([col as f64 + 1.0, h]);
            outline.push([col as f64, h]);
        }
        outline.push([0.0, 0.0]);
        let mut modes = Vec::new();
        let mut hz = Vec::new();
        for k in 1..=3 {
            let vec = &eigen.vectors[k];
            let top = vec.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
            modes.push(vec.iter().map(|x| x / top).collect());
            hz.push(section.hertz(eigen.values[k]));
        }
        Self {
            cells,
            outline,
            modes,
            hz,
        }
    }

    /// The pressure of `mode` at `(x, y)` in cell units: bilinear between
    /// cell centres, a wall (or a missing cell) giving the value of the
    /// cell the point is in, so the field is flat against a wall.
    fn sample(&self, index: &[Vec<Option<usize>>], mode: usize, x: f64, y: f64) -> f64 {
        let here = index
            .get(y.floor().max(0.0) as usize)
            .and_then(|r| r.get(x.floor().max(0.0) as usize))
            .copied()
            .flatten();
        let own_value = here.map_or(0.0, |i| self.modes[mode][i]);
        let value = |col: i64, row: i64| -> f64 {
            let at = (row >= 0 && col >= 0)
                .then(|| {
                    index
                        .get(row as usize)
                        .and_then(|r| r.get(col as usize))
                        .copied()
                        .flatten()
                })
                .flatten();
            at.map_or(own_value, |i| self.modes[mode][i])
        };
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (c0, r0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - c0, fy - r0);
        let (c0, r0) = (c0 as i64, r0 as i64);
        let bottom = value(c0, r0) * (1.0 - tx) + value(c0 + 1, r0) * tx;
        let top = value(c0, r0 + 1) * (1.0 - tx) + value(c0 + 1, r0 + 1) * tx;
        bottom * (1.0 - ty) + top * ty
    }
}

/// `(row, col)` to a cell's index in `cells`.
fn cell_index(rows: usize, cols: usize, cells: &[(usize, usize)]) -> Vec<Vec<Option<usize>>> {
    let mut index = vec![vec![None; cols]; rows];
    for (i, &(r, c)) in cells.iter().enumerate() {
        index[r][c] = Some(i);
    }
    index
}

/// The `n`-th point of the base-`b` low-discrepancy sequence on `[0, 1)`.
fn halton(mut n: usize, base: usize) -> f64 {
    let (mut f, mut r) = (1.0, 0.0);
    while n > 0 {
        f /= base as f64;
        r += f * (n % base) as f64;
        n /= base;
    }
    r
}

/// Where the motes rest: the first `MOTES` points of a Halton sequence over
/// the attic's box that fall in a cell of air (cell units).
fn mote_rest(index: &[Vec<Option<usize>>], cols: usize, rows: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    let mut n = 1;
    while out.len() < MOTES {
        let (x, y) = (halton(n, 2) * cols as f64, halton(n, 3) * rows as f64);
        if index[y.floor() as usize][x.floor() as usize].is_some() {
            out.push([x, y]);
        }
        n += 1;
    }
    out
}

/// What the three modes of the attic do to each mote: the direction the air
/// is pushed (the pressure's gradient, scaled so that no mote swings more
/// than `SWING` cells), `[mode][mote]`.
fn mote_swing(attic: &Attic, index: &[Vec<Option<usize>>], rest: &[[f64; 2]]) -> Vec<Vec<[f64; 2]>> {
    let h = 0.35;
    (0..attic.modes.len())
        .map(|k| {
            let g: Vec<[f64; 2]> = rest
                .iter()
                .map(|&[x, y]| {
                    [
                        (attic.sample(index, k, x + h, y) - attic.sample(index, k, x - h, y)) / (2.0 * h),
                        (attic.sample(index, k, x, y + h) - attic.sample(index, k, x, y - h)) / (2.0 * h),
                    ]
                })
                .collect();
            let top = g.iter().fold(1e-9_f64, |m, d| m.max(d[0].hypot(d[1])));
            g.iter().map(|d| [d[0] / top * SWING, d[1] / top * SWING]).collect()
        })
        .collect()
}

/// A colour between two, `t` of the way.
fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    let t = t.clamp(0.0, 1.0);
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// A cell's colour at pressure `p` (-1 to 1): the room's own colour at rest,
/// magenta where the air is pressed, teal where it is thinned.
fn pressure_colour(p: f64) -> Srgb {
    if p >= 0.0 {
        mix(panel(), squeezed(), p)
    } else {
        mix(panel(), stretched(), -p)
    }
}

impl SceneConstruct for WhereModesRule {
    fn name(&self) -> &str {
        "11_where_modes_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();
        let faint = Palette::faint();

        // ------------------------------------------------------------ 11_1
        // rays bounce round the hall; the seats fill with people and the
        // decay curve shortens; Sabine's rule, in words and in the hall's
        // numbers
        let line = say(&kit, stage, "11_1")?;
        let hall = Hall::build()?;
        let (hl, hb, hh) = (hall.at(0.0, 0.0), hall.at(0.0, 0.0)[1], hall.height());
        let outline = path(
            stage,
            vec![
                v(hl[0], hb),
                v(hl[0] + HALL_W, hb),
                v(hl[0] + HALL_W, hb + hh),
                v(hl[0], hb + hh),
                v(hl[0], hb),
            ],
            dim,
            4.0,
        );
        let edge = hall.at(hall.stage_edge_m(), 0.0)[0];
        let stage_wall = seg(stage, v(edge, hb), v(edge, hb + hh), dim, 3.0);
        let (ray_trails, ray_anchor) = rays(stage, &hall)?;

        // the decay plot
        let t_full = numbers("K-H1.t60_s")?[0];
        let t_max = t_full * EMPTY_STRETCH * 1.08;
        let axes = path(
            stage,
            vec![
                v(PLOT_LEFT, PLOT_BOTTOM + PLOT_H),
                v(PLOT_LEFT, PLOT_BOTTOM),
                v(PLOT_LEFT + PLOT_W, PLOT_BOTTOM),
            ],
            faint,
            3.0,
        );
        let curve = path(
            stage,
            vec![
                v(PLOT_LEFT, PLOT_BOTTOM + PLOT_H),
                v(PLOT_LEFT + PLOT_W * t_full * EMPTY_STRETCH / t_max, PLOT_BOTTOM),
            ],
            ink,
            5.0,
        );
        let level = kit.text_left(
            stage,
            SOUND_LEVEL,
            24.0,
            dim,
            v(PLOT_LEFT + 0.15, PLOT_BOTTOM + PLOT_H + 0.25),
        )?;
        let time = kit.text(stage, TIME, 24.0, dim, v(PLOT_LEFT + PLOT_W - 0.4, PLOT_BOTTOM - 0.3))?;
        let crowd = Crowd::build(stage, &hall, curve, t_full, t_max)?;

        fade_in_all(stage, &[outline, stage_wall, crowd.seats], 0.6)?;
        stage.add_to_scene(ray_anchor)?;
        fade_in_all(stage, &ray_trails, 0.5)?;
        stage.add_to_scene(crowd.anchor)?;
        line.wait_for(stage, "Halls are designed with statistics")?;
        fade_in_all(stage, &[axes, crowd.curve, level, time], 0.6)?;
        play!(stage; tween_linear(crowd.arrive, 1.0, 3.6)?);
        // where the decay crosses the axis: the reverberation time
        let t60 = fact("K-H1.t60_s")?;
        let cross_x = PLOT_LEFT + PLOT_W * t_full / t_max;
        let t60_tick = seg(
            stage,
            v(cross_x, PLOT_BOTTOM - 0.1),
            v(cross_x, PLOT_BOTTOM + 0.1),
            ink,
            3.0,
        );
        let t60_label = kit.text(stage, &format!("{t60} s"), 30.0, ink, v(cross_x, PLOT_BOTTOM - 0.45))?;
        line.wait_for(stage, "which began with Wallace Sabine")?;
        fade_in_all(stage, &[t60_tick, t60_label], 0.5)?;
        let opened = fact("K-S.sabine.opened")?;
        let sabine = kit.text(
            stage,
            &format!("Sabine advised the architects of Boston Symphony Hall, which opened in {opened}"),
            26.0,
            dim,
            v(0.0, -2.3),
        )?;
        let search = names_to_search(
            &kit,
            stage,
            &["Sabine equation", "geometrical acoustics", "image sources"],
        )?;
        fade_in_all(stage, &[sabine, search], 0.6)?;
        line.wait_for(stage, "A bigger room lets sound linger longer")?;
        let bigger = kit.text(stage, RULE_BIGGER, 32.0, ink, v(0.0, -0.75))?;
        fade_in_all(stage, &[bigger], 0.6)?;
        let soaks = kit.text(stage, RULE_SOAKS, 32.0, ink, v(0.0, -1.3))?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[soaks], 0.6)?;
        let constant = fact("K-H4.sabine_constant")?;
        let sum = kit.text(
            stage,
            &format!(
                "{t60} s = {constant} \u{d7} {} \u{f7} {} m\u{b2} {ABSORPTION}",
                grouped("K-H1.volume_m3")?,
                grouped("K-H4.sabine_m2")?
            ),
            32.0,
            ink,
            v(0.0, -1.85),
        )?;
        fade_in_all(stage, &[sum], 0.6)?;
        pause(stage, 1.6)?;

        // ------------------------------------------------------------ 11_3
        // a small room's modes are single, counted below its crossover; the
        // card of three rooms, and the countdown
        let line = say(&kit, stage, "11_3")?;
        stage.clear_updaters(ray_anchor, true);
        stage.clear_updaters(crowd.anchor, true);
        let mut leaving = vec![outline, stage_wall, crowd.seats, crowd.curve, axes, level, time];
        leaving.extend(&ray_trails);
        leaving.extend([t60_tick, t60_label, sabine, search, bigger, soaks, sum]);
        fade_out_all(stage, &leaving, 0.6)?;

        // the study's tones, as gold peaks along a frequency line
        let study = STUDY.modes_below(200);
        let x_of = |hz: f64| LINE_X.0 + (LINE_X.1 - LINE_X.0) * hz / LINE_MAX_HZ;
        let axis = seg(stage, v(LINE_X.0, LINE_Y), v(LINE_X.1, LINE_Y), dim, 3.0);
        let mut peaks = Vec::new();
        for &(_, hz) in &study {
            peaks.push(seg(
                stage,
                v(x_of(hz), LINE_Y),
                v(x_of(hz), LINE_Y + PEAK_H),
                own(),
                4.0,
            ));
        }
        let crossover_hz = numbers("K-R4.schroeder_hz")?[0];
        let marker = seg(
            stage,
            v(x_of(crossover_hz), LINE_Y - 0.15),
            v(x_of(crossover_hz), LINE_Y + PEAK_H + 0.45),
            ink,
            3.0,
        );
        let marker_name = kit.text(
            stage,
            &format!("{CROSSOVER} {} {HZ}", whole(crossover_hz)),
            28.0,
            ink,
            v(x_of(crossover_hz) - 0.25, LINE_Y + PEAK_H + 0.8),
        )?;
        let zero = kit.text(stage, "0", 26.0, dim, v(LINE_X.0, LINE_Y - 0.4))?;
        let mut ends = vec![axis, marker, marker_name, zero];
        // the half-way mark of the line
        let hundred = crossover_hz / 2.0;
        let tick = seg(
            stage,
            v(x_of(hundred), LINE_Y - 0.1),
            v(x_of(hundred), LINE_Y),
            dim,
            3.0,
        );
        let name = kit.text(stage, &whole(hundred), 26.0, dim, v(x_of(hundred), LINE_Y - 0.4))?;
        ends.extend([tick, name]);
        let count = stage.add_value_tracker(0.0);
        let counter = kit.live_value(
            stage,
            move |s| s.tracker_value(count).unwrap_or(0.0).round(),
            0,
            false,
            56.0,
            ink,
            v(-5.9, 2.55),
            LEFT,
        )?;
        stage.remove_from_scene(counter);
        let counted = kit.text_left(
            stage,
            &format!("{MODES_BELOW} {} {HZ}", whole(crossover_hz)),
            36.0,
            ink,
            v(-4.7, 2.5),
        )?;
        fade_in_all(stage, &ends, 0.5)?;
        stage.add_to_scene(counter)?;
        fade_in_all(stage, &[counted], 0.3)?;
        let mut grow: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &peaks {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            grow.push(stage.prepare(a)?);
        }
        let wave = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, 0.05)?.with_run_time(3.4);
        play!(stage; wave, tween_linear(count, study.len() as f64, 3.4)?);
        line.wait_for(stage, "A room booms")?;
        let sides = items("K-R4.sides_m")?;
        let t60_small = fact("K-R4.t60_s")?;
        let size = kit.text(
            stage,
            &format!(
                "{} \u{d7} {} \u{d7} {} m; dies away in {t60_small} s ({A_MODEL})",
                sides[0], sides[1], sides[2]
            ),
            30.0,
            ink,
            v(0.0, -1.4),
        )?;
        fade_in_all(stage, &[size], 0.6)?;

        // the quiz card: three small rooms in outline
        let card = QuizCard::build(&kit, stage, "11_3", CARD_AT, CARD_SIZE)?;
        let (centre, room_w, _) = card.content();
        let rooms = [(&NARROW, NARROW_NAME), (&STUDY, ORDINARY_NAME), (&CUBE, CUBE_NAME)];
        let mut problem = Vec::new();
        let mut cube_box = None;
        let slot = room_w / 3.0;
        for (i, (room, name)) in rooms.iter().enumerate() {
            let x = centre[0] - room_w / 2.0 + slot * (i as f64 + 0.5);
            let (w_probe, h_probe) = {
                let [lx, ly, lz] = room.sides_m();
                (
                    lx * ROOM_SCALE + ly * ROOM_SCALE * OBLIQUE.0,
                    lz * ROOM_SCALE + ly * ROOM_SCALE * OBLIQUE.1,
                )
            };
            let corner = v(x - w_probe / 2.0, centre[1] - h_probe / 2.0 + 0.15);
            let (shape, w, h) = room_outline(stage, room, corner, ink)?;
            let caption = kit.text(stage, name, 28.0, dim, v(x, corner[1] - 0.45))?;
            problem.extend([shape, caption]);
            if room.name == CUBE.name {
                cube_box = Some((x, corner[1] + h / 2.0, w, h));
            }
        }
        line.wait_for(stage, "Your turn")?;
        fade_out_all(stage, &[counter, counted, size], 0.4)?;
        let mut gone = ends.clone();
        gone.extend(&peaks);
        fade_out_all(stage, &gone, 0.4)?;
        card.show_frame(stage, &problem, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;
        // the answer: the cube, outlined in gold, named at the ring's place
        let (cx, cy, cw, ch) = cube_box.ok_or_else(|| super::draw::fail("the card has no cube".into()))?;
        let ring = Rectangle::new()
            .width(cw + 0.5)
            .height(ch + 1.1)
            .corner_radius(0.15)
            .style(faded_line(own(), 5.0, 1.0))
            .build()?
            .moved_to(v(cx, cy - 0.15));
        let ring = stage.arena_mut().add(ring);
        let answer = kit.text(stage, THE_CUBE, 36.0, own(), card.ring_at())?;
        card.land(stage, &[ring, answer], 0.5)?;

        // ------------------------------------------------------------ 11_4
        // the cube: the tones of the three rooms below 200 Hz on three
        // number lines, the cube's stacked
        let line = say(&kit, stage, "11_4")?;
        let mut cleared = card.mobs();
        cleared.extend(&problem);
        cleared.extend([ring, answer, card.track, card.pause_line]);
        fade_out_all(stage, &cleared, 0.5)?;
        let row_x = |hz: f64| ROW_X.0 + (ROW_X.1 - ROW_X.0) * hz / ROW_MAX_HZ;
        let mut rows_ticks: Vec<Vec<Mob>> = Vec::new();
        let mut row_labels = Vec::new();
        let mut row_axes = Vec::new();
        let rooms = [
            (&NARROW, NARROW_NAME, "K-R5.modes_below_200.narrow", ROW_Y[0]),
            (&STUDY, ORDINARY_NAME, "K-R5.modes_below_200.study", ROW_Y[1]),
            (&CUBE, CUBE_NAME, "K-R5.modes_below_200.cube", ROW_Y[2]),
        ];
        let mut cube_stacks: Vec<(f64, usize)> = Vec::new();
        for (room, name, count_key, y) in rooms {
            row_axes.push(seg(stage, v(ROW_X.0, y), v(ROW_X.1, y), faint, 3.0));
            let stacks = stacked_ticks(room, 200);
            let mut ticks = Vec::new();
            for &(hz, height) in &stacks {
                let base = y + TICK_H * height as f64;
                ticks.push(seg(
                    stage,
                    v(row_x(hz), base),
                    v(row_x(hz), base + TICK_H * 0.8),
                    own(),
                    5.0,
                ));
            }
            let words = kit.text_left(stage, name, 28.0, ink, v(-6.8, y + 0.2))?;
            let many = format!("{} {MODES_BELOW} {}", fact(count_key)?, whole(200.0));
            let counted = kit.text_left(stage, &many, 24.0, dim, v(-6.8, y - 0.25))?;
            row_labels.extend([words, counted]);
            rows_ticks.push(ticks);
            if room.name == CUBE.name {
                cube_stacks = stacks;
            }
        }
        // the scale the three lines share, under the lowest: where the lines
        // begin and where they end (the storyboard's "below 200 Hz")
        let scale_y = ROW_Y[2] - SCALE_DROP;
        for hz in [0.0, ROW_MAX_HZ] {
            row_axes.push(seg(
                stage,
                v(row_x(hz), ROW_Y[2] - 0.08),
                v(row_x(hz), ROW_Y[2] - 0.2),
                dim,
                3.0,
            ));
        }
        let scale_zero = kit.text(stage, "0", 24.0, dim, v(row_x(0.0), scale_y))?;
        let scale_end = kit.text(
            stage,
            &format!("{} {HZ}", whole(ROW_MAX_HZ)),
            24.0,
            dim,
            v(0.0, scale_y),
        )?;
        stage.move_to(scale_end, v(row_x(ROW_MAX_HZ) + 0.1, scale_y), RIGHT);
        row_axes.extend([scale_zero, scale_end]);
        let mut ins = row_axes.clone();
        ins.extend(&row_labels);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_for(stage, "The cube")?;
        for ticks in &rows_ticks[..2] {
            fade_in_lagged(stage, ticks, 0.9, 0.04)?;
        }
        line.wait_for(stage, "Its three sides match")?;
        let cube_ticks = &rows_ticks[2];
        fade_in_lagged(stage, cube_ticks, 1.2, 0.04)?;
        // the lowest stack: three modes on 343 / 6 = 57.2 Hz
        let first = fact("K-R3.first_hz")?;
        let side = numbers("K-R3.side_m")?[0];
        let formula = kit.text(
            stage,
            &format!(
                "{} \u{f7} {} = {first} {HZ} \u{d7} {}",
                fact("K-R0.sound_m_per_s")?,
                whole(2.0 * side),
                fact("K-R3.first_count")?
            ),
            34.0,
            ink,
            v(row_x(numbers("K-R3.first_hz")?[0]) + 1.2, ROW_Y[2] - 0.75),
        )?;
        let pointer = seg(
            stage,
            v(row_x(numbers("K-R3.first_hz")?[0]), ROW_Y[2] - 0.15),
            v(row_x(numbers("K-R3.first_hz")?[0]) + 0.5, ROW_Y[2] - 0.5),
            ink,
            3.0,
        );
        fade_in_all(stage, &[formula, pointer], 0.6)?;
        let sharing = kit.text_left(
            stage,
            &format!(
                "{} of {} {SHARE_A_TONE}",
                fact("K-R5.sharing_200.cube")?,
                fact("K-R5.modes_below_200.cube")?
            ),
            26.0,
            own(),
            v(-6.8, ROW_Y[2] - 0.7),
        )?;
        fade_in_all(stage, &[sharing], 0.5)?;
        let sources = lower_third(&kit, stage, ROOMS_SOURCES, SHOEBOXES)?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[sources], 0.6)?;
        pause(stage, 1.2)?;
        let _ = cube_stacks;

        // ------------------------------------------------------------ 11_5
        // a room that is not a box: its air cut into cells, the table, its
        // eigenvectors
        let line = say(&kit, stage, "11_5")?;
        let mut leaving = row_axes.clone();
        leaving.extend(&row_labels);
        for ticks in &rows_ticks {
            leaving.extend(ticks);
        }
        leaving.extend([formula, pointer, sharing, sources]);
        fade_out_all(stage, &leaving, 0.6)?;
        let attic = Attic::new();
        let (section, _) = attic_section();
        let index = cell_index(section.rows, section.cols, &attic.cells);
        let at = |x: f64, y: f64| v(ATTIC_LEFT + CELL * x, ATTIC_BOTTOM + CELL * y);
        // the cells at rest: the room's own colour over air, the background
        // over a wall
        let rest: Vec<Vec<Srgb>> = (0..section.rows)
            .rev()
            .map(|r| {
                (0..section.cols)
                    .map(|c| if index[r][c].is_some() { panel() } else { ground() })
                    .collect()
            })
            .collect();
        let cells_top_left = at(0.0, section.rows as f64);
        let strips = RowStrips::build(stage, cells_top_left, (CELL, CELL), &rest)?;
        let room_line = path(stage, attic.outline.iter().map(|p| at(p[0], p[1])).collect(), dim, 4.0);
        let tag = kit.text(
            stage,
            SIDE_VIEW,
            26.0,
            dim,
            v(ATTIC_LEFT + CELL * section.cols as f64 / 2.0, ATTIC_BOTTOM - 0.45),
        )?;
        fade_in_all(stage, &[room_line], 0.8)?;
        line.wait_for(stage, "Cut the air into cells")?;
        fade_in_all(stage, &[strips.group, tag], 0.8)?;
        let count_label = kit.text(
            stage,
            &format!("{} cells", fact("K-N1.cells")?),
            30.0,
            ink,
            v(
                ATTIC_LEFT + CELL * section.cols as f64 / 2.0,
                ATTIC_BOTTOM + CELL * section.rows as f64 + 0.35,
            ),
        )?;
        fade_in_all(stage, &[count_label], 0.4)?;

        // the corner of the table: a pair of touching cells adds 1 to both
        // diagonal entries and -1 between them
        line.wait_for(stage, "build the table")?;
        let table = section.table();
        let corner: Vec<Vec<String>> = (0..4)
            .map(|i| (0..4).map(|j| whole(table.at(i, j))).collect())
            .collect();
        let grid = MatrixGrid::build(&kit, stage, &corner, 40.0, ink, v(3.9, 1.9))?;
        let corner_name = kit.text(stage, TABLE_CORNER, 24.0, dim, v(3.9, 0.35))?;
        let size_note = kit.text(
            stage,
            &format!("{} \u{d7} {}", fact("K-N1.cells")?, fact("K-N1.cells")?),
            26.0,
            dim,
            v(3.9, -0.05),
        )?;
        fade_in_all(stage, &[grid.group, corner_name, size_note], 0.7)?;
        let coder = lower_third(&kit, stage, PROGRAMMERS, CELLS_TEXT)?;
        fade_in_all(stage, &[coder], 0.5)?;

        // mode 1, then the next two, as the cells and as motes
        line.wait_for(stage, "and ask for its eigenvectors")?;
        let rows = section.rows;
        let rest_points = mote_rest(&index, section.cols, rows);
        let swing = mote_swing(&attic, &index, &rest_points);
        let mote_template: Vec<Vec3> = Dot::new().radius(0.035).build().points().to_vec();
        let mut motes = Vec::new();
        for p in &rest_points {
            let c = at(p[0], p[1]);
            let m = stage.arena_mut().add(
                Dot::new()
                    .point(c)
                    .radius(0.035)
                    .style(super::draw::flat(ink, 0.9))
                    .build(),
            );
            motes.push(m);
        }
        let mote_group = super::draw::group(stage, &motes)?;
        let which = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let hz = attic.hz.clone();
        let modes = attic.modes.clone();
        let index_for_cells = index.clone();
        let strips_for_driver = strips;
        let mut phase = 0.0_f64;
        let mut last_mode = -1.0_f64;
        let rest_for_driver = rest_points.clone();
        let motes_for_driver = motes.clone();
        stage.add_dt_updater(
            anchor,
            move |s: &mut Arena, _, dt| {
                if dt <= 0.0 {
                    return;
                }
                let k = s.tracker_value(which).unwrap_or(0.0).round().clamp(0.0, 2.0) as usize;
                if (k as f64) != last_mode {
                    last_mode = k as f64;
                    phase = 0.0;
                }
                // each mode swings at its own frequency, the first once in
                // `BASE_PERIOD_S`
                phase += std::f64::consts::TAU * (hz[k] / hz[0]) * dt / BASE_PERIOD_S;
                let amp = phase.sin();
                let colors: Vec<Vec<Srgb>> = (0..rows)
                    .rev()
                    .map(|r| {
                        index_for_cells[r]
                            .iter()
                            .map(|i| i.map_or_else(ground, |i| pressure_colour(modes[k][i] * amp)))
                            .collect()
                    })
                    .collect();
                strips_for_driver.recolor(s, &colors);
                for (j, &m) in motes_for_driver.iter().enumerate() {
                    let p = rest_for_driver[j];
                    let (x, y) = (p[0] + amp * swing[k][j][0], p[1] + amp * swing[k][j][1]);
                    let c = v(ATTIC_LEFT + CELL * x, ATTIC_BOTTOM + CELL * y);
                    let pts: Vec<Vec3> = mote_template.iter().map(|q| [q[0] + c[0], q[1] + c[1], q[2]]).collect();
                    let _ = s.set_points(m, &pts);
                }
            },
            false,
        )?;
        stage.add_to_scene(anchor)?;
        fade_in_all(stage, &[mote_group], 0.5)?;

        // the three lowest tones of the slice, the playing one in gold
        let mut names = Vec::new();
        for (k, &hertz) in attic.hz.iter().enumerate() {
            let y = -1.0 - 0.5 * k as f64;
            let text = format!("{MODE} {}   {} {HZ}", k + 1, show(hertz, 1)?);
            names.push(kit.text_left(stage, &text, 28.0, ink, v(2.1, y))?);
        }
        light_the_playing(stage, &names, 0);
        fade_in_all(stage, &names, 0.5)?;
        line.wait_for(stage, "The same calculation as the bridge")?;
        stage.set_tracker_value(which, 1.0)?;
        light_the_playing(stage, &names, 1);
        pause(stage, 1.8)?;
        stage.set_tracker_value(which, 2.0)?;
        light_the_playing(stage, &names, 2);
        let frank = lower_third(
            &kit,
            stage,
            FRANKENSIM,
            &format!(
                "finds certified windows of the lowest modes of a {}-cell grid",
                grouped("K-S.frankensim.grid_unknowns")?
            ),
        )?;
        fade_out_all(stage, &[coder], 0.4)?;
        fade_in_all(stage, &[frank], 0.5)?;
        line.wait_end(stage)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        stage.clear_updaters(anchor, true);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// How many of the stacked ticks stand on a tone another shares.
    fn sharing_from_stacks(stacks: &[(f64, usize)]) -> usize {
        stacks
            .iter()
            .filter(|&&(f, _)| stacks.iter().filter(|(g, _)| (g - f).abs() < 1e-9).count() > 1)
            .count()
    }

    /// 11_3 and 11_4: the ticks of the three rooms are the model's modes
    /// below 200 Hz, counted and stacked as the facts say; the quiz's
    /// answer is the room whose stacks are three high.
    #[test]
    fn the_ticks_of_the_three_rooms_are_the_facts() {
        for (room, name) in [(&NARROW, "narrow"), (&STUDY, "study"), (&CUBE, "cube")] {
            let stacks = stacked_ticks(room, 200);
            assert_eq!(
                stacks.len().to_string(),
                fact(&format!("K-R5.modes_below_200.{name}")).unwrap(),
                "{name}"
            );
            assert_eq!(
                sharing_from_stacks(&stacks).to_string(),
                fact(&format!("K-R5.sharing_200.{name}")).unwrap(),
                "{name}"
            );
        }
        // the small room of 11_3 is the ordinary one: 67 modes below its
        // crossover, none on a shared tone
        assert_eq!(
            STUDY.modes_below(200).len().to_string(),
            fact("K-R4.modes_below").unwrap()
        );
        assert_eq!(fact("K-R4.modes_below").unwrap(), "67");
        assert_eq!(numbers("K-R4.schroeder_hz").unwrap(), [200.0]);
        assert_eq!(fact("K-R4.sharing_a_tone").unwrap(), "0");
        let sides = items("K-R4.sides_m").unwrap();
        assert_eq!(sides, ["4.8", "4", "2.6"]);
        assert_eq!(STUDY.sides_m(), [4.8, 4.0, 2.6]);
        // the answer of the quiz is the room with the most on shared tones
        assert_eq!(fact("K-Q.11_3.answer").unwrap(), "cube");
        assert_eq!(fact("K-R5.worst").unwrap(), CUBE.name);
        assert_eq!(super::super::draw::quiz_at("11_3").unwrap().seconds, 5);
        // the cube's lowest tone: three modes stacked on 343 / 6 = 57.2 Hz
        let cube = stacked_ticks(&CUBE, 200);
        let lowest: Vec<&(f64, usize)> = cube.iter().filter(|(f, _)| (f - cube[0].0).abs() < 1e-9).collect();
        assert_eq!(lowest.iter().map(|&&(_, h)| h).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(show(cube[0].0, 1).unwrap(), fact("K-R3.first_hz").unwrap());
        assert_eq!(lowest.len().to_string(), fact("K-R3.first_count").unwrap());
        let side = numbers("K-R3.side_m").unwrap()[0];
        let c = numbers("K-R0.sound_m_per_s").unwrap()[0];
        assert_eq!(show(c / (2.0 * side), 1).unwrap(), fact("K-R3.first_hz").unwrap());
        assert_eq!(whole(2.0 * side), "6");
        // the tallest stack stands clear of the line above it
        let tallest = cube.iter().map(|&(_, h)| h + 1).max().unwrap();
        assert_eq!(tallest, 6);
        assert!(
            TICK_H * tallest as f64 + 0.15 < ROW_Y[1] - ROW_Y[2],
            "a stack of {tallest} reaches the line above"
        );
        // the narrow and the ordinary room are single ticks
        for room in [&NARROW, &STUDY] {
            assert!(stacked_ticks(room, 200).iter().all(|&(_, h)| h == 0));
        }
    }

    /// 11_1: the hall is `K-H1.sides_m`, the rays never leave it, and the
    /// sum on the screen is Sabine's rule on the facts.
    #[test]
    fn the_hall_holds_its_rays_and_the_sum_is_sabines_rule() {
        let hall = Hall::build().unwrap();
        assert_eq!(numbers("K-H1.sides_m").unwrap(), [45.0, 23.0, 18.0]);
        assert_eq!((hall.length_m, hall.width_m), (45.0, 23.0));
        let (left, bottom) = (hall.left, hall.bottom);
        for i in 0..RAYS {
            for step in 0..2000 {
                let p = hall.ray_at(ray_angle(i), f64::from(step) * 0.37);
                assert!(
                    p[0] >= left - 1e-9 && p[0] <= left + HALL_W + 1e-9,
                    "ray {i}: x {}",
                    p[0]
                );
                assert!(
                    p[1] >= bottom - 1e-9 && p[1] <= bottom + hall.height() + 1e-9,
                    "ray {i}: y {}",
                    p[1]
                );
            }
        }
        // the rays fan out either side of straight down the hall
        assert!(ray_angle(0) < 0.0 && ray_angle(RAYS - 1) > 0.0);
        assert!((ray_angle(0) + ray_angle(RAYS - 1)).abs() < 1e-12);
        // the volume is the sides' product, the time is Sabine's rule
        let [x, y, z] = [45.0_f64, 23.0, 18.0];
        assert_eq!((x * y * z).to_string(), "18630");
        assert_eq!(fact("K-H1.volume_m3").unwrap(), "18630");
        let t = numbers("K-H4.sabine_constant").unwrap()[0] * numbers("K-H1.volume_m3").unwrap()[0]
            / numbers("K-H4.sabine_m2").unwrap()[0];
        assert_eq!(show(t, 1).unwrap(), fact("K-H1.t60_s").unwrap());
        assert_eq!(fact("K-H1.t60_s").unwrap(), "1.9");
        assert_eq!(grouped("K-H1.volume_m3").unwrap(), "18,630");
        assert_eq!(grouped("K-H4.sabine_m2").unwrap(), "1,579");
        assert_eq!(fact("K-S.sabine.opened").unwrap(), "1900");
        // a ray folds back at each wall
        assert_eq!(bounce(-3.0, 10.0), 3.0);
        assert_eq!(bounce(13.0, 10.0), 7.0);
        assert_eq!(bounce(23.0, 10.0), 3.0);
    }

    /// 11_5: the attic is `K-N1.*`: its cells, its three lowest modes at the
    /// frequencies the facts and the storyboard print, motes at rest in
    /// cells of air that never swing further than `SWING`.
    #[test]
    fn the_attic_is_the_facts_and_its_motes_stay_in_air() {
        let attic = Attic::new();
        let (section, _) = attic_section();
        assert_eq!(attic.cells.len().to_string(), fact("K-N1.cells").unwrap());
        assert_eq!(fact("K-N1.cells").unwrap(), "232");
        assert_eq!(
            (section.cols.to_string(), section.rows.to_string()),
            (
                fact("K-N1.cols").unwrap().to_owned(),
                fact("K-N1.rows").unwrap().to_owned()
            )
        );
        for (k, key) in ["K-N1.hz.1", "K-N1.hz.2", "K-N1.hz.3"].into_iter().enumerate() {
            assert_eq!(show(attic.hz[k], 2).unwrap(), fact(key).unwrap(), "{key}");
        }
        let shown: Vec<String> = attic.hz.iter().map(|&h| show(h, 1).unwrap()).collect();
        assert_eq!(shown, ["29.8", "56.1", "59.9"]);
        assert!(attic.hz.windows(2).all(|w| w[0] < w[1]));
        // the outline starts and ends at the floor's left corner, and its
        // right wall is the lowest end of the ceiling
        assert_eq!(attic.outline.first(), attic.outline.last());
        assert_eq!(attic.outline[1], [section.cols as f64, 0.0]);
        assert_eq!(attic.outline[2], [section.cols as f64, 5.0]);
        assert_eq!(fact("K-N1.low_end_m").unwrap(), "1.25");
        // every mode is scaled to 1 at its largest, and moves air both ways
        for (k, mode) in attic.modes.iter().enumerate() {
            assert_eq!(mode.len(), attic.cells.len());
            let top = mode.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
            assert!((top - 1.0).abs() < 1e-12, "mode {}", k + 1);
            assert!(
                mode.iter().any(|&x| x > 0.1) && mode.iter().any(|&x| x < -0.1),
                "mode {}",
                k + 1
            );
        }
        // the motes rest in air, there are three hundred, and none swings
        // more than `SWING` cells
        let index = cell_index(section.rows, section.cols, &attic.cells);
        let rest = mote_rest(&index, section.cols, section.rows);
        assert_eq!(rest.len(), MOTES);
        assert!(
            rest.iter()
                .all(|p| index[p[1].floor() as usize][p[0].floor() as usize].is_some())
        );
        let swing = mote_swing(&attic, &index, &rest);
        assert_eq!(swing.len(), 3);
        for (k, mode) in swing.iter().enumerate() {
            let most = mode.iter().fold(0.0_f64, |m, d| m.max(d[0].hypot(d[1])));
            assert!(most <= SWING + 1e-9 && most > 0.9 * SWING, "mode {}: {most}", k + 1);
        }
        // the colours: rest is the room's own, +1 is magenta, -1 is teal
        assert_eq!(pressure_colour(0.0), panel());
        assert_eq!(pressure_colour(1.0), squeezed());
        assert_eq!(pressure_colour(-1.0), stretched());
    }

    /// 11_1: the rays are really drawn. They were built as two vertices and
    /// rewritten as thirteen, which the engine refuses without a word, so no
    /// ray was ever on the screen. A test of the state, not the lint.
    #[test]
    fn the_rays_are_drawn_and_run_across_the_hall() {
        super::super::carts::probe::run_scene(|stage| {
            let hall = Hall::build()?;
            let (trails, anchor) = rays(stage, &hall)?;
            stage.add_to_scene(anchor)?;
            for &m in &trails {
                stage.add_to_scene(m)?;
            }
            assert_eq!(trails.len(), RAYS);
            let before = stage.get_points(trails[0]).expect("a ray has points");
            assert_eq!(before.len(), 2 * (RAY_STEPS + 1) - 1);
            // a second of the picture: the head has run RAY_SPEED metres
            stage.wait(1.0)?;
            for &m in &trails {
                let pts = stage.get_points(m).expect("a ray has points");
                assert_eq!(pts.len(), before.len(), "the ray's size changed");
                let (head, tail) = (pts[0], pts[pts.len() - 1]);
                let trail = (head[0] - tail[0]).hypot(head[1] - tail[1]);
                assert!(trail > 0.5, "the ray is a speck: {trail} units");
                // and it is inside the hall
                for p in &pts {
                    assert!(p[0] >= hall.left - 1e-6 && p[0] <= hall.left + hall.scale * hall.length_m + 1e-6);
                    assert!(p[1] >= hall.bottom - 1e-6 && p[1] <= hall.bottom + hall.height() + 1e-6);
                }
            }
            Ok(())
        });
    }

    /// The colour of the first glyph of a text (a text is a group whose
    /// parts carry the colour).
    fn colour_of(stage: &Stage<'_>, m: Mob) -> Option<Srgb> {
        stage.get_color(m).or_else(|| {
            explainer_kit::children(stage, m)
                .into_iter()
                .find_map(|c| colour_of(stage, c))
        })
    }

    /// 11_1: people are a tint, between the empty seat's grey and the page's
    /// white (a block of 182 in full white was the brightest thing of the
    /// video); 11_5: the playing mode's name is the gold one.
    #[test]
    fn the_seats_are_a_tint_and_the_playing_mode_is_gold() {
        let (faint, ink, lit) = (Palette::faint(), Palette::ink(), lit_seat());
        for (a, f, i) in [
            (lit.r, faint.r, ink.r),
            (lit.g, faint.g, ink.g),
            (lit.b, faint.b, ink.b),
        ] {
            let (lo, hi) = (f.min(i), f.max(i));
            assert!(a > lo + 1e-3 && a < hi - 1e-3, "{a} is not between {f} and {i}");
        }
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let names: Vec<Mob> = (0..3)
                .map(|k| kit.text_left(stage, &format!("mode {k}"), 28.0, ink, v(0.0, -f64::from(k))))
                .collect::<fmn::Result<_>>()?;
            for playing in 0..3 {
                light_the_playing(stage, &names, playing);
                for (k, &m) in names.iter().enumerate() {
                    let want = if k == playing { own() } else { ink };
                    let got = colour_of(stage, m).expect("a name has a colour");
                    assert!(
                        (got.r - want.r).abs() < 2e-3 && (got.g - want.g).abs() < 2e-3 && (got.b - want.b).abs() < 2e-3,
                        "name {k} with {playing} playing is {got:?}, not {want:?}"
                    );
                }
            }
            Ok(())
        });
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("11_where_modes_rule").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        // 11_4's three lines have a scale: the end of their run, by itself
        assert!(words.contains(&"200 Hz"), "the lines have no scale: {words:?}");
        for want in [
            "1.9 s = 0.161 \u{d7} 18,630 \u{f7} 1,579 m\u{b2} of absorption",
            "Sabine advised the architects of Boston Symphony Hall, which opened in 1900",
            "crossover 200 Hz",
            "modes below 200 Hz",
            "4.8 \u{d7} 4 \u{d7} 2.6 m; dies away in 0.5 s (a model)",
            "a perfect cube",
            "343 \u{f7} 6 = 57.2 Hz \u{d7} 3",
            "36 of 38 share a tone",
            "232 cells",
            "mode 1   29.8 Hz",
            "mode 3   59.9 Hz",
            "100,489-cell grid",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
