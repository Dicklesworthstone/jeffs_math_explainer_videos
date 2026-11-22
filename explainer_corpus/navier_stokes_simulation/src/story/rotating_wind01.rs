//! Chapter 02, "What a flow picture shows": the rotating wind, cues 02_1 to
//! 02_10, native and exact.
//!
//! LavenderPike `wind-scene01` for root's `navier_stokes_simulation` crate
//! (Mail 36194). Cue text and pictures: `navier-full-script03.md` chapter 02,
//! copied there from `navier-opening-script03.md` lines 143 to 191 (sha256
//! 10760e12…). Fixture: `navier-opening-script02/tools/wind02.py` (a731dfc2…)
//! and its run `runs/wind02-run01.txt` (253abe8b…, 13 facts, ALL PASS).
//!
//! The toy wind is the same everywhere at each instant,
//! u(t) = (cos ωt, sin ωt) m/s with ω = 2π/8 per second, and the nozzle is
//! the origin of the toy's metres. Every picture is computed from closed
//! forms at every frame, never drawn by hand:
//! - the dye released at τ and seen at T sits at
//!   r (sin ωT − sin ωτ, cos ωτ − cos ωT), with r = 1/ω = 4/π m. So the dye is
//!   an arc of radius r about the centre r (sin ωT, −cos ωT), and that centre
//!   turns around the nozzle;
//! - the dye's near end (τ = T) is the nozzle;
//! - its far end (τ = 0) is the first puff, r (sin ωT, 1 − cos ωT), which
//!   rides a fixed circle about (0, r) above the nozzle.
//!
//! The reveal (02_8): the gold first puff traces its fixed circle while the
//! rose dye curls about a rose centre that travels around the nozzle; both are
//! on screen together, and at 8 s they stand as the circle above and the
//! circle below.
//!
//! Root owns the tunnel, its thumbnail and the open question card. The tunnel
//! halves of 02_1 and 02_2 are therefore [`TunnelCues`] hooks (empty in
//! [`NoTunnel`]), and 02_11 to 02_13 are root's. Nothing in the window is
//! rescaled or redrawn at another scale between beats.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

// ------------------------------------------------------------ the toy wind

/// Seconds for the wind's direction to turn once around (fact wind.period).
pub const PERIOD: f64 = 8.0;
/// The direction's turning rate, radians per second.
pub const OMEGA: f64 = TAU / PERIOD;
/// The radius of every loop: speed / ω = 4/π m (fact wind.streak.radius).
pub const R: f64 = 1.0 / OMEGA;

/// The wind at time `t`, metres per second: speed one, the same everywhere.
pub fn wind(t: f64) -> (f64, f64) {
    ((OMEGA * t).cos(), (OMEGA * t).sin())
}

/// Where the dye released at `tau` is at time `now`, metres from the nozzle.
pub fn streak(now: f64, tau: f64) -> (f64, f64) {
    (
        R * ((OMEGA * now).sin() - (OMEGA * tau).sin()),
        R * ((OMEGA * tau).cos() - (OMEGA * now).cos()),
    )
}

/// The first puff at time `now`: the dye's far end, on its pathline.
pub fn puff(now: f64) -> (f64, f64) {
    (R * (OMEGA * now).sin(), R * (1.0 - (OMEGA * now).cos()))
}

/// The centre of the dye's arc at time `now`. It turns around the nozzle.
pub fn streak_centre(now: f64) -> (f64, f64) {
    (R * (OMEGA * now).sin(), -R * (OMEGA * now).cos())
}

/// The centre of the first puff's pathline circle. It never moves.
pub const PUFF_CENTRE: (f64, f64) = (0.0, R);

/// Polyline samples for a whole loop: 1.5° chords, whose sag (2.2e-4 scene
/// units) is about 0.05 px at 4K with the crate's 9-unit frame height.
pub const SAMPLES: usize = 240;

fn samples(now: f64) -> usize {
    ((SAMPLES as f64 * now / PERIOD).ceil() as usize).max(1) + 1
}

/// The `i`-th of `n` even steps from 0 to `end`, the last exactly `end`.
fn step(end: f64, i: usize, n: usize) -> f64 {
    if i + 1 == n {
        end
    } else {
        end * i as f64 / (n - 1) as f64
    }
}

/// The dye at time `now`, from its far end (τ = 0, the first puff) to its
/// near end (τ = `now`, exactly the nozzle).
pub fn dye_points(now: f64) -> Vec<(f64, f64)> {
    let n = samples(now);
    (0..n).map(|i| streak(now, step(now, i, n))).collect()
}

/// The first puff's track up to time `now`, from the nozzle to the puff.
pub fn path_points(now: f64) -> Vec<(f64, f64)> {
    let n = samples(now);
    (0..n).map(|i| puff(step(now, i, n))).collect()
}

/// The flow time shown on frame `k` of an `n`-frame time-lapse. Exact
/// integers in f64, so 2, 4, 6 and 8 s fall on whole frames at 10, 30 and
/// 60 fps (frames n/4, n/2, 3n/4 and n).
pub fn flow_at(k: u64, n: u64) -> f64 {
    PERIOD * k as f64 / n as f64
}

/// Checkpoints the storyboard names, `(T, τ, x, y)` in metres from the
/// nozzle. `check_wind01.py` verifies these literals by its own quadrature;
/// the tests verify [`streak`] against them.
#[cfg_attr(not(test), allow(dead_code))]
pub const CHECKPOINTS: [(f64, f64, f64, f64); 12] = [
    // 2 s: a quarter arc up and to the right, about (r, 0)
    (2.0, 0.0, 1.2732395447351628, 1.2732395447351628),
    (2.0, 1.0, 0.3729232285780567, 0.9003163161571061),
    (2.0, 2.0, 0.0, 0.0),
    // 4 s: the left half of the circle above, its far end 2.55 m up
    (4.0, 0.0, 0.0, 2.5464790894703255),
    (4.0, 2.0, -1.2732395447351628, 1.2732395447351628),
    // 6 s: out to 2.55 m left of the nozzle
    (6.0, 0.0, -1.2732395447351628, 1.2732395447351628),
    (6.0, 2.0, -2.5464790894703255, 0.0),
    // 8 s: closed into the circle below, about (0, −r)
    (8.0, 0.0, 0.0, 0.0),
    (8.0, 2.0, -1.2732395447351628, -1.2732395447351628),
    (8.0, 4.0, 0.0, -2.5464790894703255),
    (8.0, 6.0, 1.2732395447351628, -1.2732395447351628),
    (8.0, 8.0, 0.0, 0.0),
];

/// The dye arc's centre at the quarter turns, `(T, x, y)` in metres: it goes
/// once around the nozzle while the pathline's centre stays at (0, r).
#[cfg_attr(not(test), allow(dead_code))]
pub const CENTRES: [(f64, f64, f64); 5] = [
    (0.0, 0.0, -1.2732395447351628),
    (2.0, 1.2732395447351628, 0.0),
    (4.0, 0.0, 1.2732395447351628),
    (6.0, -1.2732395447351628, 0.0),
    (8.0, 0.0, -1.2732395447351628),
];

// ------------------------------------------------------------------ layout

/// Scene units per metre, fixed from 02_3 to 02_10.
pub const SCALE: f64 = 0.9;
/// The 6 m window's centre and half side, scene units: x −6.15 to −0.75,
/// y −2.7 to 2.7.
pub const WINDOW: (f64, f64) = (-3.45, 0.0);
pub const HALF: f64 = 2.7;
/// The nozzle, 2/π m right of the window's centre: (−2.877, 0).
pub const NOZZLE: (f64, f64) = (WINDOW.0 + SCALE * 2.0 / PI, WINDOW.1);
/// The 1 m ruler (0.9 units), its end ticks and its label above it.
pub const RULER: [(f64, f64); 2] = [(-6.0, -2.5), (-5.1, -2.5)];
pub const RULER_TICK: f64 = 0.08;
pub const RULER_LABEL_AT: (f64, f64) = (-5.55, -2.27);
/// The velocity arrows: a 4 × 4 grid 1.2 m apart about the window's centre,
/// each 0.8 m long (0.72 units) and turning about its own middle.
pub const ARROW_OFFSETS: [f64; 4] = [-1.62, -0.54, 0.54, 1.62];
pub const ARROW_LENGTH: f64 = 0.72;
pub const ARROW_THICKNESS: f64 = 3.0;
/// The streamlines at 8 s: horizontal, through each arrow row and halfway
/// between rows, inset from the window's sides.
pub const STREAM_ROWS: [f64; 7] = [-1.62, -1.08, -0.54, 0.0, 0.54, 1.08, 1.62];
pub const STREAM_INSET: f64 = 0.1;

pub const TITLE: &str = "What a flow picture shows";
pub const TITLE_AT: (f64, f64) = (-7.0, 3.4);
/// The clock, on one baseline: "t =" at the left, the flow time, then "s".
pub const CLOCK_AT: (f64, f64) = (0.9, 1.35);
pub const CLOCK_NUM_DX: f64 = 0.7;
pub const CLOCK_UNIT_DX: f64 = 1.3;
pub const CLOCK_NOTE: &str = "time-lapse: 8 s of flow in 4 s of film";
pub const CLOCK_NOTE_AT: (f64, f64) = (0.9, 0.95);
/// The prediction card (02_5 to 02_6).
pub const CARD_AT: (f64, f64) = (3.65, -0.7);
pub const CARD_SIZE: (f64, f64) = (5.0, 2.7);
pub const QUESTION: &str = "Is the air swirling?";
pub const CHOICES: [&str; 2] = ["yes", "no"];
pub const PAUSE_NOTE: &str = "pause for more time";
pub const RING_R: f64 = 0.36;
/// The key (02_10): a swatch in each curve's own style, then its name.
pub const KEY_AT: (f64, f64) = (0.9, 0.35);
pub const KEY_STEP: f64 = 0.65;
pub const KEY_SWATCH: f64 = 0.6;
pub const KEY: [&str; 3] = ["streamline (instant)", "pathline (one parcel)", "streakline (dye)"];
/// The Precisely lower third (02_10), the script's words in three lines.
pub const PRECISELY: [&str; 3] = [
    "Precisely: In a steady flow, streamlines, pathlines and streaklines coincide.",
    "In an unsteady flow they can differ, but need not: if the direction at every point",
    "stays fixed and only the speed changes, all three still lie along the same curves.",
];
pub const PRECISELY_Y: [f64; 3] = [-3.12, -3.47, -3.82];

pub const TITLE_PT: f64 = 42.0;
pub const READOUT_PT: f64 = 34.0;
pub const CARD_PT: f64 = 34.0;
pub const KEY_PT: f64 = 30.0;
pub const FINE_PT: f64 = 24.0;

const FRAME_W: f64 = 2.0;
const RULER_W: f64 = 3.0;
const DYE_W: f64 = 6.0;
const PATH_W: f64 = 5.0;
const STREAM_W: f64 = 2.5;
const STREAM_O: f64 = 0.55;
const HALO_W: f64 = 14.0;
const HALO_O: f64 = 0.9;
const CENTRE_W: f64 = 3.0;
const NOZZLE_R: f64 = 0.09;
const NOZZLE_RING: f64 = 0.14;
const PUFF_R: f64 = 0.085;
const CENTRE_R: f64 = 0.075;

// Draw order: field under the halo under the curves under the marks.
const Z_FRAME: i32 = 0;
const Z_FIELD: i32 = 1;
const Z_HALO: i32 = 2;
const Z_CURVES: i32 = 3;
const Z_MARKS: i32 = 4;
const Z_PUFF: i32 = 5;
const Z_TEXT: i32 = 6;

// Timing, seconds. Every value is exact in binary and a whole number of
// frames at the 60 fps delivery rate, so no segment gains a frame from f64
// noise (fmn rounds a segment up on its exact f64). At 30 fps the 0.25 and
// 0.75 s steps round up to 8 and 23 frames; the time-lapse and the
// countdown are frame loops and stay exact at any rate.
/// Film seconds for the eight seconds of flow (02_4, 02_6, 02_8).
pub const FILM_SECONDS: f64 = 4.0;
/// The 02_h1 prediction countdown.
pub const HOLD_SECONDS: f64 = 5.0;
/// The clean hold after 02_10's line and breath, before root's 02_11.
pub const FINAL_HOLD: f64 = 2.0;
const OPEN: f64 = 0.75;
const FADE: f64 = 0.5;
const QUICK: f64 = 0.25;
const ARROWS_IN: f64 = 0.75;
const LINES_IN: f64 = 1.0;
const HALO_IN: f64 = 1.0;

/// Dye (history): rose, its only role in the film (opening03 colour table).
pub fn rose() -> Srgb {
    hex("#EC92AB")
}

/// Streamlines (instant): pale sky, thin, kept apart from the velocity blue.
pub fn sky() -> Srgb {
    hex("#BFE3F2")
}

/// The teaching highlight: a white outline.
pub fn white() -> Srgb {
    hex("#FFFFFF")
}

fn card_fill() -> Srgb {
    hex("#141C29")
}

// ----------------------------------------------------------------- helpers

fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

/// Toy metres (the nozzle at the origin) to scene units, at the one scale.
pub fn to_scene(p: (f64, f64)) -> Vec3 {
    v(NOZZLE.0 + SCALE * p.0, NOZZLE.1 + SCALE * p.1)
}

fn scene_points(points: &[(f64, f64)]) -> Vec<Vec3> {
    points.iter().map(|&p| to_scene(p)).collect()
}

fn line_style(color: Srgb, width: f64, opacity: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, opacity)
}

/// A detached mobject at draw layer `z`.
fn add(stage: &mut Stage<'_>, object: VMobject, z: i32) -> Mob {
    let m = stage.arena_mut().add(object);
    stage.set_z_index(m, z, true);
    m
}

/// The grid points the arrows turn about, top row first, left to right.
pub fn arrow_centres() -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for &oy in ARROW_OFFSETS.iter().rev() {
        for &ox in &ARROW_OFFSETS {
            out.push((WINDOW.0 + ox, WINDOW.1 + oy));
        }
    }
    out
}

/// The arrow about scene point `centre` along the wind at time `now`.
pub fn arrow_at(centre: (f64, f64), now: f64) -> Arrow {
    let (ux, uy) = wind(now);
    let h = ARROW_LENGTH / 2.0;
    Arrow::new(
        v(centre.0 - h * ux, centre.1 - h * uy),
        v(centre.0 + h * ux, centre.1 + h * uy),
    )
    .buff(0.0)
    .thickness(ARROW_THICKNESS)
    .color(Palette::blue())
}

/// What is left of a ring of radius `r` at `c`, drained clockwise from the
/// top (the corpus's countdown ring).
fn ring_points(c: (f64, f64), r: f64, share: f64) -> Vec<Vec3> {
    let share = share.clamp(0.002, 1.0);
    let steps = ((64.0 * share).ceil() as usize).max(1);
    (0..=steps)
        .map(|k| {
            let a = FRAC_PI_2 + TAU * share * k as f64 / steps as f64;
            v(c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// Fade `out` away and `into` in, together, over `rt` seconds.
fn swap(stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in into {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, &[], mobs, rt)
}

fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, mobs, &[], rt)
}

/// Draw `mobs` along their paths together, over `rt` seconds.
fn create_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        anims.push(stage.prepare(show_creation(m).rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Frames of film for `seconds` at the scene's rate.
fn film_frames(stage: &Stage<'_>, seconds: f64) -> u64 {
    frames_covering(seconds, stage.scene().time().fps())
}

/// Text placed by its lower-left corner, so a row shares one baseline.
fn text_on_baseline(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, Palette::ink(), v(at.0, at.1))?;
    stage.move_to(m, v(at.0, at.1), DL);
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

// ------------------------------------------------------------- the window

/// Every object the window and its side panel use, built once at 02_1 and
/// never rescaled. One tracker, [`Wind::flow`], drives every curve, mark,
/// arrow and the clock.
pub struct Wind {
    /// Seconds of flow on show, 0 to 8.
    pub flow: Mob,
    dye_alpha: Mob,
    gold_alpha: Mob,
    pub frame: Mob,
    pub title: Option<Mob>,
    pub ruler: Vec<Mob>,
    pub nozzle: Vec<Mob>,
    pub dye: Mob,
    pub gold: Mob,
    pub puff: Mob,
    /// The first puff's fixed centre (gold) and the dye arc's moving centre
    /// (rose).
    pub centres: [Mob; 2],
    pub arrows: Vec<Mob>,
    pub streamlines: Vec<Mob>,
    pub halo: Mob,
    /// "t =", "s", the time-lapse note and (from 02_4) the live flow time.
    pub clock: Vec<Mob>,
    pub key: Vec<Mob>,
    pub precisely: Vec<Mob>,
    /// Read only by [`Wind::leave`], root's exit.
    #[allow(dead_code)]
    curves: Mob,
    field: Mob,
}

impl Wind {
    /// Build everything detached, with its updaters.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, title: bool) -> fmn::Result<Self> {
        let flow = stage.add_value_tracker(0.0);
        let dye_alpha = stage.add_value_tracker(1.0);
        let gold_alpha = stage.add_value_tracker(0.0);
        let ink = Palette::ink();

        let outline = Rectangle::new()
            .width(2.0 * HALF)
            .height(2.0 * HALF)
            .style(line_style(Palette::faint(), FRAME_W, 1.0))
            .build()?
            .moved_to(v(WINDOW.0, WINDOW.1));
        let frame = add(stage, outline, Z_FRAME);
        let title = if title {
            let t = kit.text_left(stage, TITLE, TITLE_PT, ink, v(TITLE_AT.0, TITLE_AT.1))?;
            Some(t)
        } else {
            None
        };

        let [(x0, y0), (x1, _)] = RULER;
        let ruler_line = |a: Vec3, b: Vec3| polyline(vec![a, b]).style(line_style(ink, RULER_W, 1.0)).build();
        let ruler = vec![
            add(stage, ruler_line(v(x0, y0), v(x1, y0)), Z_MARKS),
            add(
                stage,
                ruler_line(v(x0, y0 - RULER_TICK), v(x0, y0 + RULER_TICK)),
                Z_MARKS,
            ),
            add(
                stage,
                ruler_line(v(x1, y0 - RULER_TICK), v(x1, y0 + RULER_TICK)),
                Z_MARKS,
            ),
            kit.text(stage, "1 m", FINE_PT, ink, v(RULER_LABEL_AT.0, RULER_LABEL_AT.1))?,
        ];

        let at = to_scene((0.0, 0.0));
        let nozzle = vec![
            add(
                stage,
                Dot::new().point(at).radius(NOZZLE_R).color(rose()).build(),
                Z_MARKS,
            ),
            add(
                stage,
                Circle::new()
                    .radius(NOZZLE_RING)
                    .arc_center(at)
                    .style(line_style(ink, 2.5, 1.0))
                    .build(),
                Z_MARKS,
            ),
        ];

        // The curves start as their 8 s shapes, invisible; the updater
        // redraws them from the formulas whenever the flow time moves.
        let full_loop = scene_points(&dye_points(PERIOD));
        let dye = add(
            stage,
            polyline(full_loop.clone())
                .style(line_style(rose(), DYE_W, 0.0))
                .build(),
            Z_CURVES,
        );
        let gold = add(
            stage,
            polyline(scene_points(&path_points(PERIOD)))
                .style(line_style(Palette::gold(), PATH_W, 0.0))
                .build(),
            Z_CURVES,
        );
        let halo = add(
            stage,
            polyline(full_loop).style(line_style(white(), HALO_W, HALO_O)).build(),
            Z_HALO,
        );
        let puff_dot = add(
            stage,
            Dot::new()
                .point(to_scene(puff(0.0)))
                .radius(PUFF_R)
                .color(Palette::gold())
                .build(),
            Z_PUFF,
        );
        let ring = |c: (f64, f64), color: Srgb| {
            Circle::new()
                .radius(CENTRE_R)
                .arc_center(to_scene(c))
                .style(line_style(color, CENTRE_W, 1.0))
                .build()
        };
        let fixed_centre = add(stage, ring(PUFF_CENTRE, Palette::gold()), Z_MARKS);
        let moving_centre = add(stage, ring(streak_centre(0.0), rose()), Z_MARKS);

        let grid = arrow_centres();
        let mut arrows = Vec::with_capacity(grid.len());
        for &c in &grid {
            arrows.push(add(stage, arrow_at(c, PERIOD).build()?, Z_FIELD));
        }
        let (sx0, sx1) = (WINDOW.0 - HALF + STREAM_INSET, WINDOW.0 + HALF - STREAM_INSET);
        let mut streamlines = Vec::with_capacity(STREAM_ROWS.len());
        for &dy in &STREAM_ROWS {
            let y = WINDOW.1 + dy;
            let line = polyline(vec![v(sx0, y), v(sx1, y)])
                .style(line_style(sky(), STREAM_W, STREAM_O))
                .build();
            streamlines.push(add(stage, line, Z_FIELD));
        }

        let clock = vec![
            text_on_baseline(kit, stage, "t =", READOUT_PT, CLOCK_AT)?,
            text_on_baseline(kit, stage, "s", READOUT_PT, (CLOCK_AT.0 + CLOCK_UNIT_DX, CLOCK_AT.1))?,
            kit.text_left(
                stage,
                CLOCK_NOTE,
                FINE_PT,
                Palette::dim(),
                v(CLOCK_NOTE_AT.0, CLOCK_NOTE_AT.1),
            )?,
        ];

        let swatches = [
            (sky(), STREAM_W, STREAM_O),
            (Palette::gold(), PATH_W, 1.0),
            (rose(), DYE_W, 1.0),
        ];
        let mut key = Vec::with_capacity(2 * KEY.len());
        for (i, (label, (color, width, opacity))) in KEY.iter().zip(swatches).enumerate() {
            let y = KEY_AT.1 - KEY_STEP * i as f64;
            let swatch = polyline(vec![v(KEY_AT.0, y), v(KEY_AT.0 + KEY_SWATCH, y)])
                .style(line_style(color, width, opacity))
                .build();
            key.push(add(stage, swatch, Z_TEXT));
            key.push(kit.text_left(stage, label, KEY_PT, ink, v(KEY_AT.0 + KEY_SWATCH + 0.25, y))?);
        }
        let mut precisely = Vec::with_capacity(PRECISELY.len());
        for (line, y) in PRECISELY.iter().zip(PRECISELY_Y) {
            precisely.push(kit.text(stage, line, FINE_PT, Palette::dim(), v(0.0, y))?);
        }

        // One updater for the curves and marks: dye and gold track from the
        // formulas, the puff on the dye's far end, the arc's centre.
        let curves = stage.arena_mut().add(VMobject::new());
        let mut last = (f64::NAN, f64::NAN, f64::NAN);
        stage.add_updater(
            curves,
            move |s, _| {
                let now = s.tracker_value(flow).unwrap_or(0.0);
                let dye_a = s.tracker_value(dye_alpha).unwrap_or(1.0);
                let gold_a = s.tracker_value(gold_alpha).unwrap_or(0.0);
                if (now, dye_a, gold_a) == last {
                    return;
                }
                let drawn = now > 0.0;
                if now != last.0 {
                    if drawn {
                        let _ = s.set_points(dye, polyline(scene_points(&dye_points(now))).build().points());
                        let _ = s.set_points(gold, polyline(scene_points(&path_points(now))).build().points());
                    }
                    s.move_to(puff_dot, to_scene(puff(now)), ORIGIN);
                    s.move_to(moving_centre, to_scene(streak_centre(now)), ORIGIN);
                }
                s.set_stroke(dye, None, None, Some(if drawn { dye_a } else { 0.0 }), None, false);
                s.set_stroke(gold, None, None, Some(if drawn { gold_a } else { 0.0 }), None, false);
                last = (now, dye_a, gold_a);
            },
            true,
        )?;
        stage.add_to_scene(curves)?;

        // The arrows' updater runs only while `field` is on the scene
        // (02_6 to the start of 02_8): every arrow turns with the wind.
        let field = stage.arena_mut().add(VMobject::new());
        let turning = arrows.clone();
        let mut last_now = f64::NAN;
        stage.add_updater(
            field,
            move |s, _| {
                let now = s.tracker_value(flow).unwrap_or(0.0);
                if now == last_now {
                    return;
                }
                for (&m, &c) in turning.iter().zip(&grid) {
                    if let Ok(shape) = arrow_at(c, now).build() {
                        let _ = s.set_points(m, shape.points());
                    }
                }
                last_now = now;
            },
            true,
        )?;

        Ok(Self {
            flow,
            dye_alpha,
            gold_alpha,
            frame,
            title,
            ruler,
            nozzle,
            dye,
            gold,
            puff: puff_dot,
            centres: [fixed_centre, moving_centre],
            arrows,
            streamlines,
            halo,
            clock,
            key,
            precisely,
            curves,
            field,
        })
    }

    /// The clock comes in with the first time-lapse: "t = 0.0 s" on one
    /// baseline, live from the flow time, and the time-lapse note. A digital
    /// readout only: a turning clock hand would be a direction indicator,
    /// which 02_4 and the 02_h1 hold must not show.
    fn clock_in(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let flow = self.flow;
        let readout = kit.live_value(
            stage,
            move |s| s.tracker_value(flow).unwrap_or(0.0),
            1,
            false,
            READOUT_PT,
            Palette::ink(),
            v(CLOCK_AT.0 + CLOCK_NUM_DX, CLOCK_AT.1),
            DL,
        )?;
        for &m in &self.clock {
            stage.add_to_scene(m)?;
        }
        self.clock.push(readout);
        Ok(())
    }

    /// Eight seconds of flow in [`FILM_SECONDS`] of film, set frame by
    /// frame from [`flow_at`], so every frame is drawn at its exact time.
    pub fn time_lapse(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = film_frames(stage, FILM_SECONDS);
        for k in 1..=n {
            stage.set_tracker_value(self.flow, flow_at(k, n))?;
            wait_frames(stage, 1)?;
        }
        Ok(())
    }

    /// The time-lapse again: the loop fades, the clock returns to 0, `with`
    /// fades in at its 0 s place, and the dye regrows from the same formula.
    fn replay(&self, stage: &mut Stage<'_>, with: &[Mob], gold: bool) -> fmn::Result<()> {
        stage.play(explainer_kit::tween(self.dye_alpha, 0.0, QUICK)?)?;
        stage.set_tracker_value(self.flow, 0.0)?;
        stage.set_tracker_value(self.dye_alpha, 1.0)?;
        if gold {
            stage.set_tracker_value(self.gold_alpha, 1.0)?;
            stage.add_to_scene(self.gold)?;
        }
        if with.is_empty() {
            wait_frames(stage, 1)?;
        } else {
            fade_in_all(stage, with, QUICK)?;
        }
        self.time_lapse(stage)
    }

    fn arrows_in(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.field)?;
        fade_in_all(stage, &self.arrows, ARROWS_IN)
    }

    /// The instant's pictures go before the replay that follows one parcel.
    fn field_out(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.remove_from_scene(self.field);
        let mut out = self.arrows.clone();
        out.extend(&self.streamlines);
        fade_out_all(stage, &out, FADE)
    }

    /// Everything of the window's, for root's exit at 02_11.
    #[allow(dead_code)]
    pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.remove_from_scene(self.curves);
        stage.remove_from_scene(self.field);
        let mut all = vec![self.frame, self.dye, self.gold, self.puff, self.halo];
        all.extend(self.title);
        for group in [
            &self.ruler,
            &self.nozzle,
            &self.arrows,
            &self.streamlines,
            &self.clock,
            &self.key,
            &self.precisely,
        ] {
            all.extend(group);
        }
        all.extend(self.centres);
        let shown: Vec<Mob> = {
            let on = stage.scene().mobjects();
            all.into_iter().filter(|m| on.contains(m)).collect()
        };
        fade_out_all(stage, &shown, rt)
    }
}

// ---------------------------------------------------------- the 02_h1 card

/// "Is the air swirling?" with yes and no, "pause for more time" and a ring
/// that drains over [`HOLD_SECONDS`] with the seconds left in its middle.
pub struct SwirlCard {
    pub back: Mob,
    pub question: Mob,
    pub yes: Mob,
    pub no: Mob,
    pub note: Mob,
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub answer: Mob,
    left: Mob,
    anchor: Mob,
}

impl SwirlCard {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (cx, cy) = CARD_AT;
        let (w, h) = CARD_SIZE;
        let ink = Palette::ink();
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18)
            .style(
                Style::default()
                    .fill(card_fill(), 0.96)
                    .stroke(Palette::dim(), 2.5, 1.0),
            )
            .build()?
            .moved_to(v(cx, cy));
        let back = add(stage, back, Z_TEXT);
        let question = kit.text(stage, QUESTION, CARD_PT, ink, v(cx, cy + 0.85))?;
        let yes = kit.text(stage, CHOICES[0], CARD_PT, ink, v(cx - 1.0, cy + 0.05))?;
        let no = kit.text(stage, CHOICES[1], CARD_PT, ink, v(cx + 1.0, cy + 0.05))?;
        let ring_at = (cx + w / 2.0 - 0.75, cy - h / 2.0 + 0.6);
        let note = kit.text_left(
            stage,
            PAUSE_NOTE,
            FINE_PT,
            Palette::dim(),
            v(cx - w / 2.0 + 0.35, ring_at.1),
        )?;
        let track = add(
            stage,
            Circle::new()
                .radius(RING_R)
                .arc_center(v(ring_at.0, ring_at.1))
                .style(line_style(Palette::faint(), 6.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let ring = add(
            stage,
            polyline(ring_points(ring_at, RING_R, 1.0))
                .style(line_style(ink, 7.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let seconds = HOLD_SECONDS as usize;
        let mut digits = Vec::with_capacity(seconds);
        for second in 1..=seconds {
            let d = kit.text(stage, &second.to_string(), CARD_PT, ink, v(ring_at.0, ring_at.1))?;
            stage.set_z_index(d, Z_TEXT, true);
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let answer = Rectangle::new()
            .width(1.0)
            .height(0.62)
            .corner_radius(0.1)
            .style(line_style(Palette::green(), 3.0, 1.0))
            .build()?
            .moved_to(v(cx + 1.0, cy + 0.05));
        let answer = add(stage, answer, Z_TEXT);
        for m in [question, yes, no, note] {
            stage.set_z_index(m, Z_TEXT, true);
        }

        let left = stage.add_value_tracker(HOLD_SECONDS);
        let anchor = stage.arena_mut().add(VMobject::new());
        let shown = digits.clone();
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let l = s.tracker_value(left).unwrap_or(HOLD_SECONDS);
                if l == last {
                    return;
                }
                let _ = s.set_points(
                    ring,
                    polyline(ring_points(ring_at, RING_R, l / HOLD_SECONDS))
                        .build()
                        .points(),
                );
                let current = l.ceil() as usize;
                for (k, &d) in shown.iter().enumerate() {
                    s.set_opacity(d, if l > 0.0 && current == k + 1 { 1.0 } else { 0.0 }, true);
                }
                last = l;
            },
            true,
        )?;
        Ok(Self {
            back,
            question,
            yes,
            no,
            note,
            track,
            ring,
            digits,
            answer,
            left,
            anchor,
        })
    }

    /// The card, its question and choices, the note and the ring's track.
    pub fn show(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        fade_in_all(
            stage,
            &[self.back, self.question, self.yes, self.no, self.note, self.track],
            FADE,
        )
    }

    /// The countdown, frame by frame: "5" for the first second, "1" for the
    /// last, nothing at zero; then the ring, its track and the note go.
    pub fn count(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        let n = film_frames(stage, HOLD_SECONDS);
        for k in 1..=n {
            stage.set_tracker_value(self.left, HOLD_SECONDS * (n - k) as f64 / n as f64)?;
            wait_frames(stage, 1)?;
        }
        stage.remove_from_scene(self.anchor);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        fade_out_all(stage, &[self.ring, self.track, self.note], QUICK)
    }

    /// "No.": the answer is marked green.
    pub fn answer(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.set_color(self.no, Palette::green(), None, true);
        fade_in_all(stage, &[self.answer], QUICK)
    }

    pub fn leave(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        fade_out_all(stage, &[self.back, self.question, self.yes, self.no, self.answer], FADE)
    }
}

// --------------------------------------------------------------- the cues

/// Root's tunnel halves of 02_1 and 02_2: the tunnel, its thumbnail and the
/// open question card are root's objects. Each hook runs right after its
/// line starts, before the window's own picture.
pub trait TunnelCues {
    /// 02_1: the full tunnel shrinks to its thumbnail (top right, x 3.9 to
    /// 6.9, y 2.75 to 3.75). The window opens when this returns.
    fn shrink_to_thumbnail(&mut self, _kit: &Kit, _stage: &mut Stage<'_>) -> fmn::Result<()> {
        Ok(())
    }

    /// 02_2: the word "velocity" and a clock stamp in the thumbnail.
    /// FUTURE-SIM F3: no arrows in the tunnel until a validated saved state
    /// exists. The window stays empty through this line.
    fn velocity_stamp(&mut self, _kit: &Kit, _stage: &mut Stage<'_>) -> fmn::Result<()> {
        Ok(())
    }
}

/// No tunnel: the window alone (previews and tests).
#[cfg(test)]
pub struct NoTunnel;

#[cfg(test)]
impl TunnelCues for NoTunnel {}

/// 02_1 "Before you trust any picture of a flow…": after root's shrink, the
/// clean window opens on the left (and the title, if asked for).
pub fn cue_02_1(kit: &Kit, stage: &mut Stage<'_>, tunnel: &mut dyn TunnelCues, title: bool) -> fmn::Result<Wind> {
    kit.say(stage, "02_1")?;
    tunnel.shrink_to_thumbnail(kit, stage)?;
    let wind = Wind::build(kit, stage, title)?;
    let mut opening = vec![wind.frame];
    opening.extend(wind.title);
    fade_in_all(stage, &opening, OPEN)?;
    Ok(wind)
}

/// 02_2 "What the program computes is the velocity…": root's thumbnail
/// stamp only (FUTURE-SIM F3).
pub fn cue_02_2(kit: &Kit, stage: &mut Stage<'_>, tunnel: &mut dyn TunnelCues) -> fmn::Result<()> {
    kit.say(stage, "02_2")?;
    tunnel.velocity_stamp(kit, stage)
}

/// 02_3 "Dye is different…": one rose nozzle at its place, with the ruler.
pub fn cue_02_3(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind) -> fmn::Result<()> {
    kit.say(stage, "02_3")?;
    let mut shown = wind.nozzle.clone();
    shown.extend(&wind.ruler);
    fade_in_all(stage, &shown, FADE)
}

/// 02_4 "…A nozzle releases dye steadily for eight seconds.": the clock and
/// the time-lapse from sentence 1. No arrows, no direction indicator and no
/// mark on the dye's far end.
pub fn cue_02_4(kit: &Kit, stage: &mut Stage<'_>, wind: &mut Wind) -> fmn::Result<()> {
    kit.say(stage, "02_4")?;
    kit.until_sentence(stage, "02_4", 1)?;
    wind.clock_in(kit, stage)?;
    stage.add_to_scene(wind.dye)?;
    wind.time_lapse(stage)
}

/// 02_5 "…Is the air swirling?": the loop and the ruler stand; the card
/// comes with the question (sentence 1).
pub fn cue_02_5(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<SwirlCard> {
    kit.say(stage, "02_5")?;
    let card = SwirlCard::build(kit, stage)?;
    kit.until_sentence(stage, "02_5", 1)?;
    card.show(stage)?;
    Ok(card)
}

/// HOLD 02_h1: the question finishes, the moment is marked for the score,
/// and the 5 s countdown runs. Nothing new enters the window.
pub fn hold_02_h1(kit: &Kit, stage: &mut Stage<'_>, card: &SwirlCard) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    card.count(stage)
}

/// 02_6 "No. …": the answer; identical blue arrows pointing right (the 8 s
/// direction) with sentence 1; the card goes and the time-lapse replays with
/// every arrow turning together from sentence 2.
pub fn cue_02_6(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind, card: &SwirlCard) -> fmn::Result<()> {
    kit.say(stage, "02_6")?;
    card.answer(stage)?;
    kit.until_sentence(stage, "02_6", 1)?;
    wind.arrows_in(stage)?;
    kit.until_sentence(stage, "02_6", 2)?;
    card.leave(stage)?;
    wind.replay(stage, &[], false)
}

/// 02_7 "…A streamline follows the arrows at one instant…": the pale
/// horizontal streamlines of 8 s, drawn along the arrows with sentence 1.
pub fn cue_02_7(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind) -> fmn::Result<()> {
    kit.say(stage, "02_7")?;
    kit.until_sentence(stage, "02_7", 1)?;
    create_all(stage, &wind.streamlines, LINES_IN)
}

/// 02_8 "A pathline is the track of one bit of air…": the instant's arrows
/// and streamlines go; with sentence 1 the time-lapse replays once more. The
/// gold puff rides the dye's far end and traces its fixed circle about the
/// gold centre, while the rose arc curls about a rose centre that travels
/// around the nozzle. At 8 s the gold circle above and the rose circle
/// below stand together.
pub fn cue_02_8(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind) -> fmn::Result<()> {
    kit.say(stage, "02_8")?;
    wind.field_out(stage)?;
    kit.until_sentence(stage, "02_8", 1)?;
    let marks = [wind.puff, wind.centres[0], wind.centres[1]];
    wind.replay(stage, &marks, true)
}

/// 02_9 "And a streakline is everything that passed through one point…":
/// the puff and the centres go; the rose loop is outlined white with
/// sentence 2.
pub fn cue_02_9(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind) -> fmn::Result<()> {
    kit.say(stage, "02_9")?;
    fade_out_all(stage, &[wind.puff, wind.centres[0], wind.centres[1]], FADE)?;
    kit.until_sentence(stage, "02_9", 2)?;
    create_all(stage, &[wind.halo], HALO_IN)
}

/// 02_10 "Three curves, one flow…": the outline goes and the streamlines
/// return where they were; the key names each curve by its colour; the
/// Precisely with sentence 1; then the line's end, its breath and a clean
/// hold with nothing moving.
pub fn cue_02_10(kit: &Kit, stage: &mut Stage<'_>, wind: &Wind) -> fmn::Result<()> {
    kit.say(stage, "02_10")?;
    swap(stage, &[wind.halo], &wind.streamlines, FADE)?;
    fade_in_all(stage, &wind.key, FADE)?;
    kit.until_sentence(stage, "02_10", 1)?;
    fade_in_all(stage, &wind.precisely, FADE)?;
    kit.hold(stage)?;
    let still = film_frames(stage, FINAL_HOLD);
    wait_frames(stage, still)
}

/// Cues 02_1 to 02_10 and the 02_h1 hold, in order. Returns the window, still
/// on screen, for root's 02_11.
pub fn chapter02(kit: &Kit, stage: &mut Stage<'_>, tunnel: &mut dyn TunnelCues, title: bool) -> fmn::Result<Wind> {
    let mut wind = cue_02_1(kit, stage, tunnel, title)?;
    cue_02_2(kit, stage, tunnel)?;
    cue_02_3(kit, stage, &wind)?;
    cue_02_4(kit, stage, &mut wind)?;
    let card = cue_02_5(kit, stage)?;
    hold_02_h1(kit, stage, &card)?;
    cue_02_6(kit, stage, &wind, &card)?;
    cue_02_7(kit, stage, &wind)?;
    cue_02_8(kit, stage, &wind)?;
    cue_02_9(kit, stage, &wind)?;
    cue_02_10(kit, stage, &wind)?;
    Ok(wind)
}

/// The window alone as a scene (no tunnel).
#[cfg(test)]
pub struct RotatingWindScene {
    pub kit: Kit,
    pub title: bool,
}

#[cfg(test)]
impl SceneConstruct for RotatingWindScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter02(&self.kit, stage, &mut NoTunnel, self.title).map(|_| ())
    }
}

/// The spoken lines this scene says, verbatim from `navier-full-script03.md`
/// (`check_wind01.py` compares them byte for byte), as `script.tsv` rows.
/// 02_h1 is a hold, not a line.
#[cfg(test)]
pub const SCRIPT02: &str = concat!(
    "02_1\tBefore you trust any picture of a flow, ask what it's a picture of.\n",
    "02_2\tWhat the program computes is the velocity, which way the air moves and how fast, all through the tunnel. Drawn as arrows, it's a snapshot of one instant.\n",
    "02_3\tDye is different. The air carries it along, and it doesn't push back. It also remembers. The dye you see now carries a record of the past.\n",
    "02_4\tHere's a flow we can work out exactly. A nozzle releases dye steadily for eight seconds.\n",
    "02_5\tThe dye has closed into a loop, eight meters around. Is the air swirling?\n",
    "02_6\tNo. At every instant, all the air moves the same way, at one meter per second, so nothing spins and there's no center it all turns around. Only the direction turns, once around every eight seconds. The loop is a record of that turning.\n",
    "02_7\tSo one flow gives three different curves. A streamline follows the arrows at one instant. Here, the streamlines are straight lines.\n",
    "02_8\tA pathline is the track of one bit of air over time. The first puff from the nozzle went around a circle above it, and it has just arrived back. Every bit of air makes the same loop, each around its own center.\n",
    "02_9\tAnd a streakline is everything that passed through one point, wherever it is now. That's what continuous dye shows. Here, it's the circle below the nozzle.\n",
    "02_10\tThree curves, one flow. When the flow doesn't change in time, all three are the same curve. When it does change, they can come apart, as they did here.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::{Narrator, Script};

    const EXACT: f64 = 1e-12;

    fn near(a: (f64, f64), b: (f64, f64), tol: f64) -> bool {
        (a.0 - b.0).abs() <= tol && (a.1 - b.1).abs() <= tol
    }

    fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
        (a.0 - b.0).hypot(a.1 - b.1)
    }

    #[test]
    fn the_named_checkpoints_follow_from_the_formula() {
        for &(now, tau, x, y) in &CHECKPOINTS {
            assert!(
                near(streak(now, tau), (x, y), EXACT),
                "T {now}, tau {tau}: {:?}",
                streak(now, tau)
            );
        }
        for &(now, x, y) in &CENTRES {
            assert!(near(streak_centre(now), (x, y), EXACT), "centre at T {now}");
        }
    }

    /// The quarter turns, on the frames that show them, at every rate.
    #[test]
    fn quarter_turns_fall_on_whole_frames() {
        for fps in [10_u32, 30, 60] {
            let n = frames_covering(FILM_SECONDS, fps);
            assert_eq!(n, u64::from(4 * fps));
            for (k, want) in [(n / 4, 2.0), (n / 2, 4.0), (3 * n / 4, 6.0), (n, 8.0)] {
                assert_eq!(flow_at(k, n), want, "{fps} fps, frame {k}");
            }
        }
        // 2 s: the quarter arc from the nozzle to (r, r), about (r, 0)
        let quarter = dye_points(2.0);
        assert!(near(quarter[0], (R, R), EXACT) && near(*quarter.last().unwrap(), (0.0, 0.0), 0.0));
        assert!(quarter.iter().all(|&p| (dist(p, (R, 0.0)) - R).abs() < EXACT));
        // 4 s: the left half of the circle above, its far end at the top
        let half = dye_points(4.0);
        assert!(near(half[0], (0.0, 2.0 * R), EXACT));
        assert!(
            half.iter()
                .all(|&p| p.0 <= EXACT && (dist(p, PUFF_CENTRE) - R).abs() < EXACT)
        );
        // 6 s: reaching 2.55 m left of the nozzle
        let left = dye_points(6.0).iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        assert!((left + 2.0 * R).abs() < 1e-4, "{left}");
        // 8 s: closed into the circle below
        let closed = dye_points(8.0);
        assert!(near(closed[0], *closed.last().unwrap(), EXACT));
        assert!(closed.iter().all(|&p| (dist(p, (0.0, -R)) - R).abs() < EXACT));
        assert!((TAU * R - 8.0).abs() < EXACT, "eight metres around");
    }

    /// Both common endpoints, on every frame of a 60 fps time-lapse: the
    /// near end is the nozzle and the far end is the first puff on its
    /// pathline circle.
    #[test]
    fn both_ends_hold_on_every_frame() {
        let n = frames_covering(FILM_SECONDS, 60);
        for k in 1..=n {
            let now = flow_at(k, n);
            let dye = dye_points(now);
            assert_eq!(*dye.last().unwrap(), (0.0, 0.0), "near end at T {now}");
            assert_eq!(dye[0], puff(now), "far end at T {now}");
            assert_eq!(*path_points(now).last().unwrap(), puff(now));
            assert_eq!(path_points(now)[0], (0.0, 0.0));
            assert!((dist(puff(now), PUFF_CENTRE) - R).abs() < EXACT);
            let c = streak_centre(now);
            assert!(
                dye.iter().all(|&p| (dist(p, c) - R).abs() < EXACT),
                "arc about its centre at T {now}"
            );
        }
    }

    /// The reveal: the arc's centre travels once around the nozzle (2r
    /// across at the half turn) while the pathline's centre stays put.
    #[test]
    fn the_arc_centre_moves_and_the_path_centre_does_not() {
        assert!((dist(streak_centre(4.0), streak_centre(0.0)) - 2.0 * R).abs() < EXACT);
        for k in 0..=64 {
            let now = PERIOD * f64::from(k) / 64.0;
            assert!((dist(streak_centre(now), (0.0, 0.0)) - R).abs() < EXACT);
            let path = path_points(now);
            assert!(path.iter().all(|&p| (dist(p, PUFF_CENTRE) - R).abs() < EXACT));
        }
    }

    /// Every drawn point of every frame stays in the window with the margins
    /// wind02 measured, and clear of the ruler.
    #[test]
    fn everything_stays_in_the_window() {
        let (lo_x, hi_x) = (WINDOW.0 - HALF, WINDOW.0 + HALF);
        let (lo_y, hi_y) = (WINDOW.1 - HALF, WINDOW.1 + HALF);
        let n = frames_covering(FILM_SECONDS, 60);
        let ruler: Vec<(f64, f64)> = (0..=20)
            .map(|k| (RULER[0].0 + 0.9 * f64::from(k) / 20.0, RULER[0].1))
            .collect();
        let (mut margin_x, mut margin_y, mut clear) = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
        for k in 0..=n {
            let now = flow_at(k, n);
            for p in dye_points(now).into_iter().chain(path_points(now)) {
                let q = to_scene(p);
                margin_x = margin_x.min(q[0] - lo_x).min(hi_x - q[0]);
                margin_y = margin_y.min(q[1] - lo_y).min(hi_y - q[1]);
                for &r in &ruler {
                    clear = clear.min(dist((q[0], q[1]), r));
                }
            }
        }
        assert!(margin_x > 0.975 && margin_y > 0.405, "{margin_x} {margin_y}");
        assert!(clear > 1.045, "{clear}");
        let h = ARROW_LENGTH / 2.0;
        for c in arrow_centres() {
            assert!(c.0 - h > lo_x && c.0 + h < hi_x && c.1 - h > lo_y && c.1 + h < hi_y);
        }
        assert!(((NOZZLE.0 + 2.877).abs() < 5e-4) && NOZZLE.1 == 0.0);
    }

    /// Every sentence a cue waits for exists in its line.
    #[test]
    fn every_sentence_gate_exists() {
        let script = Script::parse(SCRIPT02).unwrap();
        let narrator = Narrator::estimate(&script);
        for (id, j) in [
            ("02_4", 1),
            ("02_5", 1),
            ("02_6", 1),
            ("02_6", 2),
            ("02_7", 1),
            ("02_8", 1),
            ("02_9", 2),
            ("02_10", 1),
        ] {
            assert!(narrator.sentence_starts(id).len() > j, "{id} sentence {j}");
        }
    }

    /// The whole chapter runs headless on the scratch voice, at 10 fps.
    #[test]
    fn the_chapter_runs_on_the_scratch_voice() {
        let script = Script::parse(SCRIPT02).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let mut scene = RotatingWindScene { kit, title: true };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).expect("chapter 02 runs");
    }
}
