//! Chapter 05, "Nothing piles up": cues 05_1 to 05_11 and HOLD 05_h1, native
//! and exact.
//!
//! LavenderPike `lavender-continuity01` for root's `navier_stokes_simulation`
//! crate (Mail 36251). Cue text and pictures: `navier-full-script03.md`
//! chapter 05 (sha256 31a83a2b…), the same lines as `navier-opening-script03.md`
//! 285 to 324. Facts: `navier-opening-script01/runs/facts01-run02.txt`
//! (08e3870a…): ch.parcel.area, cell.in.left, cell.out.right, cell.in.top,
//! cell.bottom, cell.in.total, cell.net, ch.divergence, tunnel.blocked and
//! tunnel.gap.ratio. `check_continuity01.py` recomputes every one by its own
//! route.
//!
//! The chapter opens on chapter 04's last frame, rebuilt from
//! `parcel_forces01::Forces` (the channel, its pressure plate, the gold parcel
//! square and the law in the side panel), so the join is the same picture.
//! - 05_1: a "?" over the pressure slot.
//! - 05_2: the pressure picture goes. The parcel square is carried by the
//!   channel's own field, u = 2 + x, v = −y: x + 2 grows as eᵗ and y shrinks
//!   as e⁻ᵗ, so it stretches along the flow and narrows across it while its
//!   area stays 0.25 m². That is ch.parcel.area, exact at every frame. Its
//!   outline is redrawn from the same map, and a faint copy keeps the start.
//! - 05_3 to 05_6: the fixed cell square [0, 1] × [0, 1] m, in the same
//!   coordinates. Its face arrows show the flow across each face (the
//!   normal component), and their values come from integrating the field
//!   along each face: 2 in, 3 out, 1 in, 0. The ledger counts in 3 and out 3.
//!   The badge (§0.4: a fixed frame, the value swapped) is red at −3 while
//!   only the inflow is counted and green at 0 once the outflow is.
//!   ∇·u = ∂u/∂x + ∂v/∂y = 1 + (−1) comes from the field's differences.
//! - 05_7 to 05_11: root's tunnel, drawn as `chapter02-tunnel.rs` and
//!   `opening.rs` draw it. It has the benchmark geometry
//!   at the full view (width 11.5, centred at
//!   y 1.15). On it go two white slices, the region between shaded, "0" on
//!   the walls and the cylinder, braces for the supposed quarter, the
//!   free-answer card, and the 4/3 reveal with the gap arrows 4/3 as long as
//!   the inlet's. The law glides below the tunnel, and its "?" becomes
//!   ∇·u = 0, linked to the pressure term by a white line.
//!
//! The 05_8 line supposes a quarter. The benchmark cylinder blocks
//! 0.1/0.41 ≈ 0.244 of the height as drawn, and the braces carry the line's
//! supposition: see `HANDOFF.md`.
//!
//! Root's tunnel thumbnail goes through [`TunnelHandoff`] (nothing in
//! [`NoThumbnail`]). While the tunnel is full, nothing here enters
//! y ∈ [−1.65, −0.85] for x ∈ [−4.1, 4.1], the place root's chapter 09 gives
//! the open question card under the full tunnel.

use std::f64::consts::{FRAC_PI_2, PI};

use explainer_kit::narration::wait_frames;
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

use super::chapter03::{
    CardSpec, FADE, FINE_PT, HIGHLIGHT_W, LABEL_PT, LENGTH, OPEN, ORIGIN_AT, PredictCard, QUICK, READOUT_PT,
    RUN_SECONDS, SCALE, TITLE_AT, TITLE_PT, Z_FIELD, Z_HIGHLIGHT, Z_MARKS, Z_TEXT, Z_TRAIL, Z_WALLS, add, create_all,
    fade_in_all, fade_out_all, fail, film_frames, line_style, run_share, swap, text_at, to_scene, v, velocity, white,
};
use super::chapter04::{EQ_BASE, EQ_PT, Forces, PARCEL_CENTRE, PARCEL_SIDE, SLOT_H, SLOT_LIFT};

// ------------------------------------------------------ the parcel (05_2)

/// Where the flow carries the point that started at (x, y), channel metres,
/// once x + 2 has grown by `growth` (eᵗ after t seconds): the exact solution
/// of dx/dt = 2 + x, dy/dt = −y.
pub fn carried(x: f64, y: f64, growth: f64) -> (f64, f64) {
    ((2.0 + x) * growth - 2.0, y / growth)
}

/// The growth at which the square's leading edge reaches the exit: 16/9.
pub fn stretch_end() -> f64 {
    (2.0 + LENGTH) / (2.0 + PARCEL_CENTRE.0 + PARCEL_SIDE / 2.0)
}

/// Seconds of flow the stretch shows: ln(16/9) ≈ 0.58 s.
pub fn stretch_time() -> f64 {
    stretch_end().ln()
}

/// The growth at share `s` of the stretch: 1 at the start, [`stretch_end`]
/// at the end (equal flow time on every frame).
pub fn growth_at(s: f64) -> f64 {
    stretch_end().powf(s)
}

/// Left, right, bottom and top of the carried square at share `s`, metres.
pub fn parcel_box(s: f64) -> [f64; 4] {
    let g = growth_at(s);
    let half = PARCEL_SIDE / 2.0;
    let (l, b) = carried(PARCEL_CENTRE.0 - half, PARCEL_CENTRE.1 - half, g);
    let (r, t) = carried(PARCEL_CENTRE.0 + half, PARCEL_CENTRE.1 + half, g);
    [l, r, b, t]
}

/// The carried square's area, m²: 0.25 at every share (ch.parcel.area).
pub fn parcel_area(s: f64) -> f64 {
    let [l, r, b, t] = parcel_box(s);
    (r - l) * (t - b)
}

/// A scene point carried by the flow, through the channel's one scale.
fn carried_scene(p: Vec3, growth: f64) -> Vec3 {
    let (x, y) = carried((p[0] - ORIGIN_AT.0) / SCALE, (p[1] - ORIGIN_AT.1) / SCALE, growth);
    to_scene(x, y)
}

pub fn area_label() -> String {
    format!(r"\text{{area }}{}\,\mathrm{{m^{{2}}}}", parcel_area(0.0))
}

pub fn slow_note() -> String {
    format!(
        "slow motion: {:.2} s of flow in {RUN_SECONDS} s of film",
        stretch_time()
    )
}

// ------------------------------------------------------ the cell (05_3–6)

/// The cell square, channel metres: left, right, bottom, top.
pub const CELL: [f64; 4] = [0.0, 1.0, 0.0, 1.0];

/// The flow out through each face (left, right, bottom, top) per metre of
/// depth, m³/s: the field's outward normal component integrated along the
/// face by the midpoint rule, exact for this linear field. Positive is out,
/// as root's chapter 07 counts (`CELL = [-2, 3, 0, -1]`).
pub fn face_outflows() -> [f64; 4] {
    const PIECES: u32 = 64;
    let [l, r, b, t] = CELL;
    let (dx, dy) = ((r - l) / f64::from(PIECES), (t - b) / f64::from(PIECES));
    let mut out = [0.0; 4];
    for k in 0..PIECES {
        let share = (f64::from(k) + 0.5) / f64::from(PIECES);
        let (x, y) = (l + (r - l) * share, b + (t - b) * share);
        out[0] -= velocity(l, y).0 * dy;
        out[1] += velocity(r, y).0 * dy;
        out[2] -= velocity(x, b).1 * dx;
        out[3] += velocity(x, t).1 * dx;
    }
    out
}

/// What flows in, m³/s per metre of depth: 2 + 1 = 3 (cell.in.total).
pub fn inflow() -> f64 {
    face_outflows().iter().filter(|&&q| q < 0.0).map(|q| -q).sum()
}

/// What flows out: 3.
pub fn outflow() -> f64 {
    face_outflows().iter().filter(|&&q| q > 0.0).sum()
}

/// Out minus in: 0 (cell.net).
pub fn net_outflow() -> f64 {
    face_outflows().iter().sum()
}

/// The speed across each face (left, right, top, bottom), m/s: the normal
/// component at the face's middle, uniform along each face here.
pub fn face_speeds() -> [f64; 4] {
    let [l, r, b, t] = CELL;
    let (mx, my) = ((l + r) / 2.0, (b + t) / 2.0);
    [
        velocity(l, my).0,
        velocity(r, my).0,
        -velocity(mx, t).1,
        velocity(mx, b).1.abs(),
    ]
}

fn speed_tex(s: f64) -> String {
    if s == 0.0 {
        "0".to_owned()
    } else {
        format!(r"{s}\,\mathrm{{m/s}}")
    }
}

/// "2 m/s", "3 m/s", "1 m/s", "0": the face labels.
pub fn face_labels() -> [String; 4] {
    face_speeds().map(speed_tex)
}

/// "2 + 1 = 3 m³/s": the inflow, left then top.
pub fn in_row() -> String {
    let q = face_outflows();
    format!(r"{}+{}={}\,\mathrm{{m^{{3}}/s}}", -q[0], -q[3], inflow())
}

pub fn out_row() -> String {
    format!(r"{}\,\mathrm{{m^{{3}}/s}}", outflow())
}

/// The badge's two values: the net outflow with only the inflow counted
/// (−3, red), then with both (0, green).
pub fn badge_values() -> [String; 2] {
    [format!("{}", -inflow()), format!("{}", net_outflow())]
}

/// ∂u/∂x and ∂v/∂y at (x, y), per second, by central differences of the
/// field: 1 and −1 everywhere.
pub fn rates(x: f64, y: f64) -> (f64, f64) {
    let h = 1e-3;
    (
        (velocity(x + h, y).0 - velocity(x - h, y).0) / (2.0 * h),
        (velocity(x, y + h).1 - velocity(x, y - h).1) / (2.0 * h),
    )
}

/// ∇·u = ∂u/∂x + ∂v/∂y = 1 + (−1) = 0, the numbers from [`rates`] at the
/// cell's centre.
pub fn divergence_line() -> String {
    let [l, r, b, t] = CELL;
    let (a, c) = rates((l + r) / 2.0, (b + t) / 2.0);
    let (a, c) = (a.round(), c.round());
    format!(
        r"\nabla\cdot u=\partial u/\partial x+\partial v/\partial y={a}+({c})={}",
        a + c
    )
}

// ----------------------------------------------------- the tunnel (05_7–11)

/// Root's tunnel geometry (`field.rs` `TunnelGeometry`), metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TunnelShape {
    pub length: f64,
    pub height: f64,
    /// Centre x, centre y, radius.
    pub cylinder: [f64; 3],
}

/// Root's full view: `chapter02-tunnel.rs` FULL and `chapter09` TUNNEL_*.
pub const FULL_AT: (f64, f64) = (0.0, 1.15);
pub const FULL_WIDTH: f64 = 11.5;

impl TunnelShape {
    /// The solver's tunnel, the only one `field.rs` accepts.
    #[cfg(test)]
    pub const BENCHMARK: Self = Self {
        length: 2.2,
        height: 0.41,
        cylinder: [0.2, 0.2, 0.05],
    };

    pub fn scale(&self) -> f64 {
        FULL_WIDTH / self.length
    }

    /// Tunnel metres to scene units in the full view.
    pub fn at(&self, x: f64, y: f64) -> Vec3 {
        let k = self.scale();
        v(
            FULL_AT.0 + k * (x - self.length / 2.0),
            FULL_AT.1 + k * (y - self.height / 2.0),
        )
    }

    /// The share of the height the cylinder blocks as drawn (0.1/0.41).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn drawn_blockage(&self) -> f64 {
        2.0 * self.cylinder[2] / self.height
    }
}

/// The 05_8 supposition: the cylinder blocks 1/4 of the height
/// (tunnel.blocked).
pub const BLOCKED: (u32, u32) = (1, 4);

/// The open share, 3/4.
pub fn open_share() -> (u32, u32) {
    (BLOCKED.1 - BLOCKED.0, BLOCKED.1)
}

/// The gap's average over the inlet's, 4/3: the same flow through 3/4 of
/// the height (tunnel.gap.ratio).
pub fn gap_ratio() -> (u32, u32) {
    (BLOCKED.1, BLOCKED.1 - BLOCKED.0)
}

pub fn fraction(f: (u32, u32)) -> String {
    format!("{}/{}", f.0, f.1)
}

pub fn gap_readout() -> String {
    format!(r"{}\times\text{{inlet average}}", fraction(gap_ratio()))
}

pub fn open_line() -> String {
    format!("open: {} of the height", fraction(open_share()))
}

/// Root's side of the 05_7 handoff.
pub trait TunnelHandoff {
    /// 05_7, before the full tunnel fades in: root takes its tunnel thumbnail
    /// (top right since 02_13) off the screen.
    fn leave_thumbnail(&mut self, _kit: &Kit, _stage: &mut Stage<'_>) -> fmn::Result<()> {
        Ok(())
    }

    /// [`back_to_panel`], once the full tunnel has gone: root brings its
    /// thumbnail back.
    fn return_thumbnail(&mut self, _kit: &Kit, _stage: &mut Stage<'_>) -> fmn::Result<()> {
        Ok(())
    }
}

/// No thumbnail (the preview).
pub struct NoThumbnail;

impl TunnelHandoff for NoThumbnail {}

// ------------------------------------------------------------------ words

pub const TITLE: &str = "Nothing piles up";
pub const UNKNOWN: &str = "?";
pub const SIDE_LABEL: &str = "1 m";
pub const LEDGER_NAMES: [&str; 2] = ["in", "out"];
pub const NET_NAME: &str = "net outflow";
pub const DIVERGENCE_WORDS: &str = r"\text{divergence}=\text{net outflow}\div\text{size}";
pub const RULE: &str = r"\nabla\cdot u=0";
pub const COMPARE_NAMES: [&str; 2] = ["left", "right"];
pub const ZERO: &str = "0";
pub const INLET_LABEL: &str = "inlet average";
pub const IN_OUT: &str = "flow in = flow out";
pub const OPEN_WORD: &str = "open";

/// The three lower thirds (the source's "Precisely" lines for 05_2, 05_3 and
/// 05_10), two lines each; each pair joined by a space is the source's text.
pub const PRECISELY: [[&str; 2]; 3] = [
    [
        "This is our modelling choice for a low-speed tunnel.",
        "Sound waves, sonic booms and blasts need compressible flow.",
    ],
    [
        "Each face is one meter long, so per meter of depth",
        "these are also flows in cubic meters per second.",
    ],
    [
        "Average means the flow through the slice divided by its open height,",
        "per meter of depth. Near the walls the air is slower, and elsewhere faster.",
    ],
];

// ----------------------------------------------------------------- layout

/// The "?" sits this far above the pressure slot's top.
pub const UNKNOWN_LIFT: f64 = 0.3;
pub const UNKNOWN_PT: f64 = 48.0;
pub const AREA_AT: (f64, f64) = (3.65, -0.55);
pub const SLOW_AT: (f64, f64) = (3.65, -1.1);
pub const RING_PAD: f64 = 0.12;
pub const PRECISELY_Y: [f64; 2] = [-3.35, -3.67];

/// Scene units per m/s for the face arrows (across each face's middle).
pub const FACE_PER: f64 = 0.3;
pub const FLOW_THICK: f64 = 4.0;
pub const CELL_W: f64 = 3.0;
pub const CELL_FILL: f64 = 0.06;
pub const FACE_LABEL_GAP: f64 = 0.1;
pub const TOP_LABEL_GAP: f64 = 0.32;
pub const SIDE_LABEL_AT: (f64, f64) = (-4.75, 0.2);
pub const ZERO_BELOW: f64 = 0.22;

pub const LEDGER_LEFT: f64 = 1.0;
pub const LEDGER_VALUE_X: f64 = 2.0;
pub const LEDGER_Y: [f64; 2] = [-0.45, -0.95];
pub const RULE_Y: f64 = -1.3;
pub const RULE_X: (f64, f64) = (0.9, 5.2);
pub const NET_Y: f64 = -1.75;
pub const BADGE_AT: (f64, f64) = (4.4, -1.75);
pub const BADGE_SIZE: (f64, f64) = (1.1, 0.62);
pub const BADGE_STROKE: f64 = 2.2;

pub const DIVERGENCE_AT: [(f64, f64); 2] = [(3.65, -2.35), (3.65, -2.9)];
pub const DIVERGENCE_PT: f64 = 34.0;
pub const RULE_AT: (f64, f64) = (3.65, -2.6);

pub const COMPARE_TAIL_X: f64 = 1.9;
pub const COMPARE_PER: f64 = 0.5;
pub const COMPARE_Y: [f64; 2] = [-0.5, -1.1];

/// The law moves below the full tunnel at 05_7.
pub const GLIDE_DY: f64 = -3.3;
pub const LINK_GAP: f64 = 0.03;

/// The upstream slice, tunnel metres from the inlet.
pub const SLICE_A: f64 = 0.02;
pub const SLICE_W: f64 = 3.0;
pub const SHADE_ALPHA: f64 = 0.12;
pub const ARC_STEPS: u32 = 48;
/// The inlet's average-speed arrows: length (scene units) and heights (shares
/// of the height); the gap's arrows are [`gap_ratio`] times as long.
pub const INLET_L: f64 = 0.45;
pub const INLET_ROWS: [f64; 3] = [0.25, 0.5, 0.75];
pub const ZERO_GAP: f64 = 0.2;
pub const BELOW_Y: f64 = -0.5;
pub const IN_OUT_X: f64 = -2.9;
pub const OPEN_LINE_X: f64 = 1.0;
pub const BRACE_GAP: f64 = 0.08;
pub const BRACE_INSET: f64 = 0.03;
pub const BRACE_LABEL_GAP: f64 = 0.12;
pub const GAP_READOUT_X: f64 = -3.15;

/// The 05_h1 card, under the tunnel at the left (free answer: no choices).
pub const CARD05: CardSpec = CardSpec {
    at: (-3.31, -2.42),
    size: (7.0, 1.4),
    text_pt: 30.0,
    question: &["How fast, on average, through", "the open part of the slice?"],
    question_dy: &[0.42, 0.02],
    choices: &[],
    choice_dx: 0.0,
    choice_dy: 0.0,
    unit: None,
    answer: 0,
    answer_box: (0.1, 0.1),
    ring_lift: 0.42,
};

// ---------------------------------------------------------------- helpers

/// Detached Tex at `at`, moved so its `edge` is there.
fn tex_at(
    kit: &Kit,
    stage: &mut Stage<'_>,
    src: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, size, color, &[], at)?;
    stage.move_to(m, at, edge);
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

fn arrow(stage: &mut Stage<'_>, tail: Vec3, tip: Vec3, color: Srgb, z: i32) -> fmn::Result<Mob> {
    let a = Arrow::new(tail, tip)
        .buff(0.0)
        .thickness(FLOW_THICK)
        .color(color)
        .build()?;
    Ok(add(stage, a, z))
}

/// Left, right, bottom, top.
pub fn bounds(stage: &Stage<'_>, m: Mob) -> (f64, f64, f64, f64) {
    (
        stage.get_left(m)[0],
        stage.get_right(m)[0],
        stage.get_bottom(m)[1],
        stage.get_top(m)[1],
    )
}

/// A white teaching outline round `m`.
fn outline(stage: &mut Stage<'_>, m: Mob, pad: f64) -> fmn::Result<Mob> {
    let (l, r, b, t) = bounds(stage, m);
    let frame = Rectangle::new()
        .width(r - l + 2.0 * pad)
        .height(t - b + 2.0 * pad)
        .corner_radius(0.1)
        .style(line_style(white(), HIGHLIGHT_W, 1.0))
        .build()?
        .moved_to(v((l + r) / 2.0, (b + t) / 2.0));
    Ok(add(stage, frame, Z_HIGHLIGHT))
}

fn badge_frame(stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(BADGE_SIZE.0)
        .height(BADGE_SIZE.1)
        .corner_radius(0.07)
        .style(Style::default().fill(color, 0.0).stroke(color, BADGE_STROKE, 1.0))
        .build()?
        .moved_to(v(BADGE_AT.0, BADGE_AT.1));
    Ok(add(stage, frame, Z_TEXT))
}

/// A brace curling right of the vertical span y0..y1 at x, and its tip (as
/// chapter 03's length brace).
fn brace(stage: &mut Stage<'_>, x: f64, y0: f64, y1: f64) -> (Mob, Vec3) {
    let span = polyline(vec![v(x, y0), v(x, y1)]).build();
    let b = Brace::around(&span, RIGHT).buff(0.0).color(Palette::ink());
    let tip = b.tip();
    (add(stage, b.build(), Z_MARKS), tip)
}

/// The mobs of `mobs` on screen now.
pub fn shown(stage: &Stage<'_>, mobs: &[Mob]) -> Vec<Mob> {
    let on = stage.scene().mobjects();
    mobs.iter().copied().filter(|m| on.contains(m)).collect()
}

/// Fade `out` away and `into` in while `moving` slides by `delta`, all in one
/// play of `rt` seconds.
fn glide(stage: &mut Stage<'_>, out: &[Mob], moving: &[Mob], into: &[Mob], delta: Vec3, rt: f64) -> fmn::Result<()> {
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
    for &m in moving {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(delta)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Chapter 04's last frame, all at once: what `Forces::final_picture` lists
/// (the walls are already on screen from `Forces::build`).
pub fn open_on_chapter04(stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    for m in forces.final_picture() {
        if !stage.scene().mobjects().contains(&m) {
            stage.add_to_scene(m)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- objects

/// What chapter 04 leaves that this chapter changes.
pub struct FromChapter04 {
    pub title: Mob,
    pub units: Vec<Mob>,
    /// The plate, its legend, the net push arrow and its label.
    pub pressure: Vec<Mob>,
    pub parcel: Mob,
    pub walls: Vec<Mob>,
    /// Du/Dt, =, the three terms (the pressure term first) and the two plus
    /// signs.
    pub law: Vec<Mob>,
}

/// The cell square and its faces (05_3 to 05_6).
pub struct CellPicture {
    pub square: Mob,
    pub centre_line: Mob,
    pub side: Mob,
    /// Left (in), right (out), top (in, pointing down).
    pub arrows: [Mob; 3],
    /// Left, right, top, bottom.
    pub labels: [Mob; 4],
    pub top_ring: Mob,
}

impl CellPicture {
    pub fn all(&self) -> Vec<Mob> {
        let mut all = vec![self.square, self.centre_line, self.side, self.top_ring];
        all.extend(self.arrows);
        all.extend(self.labels);
        all
    }
}

/// The ledger and its badge (05_4).
pub struct Ledger {
    pub names: [Mob; 2],
    pub rows: [Mob; 2],
    pub rule: Mob,
    pub net_name: Mob,
    /// Red, then green.
    pub frames: [Mob; 2],
    /// −3, then 0.
    pub values: [Mob; 2],
}

impl Ledger {
    pub fn all(&self) -> Vec<Mob> {
        let mut all = vec![self.rule, self.net_name];
        all.extend(self.names);
        all.extend(self.rows);
        all.extend(self.frames);
        all.extend(self.values);
        all
    }
}

/// The full tunnel and what 05_7 to 05_10 draw on it.
pub struct TunnelPicture {
    pub tank: Mob,
    pub cylinder: Mob,
    pub slices: [Mob; 2],
    pub shade: Mob,
    /// Top wall, bottom wall, cylinder.
    pub zeros: [Mob; 3],
    pub inlet: Vec<Mob>,
    pub inlet_label: Mob,
    pub in_out: Mob,
    /// Upper gap, cylinder, lower gap.
    pub braces: [Mob; 3],
    pub brace_labels: [Mob; 3],
    pub open_line: Mob,
    /// Upper gap, lower gap.
    pub gap: Vec<Mob>,
    pub gap_readout: Mob,
}

impl TunnelPicture {
    /// Everything drawn on the tunnel, not the tunnel itself.
    pub fn annotations(&self) -> Vec<Mob> {
        let mut all = vec![
            self.shade,
            self.inlet_label,
            self.in_out,
            self.open_line,
            self.gap_readout,
        ];
        all.extend(self.slices);
        all.extend(self.zeros);
        all.extend(&self.inlet);
        all.extend(self.braces);
        all.extend(self.brace_labels);
        all.extend(&self.gap);
        all
    }
}

/// Every object of the chapter, built detached at its final place.
pub struct Continuity {
    pub from04: FromChapter04,
    pub title: Mob,
    pub unknown: Mob,
    pub area: Mob,
    pub slow: Mob,
    pub area_ring: Mob,
    pub ghost: Mob,
    /// The share of the stretch, 0 to 1, and the updater that redraws the
    /// parcel from it.
    pub stretch: Mob,
    pub mover: Mob,
    pub precisely: [Vec<Mob>; 3],
    pub cell: CellPicture,
    pub ledger: Ledger,
    /// The words and the formula, then ∇·u = 0 alone.
    pub divergence: [Mob; 2],
    pub div_zero: Mob,
    pub compare: Vec<Mob>,
    pub tunnel: TunnelPicture,
    /// ∇·u = 0 where the "?" ends, and the white line to the pressure term.
    pub rule_zero: Mob,
    pub link: Mob,
}

impl Continuity {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, f: &Forces, shape: TunnelShape) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let dim = Palette::dim();
        let blue = Palette::blue();
        let mut law = f.lhs.to_vec();
        law.extend(f.terms);
        law.extend(f.plus);
        let from04 = FromChapter04 {
            title: f.title,
            units: f.units.clone(),
            pressure: vec![f.plate, f.legend[0], f.legend[1], f.push, f.push_labels[1]],
            parcel: f.parcel,
            walls: f.walls.clone(),
            law,
        };
        let title = kit.text_left(stage, TITLE, TITLE_PT, ink, v(TITLE_AT.0, TITLE_AT.1))?;
        stage.set_z_index(title, Z_TEXT, true);

        // 05_1: the "?" over the pressure slot.
        let slot_top = EQ_BASE + SLOT_LIFT + SLOT_H / 2.0;
        let term_x = stage.get_center(f.terms[0])[0];
        let unknown_at = v(term_x, slot_top + UNKNOWN_LIFT);
        let unknown = text_at(kit, stage, UNKNOWN, UNKNOWN_PT, ink, unknown_at, ORIGIN)?;

        // 05_2: the area readout, the faint start square and the stretch.
        let gold = Palette::gold();
        let area = tex_at(
            kit,
            stage,
            &area_label(),
            READOUT_PT,
            gold,
            v(AREA_AT.0, AREA_AT.1),
            ORIGIN,
        )?;
        let slow = text_at(kit, stage, &slow_note(), FINE_PT, dim, v(SLOW_AT.0, SLOW_AT.1), ORIGIN)?;
        let area_ring = outline(stage, area, RING_PAD)?;
        let side = PARCEL_SIDE * SCALE;
        let ghost = Rectangle::new()
            .width(side)
            .height(side)
            .style(line_style(Palette::faint(), 3.0, 1.0))
            .build()?
            .moved_to(to_scene(PARCEL_CENTRE.0, PARCEL_CENTRE.1));
        let ghost = add(stage, ghost, Z_MARKS);
        let stretch = stage.add_value_tracker(0.0);
        let parcel = f.parcel;
        let start = stage
            .get_points(parcel)
            .ok_or_else(|| fail("the parcel square has no points"))?;
        let mover = stage.arena_mut().add(VMobject::new());
        let mut last = 0.0;
        stage.add_updater(
            mover,
            move |s, _| {
                let share = s.tracker_value(stretch).unwrap_or(0.0);
                if share == last {
                    return;
                }
                let g = growth_at(share);
                let points: Vec<Vec3> = start.iter().map(|&p| carried_scene(p, g)).collect();
                let _ = s.set_points(parcel, &points);
                last = share;
            },
            true,
        )?;
        stage.add_to_scene(mover)?;

        let mut precisely: [Vec<Mob>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for (k, lines) in PRECISELY.iter().enumerate() {
            for (line, y) in lines.iter().zip(PRECISELY_Y) {
                precisely[k].push(text_at(kit, stage, line, LABEL_PT, dim, v(0.0, y), ORIGIN)?);
            }
        }

        // 05_3: the cell square, its faces and their values.
        let [cl, cr, cb, ct] = CELL;
        let (lo, hi) = (to_scene(cl, cb), to_scene(cr, ct));
        let square = Rectangle::new()
            .width(hi[0] - lo[0])
            .height(hi[1] - lo[1])
            .style(Style::default().fill(ink, CELL_FILL).stroke(ink, CELL_W, 1.0))
            .build()?
            .moved_to(v((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0));
        let square = add(stage, square, Z_MARKS);
        let centre_line = add(
            stage,
            polyline(vec![to_scene(0.0, 0.0), to_scene(LENGTH, 0.0)])
                .style(line_style(Palette::faint(), 2.0, 1.0))
                .build(),
            Z_FIELD,
        );
        let side_label = text_at(
            kit,
            stage,
            SIDE_LABEL,
            FINE_PT,
            dim,
            v(SIDE_LABEL_AT.0, SIDE_LABEL_AT.1),
            RIGHT,
        )?;
        let speeds = face_speeds();
        let words = face_labels();
        let (mx, my) = ((cl + cr) / 2.0, (cb + ct) / 2.0);
        let left_mid = to_scene(cl, my);
        let right_mid = to_scene(cr, my);
        let top_mid = to_scene(mx, ct);
        let bottom_mid = to_scene(mx, cb);
        let half = |s: f64| FACE_PER * s / 2.0;
        let arrows = [
            arrow(
                stage,
                v(left_mid[0] - half(speeds[0]), left_mid[1]),
                v(left_mid[0] + half(speeds[0]), left_mid[1]),
                blue,
                Z_TRAIL,
            )?,
            arrow(
                stage,
                v(right_mid[0] - half(speeds[1]), right_mid[1]),
                v(right_mid[0] + half(speeds[1]), right_mid[1]),
                blue,
                Z_TRAIL,
            )?,
            arrow(
                stage,
                v(top_mid[0], top_mid[1] + half(speeds[2])),
                v(top_mid[0], top_mid[1] - half(speeds[2])),
                blue,
                Z_TRAIL,
            )?,
        ];
        let labels = [
            tex_at(
                kit,
                stage,
                &words[0],
                LABEL_PT,
                blue,
                v(left_mid[0] - half(speeds[0]) - FACE_LABEL_GAP, left_mid[1]),
                RIGHT,
            )?,
            tex_at(
                kit,
                stage,
                &words[1],
                LABEL_PT,
                blue,
                v(right_mid[0] + half(speeds[1]) + FACE_LABEL_GAP, right_mid[1]),
                LEFT,
            )?,
            tex_at(
                kit,
                stage,
                &words[2],
                LABEL_PT,
                blue,
                v(top_mid[0] - TOP_LABEL_GAP, top_mid[1] + half(speeds[2])),
                RIGHT,
            )?,
            tex_at(
                kit,
                stage,
                &words[3],
                LABEL_PT,
                blue,
                v(bottom_mid[0], bottom_mid[1] - ZERO_BELOW),
                ORIGIN,
            )?,
        ];
        let top_ring = outline(stage, arrows[2], RING_PAD)?;
        let cell = CellPicture {
            square,
            centre_line,
            side: side_label,
            arrows,
            labels,
            top_ring,
        };

        // 05_4: the ledger and its badge.
        let mut names = Vec::with_capacity(2);
        let mut rows = Vec::with_capacity(2);
        for (k, (name, src)) in LEDGER_NAMES.iter().zip([in_row(), out_row()]).enumerate() {
            names.push(text_at(
                kit,
                stage,
                name,
                LABEL_PT,
                dim,
                v(LEDGER_LEFT, LEDGER_Y[k]),
                LEFT,
            )?);
            rows.push(tex_at(
                kit,
                stage,
                &src,
                READOUT_PT,
                blue,
                v(LEDGER_VALUE_X, LEDGER_Y[k]),
                LEFT,
            )?);
        }
        let rule = add(
            stage,
            polyline(vec![v(RULE_X.0, RULE_Y), v(RULE_X.1, RULE_Y)])
                .style(line_style(dim, 2.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let net_name = text_at(kit, stage, NET_NAME, LABEL_PT, ink, v(LEDGER_LEFT, NET_Y), LEFT)?;
        let (red, green) = (Palette::red(), Palette::green());
        let frames = [badge_frame(stage, red)?, badge_frame(stage, green)?];
        let [before, after] = badge_values();
        let badge_at = v(BADGE_AT.0, BADGE_AT.1);
        let values = [
            tex_at(kit, stage, &before, READOUT_PT, red, badge_at, ORIGIN)?,
            tex_at(kit, stage, &after, READOUT_PT, green, badge_at, ORIGIN)?,
        ];
        let ledger = Ledger {
            names: [names[0], names[1]],
            rows: [rows[0], rows[1]],
            rule,
            net_name,
            frames,
            values,
        };

        // 05_5: what divergence measures, then the rule alone.
        let divergence = [
            tex_at(
                kit,
                stage,
                DIVERGENCE_WORDS,
                LABEL_PT,
                ink,
                v(DIVERGENCE_AT[0].0, DIVERGENCE_AT[0].1),
                ORIGIN,
            )?,
            tex_at(
                kit,
                stage,
                &divergence_line(),
                DIVERGENCE_PT,
                ink,
                v(DIVERGENCE_AT[1].0, DIVERGENCE_AT[1].1),
                ORIGIN,
            )?,
        ];
        let div_zero = tex_at(kit, stage, RULE, EQ_PT, ink, v(RULE_AT.0, RULE_AT.1), ORIGIN)?;

        // 05_6: the left and right arrows side by side.
        let mut compare = Vec::with_capacity(6);
        // left and right: the first two face speeds and their words
        for ((&y, name), (&speed, word)) in COMPARE_Y.iter().zip(COMPARE_NAMES).zip(speeds.iter().zip(&words)) {
            let tip = COMPARE_TAIL_X + COMPARE_PER * speed;
            compare.push(arrow(stage, v(COMPARE_TAIL_X, y), v(tip, y), blue, Z_TEXT)?);
            compare.push(text_at(
                kit,
                stage,
                name,
                LABEL_PT,
                dim,
                v(COMPARE_TAIL_X - FACE_LABEL_GAP - 0.05, y),
                RIGHT,
            )?);
            compare.push(tex_at(
                kit,
                stage,
                word,
                LABEL_PT,
                blue,
                v(tip + FACE_LABEL_GAP + 0.05, y),
                LEFT,
            )?);
        }

        // 05_7 to 05_10: the full tunnel and what goes on it.
        let tunnel = build_tunnel(kit, stage, shape)?;

        // 05_11: ∇·u = 0 where the "?" ends, linked to the pressure term.
        let rule_at = v(unknown_at[0], unknown_at[1] + GLIDE_DY);
        let rule_zero = tex_at(kit, stage, RULE, EQ_PT, ink, rule_at, ORIGIN)?;
        let link_top = stage.get_bottom(rule_zero)[1] - LINK_GAP;
        let link_bottom = stage.get_top(f.terms[0])[1] + GLIDE_DY + LINK_GAP;
        if link_top - link_bottom < 0.1 {
            return Err(fail("no room for the white link"));
        }
        let link = add(
            stage,
            polyline(vec![v(term_x, link_top), v(term_x, link_bottom)])
                .style(line_style(white(), HIGHLIGHT_W, 1.0))
                .build(),
            Z_HIGHLIGHT,
        );

        Ok(Self {
            from04,
            title,
            unknown,
            area,
            slow,
            area_ring,
            ghost,
            stretch,
            mover,
            precisely,
            cell,
            ledger,
            divergence,
            div_zero,
            compare,
            tunnel,
            rule_zero,
            link,
        })
    }

    /// The law and its "?": what glides below the tunnel at 05_7.
    pub fn equation(&self) -> Vec<Mob> {
        let mut all = self.from04.law.clone();
        all.push(self.unknown);
        all
    }

    /// The stretch, [`stretch_time`] of flow in [`RUN_SECONDS`] of film, set
    /// frame by frame: the last frame is exactly the end.
    pub fn run_stretch(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = film_frames(stage, RUN_SECONDS);
        for k in 1..=n {
            stage.set_tracker_value(self.stretch, run_share(k, n))?;
            wait_frames(stage, 1)?;
        }
        stage.remove_from_scene(self.mover);
        Ok(())
    }

    /// What is on screen when the chapter ends, for chapter 06's opening:
    /// the title, the full tunnel, the law with ∇·u = 0 linked to its
    /// pressure term.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn final_picture(&self) -> Vec<Mob> {
        let mut all = vec![self.title, self.tunnel.tank, self.tunnel.cylinder];
        all.extend(&self.from04.law);
        all.extend([self.rule_zero, self.link]);
        all
    }
}

fn build_tunnel(kit: &Kit, stage: &mut Stage<'_>, t: TunnelShape) -> fmn::Result<TunnelPicture> {
    let ink = Palette::ink();
    let blue = Palette::blue();
    let k = t.scale();
    let [cx, cy, r] = t.cylinder;
    // As root's chapter02-tunnel.rs `set_view` draws them.
    let tank = Rectangle::new()
        .width(FULL_WIDTH)
        .height(t.height * k)
        .style(
            Style::default()
                .fill(hex("#17283C"), 1.0)
                .stroke(Palette::dim(), 1.7, 1.0),
        )
        .build()?
        .moved_to(v(FULL_AT.0, FULL_AT.1));
    let tank = stage.arena_mut().add(tank);
    let cylinder = Circle::new()
        .radius(r * k)
        .arc_center(t.at(cx, cy))
        .style(
            Style::default()
                .fill(hex("#0A121F"), 1.0)
                .stroke(Palette::ink(), 1.6, 1.0),
        )
        .build();
    let cylinder = add(stage, cylinder, Z_WALLS);

    let slice = |stage: &mut Stage<'_>, x: f64| {
        add(
            stage,
            polyline(vec![t.at(x, 0.0), t.at(x, t.height)])
                .style(line_style(white(), SLICE_W, 1.0))
                .build(),
            Z_HIGHLIGHT,
        )
    };
    let slices = [slice(stage, SLICE_A), slice(stage, cx)];

    // The region between the slices: up the cylinder's centre line, round
    // its upstream half, and back along the top.
    let mut ring = vec![t.at(SLICE_A, 0.0), t.at(cx, 0.0), t.at(cx, cy - r)];
    for step in 1..ARC_STEPS {
        let a = -FRAC_PI_2 - PI * f64::from(step) / f64::from(ARC_STEPS);
        ring.push(t.at(cx + r * a.cos(), cy + r * a.sin()));
    }
    ring.extend([t.at(cx, cy + r), t.at(cx, t.height), t.at(SLICE_A, t.height)]);
    let shade = Polygon::new(ring)
        .style(Style::default().fill(white(), SHADE_ALPHA).stroke(white(), 0.0, 0.0))
        .build();
    let shade = add(stage, shade, Z_FIELD);

    let (xa, xb) = (t.at(SLICE_A, 0.0)[0], t.at(cx, 0.0)[0]);
    let (bottom, top) = (t.at(0.0, 0.0)[1], t.at(0.0, t.height)[1]);
    let middle = (xa + xb) / 2.0;
    let cylinder_left = t.at(cx - r, cy);
    let zeros = [
        text_at(kit, stage, ZERO, LABEL_PT, ink, v(middle, top + ZERO_GAP), ORIGIN)?,
        text_at(kit, stage, ZERO, LABEL_PT, ink, v(middle, bottom - ZERO_GAP), ORIGIN)?,
        text_at(
            kit,
            stage,
            ZERO,
            LABEL_PT,
            ink,
            v(cylinder_left[0] - BRACE_GAP, cylinder_left[1]),
            RIGHT,
        )?,
    ];

    // Average arrows: equal at the inlet slice, 4/3 as long in each gap.
    let mut inlet = Vec::with_capacity(INLET_ROWS.len());
    for share in INLET_ROWS {
        let y = t.at(0.0, share * t.height)[1];
        inlet.push(arrow(stage, v(xa - INLET_L, y), v(xa, y), blue, Z_MARKS)?);
    }
    let (n, d) = gap_ratio();
    let gap_l = INLET_L * f64::from(n) / f64::from(d);
    let gap_mid = [(cy + r + t.height) / 2.0, (cy - r) / 2.0];
    let mut gap = Vec::with_capacity(2);
    for y in gap_mid {
        let y = t.at(0.0, y)[1];
        gap.push(arrow(stage, v(xb - gap_l, y), v(xb, y), blue, Z_MARKS)?);
    }
    let below = v(xa - INLET_L / 2.0, BELOW_Y);
    let inlet_label = text_at(kit, stage, INLET_LABEL, LABEL_PT, blue, below, ORIGIN)?;
    let in_out = text_at(kit, stage, IN_OUT, LABEL_PT, ink, v(IN_OUT_X, BELOW_Y), ORIGIN)?;
    let open_words = open_line();
    let open_line = text_at(kit, stage, &open_words, LABEL_PT, ink, v(OPEN_LINE_X, BELOW_Y), ORIGIN)?;

    // Braces right of the cylinder: the upper gap, the cylinder, the lower gap.
    let brace_x = t.at(cx + r, cy)[0] + BRACE_GAP;
    let edges = [
        (t.at(0.0, cy + r)[1], top),
        (t.at(0.0, cy - r)[1], t.at(0.0, cy + r)[1]),
        (bottom, t.at(0.0, cy - r)[1]),
    ];
    let mut braces = Vec::with_capacity(3);
    let mut brace_labels = Vec::with_capacity(3);
    let blocked = fraction(BLOCKED);
    for (i, &(y0, y1)) in edges.iter().enumerate() {
        let (b, tip) = brace(stage, brace_x, y0 + BRACE_INSET, y1 - BRACE_INSET);
        braces.push(b);
        let words = if i == 1 { blocked.as_str() } else { OPEN_WORD };
        brace_labels.push(text_at(
            kit,
            stage,
            words,
            LABEL_PT,
            ink,
            v(tip[0] + BRACE_LABEL_GAP, tip[1]),
            LEFT,
        )?);
    }
    let gap_readout = tex_at(
        kit,
        stage,
        &gap_readout(),
        LABEL_PT + 2.0,
        blue,
        v(GAP_READOUT_X, t.at(0.0, gap_mid[0])[1]),
        LEFT,
    )?;
    Ok(TunnelPicture {
        tank,
        cylinder,
        slices,
        shade,
        zeros,
        inlet,
        inlet_label,
        in_out,
        braces: [braces[0], braces[1], braces[2]],
        brace_labels: [brace_labels[0], brace_labels[1], brace_labels[2]],
        open_line,
        gap,
        gap_readout,
    })
}

// ---------------------------------------------------------------- the cues

/// 05_1 "We still haven't said what sets the pressure. …": the title changes,
/// chapter 04's units go, and a "?" stands over the pressure slot.
pub fn cue_05_1(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_1")?;
    let mut out = vec![c.from04.title];
    out.extend(&c.from04.units);
    swap(stage, &out, &[c.title], FADE)?;
    fade_in_all(stage, &[c.unknown], FADE)
}

/// 05_2 "Air can be squeezed. …": the pressure picture goes; with sentence 1
/// ("But we'll keep the air … incompressible.") the lower third, the area
/// readout and the stretch; with sentence 2 ("Every parcel keeps its
/// volume.") the readout is outlined.
pub fn cue_05_2(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_2")?;
    fade_out_all(stage, &c.from04.pressure, FADE)?;
    kit.until_sentence(stage, "05_2", 1)?;
    let mut words = c.precisely[0].clone();
    words.extend([c.area, c.slow]);
    fade_in_all(stage, &words, FADE)?;
    fade_in_all(stage, &[c.ghost], QUICK)?;
    c.run_stretch(stage)?;
    kit.until_sentence(stage, "05_2", 2)?;
    fade_in_all(stage, &[c.area_ring], FADE)
}

/// 05_3 "Here's what that means for one square of the channel, …": the cell
/// square on the centre line; each face's arrow and value with its sentence;
/// then the lower third.
pub fn cue_05_3(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_3")?;
    let mut out = vec![c.from04.parcel, c.ghost, c.area, c.slow, c.area_ring];
    out.extend(&c.precisely[0]);
    let cell = &c.cell;
    swap(stage, &out, &[cell.square, cell.centre_line, cell.side], FADE)?;
    kit.until_sentence(stage, "05_3", 1)?;
    fade_in_all(stage, &[cell.arrows[0], cell.labels[0]], FADE)?;
    kit.until_sentence(stage, "05_3", 2)?;
    fade_in_all(stage, &[cell.arrows[1], cell.labels[1]], FADE)?;
    kit.until_sentence(stage, "05_3", 3)?;
    fade_in_all(stage, &[cell.arrows[2], cell.labels[2], cell.labels[3]], FADE)?;
    fade_in_all(stage, &c.precisely[1], FADE)
}

/// 05_4 "In, two plus one. Out, three. …": the in row, then the out row
/// with sentence 1. Show the net zero only after both rows are visible.
pub fn cue_05_4(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_4")?;
    let l = &c.ledger;
    fade_in_all(stage, &[l.names[0], l.rows[0]], FADE)?;
    kit.until_sentence(stage, "05_4", 1)?;
    fade_in_all(stage, &[l.names[1], l.rows[1], l.rule], FADE)?;
    fade_in_all(stage, &[l.net_name, l.frames[1], l.values[1]], QUICK)
}

/// 05_5 "Net outflow from a small region, divided by its size, is called the
/// divergence. …": the words and ∇·u = ∂u/∂x + ∂v/∂y = 1 + (−1) = 0; with
/// sentence 1 ∇·u = 0 alone.
pub fn cue_05_5(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_5")?;
    swap(stage, &c.precisely[1], &c.divergence, FADE)?;
    kit.until_sentence(stage, "05_5", 1)?;
    swap(stage, &c.divergence, &[c.div_zero], FADE)
}

/// 05_6 "Notice what didn't happen. …": the ledger goes; with sentence 1 the
/// left and right arrows side by side; with sentence 2 the top arrow
/// outlined.
pub fn cue_05_6(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_6")?;
    let ledger = shown(stage, &c.ledger.all());
    fade_out_all(stage, &ledger, FADE)?;
    kit.until_sentence(stage, "05_6", 1)?;
    fade_in_all(stage, &c.compare, FADE)?;
    kit.until_sentence(stage, "05_6", 2)?;
    fade_in_all(stage, &[c.cell.top_ring], FADE)
}

/// 05_7 "Now the tunnel. …": root's thumbnail goes; then, in one play, the
/// channel goes, the law glides below and the full tunnel comes in its place;
/// with sentence 1 two white slices; with sentence 2
/// "0" on the walls and the cylinder; with sentence 3 the region between
/// shaded, the inlet's average arrows and "flow in = flow out".
pub fn cue_05_7(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity, handoff: &mut dyn TunnelHandoff) -> fmn::Result<()> {
    kit.say(stage, "05_7")?;
    let mut out = c.from04.walls.clone();
    out.extend(c.cell.all());
    out.extend(&c.compare);
    out.push(c.div_zero);
    let out = shown(stage, &out);
    handoff.leave_thumbnail(kit, stage)?;
    let t = &c.tunnel;
    glide(
        stage,
        &out,
        &c.equation(),
        &[t.tank, t.cylinder],
        v(0.0, GLIDE_DY),
        OPEN,
    )?;
    kit.until_sentence(stage, "05_7", 1)?;
    create_all(stage, &t.slices, FADE)?;
    kit.until_sentence(stage, "05_7", 2)?;
    fade_in_all(stage, &t.zeros, FADE)?;
    kit.until_sentence(stage, "05_7", 3)?;
    let mut flow = vec![t.shade, t.inlet_label, t.in_out];
    flow.extend(&t.inlet);
    fade_in_all(stage, &flow, FADE)
}

/// 05_8 "Suppose the cylinder blocks a quarter of the tunnel's height.":
/// braces for the gaps and the cylinder, then "open: 3/4 of the height".
pub fn cue_05_8(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_8")?;
    let t = &c.tunnel;
    let mut braces = t.braces.to_vec();
    braces.extend(t.brace_labels);
    fade_in_all(stage, &braces, FADE)?;
    fade_in_all(stage, &[t.open_line], FADE)
}

/// 05_9 "Compared with the inlet, how fast must the air move, …": the card.
pub fn cue_05_9(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<PredictCard> {
    let card = PredictCard::build(kit, stage, &CARD05)?;
    kit.say(stage, "05_9")?;
    card.show(stage)?;
    Ok(card)
}

/// HOLD 05_h1: the question finishes, then 5 s of countdown. The gap arrows,
/// the gap readout and the 05_10 lower third stay hidden.
pub fn hold_05_h1(kit: &Kit, stage: &mut Stage<'_>, card: &PredictCard) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    card.count(stage)
}

/// 05_10 "Four thirds as fast. …": the card gives way to the gap arrows and
/// "4/3 × inlet average"; with sentence 1 the lower third.
pub fn cue_05_10(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity, card: &PredictCard) -> fmn::Result<()> {
    kit.say(stage, "05_10")?;
    let mut out = vec![card.back];
    out.extend(&card.question);
    let t = &c.tunnel;
    let mut reveal = t.gap.clone();
    reveal.push(t.gap_readout);
    swap(stage, &out, &reveal, FADE)?;
    kit.until_sentence(stage, "05_10", 1)?;
    fade_in_all(stage, &c.precisely[2], FADE)
}

/// 05_11 "That's a rule the program must keep in every cell, at every step.
/// …": the lower third goes; with sentence 1 ("Keeping it is pressure's
/// job.") the "?" becomes ∇·u = 0 and a white line links it to the pressure
/// term, and the tunnel's marks go, leaving the chapter 06 opening.
pub fn cue_05_11(kit: &Kit, stage: &mut Stage<'_>, c: &Continuity) -> fmn::Result<()> {
    kit.say(stage, "05_11")?;
    fade_out_all(stage, &c.precisely[2], FADE)?;
    kit.until_sentence(stage, "05_11", 1)?;
    swap(stage, &[c.unknown], &[c.rule_zero], FADE)?;
    create_all(stage, &[c.link], QUICK)?;
    let marks = shown(stage, &c.tunnel.annotations());
    fade_out_all(stage, &marks, FADE)?;
    kit.hold(stage)
}

/// The chapter from chapter 04's last frame, with the retained tunnel geometry.
pub fn chapter05(
    kit: &Kit,
    stage: &mut Stage<'_>,
    tunnel: TunnelShape,
    handoff: &mut dyn TunnelHandoff,
) -> fmn::Result<Continuity> {
    let forces = Forces::build(kit, stage)?;
    open_on_chapter04(stage, &forces)?;
    let c = Continuity::build(kit, stage, &forces, tunnel)?;
    cue_05_1(kit, stage, &c)?;
    cue_05_2(kit, stage, &c)?;
    cue_05_3(kit, stage, &c)?;
    cue_05_4(kit, stage, &c)?;
    cue_05_5(kit, stage, &c)?;
    cue_05_6(kit, stage, &c)?;
    cue_05_7(kit, stage, &c, handoff)?;
    cue_05_8(kit, stage, &c)?;
    let card = cue_05_9(kit, stage)?;
    hold_05_h1(kit, stage, &card)?;
    cue_05_10(kit, stage, &c, &card)?;
    cue_05_11(kit, stage, &c)?;
    Ok(c)
}

/// Back to the side panel for chapter 06's opening ("the side-panel slots
/// slide into one line"): the full tunnel goes while the law, ∇·u = 0 and
/// the white link glide back up to the law's chapter 04 place, then root's
/// thumbnail returns. Not one of chapter 05's cues (its last picture keeps
/// the full tunnel); root calls it where chapter 06 starts.
#[cfg_attr(not(test), allow(dead_code))]
pub fn back_to_panel(
    kit: &Kit,
    stage: &mut Stage<'_>,
    c: &Continuity,
    handoff: &mut dyn TunnelHandoff,
) -> fmn::Result<()> {
    let mut moving = c.from04.law.clone();
    moving.extend([c.rule_zero, c.link]);
    glide(
        stage,
        &[c.tunnel.tank, c.tunnel.cylinder],
        &moving,
        &[],
        v(0.0, -GLIDE_DY),
        OPEN,
    )?;
    handoff.return_thumbnail(kit, stage)
}

/// The spoken lines, verbatim from `navier-full-script03.md`
/// (`check_continuity01.py` compares them byte for byte), as `script.tsv`
/// rows. 05_h1 is a hold, not a line.
#[cfg(test)]
pub const SCRIPT05: &str = concat!(
    "05_1\tWe still haven't said what sets the pressure. The answer is a rule about volume.\n",
    "05_2\tAir can be squeezed. But we'll keep the air in our tunnel far slower than sound, where its density barely changes, and treat it as incompressible. Every parcel keeps its volume.\n",
    "05_3\tHere's what that means for one square of the channel, one meter on a side. Air comes in through its left side at two meters per second. It leaves through the right at three. And through the top, it comes in at one.\n",
    "05_4\tIn, two plus one. Out, three. Nothing piles up, and nothing drains.\n",
    "05_5\tNet outflow from a small region, divided by its size, is called the divergence. Incompressible means the divergence is zero, everywhere, at every moment.\n",
    "05_6\tNotice what didn't happen. The speeds aren't all equal. Air leaves this square faster than it entered from the left, because more arrives through the top. Constant density doesn't mean constant speed.\n",
    "05_7\tNow the tunnel. Draw two slices across it, from wall to wall. The walls and the cylinder let nothing through. So whatever flows in through one slice must flow out through the other, at the same moment.\n",
    "05_8\tSuppose the cylinder blocks a quarter of the tunnel's height.\n",
    "05_9\tCompared with the inlet, how fast must the air move, on average, through the open part of that slice?\n",
    "05_10\tFour thirds as fast. The same flow has to fit through three quarters of the height.\n",
    "05_11\tThat's a rule the program must keep in every cell, at every step. Keeping it is pressure's job.\n",
);

#[cfg(test)]
mod tests {
    use super::super::chapter03::wall;
    use super::*;
    use explainer_kit::{Narrator, Script};

    /// facts01-run02's values for this chapter's keys.
    const FACTS: [(&str, &str); 9] = [
        ("cell.in.left", "2"),
        ("cell.out.right", "3"),
        ("cell.in.top", "1"),
        ("cell.bottom", "0"),
        ("cell.in.total", "3"),
        ("cell.net", "0"),
        ("ch.divergence", "0"),
        ("tunnel.blocked", "1/4"),
        ("tunnel.gap.ratio", "4/3"),
    ];

    fn computed(key: &str) -> String {
        let q = face_outflows();
        match key {
            "cell.in.left" => format!("{}", -q[0]),
            "cell.out.right" => format!("{}", q[1]),
            "cell.in.top" => format!("{}", -q[3]),
            "cell.bottom" => format!("{}", q[2]),
            "cell.in.total" => format!("{}", inflow()),
            "cell.net" => format!("{}", net_outflow()),
            "ch.divergence" => {
                let (a, c) = rates(0.5, 0.5);
                format!("{}", (a + c).round() + 0.0)
            }
            "tunnel.blocked" => fraction(BLOCKED),
            "tunnel.gap.ratio" => fraction(gap_ratio()),
            _ => unreachable!("{key}"),
        }
    }

    #[test]
    fn the_numbers_are_the_facts() {
        for (key, value) in FACTS {
            assert_eq!(computed(key), value, "{key}");
        }
        // root's chapter 07 counts the same faces, positive out
        assert_eq!(face_outflows(), [-2.0, 3.0, 0.0, -1.0]);
        assert_eq!((inflow(), outflow(), net_outflow()), (3.0, 3.0, 0.0));
    }

    /// ch.parcel.area: the carried square keeps 0.25 m² at every share,
    /// stretches along and narrows across, stays between the walls, and the
    /// closed form is the field's own motion.
    #[test]
    fn the_parcel_keeps_its_area() {
        for k in 0..=240 {
            let s = f64::from(k) / 240.0;
            let area = parcel_area(s);
            assert!((area - 0.25).abs() < 1e-12, "area {area} at share {s}");
            let [l, r, b, t] = parcel_box(s);
            assert!(r - l >= PARCEL_SIDE - 1e-12 && t - b <= PARCEL_SIDE + 1e-12);
            if l >= 0.0 {
                assert!(t < wall(r) && -b < wall(r), "inside the walls at share {s}");
            }
        }
        let [l, r, b, t] = parcel_box(1.0);
        assert!((r - LENGTH).abs() < 1e-12, "the leading edge ends at the exit");
        assert!(((r - l) - 8.0 / 9.0).abs() < 1e-12 && ((t - b) - 9.0 / 32.0).abs() < 1e-12);
        assert!((stretch_end() - 16.0 / 9.0).abs() < 1e-15);
        assert_eq!(format!("{:.2}", stretch_time()), "0.58");
        let h = 1e-6;
        for &(x, y) in &[(-0.25, 0.25), (0.25, -0.25), (0.0, 0.1)] {
            let at = |time: f64| carried(x, y, time.exp());
            let (ahead, behind, now) = (at(0.3 + h), at(0.3 - h), at(0.3));
            let rate = ((ahead.0 - behind.0) / (2.0 * h), (ahead.1 - behind.1) / (2.0 * h));
            let (u, w) = velocity(now.0, now.1);
            assert!((rate.0 - u).abs() < 1e-6 && (rate.1 - w).abs() < 1e-6, "dx/dt = u");
        }
        assert_eq!(area_label(), r"\text{area }0.25\,\mathrm{m^{2}}");
        assert_eq!(slow_note(), "slow motion: 0.58 s of flow in 4 s of film");
    }

    #[test]
    fn the_cell_words_are_its_flows() {
        assert_eq!(face_speeds(), [2.0, 3.0, 1.0, 0.0]);
        assert_eq!(
            face_labels(),
            [r"2\,\mathrm{m/s}", r"3\,\mathrm{m/s}", r"1\,\mathrm{m/s}", "0"].map(String::from)
        );
        assert_eq!(in_row(), r"2+1=3\,\mathrm{m^{3}/s}");
        assert_eq!(out_row(), r"3\,\mathrm{m^{3}/s}");
        assert_eq!(badge_values(), ["-3", "0"].map(String::from));
    }

    /// ch.divergence: ∂u/∂x = 1 and ∂v/∂y = −1 at every point inside the
    /// walls.
    #[test]
    fn the_divergence_is_zero_everywhere() {
        for i in 0..=8 {
            let x = LENGTH * f64::from(i) / 8.0;
            for j in -8..=8 {
                let y = wall(x) * f64::from(j) / 9.0;
                let (a, c) = rates(x, y);
                assert!((a - 1.0).abs() < 1e-9 && (c + 1.0).abs() < 1e-9, "({x}, {y})");
            }
        }
        assert_eq!(
            divergence_line(),
            r"\nabla\cdot u=\partial u/\partial x+\partial v/\partial y=1+(-1)=0"
        );
    }

    /// tunnel.blocked and tunnel.gap.ratio; the gap arrows are 4/3 of the
    /// inlet's; the tunnel is root's, whose cylinder blocks 0.1/0.41 as drawn.
    #[test]
    fn the_gap_takes_four_thirds() {
        assert_eq!((open_share(), gap_ratio()), ((3, 4), (4, 3)));
        let ((a, b), (c, d)) = (open_share(), gap_ratio());
        assert_eq!(a * c, b * d, "the same flow: 3/4 of the height at 4/3 the speed");
        assert_eq!(gap_readout(), r"4/3\times\text{inlet average}");
        assert_eq!(open_line(), "open: 3/4 of the height");
        let shape = TunnelShape::BENCHMARK;
        assert_eq!(
            (shape.length, shape.height, shape.cylinder),
            (2.2, 0.41, [0.2, 0.2, 0.05])
        );
        assert_eq!((FULL_AT, FULL_WIDTH), ((0.0, 1.15), 11.5));
        let drawn = shape.drawn_blockage();
        assert!(
            (drawn - 0.1 / 0.41).abs() < 1e-12 && (drawn - 0.25).abs() < 0.01,
            "drawn {drawn}"
        );
    }

    #[test]
    fn the_words_are_the_scripts() {
        let joined: Vec<String> = PRECISELY.iter().map(|pair| pair.join(" ")).collect();
        assert_eq!(
            joined,
            [
                "This is our modelling choice for a low-speed tunnel. Sound waves, sonic booms and blasts need \
                 compressible flow.",
                "Each face is one meter long, so per meter of depth these are also flows in cubic meters per second.",
                "Average means the flow through the slice divided by its open height, per meter of depth. Near the \
                 walls the air is slower, and elsewhere faster.",
            ]
        );
        assert_eq!(
            CARD05.question.join(" "),
            "How fast, on average, through the open part of the slice?"
        );
        assert!(CARD05.choices.is_empty(), "a free answer");
    }

    #[test]
    fn every_sentence_gate_exists() {
        let script = Script::parse(SCRIPT05).unwrap();
        assert_eq!(script.lines().len(), 11);
        let narrator = Narrator::estimate(&script);
        for (id, j) in [
            ("05_2", 1),
            ("05_2", 2),
            ("05_3", 1),
            ("05_3", 2),
            ("05_3", 3),
            ("05_4", 1),
            ("05_5", 1),
            ("05_6", 1),
            ("05_6", 2),
            ("05_7", 1),
            ("05_7", 2),
            ("05_7", 3),
            ("05_10", 1),
            ("05_11", 1),
        ] {
            assert!(narrator.sentence_starts(id).len() > j, "{id} sentence {j}");
        }
    }

    /// Runs `run` in a scene on the scratch voice at 10 fps.
    struct Probe<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>> {
        kit: Kit,
        run: F,
    }

    impl<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.run)(&self.kit, stage)
        }
    }

    fn probe(run: impl FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>) {
        let script = Script::parse(SCRIPT05).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe { kit, run }, config, 7, &mut fmn::scene::NullSceneSink).expect("the probe runs");
    }

    fn opening(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Forces, Continuity)> {
        let forces = Forces::build(kit, stage)?;
        open_on_chapter04(stage, &forces)?;
        let c = Continuity::build(kit, stage, &forces, TunnelShape::BENCHMARK)?;
        Ok((forces, c))
    }

    type Window = (f64, f64, f64, f64);
    const MAIN: Window = (-7.1, 0.2, -3.15, 3.9);
    const PANEL: Window = (0.4, 6.95, -3.15, 1.75);
    const LOWER: Window = (-7.1, 7.1, -3.95, -3.15);
    const FRAME: Window = (-7.1, 7.1, -3.15, 3.9);
    /// Root's open question card under the full tunnel (chapter 09).
    const ROOT_CARD: Window = (-4.1, 4.1, -1.65, -0.85);

    fn within(b: Window, w: Window) -> bool {
        w.0 <= b.0 && b.1 <= w.1 && w.2 <= b.2 && b.3 <= w.3
    }

    fn overlaps(a: Window, b: Window) -> bool {
        a.0 < b.1 && b.0 < a.1 && a.2 < b.3 && b.2 < a.3
    }

    /// Each object sits in its window; the lower thirds in the caption slot;
    /// while the tunnel is full, nothing (the law after its glide included)
    /// enters root's question-card place.
    #[test]
    fn everything_sits_in_its_window() {
        probe(|kit, stage| {
            let (_, c) = opening(kit, stage)?;
            let mut main = c.cell.all();
            main.push(c.ghost);
            for m in main {
                assert!(within(bounds(stage, m), MAIN), "main {m:?}: {:?}", bounds(stage, m));
            }
            let mut panel = vec![c.unknown, c.area, c.slow, c.area_ring, c.div_zero];
            panel.extend(c.ledger.all());
            panel.extend(c.divergence);
            panel.extend(&c.compare);
            for m in panel {
                assert!(within(bounds(stage, m), PANEL), "panel {m:?}: {:?}", bounds(stage, m));
            }
            for &m in c.precisely.iter().flatten() {
                assert!(
                    within(bounds(stage, m), LOWER),
                    "lower third {m:?}: {:?}",
                    bounds(stage, m)
                );
            }
            let card = PredictCard::build(kit, stage, &CARD05)?;
            let mut full = c.tunnel.annotations();
            full.extend([
                c.tunnel.tank,
                c.tunnel.cylinder,
                c.rule_zero,
                c.link,
                card.back,
                card.note,
                card.track,
            ]);
            full.extend(&card.question);
            for m in full {
                let b = bounds(stage, m);
                assert!(within(b, FRAME) && !overlaps(b, ROOT_CARD), "tunnel view {m:?}: {b:?}");
            }
            for m in c.equation() {
                let (l, r, b, t) = bounds(stage, m);
                let moved = (l, r, b + GLIDE_DY, t + GLIDE_DY);
                assert!(
                    within(moved, FRAME) && !overlaps(moved, ROOT_CARD),
                    "law {m:?}: {moved:?}"
                );
            }
            let (_, _, _, card_top) = bounds(stage, card.back);
            for m in c.equation() {
                let (l, _, b, _) = bounds(stage, m);
                assert!(
                    l > CARD05.at.0 + CARD05.size.0 / 2.0 || b + GLIDE_DY > card_top,
                    "law clears the card"
                );
            }
            // the words on and under the tunnel keep apart
            let t = &c.tunnel;
            let mut apart = vec![t.gap_readout, t.inlet_label, t.in_out, t.open_line];
            apart.extend(t.zeros);
            apart.extend(t.brace_labels);
            for (i, &a) in apart.iter().enumerate() {
                for &z in &apart[i + 1..] {
                    let (p, q) = (bounds(stage, a), bounds(stage, z));
                    let dx = (p.0 - q.1).max(q.0 - p.1).max(0.0);
                    let dy = (p.2 - q.3).max(q.2 - p.3).max(0.0);
                    assert!(dx.hypot(dy) > 0.1, "{a:?} {p:?} and {z:?} {q:?} too close");
                }
            }
            Ok(())
        });
    }

    /// The chapter opens on chapter 04's last frame and nothing else; the
    /// plate is at full opacity, as chapter 04's fade leaves it.
    #[test]
    fn the_chapter_opens_on_chapter_04s_last_frame() {
        probe(|kit, stage| {
            let forces = Forces::build(kit, stage)?;
            open_on_chapter04(stage, &forces)?;
            let on = stage.scene().mobjects().to_vec();
            let end = forces.final_picture();
            assert!(end.iter().all(|m| on.contains(m)), "chapter 04's last frame");
            assert!(on.iter().all(|m| end.contains(m)), "and nothing else");
            for k in stage.family(forces.plate) {
                if let Some(e) = stage.get(k)
                    && e.image_resource().is_some()
                {
                    let opacity = e.buffer.read_column("opacity").unwrap_or_default();
                    assert!(opacity.iter().all(|&a| a == 1.0), "plate opacity {opacity:?}");
                }
            }
            Ok(())
        });
    }

    /// After 05_2 the parcel's outline is the carried square's end box.
    #[test]
    fn the_drawn_parcel_is_the_carried_square() {
        probe(|kit, stage| {
            let (forces, c) = opening(kit, stage)?;
            cue_05_1(kit, stage, &c)?;
            cue_05_2(kit, stage, &c)?;
            let [l, r, b, t] = parcel_box(1.0);
            let (lo, hi) = (to_scene(l, b), to_scene(r, t));
            let points = stage.get_points(forces.parcel).expect("points");
            let xs = points.iter().map(|p| p[0]);
            let ys = points.iter().map(|p| p[1]);
            let (x0, x1) = (xs.clone().fold(f64::MAX, f64::min), xs.fold(f64::MIN, f64::max));
            let (y0, y1) = (ys.clone().fold(f64::MAX, f64::min), ys.fold(f64::MIN, f64::max));
            for (got, want) in [(x0, lo[0]), (x1, hi[0]), (y0, lo[1]), (y1, hi[1])] {
                assert!((got - want).abs() < 1e-4, "outline {got} against {want}");
            }
            Ok(())
        });
    }

    /// During the 05_h1 countdown the gap arrows, the gap readout and the
    /// 05_10 lower third are hidden; the card and the inlet arrows show.
    #[test]
    fn the_hold_hides_the_answer() {
        probe(|kit, stage| {
            let (_, c) = opening(kit, stage)?;
            cue_05_1(kit, stage, &c)?;
            cue_05_2(kit, stage, &c)?;
            cue_05_3(kit, stage, &c)?;
            cue_05_4(kit, stage, &c)?;
            cue_05_5(kit, stage, &c)?;
            cue_05_6(kit, stage, &c)?;
            cue_05_7(kit, stage, &c, &mut NoThumbnail)?;
            cue_05_8(kit, stage, &c)?;
            let card = cue_05_9(kit, stage)?;
            kit.hold(stage)?;
            let mut hidden = c.tunnel.gap.clone();
            hidden.push(c.tunnel.gap_readout);
            hidden.extend(&c.precisely[2]);
            assert!(shown(stage, &hidden).is_empty(), "the answer shows before 05_10");
            assert_eq!(shown(stage, &[card.back]).len(), 1);
            assert_eq!(shown(stage, &c.tunnel.inlet).len(), INLET_ROWS.len());
            Ok(())
        });
    }

    /// The whole chapter runs headless and ends on the chapter 06 opening.
    #[test]
    fn the_chapter_runs_on_the_scratch_voice() {
        probe(|kit, stage| {
            let c = chapter05(kit, stage, TunnelShape::BENCHMARK, &mut NoThumbnail)?;
            let end = c.final_picture();
            assert_eq!(shown(stage, &end).len(), end.len(), "the final picture is on screen");
            let mut gone = c.tunnel.annotations();
            gone.push(c.unknown);
            assert!(shown(stage, &gone).is_empty(), "the tunnel's marks are gone");
            let law_y = stage.get_center(c.from04.law[0])[1];
            back_to_panel(kit, stage, &c, &mut NoThumbnail)?;
            assert!(shown(stage, &[c.tunnel.tank, c.tunnel.cylinder]).is_empty());
            let back = stage.get_center(c.from04.law[0])[1];
            assert!((back - law_y + GLIDE_DY).abs() < 1e-4, "the law is back in the panel");
            for m in [c.rule_zero, c.link] {
                assert!(
                    within(bounds(stage, m), PANEL),
                    "{m:?} in the panel: {:?}",
                    bounds(stage, m)
                );
            }
            Ok(())
        });
    }
}
