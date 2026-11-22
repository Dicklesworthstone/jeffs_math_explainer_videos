//! Chapter 04 "What pushes a parcel": cues 04_1 to 04_14 and HOLD 04_h1, from
//! `navier-full-script03.md` (LavenderPike lavender-forces01, Mail 36229).
//!
//! The chapter opens on chapter 03's last picture, the channel's walls alone
//! (`follow_parcel02.rs`, whose coordinates, helpers and card it borrows), and
//! builds the parcel's law one force at a time in three slots of the side
//! panel:
//! - pressure (04_1 to 04_5): a gold parcel square at the entrance, equal
//!   orange arrows on its four faces, then a longer one behind and a net
//!   arrow forward; −∇p, then −∇p/ρ, in the first slot; the channel's exact
//!   pressure as one raster plate, orange at the entrance shading to teal at
//!   the exit; ρ = 1.3 kg/m³ and 1.3 kg/m³ × 2 m/s² = 2.6 Pa per m.
//! - viscosity (04_6 to 04_11): three layers at 1, 2 and 3 m/s with violet
//!   drag arrows along their two boundaries; the 04_h1 prediction card; the
//!   middle layer's two signed pulls, +1 and −1, net 0; then 1, 3, 3, pulls 0
//!   and −2, net −2 against the neighbours' average 2; ν∇²u in the second
//!   slot; back to the channel, whose straight-line velocity leaves "net 0 for
//!   this flow".
//! - body force (04_12): a slate arrow through the parcel, f in the third
//!   slot, then "f = 0 (gravity folded into p)".
//!   Then Du/Dt = −∇p/ρ + ν∇²u + f (04_13) with N/kg = m/s² under each term
//!   (04_14), left on screen for chapter 05.
//!
//! The pressure fill evaluates p/ρ = C − ((2 + x)² + y²)/2 at every texel's
//! centre and colours it after (no solver, no numeric scale), through the
//! adopted raster route (fluid-native-route01 §2 and §7): one `ImageMobject`
//! plate inside the walls, faded by `plate_fade` beside the engine's fades.
//! Its pixels need the qualified engine named in the opening script's native
//! notes; this module was written against the crate's a399d626 pin.
//!
//! Every number on screen is computed here from the module's own formulas,
//! which the tests pin against facts01-run02 (`ch.pressure.push.in`,
//! `ch.pressure.gradient.air`, `ch.laplacian`, `layers.123.net`,
//! `layers.133.net`, `layers.133.avg`, `units.N.per.kg`).

use explainer_kit::{Kit, Palette, children, hex};
use fmn::core::color::interpolate_color;
use fmn::mobject::{ImageColorSpace, ImageResource, ImageSampler, ImageWrap};
use fmn::prelude::*;

use super::chapter03::{
    CardSpec, FADE, FINE_PT, HIGHLIGHT_W, LABEL_PT, LENGTH, OPEN, ORIGIN_AT, PredictCard, QUICK, SCALE, TITLE_AT,
    TITLE_PT, Z_FIELD, Z_HIGHLIGHT, Z_MARKS, Z_PARCEL, Z_TEXT, Z_TRAIL, add, build_walls, create_all, fade_in_all,
    fade_out_all, fail, line_style, parcel_accel, swap, to_scene, v, wall, white,
};

// ------------------------------------------------------------- the physics

/// How far p/ρ sits below its constant at channel point (x, y), J/kg:
/// p/ρ = C − q with q = ((2 + x)² + y²)/2. Only differences matter, so the
/// picture needs no C.
pub fn pressure_drop(x: f64, y: f64) -> f64 {
    ((2.0 + x).powi(2) + y * y) / 2.0
}

/// The pressure push on each kilogram, −∇(p/ρ) = ∇q, N/kg: for this steady
/// flow it equals (u·∇)u, the parcel's whole acceleration.
pub fn push(x: f64, y: f64) -> (f64, f64) {
    (2.0 + x, y)
}

/// Air's density for this example, kg/m³ (our choice, stated on screen).
pub const RHO: f64 = 1.3;

/// The pressure drop each metre ahead of the parcel at the entrance, Pa per
/// m: ρ times the push there.
pub fn gradient_air() -> f64 {
    RHO * push(0.0, 0.0).0
}

/// Three layers' speeds, bottom to top, m/s: even (04_7) and bent (04_9).
pub const SPEEDS_EVEN: [f64; 3] = [1.0, 2.0, 3.0];
pub const SPEEDS_BENT: [f64; 3] = [1.0, 3.0, 3.0];

/// The middle layer's two signed pulls, from the layer above and from the
/// layer below, in steps of ν/h² × 1 m/s (h the spacing).
pub fn pulls(speeds: [f64; 3]) -> [f64; 2] {
    [speeds[2] - speeds[1], speeds[0] - speeds[1]]
}

/// The middle layer's net pull: the second difference above − 2·mid + below.
pub fn net_pull(speeds: [f64; 3]) -> f64 {
    let [above, below] = pulls(speeds);
    above + below
}

/// The middle layer's neighbours' average, m/s.
pub fn neighbours_average(speeds: [f64; 3]) -> f64 {
    (speeds[0] + speeds[2]) / 2.0
}

// --------------------------------------------------------------- the words

/// Rust's `{}` for a pull: "+1", "-1", "0", "-2" (Tex draws the minus).
pub fn signed(x: f64) -> String {
    if x > 0.0 { format!("+{x}") } else { format!("{x}") }
}

/// "2\,\mathrm{m/s^{2}}": the parcel's acceleration at the entrance (04_4).
pub fn accel_label() -> String {
    format!(r"{}\,\mathrm{{m/s^{{2}}}}", parcel_accel(0.0))
}

/// "2\,\mathrm{N\ per\ kg}": the pressure push at the entrance (04_4).
pub fn push_label() -> String {
    format!(r"{}\,\mathrm{{N\ per\ kg}}", push(0.0, 0.0).0)
}

/// 04_5's two lines: the density, then the pressure drop it needs.
pub fn density_line() -> String {
    format!(r"\rho = {RHO}\,\mathrm{{kg/m^{{3}}}}")
}

pub fn drop_line() -> String {
    format!(
        r"{RHO}\,\mathrm{{kg/m^{{3}}}}\times {}\,\mathrm{{m/s^{{2}}}} = {}\,\mathrm{{Pa\ per\ m}}",
        push(0.0, 0.0).0,
        gradient_air()
    )
}

/// "1 m/s", "2 m/s", "3 m/s": a layer's speed.
pub fn layer_label(speed: f64) -> String {
    format!("{speed} m/s")
}

/// "\mathrm{net}\;{0}", "\mathrm{net}\;{-2}": the middle layer's net pull
/// (braced, so the minus is a sign, not a subtraction from "net").
pub fn net_label(speeds: [f64; 3]) -> String {
    format!(r"\mathrm{{net}}\;{{{}}}", net_pull(speeds))
}

/// "neighbors' average: (1 + 3)/2 = 2 m/s" (04_9).
pub fn average_line(speeds: [f64; 3]) -> String {
    format!(
        "neighbors' average: ({} + {})/2 = {} m/s",
        speeds[0],
        speeds[2],
        neighbours_average(speeds)
    )
}

pub const TITLE: &str = "What pushes a parcel";
pub const DENSITY_NOTE: &str = "(our choice for this example)";
pub const FINE: &str = r"\text{pulls in steps of }\nu\,(1\,\mathrm{m/s})/h^{2}\text{, with }h\text{ the layer spacing}";
pub const VISCOUS_NOTE: &str = "net 0 for this flow";
pub const BODY_NOTE: &str = "f = 0 (gravity folded into p)";
pub const LEGEND: [&str; 2] = ["high pressure", "low pressure"];

/// The equation's pieces: Du/Dt, =, the three terms, two pluses; the first
/// slot's −∇p before ρ divides it; the units under the terms (04_14).
pub const LHS: [&str; 2] = [r"Du/Dt", r"="];
pub const TERMS: [&str; 3] = [r"-\nabla p/\rho", r"\nu\nabla^{2}u", r"f"];
pub const GRADIENT: &str = r"-\nabla p";
pub const PLUS: &str = r"+";
pub const UNIT_ROWS: [&str; 2] = [r"\mathrm{N/kg}", r"=\mathrm{m/s^{2}}"];
pub const LHS_UNIT: &str = r"\mathrm{m/s^{2}}";

// ---------------------------------------------------------------- layout

/// The parcel: a square of side `PARCEL_SIDE` metres centred on the
/// entrance's centre line, so its faces' average push is exactly the
/// entrance value (the push is linear in x). Chapter 05 may reuse both.
pub const PARCEL_CENTRE: (f64, f64) = (0.0, 0.0);
pub const PARCEL_SIDE: f64 = 0.5;
/// Face arrows point at the faces from `FACE_GAP` outside them; the back one
/// grows to `BACK_L`. The net arrow sits inside the square.
pub const FACE_L: f64 = 0.35;
pub const FACE_GAP: f64 = 0.05;
pub const BACK_L: f64 = 0.8;
pub const NET_PUSH_L: f64 = 0.45;
pub const PUSH_THICK: f64 = 4.0;
pub const NET_THICK: f64 = 5.0;
/// "net 0" under the square (04_2); the push label right-aligned left of the
/// square, clear of the plate (04_4).
pub const BALANCE_DY: f64 = -0.95;
pub const PUSH_LABEL_GAP: f64 = 0.3;
/// The legend: "high pressure" under the entrance, "low pressure" under the
/// exit, each `LEGEND_GAP` below its end of the lower wall.
pub const LEGEND_GAP: f64 = 0.4;

/// The plate: texels per scene unit (1.25 a pixel of the 4K frame's 240),
/// and its opacity inside the walls (root's chapter 09 fills its pressure
/// cells at 0.36).
pub const PLATE_TEXELS: f64 = 300.0;
pub const PLATE_ALPHA: f64 = 0.36;
pub const PLATE_IN: f64 = 1.0;

/// The side panel's equation: its baseline, piece centres and slots.
pub const EQ_PT: f64 = 40.0;
pub const EQ_BASE: f64 = 0.55;
pub const LHS_X: [f64; 2] = [1.05, 1.95];
pub const TERM_X: [f64; 3] = [2.98, 4.81, 6.39];
pub const PLUS_X: [f64; 2] = [3.97, 5.65];
pub const SLOT_W: [f64; 3] = [1.45, 1.15, 0.95];
pub const SLOT_H: f64 = 0.8;
pub const SLOT_LIFT: f64 = 0.12;
pub const FOCUS_PAD: f64 = 0.08;
/// The unit rows' baselines under the equation's (04_14).
pub const UNIT_BASE: [f64; 2] = [EQ_BASE - 0.62, EQ_BASE - 0.97];
/// The notes under the slots (04_11, 04_12).
pub const VISCOUS_NOTE_Y: f64 = -0.1;
pub const BODY_NOTE_AT: (f64, f64) = (6.6, -0.55);
/// 04_5's lines.
pub const DENSITY_PT: f64 = 36.0;
pub const DENSITY_LEFT: f64 = 0.6;
pub const DENSITY_BASE: [f64; 2] = [-1.0, -1.65];
pub const DENSITY_NOTE_GAP: f64 = 0.2;

/// The layers (04_6 to 04_9): bands across the main window, bottom first.
pub const LAYER_X: (f64, f64) = (-6.8, -1.7);
pub const LAYER_H: f64 = 1.1;
pub const LAYER_Y: [f64; 3] = [-1.0, 0.1, 1.2];
pub const SPEED_TAIL_X: f64 = -6.5;
pub const SPEED_PER: f64 = 0.5;
pub const SPEED_LABEL_X: f64 = -4.85;
/// Drag arrows, centred at `DRAG_X`, `DRAG_OFF` either side of a boundary,
/// `DRAG_PER` units long per m/s of difference.
pub const DRAG_X: f64 = -3.0;
pub const DRAG_OFF: f64 = 0.2;
pub const DRAG_PER: f64 = 0.55;
pub const PULL_LABEL_X: f64 = -2.05;
pub const NET_LABEL_LEFT: f64 = -1.6;
pub const AVERAGE_AT: (f64, f64) = (-3.6, -2.15);
pub const FINE_AT: (f64, f64) = (-3.6, -2.65);
pub const AVERAGE_PT: f64 = 26.0;
pub const PULL_PT: f64 = 30.0;

/// The 04_h1 card.
pub const CARD04: CardSpec = CardSpec {
    at: (3.65, -1.3),
    size: (5.6, 2.7),
    text_pt: 30.0,
    question: &["Is the middle layer pulled", "forward, held back, or neither?"],
    question_dy: &[0.95, 0.5],
    choices: &["forward", "back", "neither"],
    choice_dx: 1.6,
    choice_dy: -0.15,
    unit: None,
    answer: 2,
    answer_box: (1.4, 0.55),
    ring_lift: 0.5,
};

/// Pressure's orange (high) and the colours the palette lacks: viscosity's
/// violet and the body force's slate (the opening script's colour roles).
pub fn orange() -> Srgb {
    hex("#FF7A47")
}

pub fn violet() -> Srgb {
    hex("#B48CFF")
}

pub fn slate() -> Srgb {
    hex("#7F96B2")
}

// ------------------------------------------------------------- the plate

/// The plate's texel columns and rows: the channel's bounding box, x 0 to
/// `LENGTH`, y between the entrance's walls.
pub fn plate_size() -> (u32, u32) {
    let w = LENGTH * SCALE * PLATE_TEXELS;
    let h = 2.0 * wall(0.0) * SCALE * PLATE_TEXELS;
    (w.round() as u32, h.round() as u32)
}

/// The plate's top-left corner, scene units.
pub fn plate_corner() -> (f64, f64) {
    let c = to_scene(0.0, wall(0.0));
    (c[0], c[1])
}

/// The channel point under the centre of texel (`i`, `j`), row 0 at the top.
pub fn texel_point(i: u32, j: u32) -> (f64, f64) {
    let (left, top) = plate_corner();
    let x = left + (f64::from(i) + 0.5) / PLATE_TEXELS;
    let y = top - (f64::from(j) + 0.5) / PLATE_TEXELS;
    ((x - ORIGIN_AT.0) / SCALE, (y - ORIGIN_AT.1) / SCALE)
}

/// q's least and greatest values over the channel: the entrance's centre and
/// the exit's corners (q grows with x and with |y|, and |y| ≤ 4/(2 + x)).
pub fn drop_range() -> (f64, f64) {
    (pressure_drop(0.0, 0.0), pressure_drop(LENGTH, wall(LENGTH)))
}

/// The pressure's colour at channel point (x, y): orange at the highest
/// pressure, teal at the lowest, mixed as root's chapter 09 mixes its plate.
pub fn pressure_color(x: f64, y: f64) -> Srgb {
    let (lo, hi) = drop_range();
    let share = ((hi - pressure_drop(x, y)) / (hi - lo)).clamp(0.0, 1.0);
    interpolate_color(Palette::teal(), orange(), share)
}

/// One texel's RGBA8: the pressure colour, at `PLATE_ALPHA` inside the walls
/// and clear outside them.
pub fn plate_texel(i: u32, j: u32) -> [u8; 4] {
    let (x, y) = texel_point(i, j);
    let c = pressure_color(x, y);
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let alpha = if y.abs() < wall(x) { byte(PLATE_ALPHA) } else { 0 };
    [byte(c.r), byte(c.g), byte(c.b), alpha]
}

/// The plate's pixels, top row first.
pub fn plate_rgba() -> Vec<u8> {
    let (w, h) = plate_size();
    let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
    for j in 0..h {
        for i in 0..w {
            rgba.extend(plate_texel(i, j));
        }
    }
    rgba
}

/// The plate, detached, inside an empty group (the fades act on the group,
/// `plate_fade` on the image inside it).
fn build_plate(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let (w, h) = plate_size();
    let resource = ImageResource::rgba8(
        w,
        h,
        plate_rgba(),
        ImageColorSpace::Srgb,
        ImageSampler {
            wrap_u: ImageWrap::ClampToEdge,
            wrap_v: ImageWrap::ClampToEdge,
        },
    )
    .map_err(|e| fail(&e.to_string()))?;
    let image = ImageMobject::from_resource(resource)
        .map_err(|e| fail(&e.to_string()))?
        .with_height(f64::from(h) / PLATE_TEXELS)
        .with_z_index(Z_FIELD);
    let (left, top) = plate_corner();
    let centre = (
        left + f64::from(w) / PLATE_TEXELS / 2.0,
        top - f64::from(h) / PLATE_TEXELS / 2.0,
    );
    let mut object = Mobject::from(image);
    let mut points = object
        .buffer
        .read_column("point")
        .ok_or_else(|| fail("the plate's points are missing"))?;
    for point in points.as_chunks_mut::<3>().0 {
        point[0] += centre.0 as f32;
        point[1] += centre.1 as f32;
    }
    if !object.buffer.write_range("point", 0, &points) {
        return Err(fail("the plate could not be placed"));
    }
    let image = stage.arena_mut().add(object);
    let group = stage.arena_mut().add(VMobject::new());
    stage.attach(group, image)?;
    Ok(group)
}

// ---------------------------------------------------------------- helpers

/// Detached display math centred at `at`, then moved so its `edge` is there.
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

/// Detached text, as [`tex_at`].
fn words_at(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, color, at)?;
    stage.move_to(m, at, edge);
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

/// How far a piece's first glyph sits above the baseline: read from an
/// unshown copy with a flat-bottomed 2 appended (as `baseline_text`).
fn first_glyph_lift(stage: &Stage<'_>, probe: Mob) -> f64 {
    let glyphs = children(stage, probe);
    match (glyphs.first(), glyphs.last()) {
        (Some(&first), Some(&two)) if glyphs.len() >= 2 => stage.get_bottom(first)[1] - stage.get_bottom(two)[1],
        _ => 0.0,
    }
}

/// Moves `m` so its `edge` is at x and its baseline at `base`.
fn sit_on_baseline(stage: &mut Stage<'_>, m: Mob, lift: f64, x: f64, base: f64, edge: Vec3) {
    let head = children(stage, m).first().copied().unwrap_or(m);
    stage.move_to(m, v(x, base), edge);
    let dy = base + lift - stage.get_bottom(head)[1];
    stage.shift(m, v(0.0, dy));
    stage.set_z_index(m, Z_TEXT, true);
}

/// Display math with its baseline at `base` and its `edge` at x, so pieces
/// typeset apart share one baseline.
#[allow(clippy::too_many_arguments)]
fn baseline_tex(
    kit: &Kit,
    stage: &mut Stage<'_>,
    src: &str,
    size: f64,
    color: Srgb,
    x: f64,
    base: f64,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let probe = kit.tex(stage, &format!("{src}2"), size, color, &[], ORIGIN)?;
    let lift = first_glyph_lift(stage, probe);
    let m = kit.tex(stage, src, size, color, &[], ORIGIN)?;
    sit_on_baseline(stage, m, lift, x, base, edge);
    Ok(m)
}

/// Text with its baseline at `base`, as [`baseline_tex`].
#[allow(clippy::too_many_arguments)]
fn baseline_words(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    size: f64,
    color: Srgb,
    x: f64,
    base: f64,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let probe = kit.text(stage, &format!("{s}2"), size, color, ORIGIN)?;
    let lift = first_glyph_lift(stage, probe);
    let m = kit.text(stage, s, size, color, ORIGIN)?;
    sit_on_baseline(stage, m, lift, x, base, edge);
    Ok(m)
}

/// A detached arrow from `a` to `b`.
fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, thickness: f64, z: i32) -> fmn::Result<Mob> {
    let object = Arrow::new(a, b).buff(0.0).thickness(thickness).color(color).build()?;
    Ok(add(stage, object, z))
}

/// A horizontal arrow `length` long (its sign the direction), centred at
/// (x, y); `None` for no length.
fn level_arrow(
    stage: &mut Stage<'_>,
    x: f64,
    y: f64,
    length: f64,
    color: Srgb,
    thickness: f64,
) -> fmn::Result<Option<Mob>> {
    if length == 0.0 {
        return Ok(None);
    }
    let h = length / 2.0;
    arrow(stage, v(x - h, y), v(x + h, y), color, thickness, Z_TRAIL).map(Some)
}

/// A rounded box of `w` × `h` centred at `at`, outlined.
fn outline(stage: &mut Stage<'_>, at: Vec3, (w, h): (f64, f64), color: Srgb, width: f64, z: i32) -> fmn::Result<Mob> {
    let object = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(line_style(color, width, 1.0))
        .build()?
        .moved_to(at);
    Ok(add(stage, object, z))
}

/// Of `mobs`, those on screen.
fn shown(stage: &Stage<'_>, mobs: &[Mob]) -> Vec<Mob> {
    let on = stage.scene().mobjects();
    mobs.iter().copied().filter(|m| on.contains(m)).collect()
}

// --------------------------------------------------------------- the layers

/// Three layers of fluid and every arrow and label the viscosity beats use.
pub struct Layers {
    pub bands: Vec<Mob>,
    /// Blue speed arrows, bottom to top, at 1, 2 and 3 m/s; then the middle
    /// one at 3 (04_9).
    pub speeds: Vec<Mob>,
    pub middle_bent: Mob,
    pub labels: Vec<Mob>,
    pub middle_label_bent: Mob,
    /// Violet drag arrows: on the middle layer from above, on the top layer,
    /// on the middle layer from below, on the bottom layer.
    pub drags: Vec<Mob>,
    /// The middle layer's pull from below at 1, 3, 3 (twice as long).
    pub drag_bent: Mob,
    /// "+1" and "-1", then "0" and "-2", beside the middle layer's pulls.
    pub pulls: Vec<Mob>,
    pub pulls_bent: Vec<Mob>,
    /// "net 0", then "net -2".
    pub readouts: [Mob; 2],
    /// The middle layer's net arrow at 1, 3, 3: backward, 2 steps long.
    pub net: Mob,
    pub average: Mob,
    pub fine: Mob,
}

impl Layers {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let blue = Palette::blue();
        let width = LAYER_X.1 - LAYER_X.0;
        let centre_x = (LAYER_X.0 + LAYER_X.1) / 2.0;
        let mut bands = Vec::with_capacity(3);
        let mut speeds = Vec::with_capacity(3);
        let mut labels = Vec::with_capacity(3);
        for (k, &y) in LAYER_Y.iter().enumerate() {
            let band = Rectangle::new()
                .width(width)
                .height(LAYER_H)
                .style(Style::default().fill(blue, 0.08).stroke(Palette::faint(), 2.0, 1.0))
                .build()?
                .moved_to(v(centre_x, y));
            bands.push(add(stage, band, Z_FIELD));
            let tip = SPEED_TAIL_X + SPEED_PER * SPEEDS_EVEN[k];
            speeds.push(arrow(stage, v(SPEED_TAIL_X, y), v(tip, y), blue, NET_THICK, Z_MARKS)?);
            labels.push(words_at(
                kit,
                stage,
                &layer_label(SPEEDS_EVEN[k]),
                LABEL_PT,
                ink,
                v(SPEED_LABEL_X, y),
                LEFT,
            )?);
        }
        let middle_bent = arrow(
            stage,
            v(SPEED_TAIL_X, LAYER_Y[1]),
            v(SPEED_TAIL_X + SPEED_PER * SPEEDS_BENT[1], LAYER_Y[1]),
            blue,
            NET_THICK,
            Z_MARKS,
        )?;
        let middle_label_bent = words_at(
            kit,
            stage,
            &layer_label(SPEEDS_BENT[1]),
            LABEL_PT,
            ink,
            v(SPEED_LABEL_X, LAYER_Y[1]),
            LEFT,
        )?;

        // Each boundary carries two equal and opposite drags: on the slower
        // side forward, on the faster side back.
        let upper = (LAYER_Y[1] + LAYER_Y[2]) / 2.0;
        let lower = (LAYER_Y[0] + LAYER_Y[1]) / 2.0;
        let [from_above, from_below] = pulls(SPEEDS_EVEN);
        let s = SPEEDS_EVEN;
        let drag_rows = [
            (upper - DRAG_OFF, DRAG_PER * from_above),
            (upper + DRAG_OFF, DRAG_PER * (s[1] - s[2])),
            (lower + DRAG_OFF, DRAG_PER * from_below),
            (lower - DRAG_OFF, DRAG_PER * (s[1] - s[0])),
        ];
        let violet = violet();
        let mut drags = Vec::with_capacity(4);
        for (y, length) in drag_rows {
            drags.extend(level_arrow(stage, DRAG_X, y, length, violet, NET_THICK)?);
        }
        if drags.len() != 4 {
            return Err(fail("an even layer has no drag"));
        }
        let [_, below_bent] = pulls(SPEEDS_BENT);
        let drag_bent = level_arrow(
            stage,
            DRAG_X,
            lower + DRAG_OFF,
            DRAG_PER * below_bent,
            violet,
            NET_THICK,
        )?
        .ok_or_else(|| fail("the bent layer has no pull from below"))?;

        let rows = [upper - DRAG_OFF, lower + DRAG_OFF];
        let mut pull_labels = Vec::with_capacity(2);
        let mut pull_labels_bent = Vec::with_capacity(2);
        for (k, &y) in rows.iter().enumerate() {
            let at = v(PULL_LABEL_X, y);
            pull_labels.push(tex_at(
                kit,
                stage,
                &signed(pulls(SPEEDS_EVEN)[k]),
                PULL_PT,
                violet,
                at,
                ORIGIN,
            )?);
            pull_labels_bent.push(tex_at(
                kit,
                stage,
                &signed(pulls(SPEEDS_BENT)[k]),
                PULL_PT,
                violet,
                at,
                ORIGIN,
            )?);
        }
        let readout = |stage: &mut Stage<'_>, speeds| {
            tex_at(
                kit,
                stage,
                &net_label(speeds),
                PULL_PT,
                violet,
                v(NET_LABEL_LEFT, LAYER_Y[1]),
                LEFT,
            )
        };
        let readouts = [readout(stage, SPEEDS_EVEN)?, readout(stage, SPEEDS_BENT)?];
        let net = level_arrow(
            stage,
            0.1,
            LAYER_Y[1],
            DRAG_PER * net_pull(SPEEDS_BENT),
            violet,
            NET_THICK + 2.0,
        )?
        .ok_or_else(|| fail("the bent layer has no net pull"))?;
        let average = words_at(
            kit,
            stage,
            &average_line(SPEEDS_BENT),
            AVERAGE_PT,
            blue,
            v(AVERAGE_AT.0, AVERAGE_AT.1),
            ORIGIN,
        )?;
        let fine = tex_at(
            kit,
            stage,
            FINE,
            FINE_PT,
            Palette::dim(),
            v(FINE_AT.0, FINE_AT.1),
            ORIGIN,
        )?;
        Ok(Self {
            bands,
            speeds,
            middle_bent,
            labels,
            middle_label_bent,
            drags,
            drag_bent,
            pulls: pull_labels,
            pulls_bent: pull_labels_bent,
            readouts,
            net,
            average,
            fine,
        })
    }

    /// Every object of the layers'.
    pub fn all(&self) -> Vec<Mob> {
        let mut all = Vec::new();
        for group in [
            &self.bands,
            &self.speeds,
            &self.labels,
            &self.drags,
            &self.pulls,
            &self.pulls_bent,
        ] {
            all.extend(group);
        }
        all.extend([
            self.middle_bent,
            self.middle_label_bent,
            self.drag_bent,
            self.net,
            self.average,
            self.fine,
        ]);
        all.extend(self.readouts);
        all
    }
}

// ------------------------------------------------------------- the chapter

/// Every object of chapter 04's, built once and detached at 04_1.
pub struct Forces {
    pub title: Mob,
    /// The channel's walls, on screen from the first frame.
    pub walls: Vec<Mob>,
    pub plate: Mob,
    pub legend: [Mob; 2],
    pub parcel: Mob,
    /// Equal pressure on the four faces: behind, ahead, above, below.
    pub faces: [Mob; 4],
    /// The longer push from behind (04_2).
    pub back: Mob,
    /// "net 0" under the square (04_2).
    pub balance: Mob,
    /// The net pressure push, forward, inside the square.
    pub push: Mob,
    /// "2 m/s²", then "2 N per kg", left of the square (04_4).
    pub push_labels: [Mob; 2],
    pub slots: [Mob; 3],
    /// −∇p, then −∇p/ρ (04_3).
    pub gradient: Mob,
    /// −∇p/ρ, ν∇²u and f, in their slots.
    pub terms: [Mob; 3],
    pub lhs: [Mob; 2],
    pub plus: [Mob; 2],
    /// ρ = 1.3 kg/m³, its note, and the 2.6 Pa per m line (04_5).
    pub density: [Mob; 3],
    pub layers: Layers,
    pub viscous_note: Mob,
    pub focus: Mob,
    pub gravity: Mob,
    pub body_note: Mob,
    /// N/kg and = m/s² under each term, m/s² under Du/Dt (04_14).
    pub units: Vec<Mob>,
}

impl Forces {
    /// Build everything detached; only the walls go on screen.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let gold = Palette::gold();
        let orange = orange();
        let title = kit.text_left(stage, TITLE, TITLE_PT, ink, v(TITLE_AT.0, TITLE_AT.1))?;
        let walls = build_walls(stage);
        let plate = build_plate(stage)?;
        let (exit_x, low_exit) = {
            let p = to_scene(LENGTH, -wall(LENGTH));
            (p[0], p[1])
        };
        let (entrance_x, low_entrance) = {
            let p = to_scene(0.0, -wall(0.0));
            (p[0], p[1])
        };
        let legend = [
            words_at(
                kit,
                stage,
                LEGEND[0],
                FINE_PT,
                orange,
                v(entrance_x, low_entrance - LEGEND_GAP),
                ORIGIN,
            )?,
            words_at(
                kit,
                stage,
                LEGEND[1],
                FINE_PT,
                Palette::teal(),
                v(exit_x, low_exit - LEGEND_GAP),
                ORIGIN,
            )?,
        ];

        let c = to_scene(PARCEL_CENTRE.0, PARCEL_CENTRE.1);
        let half = PARCEL_SIDE * SCALE / 2.0;
        let square = Rectangle::new()
            .width(2.0 * half)
            .height(2.0 * half)
            .style(line_style(gold, 4.0, 1.0))
            .build()?
            .moved_to(c);
        let parcel = add(stage, square, Z_PARCEL);
        // inward arrows: (direction from the face outwards, face centre)
        let (cx, cy) = (c[0], c[1]);
        let near = half + FACE_GAP;
        let face = |stage: &mut Stage<'_>, out: (f64, f64), length: f64| {
            let tip = v(cx + out.0 * near, cy + out.1 * near);
            let tail = v(tip[0] + out.0 * length, tip[1] + out.1 * length);
            arrow(stage, tail, tip, orange, PUSH_THICK, Z_MARKS)
        };
        let faces = [
            face(stage, (-1.0, 0.0), FACE_L)?,
            face(stage, (1.0, 0.0), FACE_L)?,
            face(stage, (0.0, 1.0), FACE_L)?,
            face(stage, (0.0, -1.0), FACE_L)?,
        ];
        let back = face(stage, (-1.0, 0.0), BACK_L)?;
        let balance = tex_at(
            kit,
            stage,
            r"\mathrm{net}\;0",
            LABEL_PT,
            orange,
            v(cx, cy + BALANCE_DY),
            ORIGIN,
        )?;
        let push = arrow(
            stage,
            v(cx - NET_PUSH_L / 2.0, cy),
            v(cx + NET_PUSH_L / 2.0, cy),
            orange,
            NET_THICK,
            Z_TRAIL,
        )?;
        let label_at = v(cx - half - PUSH_LABEL_GAP, cy);
        let push_labels = [
            tex_at(kit, stage, &accel_label(), LABEL_PT, orange, label_at, RIGHT)?,
            tex_at(kit, stage, &push_label(), LABEL_PT, orange, label_at, RIGHT)?,
        ];

        let slot_y = EQ_BASE + SLOT_LIFT;
        let mut slots = Vec::with_capacity(3);
        for (x, w) in TERM_X.iter().zip(SLOT_W) {
            slots.push(outline(
                stage,
                v(*x, slot_y),
                (w, SLOT_H),
                Palette::faint(),
                2.5,
                Z_MARKS,
            )?);
        }
        let colors = [orange, violet(), slate()];
        let gradient = baseline_tex(kit, stage, GRADIENT, EQ_PT, orange, TERM_X[0], EQ_BASE, ORIGIN)?;
        let mut terms = Vec::with_capacity(3);
        for ((src, x), color) in TERMS.iter().zip(TERM_X).zip(colors) {
            terms.push(baseline_tex(kit, stage, src, EQ_PT, color, x, EQ_BASE, ORIGIN)?);
        }
        let lhs = [
            baseline_tex(kit, stage, LHS[0], EQ_PT, ink, LHS_X[0], EQ_BASE, ORIGIN)?,
            baseline_tex(kit, stage, LHS[1], EQ_PT, ink, LHS_X[1], EQ_BASE, ORIGIN)?,
        ];
        let plus = [
            baseline_tex(kit, stage, PLUS, EQ_PT, ink, PLUS_X[0], EQ_BASE, ORIGIN)?,
            baseline_tex(kit, stage, PLUS, EQ_PT, ink, PLUS_X[1], EQ_BASE, ORIGIN)?,
        ];

        let rho = baseline_tex(
            kit,
            stage,
            &density_line(),
            DENSITY_PT,
            ink,
            DENSITY_LEFT,
            DENSITY_BASE[0],
            LEFT,
        )?;
        let note_x = stage.get_right(rho)[0] + DENSITY_NOTE_GAP;
        let note = baseline_words(
            kit,
            stage,
            DENSITY_NOTE,
            FINE_PT,
            Palette::dim(),
            note_x,
            DENSITY_BASE[0],
            LEFT,
        )?;
        let product = baseline_tex(
            kit,
            stage,
            &drop_line(),
            DENSITY_PT,
            ink,
            DENSITY_LEFT,
            DENSITY_BASE[1],
            LEFT,
        )?;

        let layers = Layers::build(kit, stage)?;
        let viscous_note = words_at(
            kit,
            stage,
            VISCOUS_NOTE,
            FINE_PT,
            violet(),
            v(TERM_X[1], VISCOUS_NOTE_Y),
            ORIGIN,
        )?;
        let focus = outline(
            stage,
            v(TERM_X[0], slot_y),
            (SLOT_W[0] + 2.0 * FOCUS_PAD, SLOT_H + 2.0 * FOCUS_PAD),
            white(),
            HIGHLIGHT_W,
            Z_HIGHLIGHT,
        )?;
        let gravity = arrow(
            stage,
            v(cx, cy + half + 0.5),
            v(cx, cy - half - 0.5),
            slate(),
            NET_THICK,
            Z_HIGHLIGHT,
        )?;
        let body_note = words_at(
            kit,
            stage,
            BODY_NOTE,
            FINE_PT,
            slate(),
            v(BODY_NOTE_AT.0, BODY_NOTE_AT.1),
            RIGHT,
        )?;
        let mut units = Vec::with_capacity(7);
        for &x in &TERM_X {
            for (src, base) in UNIT_ROWS.iter().zip(UNIT_BASE) {
                units.push(baseline_tex(kit, stage, src, FINE_PT, ink, x, base, ORIGIN)?);
            }
        }
        units.push(baseline_tex(
            kit,
            stage,
            LHS_UNIT,
            FINE_PT,
            ink,
            LHS_X[0],
            UNIT_BASE[1],
            ORIGIN,
        )?);

        for &w in &walls {
            stage.add_to_scene(w)?;
        }
        Ok(Self {
            title,
            walls,
            plate,
            legend,
            parcel,
            faces,
            back,
            balance,
            push,
            push_labels,
            slots: [slots[0], slots[1], slots[2]],
            gradient,
            terms: [terms[0], terms[1], terms[2]],
            lhs,
            plus,
            density: [rho, note, product],
            layers,
            viscous_note,
            focus,
            gravity,
            body_note,
            units,
        })
    }

    /// The channel picture 04_6 takes away and 04_11 brings back: walls,
    /// pressure plate and legend, the parcel, its net push and label.
    pub fn channel(&self) -> Vec<Mob> {
        let mut all = self.walls.clone();
        all.push(self.plate);
        all.extend(self.legend);
        all.extend([self.parcel, self.push, self.push_labels[1]]);
        all
    }

    /// What is on screen when the chapter ends, for chapter 05's opening.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn final_picture(&self) -> Vec<Mob> {
        let mut all = vec![self.title];
        all.extend(self.channel());
        all.extend(self.lhs);
        all.extend(self.terms);
        all.extend(self.plus);
        all.extend(&self.units);
        all
    }
}

// ---------------------------------------------------------------- the cues

/// 04_1 "Newton's law says force equals mass times acceleration. …": the
/// title and the parcel square at the entrance, between chapter 03's walls
/// (on screen from the first frame, before the line starts); with sentence 1
/// ("…the forces come from three places.") three empty slots.
pub fn cue_04_1(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Forces> {
    let forces = Forces::build(kit, stage)?;
    kit.say(stage, "04_1")?;
    fade_in_all(stage, &[forces.title, forces.parcel], OPEN)?;
    kit.until_sentence(stage, "04_1", 1)?;
    fade_in_all(stage, &forces.slots, FADE)?;
    Ok(forces)
}

/// 04_2 "First, pressure. …": sentence 1, equal orange arrows on all four
/// faces; sentence 2 ("Equal pressure all around adds up to nothing."), "net
/// 0"; sentence 3 ("What pushes is a difference…"), the back arrow grows,
/// "net 0" goes and the net arrow points forward.
pub fn cue_04_2(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_2")?;
    kit.until_sentence(stage, "04_2", 1)?;
    fade_in_all(stage, &forces.faces, FADE)?;
    kit.until_sentence(stage, "04_2", 2)?;
    fade_in_all(stage, &[forces.balance], FADE)?;
    kit.until_sentence(stage, "04_2", 3)?;
    swap(
        stage,
        &[forces.faces[0], forces.balance],
        &[forces.back, forces.push],
        FADE,
    )
}

/// 04_3 "So the push points from high pressure toward low, …": sentence 1
/// ("That's minus the pressure gradient."), −∇p in the first slot; sentence 2
/// ("Divide by the density…"), −∇p/ρ instead.
pub fn cue_04_3(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_3")?;
    kit.until_sentence(stage, "04_3", 1)?;
    fade_in_all(stage, &[forces.gradient], FADE)?;
    kit.until_sentence(stage, "04_3", 2)?;
    swap(stage, &[forces.gradient], &forces.terms[..1], FADE)
}

/// 04_4 "In our channel, the parcel at the entrance speeds up at two meters
/// per second, every second. …": the face arrows go, "2 m/s²" beside the
/// net arrow; sentence 1 ("So the pressure must fall ahead of it…"), the
/// channel fills with its pressure, orange to teal, and the label becomes
/// "2 N per kg".
pub fn cue_04_4(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_4")?;
    let mut faces = forces.faces[1..].to_vec();
    faces.push(forces.back);
    fade_out_all(stage, &faces, QUICK)?;
    fade_in_all(stage, &forces.push_labels[..1], FADE)?;
    kit.until_sentence(stage, "04_4", 1)?;
    let mut fill = vec![forces.plate];
    fill.extend(forces.legend);
    fade_in_all(stage, &fill, PLATE_IN)?;
    swap(stage, &forces.push_labels[..1], &forces.push_labels[1..], FADE)
}

/// 04_5 "Air is light. …": sentence 1, "ρ = 1.3 kg/m³ (our choice for this
/// example)"; sentence 2, "1.3 kg/m³ × 2 m/s² = 2.6 Pa per m".
pub fn cue_04_5(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_5")?;
    kit.until_sentence(stage, "04_5", 1)?;
    fade_in_all(stage, &forces.density[..2], FADE)?;
    kit.until_sentence(stage, "04_5", 2)?;
    fade_in_all(stage, &forces.density[2..], FADE)
}

/// 04_6 "Second, viscosity. …": the channel and the density lines go, three
/// layers with their blue speed arrows come; sentence 2 ("A faster layer
/// pulls a slower neighbor forward…"), the violet drags along both shared
/// boundaries, equal and opposite.
pub fn cue_04_6(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_6")?;
    let mut out = shown(stage, &forces.channel());
    out.extend(&forces.density);
    fade_out_all(stage, &out, FADE)?;
    let mut layers = forces.layers.bands.clone();
    layers.extend(&forces.layers.speeds);
    fade_in_all(stage, &layers, FADE)?;
    kit.until_sentence(stage, "04_6", 2)?;
    create_all(stage, &forces.layers.drags, FADE)
}

/// 04_7 "Here are three layers moving at one, two and three meters per
/// second. …": the speed labels; sentence 1, the question card.
pub fn cue_04_7(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<PredictCard> {
    kit.say(stage, "04_7")?;
    let card = PredictCard::build(kit, stage, &CARD04)?;
    fade_in_all(stage, &forces.layers.labels, FADE)?;
    kit.until_sentence(stage, "04_7", 1)?;
    card.show(stage)?;
    Ok(card)
}

/// HOLD 04_h1: the question finishes, the moment is marked for the score and
/// the 5 s countdown runs. The middle layer's net arrow, its net readout and
/// the pull values stay hidden.
pub fn hold_04_h1(kit: &Kit, stage: &mut Stage<'_>, card: &PredictCard) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    card.count(stage)
}

/// 04_8 "Neither. …": the answer; sentence 1 ("Both pulls are real…"), the
/// card and the outer layers' drags go and the middle layer's two pulls read
/// +1 and −1; sentence 2 ("…viscosity's net pull is zero."), "net 0" and the
/// pulls' unit in fine print.
pub fn cue_04_8(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces, card: &PredictCard) -> fmn::Result<()> {
    kit.say(stage, "04_8")?;
    card.answer(stage)?;
    kit.until_sentence(stage, "04_8", 1)?;
    card.leave(stage)?;
    let drags = &forces.layers.drags;
    swap(stage, &[drags[1], drags[3]], &forces.layers.pulls, FADE)?;
    kit.until_sentence(stage, "04_8", 2)?;
    fade_in_all(stage, &[forces.layers.readouts[0], forces.layers.fine], FADE)
}

/// 04_9 "Make the speeds one, three and three. …": the middle layer goes to
/// 3 m/s, its pull from above vanishes (0), the pull from below doubles (−2),
/// and the old net label leaves. Sentence 1 reveals the average, the backward
/// arrow and "net −2".
pub fn cue_04_9(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_9")?;
    let l = &forces.layers;
    swap(
        stage,
        &[
            l.speeds[1],
            l.labels[1],
            l.drags[0],
            l.drags[2],
            l.pulls[0],
            l.pulls[1],
            l.readouts[0],
        ],
        &[
            l.middle_bent,
            l.middle_label_bent,
            l.drag_bent,
            l.pulls_bent[0],
            l.pulls_bent[1],
        ],
        FADE,
    )?;
    kit.until_sentence(stage, "04_9", 1)?;
    fade_in_all(stage, &[l.average, l.net, l.readouts[1]], FADE)
}

/// 04_10 "In symbols, that's nu times the Laplacian of the velocity. …": ν∇²u
/// in the second slot, violet.
pub fn cue_04_10(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_10")?;
    fade_in_all(stage, &forces.terms[1..2], FADE)
}

/// 04_11 "In our channel, each part of the velocity changes in a straight
/// line…": the layers go and the channel comes back; sentence 1 ("So
/// viscosity's pulls cancel there too…"), "net 0 for this flow" under the
/// violet slot; sentence 2 ("It's all pressure."), the first slot outlined
/// white.
pub fn cue_04_11(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_11")?;
    let out = shown(stage, &forces.layers.all());
    fade_out_all(stage, &out, FADE)?;
    fade_in_all(stage, &forces.channel(), PLATE_IN)?;
    kit.until_sentence(stage, "04_11", 1)?;
    fade_in_all(stage, &[forces.viscous_note], FADE)?;
    kit.until_sentence(stage, "04_11", 2)?;
    fade_in_all(stage, &[forces.focus], FADE)
}

/// 04_12 "Third, forces that act on the whole parcel at once, like gravity.
/// …": the white outline goes, a slate arrow points down through the parcel;
/// sentence 1 ("We write f…"), f in the third slot; sentence 3 ("So we fold
/// it into the pressure, and set f to zero."), the arrow goes and "f = 0
/// (gravity folded into p)" comes.
pub fn cue_04_12(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_12")?;
    swap(stage, &[forces.focus], &[forces.gravity], FADE)?;
    kit.until_sentence(stage, "04_12", 1)?;
    fade_in_all(stage, &forces.terms[2..], FADE)?;
    kit.until_sentence(stage, "04_12", 3)?;
    swap(stage, &[forces.gravity], &[forces.body_note], FADE)
}

/// 04_13 "Add the three, and you have the parcel's acceleration. …": the
/// notes and the empty slots go, and Du/Dt = … + … + … completes the law,
/// each term in its colour.
pub fn cue_04_13(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_13")?;
    let mut out = vec![forces.viscous_note, forces.body_note];
    out.extend(forces.slots);
    let mut into = forces.lhs.to_vec();
    into.extend(forces.plus);
    swap(stage, &out, &into, FADE)
}

/// 04_14 "Each term is a force on one kilogram, …": N/kg under each term,
/// then = m/s² (and m/s² under Du/Dt); then the line's end and its breath.
pub fn cue_04_14(kit: &Kit, stage: &mut Stage<'_>, forces: &Forces) -> fmn::Result<()> {
    kit.say(stage, "04_14")?;
    let newtons: Vec<Mob> = forces.units[..6].iter().copied().step_by(2).collect();
    let metres: Vec<Mob> = forces.units[1..6]
        .iter()
        .copied()
        .step_by(2)
        .chain(forces.units[6..].iter().copied())
        .collect();
    fade_in_all(stage, &newtons, FADE)?;
    fade_in_all(stage, &metres, FADE)?;
    kit.hold(stage)
}

/// Cues 04_1 to 04_14 and the 04_h1 hold, in order. Returns the chapter's
/// objects, the law and the channel still on screen.
pub fn chapter04(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Forces> {
    let forces = cue_04_1(kit, stage)?;
    cue_04_2(kit, stage, &forces)?;
    cue_04_3(kit, stage, &forces)?;
    cue_04_4(kit, stage, &forces)?;
    cue_04_5(kit, stage, &forces)?;
    cue_04_6(kit, stage, &forces)?;
    let card = cue_04_7(kit, stage, &forces)?;
    hold_04_h1(kit, stage, &card)?;
    cue_04_8(kit, stage, &forces, &card)?;
    cue_04_9(kit, stage, &forces)?;
    cue_04_10(kit, stage, &forces)?;
    cue_04_11(kit, stage, &forces)?;
    cue_04_12(kit, stage, &forces)?;
    cue_04_13(kit, stage, &forces)?;
    cue_04_14(kit, stage, &forces)?;
    Ok(forces)
}

/// Chapter 04 as a scene.
pub struct ParcelForcesScene {
    pub kit: Kit,
}

impl SceneConstruct for ParcelForcesScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter04(&self.kit, stage)?;
        self.kit.finish(stage)
    }
}

/// The spoken lines, verbatim from `navier-full-script03.md`
/// (`check_forces01.py` compares them byte for byte), as `script.tsv` rows.
/// 04_h1 is a hold, not a line.
#[cfg(test)]
pub const SCRIPT04: &str = concat!(
    "04_1\tNewton's law says force equals mass times acceleration. For a parcel of fluid, the forces come from three places.\n",
    "04_2\tFirst, pressure. The air around the parcel presses on every side. Equal pressure all around adds up to nothing. What pushes is a difference, more pressure behind than in front.\n",
    "04_3\tSo the push points from high pressure toward low, and its strength is how fast the pressure falls along the way. That's minus the pressure gradient. Divide by the density, and you have the acceleration it causes.\n",
    "04_4\tIn our channel, the parcel at the entrance speeds up at two meters per second, every second. So the pressure must fall ahead of it, steeply enough to push each kilogram with two newtons.\n",
    "04_5\tAir is light. For this example, we'll take its density to be one point three kilograms per cubic meter. Then that push needs a pressure drop of just two point six pascals for each meter.\n",
    "04_6\tSecond, viscosity. Neighboring layers of fluid drag on each other. A faster layer pulls a slower neighbor forward, and the slower one holds it back.\n",
    "04_7\tHere are three layers moving at one, two and three meters per second. Is the middle layer pulled forward, held back, or neither?\n",
    "04_8\tNeither. Both pulls are real, and the fast layer pulls it forward exactly as hard as the slow layer holds it back. When the speed climbs evenly from layer to layer, viscosity's net pull is zero. Only a bend leaves a net pull.\n",
    "04_9\tMake the speeds one, three and three. Now the middle layer is faster than the average of its neighbors, which is two, so it's held back. Viscosity pulls each bit of fluid toward the average of what's around it.\n",
    "04_10\tIn symbols, that's nu times the Laplacian of the velocity. Nu, the kinematic viscosity, says how quickly that drag evens out the speeds.\n",
    "04_11\tIn our channel, each part of the velocity changes in a straight line from place to place, with no bend. So viscosity's pulls cancel there too, and its net pull is zero. It's all pressure.\n",
    "04_12\tThird, forces that act on the whole parcel at once, like gravity. We write f for the force on each kilogram. In our tunnel, gravity's only effect is a pressure that rises steadily with depth. So we fold it into the pressure, and set f to zero. But f belongs in the law.\n",
    "04_13\tAdd the three, and you have the parcel's acceleration. Minus the pressure gradient over density, plus nu times the Laplacian of velocity, plus f.\n",
    "04_14\tEach term is a force on one kilogram, which is the same thing as meters per second, every second.\n",
);

#[cfg(test)]
mod tests {
    use super::super::chapter03::velocity;
    use super::*;
    use explainer_kit::{Narrator, Script};

    const H: f64 = 1e-4;

    /// (u·∇)u of the module's velocity field, by central differences.
    fn advection(x: f64, y: f64) -> (f64, f64) {
        let (u, w) = velocity(x, y);
        let dx = |k: usize| {
            let (a, b) = (velocity(x + H, y), velocity(x - H, y));
            [(a.0 - b.0) / (2.0 * H), (a.1 - b.1) / (2.0 * H)][k]
        };
        let dy = |k: usize| {
            let (a, b) = (velocity(x, y + H), velocity(x, y - H));
            [(a.0 - b.0) / (2.0 * H), (a.1 - b.1) / (2.0 * H)][k]
        };
        (u * dx(0) + w * dy(0), u * dx(1) + w * dy(1))
    }

    /// The push is minus the gradient of p/ρ, it is the steady flow's whole
    /// acceleration, the velocity has no bend, and the facts follow:
    /// ch.pressure.push.in 2, ch.laplacian 0, ch.divergence 0.
    #[test]
    fn the_pressure_push_is_the_acceleration() {
        for i in 0..=8 {
            for j in -4..=4 {
                let x = LENGTH * f64::from(i) / 8.0;
                let y = wall(x) * f64::from(j) / 5.0;
                // −∇(p/ρ) = ∇q
                let gx = (pressure_drop(x + H, y) - pressure_drop(x - H, y)) / (2.0 * H);
                let gy = (pressure_drop(x, y + H) - pressure_drop(x, y - H)) / (2.0 * H);
                let (px, py) = push(x, y);
                assert!((gx - px).abs() < 1e-8 && (gy - py).abs() < 1e-8, "push at ({x}, {y})");
                let (ax, ay) = advection(x, y);
                assert!((ax - px).abs() < 1e-8 && (ay - py).abs() < 1e-8, "(u·∇)u at ({x}, {y})");
                for k in 0..2 {
                    let at = |x: f64, y: f64| [velocity(x, y).0, velocity(x, y).1][k];
                    let lap = (at(x + H, y) + at(x - H, y) + at(x, y + H) + at(x, y - H) - 4.0 * at(x, y)) / (H * H);
                    assert!(lap.abs() < 1e-4, "∇²u component {k} at ({x}, {y})");
                }
                let div = (velocity(x + H, y).0 - velocity(x - H, y).0 + velocity(x, y + H).1 - velocity(x, y - H).1)
                    / (2.0 * H);
                assert!(div.abs() < 1e-8, "∇·u at ({x}, {y})");
            }
        }
        assert_eq!(push(0.0, 0.0), (2.0, 0.0), "ch.pressure.push.in");
        assert_eq!(push(0.0, 0.0).0, parcel_accel(0.0), "push in = accel in");
        // the square's average push is the entrance value: the push is linear
        let half = PARCEL_SIDE / 2.0;
        let mean = (0..=100)
            .map(|k| push(PARCEL_CENTRE.0 - half + PARCEL_SIDE * f64::from(k) / 100.0, 0.0).0)
            .sum::<f64>()
            / 101.0;
        assert!((mean - 2.0).abs() < 1e-12, "the square's mean push {mean}");
    }

    /// ch.pressure.gradient.air 2.6, units.N.per.kg, and the words.
    #[test]
    fn the_numbers_on_screen_are_the_facts() {
        assert_eq!(gradient_air(), 2.6);
        assert_eq!(format!("{}", gradient_air()), "2.6");
        assert_eq!(accel_label(), r"2\,\mathrm{m/s^{2}}");
        assert_eq!(push_label(), r"2\,\mathrm{N\ per\ kg}");
        assert_eq!(density_line(), r"\rho = 1.3\,\mathrm{kg/m^{3}}");
        assert_eq!(
            drop_line(),
            r"1.3\,\mathrm{kg/m^{3}}\times 2\,\mathrm{m/s^{2}} = 2.6\,\mathrm{Pa\ per\ m}"
        );
        assert_eq!(
            [layer_label(1.0), layer_label(2.0), layer_label(3.0)],
            ["1 m/s", "2 m/s", "3 m/s"]
        );
        assert_eq!(average_line(SPEEDS_BENT), "neighbors' average: (1 + 3)/2 = 2 m/s");
        assert_eq!(
            [net_label(SPEEDS_EVEN), net_label(SPEEDS_BENT)],
            [r"\mathrm{net}\;{0}", r"\mathrm{net}\;{-2}"]
        );
    }

    /// layers.123.net 0, layers.133.net −2, layers.133.avg 2; the pulls are
    /// signed (+1, −1) and (0, −2).
    #[test]
    fn the_layers_pull_as_the_facts_say() {
        assert_eq!(pulls(SPEEDS_EVEN), [1.0, -1.0]);
        assert_eq!(net_pull(SPEEDS_EVEN), 0.0);
        assert_eq!(pulls(SPEEDS_BENT), [0.0, -2.0]);
        assert_eq!(net_pull(SPEEDS_BENT), -2.0);
        assert_eq!(neighbours_average(SPEEDS_BENT), 2.0);
        assert_eq!(
            [signed(1.0), signed(-1.0), signed(0.0), signed(-2.0)],
            ["+1", "-1", "0", "-2"]
        );
        // the net is the second difference, and it points toward the average
        for s in [SPEEDS_EVEN, SPEEDS_BENT] {
            assert_eq!(net_pull(s), s[2] - 2.0 * s[1] + s[0]);
            assert!(net_pull(s) * (neighbours_average(s) - s[1]) >= 0.0);
        }
    }

    /// The plate covers the channel at 1.25 texels a 4K pixel, orange at the
    /// entrance's centre, teal at the exit's corners, clear outside the walls.
    #[test]
    fn the_plate_paints_the_pressure() {
        assert_eq!(plate_size(), (720, 1440));
        assert_eq!(PLATE_TEXELS / (2160.0 / 9.0), 1.25);
        assert_eq!(drop_range(), (2.0, 8.5));
        let (w, h) = plate_size();
        let byte = |v: f64| (v * 255.0).round() as u8;
        let o = orange();
        let t = Palette::teal();
        // the texel nearest the entrance's centre is within one texel of orange
        let entrance = plate_texel(0, h / 2);
        assert_eq!(entrance[3], byte(PLATE_ALPHA));
        for (k, c) in [o.r, o.g, o.b].into_iter().enumerate() {
            assert!(
                (i32::from(entrance[k]) - i32::from(byte(c))).abs() <= 1,
                "orange channel {k}"
            );
        }
        // inside every texel's share is in [0, 1]; outside, alpha 0
        let mut inside = 0_u64;
        for j in (0..h).step_by(7) {
            for i in (0..w).step_by(7) {
                let (x, y) = texel_point(i, j);
                let q = pressure_drop(x, y);
                let texel = plate_texel(i, j);
                if y.abs() < wall(x) {
                    inside += 1;
                    assert!((2.0..=8.5).contains(&q), "q {q} at ({x}, {y})");
                    assert_eq!(texel[3], 92);
                } else {
                    assert_eq!(texel[3], 0);
                }
            }
        }
        assert!(inside > 0);
        // the exit's corners are teal
        let corner = pressure_color(LENGTH, wall(LENGTH));
        assert_eq!(
            (byte(corner.r), byte(corner.g), byte(corner.b)),
            (byte(t.r), byte(t.g), byte(t.b))
        );
        let rgba = plate_rgba();
        assert_eq!(rgba.len(), (w * h * 4) as usize);
        // check_forces01.py counts 718672 texels inside the walls (5.5453 m²
        // against the channel's 8 ln 2 = 5.5452 m²), texel by texel
        let (pixels, trailing_bytes) = rgba.as_chunks::<4>();
        assert!(trailing_bytes.is_empty());
        let shown = pixels.iter().filter(|px| px[3] == byte(PLATE_ALPHA)).count();
        let clear = pixels.iter().filter(|px| px[3] == 0).count();
        assert_eq!((shown, shown + clear), (718_672, (w * h) as usize));
    }

    /// The card's answer is the script's, and every sentence a cue waits for
    /// exists in its line.
    #[test]
    fn every_sentence_gate_exists() {
        assert_eq!(CARD04.choices[CARD04.answer], "neither");
        assert!(SCRIPT04.contains("04_8\tNeither. "));
        let script = Script::parse(SCRIPT04).unwrap();
        let narrator = Narrator::estimate(&script);
        for (id, j) in [
            ("04_1", 1),
            ("04_2", 1),
            ("04_2", 2),
            ("04_2", 3),
            ("04_3", 1),
            ("04_3", 2),
            ("04_4", 1),
            ("04_5", 1),
            ("04_5", 2),
            ("04_6", 2),
            ("04_7", 1),
            ("04_8", 1),
            ("04_8", 2),
            ("04_9", 1),
            ("04_11", 1),
            ("04_11", 2),
            ("04_12", 1),
            ("04_12", 3),
        ] {
            assert!(narrator.sentence_starts(id).len() > j, "{id} sentence {j}");
        }
    }

    /// Runs `check` on a scene built from `cues` on the scratch voice at 10 fps.
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
        let script = Script::parse(SCRIPT04).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe { kit, run }, config, 7, &mut fmn::scene::NullSceneSink).expect("the probe runs");
    }

    /// Every object sits in its window: the channel and layers in the main
    /// window above the caption slot, the panel's in the panel; each term
    /// fits its slot.
    #[test]
    fn everything_sits_in_its_window() {
        probe(|kit, stage| {
            let f = Forces::build(kit, stage)?;
            let bounds = |stage: &Stage<'_>, m: Mob| {
                (
                    stage.get_left(m)[0],
                    stage.get_right(m)[0],
                    stage.get_bottom(m)[1],
                    stage.get_top(m)[1],
                )
            };
            let mut main = f.channel();
            main.extend([f.title, f.balance, f.back, f.gravity, f.push_labels[0]]);
            main.extend(f.faces);
            main.extend(f.layers.all().into_iter().filter(|&m| m != f.layers.net));
            for m in main {
                let (l, r, b, t) = bounds(stage, m);
                assert!(
                    l >= -7.1 && r <= 0.2 && b >= -3.15 && t <= 3.9,
                    "main {m:?}: {l} {r} {b} {t}"
                );
            }
            let (net_left, net_right, net_bottom, net_top) = bounds(stage, f.layers.net);
            assert!(
                net_left > bounds(stage, f.layers.readouts[1]).1
                    && net_left > bounds(stage, f.layers.drag_bent).1
                    && net_right < bounds(stage, f.slots[0]).0
                    && net_bottom >= -3.15
                    && net_top <= 1.75,
                "net arrow must clear its label, the local pull and the pressure slot"
            );
            let mut panel = f.slots.to_vec();
            panel.extend(f.terms);
            panel.extend(f.lhs);
            panel.extend(f.plus);
            panel.extend(f.density);
            panel.extend(&f.units);
            panel.extend([f.gradient, f.viscous_note, f.body_note, f.focus]);
            for m in panel {
                let (l, r, b, t) = bounds(stage, m);
                assert!(
                    l >= 0.4 && r <= 6.95 && b >= -3.15 && t <= 1.75,
                    "panel {m:?}: {l} {r} {b} {t}"
                );
            }
            for (k, &term) in f.terms.iter().enumerate() {
                let (l, r, b, t) = bounds(stage, term);
                let (sl, sr, sb, st) = bounds(stage, f.slots[k]);
                assert!(sl < l && r < sr && sb < b && t < st, "term {k} leaves its slot");
            }
            Ok(())
        });
    }

    /// During the 04_h1 countdown the middle layer's net arrow, its readouts
    /// and the pull values are hidden; the card and the layers are shown.
    #[test]
    fn the_hold_hides_the_answer() {
        probe(|kit, stage| {
            let forces = cue_04_1(kit, stage)?;
            cue_04_2(kit, stage, &forces)?;
            cue_04_3(kit, stage, &forces)?;
            cue_04_4(kit, stage, &forces)?;
            cue_04_5(kit, stage, &forces)?;
            cue_04_6(kit, stage, &forces)?;
            let card = cue_04_7(kit, stage, &forces)?;
            kit.hold(stage)?;
            let l = &forces.layers;
            let mut hidden = vec![l.net, l.average, l.fine, l.drag_bent, l.middle_bent];
            hidden.extend(l.readouts);
            hidden.extend(&l.pulls);
            hidden.extend(&l.pulls_bent);
            assert!(shown(stage, &hidden).is_empty(), "the answer shows before 04_8");
            assert_eq!(shown(stage, &[card.back]).len(), 1);
            assert_eq!(shown(stage, &l.drags).len(), 4);
            assert!(shown(stage, &forces.channel()).is_empty());
            Ok(())
        });
    }

    /// The whole chapter runs headless on the scratch voice and ends on the
    /// law and the channel.
    #[test]
    fn the_chapter_runs_on_the_scratch_voice() {
        probe(|kit, stage| {
            let forces = chapter04(kit, stage)?;
            let end = forces.final_picture();
            assert_eq!(shown(stage, &end).len(), end.len(), "the final picture is on screen");
            Ok(())
        });
    }
}
