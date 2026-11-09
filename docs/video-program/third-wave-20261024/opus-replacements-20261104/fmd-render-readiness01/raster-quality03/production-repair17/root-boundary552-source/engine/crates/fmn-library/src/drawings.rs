//! The drawings shelf, geometry-native tranche (fm-3kr, §12.7): `Clock`,
//! `DieFace`, and `Dartboard` — pure Marionette + Chisel compositions, no
//! new mechanisms (the shelf's quality bar). The asset-backed census
//! VectorizedEarth) load Reference-repo SVG files and are governed by
//! ADR-0020: those assets never ship — classes resolve user-supplied files,
//! refusing by name otherwise. They are deliberately absent here until
//! fm-3kr tranche 3 lands the resolver families.
//!
//! Self-goldens: one bit-locked point digest per class in
//! `tests/drawings.rs`, the fm-6l6 pattern.

use std::fmt;

use fmn_core::color::{Srgb, color_gradient};
use fmn_core::constants::{
    BLACK, BLUE_A, BLUE_B, BLUE_C, BLUE_D, DL, DOWN, DR, FRAME_WIDTH, GREEN, GREEN_E, GREEN_SCREEN,
    GREY_B, GREY_E, LEFT, MED_LARGE_BUFF, MED_SMALL_BUFF, ORIGIN, OUT, RED_E, RIGHT, TAU, UL, UP,
    UR, WHITE, YELLOW,
};
use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

use crate::arc::{AnnularSector, Circle, Dot};
use crate::boolean_ops::BooleanMobjectError;
use crate::line::Line;
use crate::poly::Rectangle;
use crate::style::Style;
use crate::svg::{SvgDocument, SvgError, svg_document_mobject};
use crate::vmobject::{VMobject, v_group};

/// drawings.py:309's tick direction: `cos(angle)·UP + sin(angle)·RIGHT`.
fn tick_direction(angle: f64) -> Vec3 {
    let cos = fmn_dmath::cos(angle);
    let sin = fmn_dmath::sin(angle);
    [
        cos * UP[0] + sin * RIGHT[0],
        cos * UP[1] + sin * RIGHT[1],
        0.0,
    ]
}

/// `Clock` (drawings.py:296): a circle, twelve tick marks (the cardinals
/// double-length), and hour/minute hands pointing up from the center.
///
/// Child order is the Reference's — `[circle, hour_hand, minute_hand,
/// ticks]` — so `ClockPassesTime` wiring can find the hands positionally.
pub struct Clock {
    vmob: VMobject,
}

impl Clock {
    /// Reference defaults: `stroke_color = WHITE`, `stroke_width = 3.0`,
    /// `hour_hand_height = 0.3`, `minute_hand_height = 0.6`,
    /// `tick_length = 0.1`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_style(WHITE, 3.0, 0.3, 0.6, 0.1)
    }

    /// The full constructor surface.
    #[must_use]
    pub fn with_style(
        stroke_color: Srgb,
        stroke_width: f64,
        hour_hand_height: f64,
        minute_hand_height: f64,
        tick_length: f64,
    ) -> Self {
        let circle = Circle::new().build();
        let mut ticks: Vec<VMobject> = Vec::new();
        for (index, step) in (0..12).enumerate() {
            let angle = f64::from(step as u32) * TAU / 12.0;
            let direction = tick_direction(angle);
            let length = if index % 3 == 0 {
                2.0 * tick_length
            } else {
                tick_length
            };
            let inner = [
                (1.0 - length) * direction[0],
                (1.0 - length) * direction[1],
                0.0,
            ];
            ticks.push(
                Line::new(direction, inner)
                    .build()
                    .expect("a straight tick never fails"),
            );
        }
        let hour_hand = Line::new(ORIGIN, [0.0, hour_hand_height, 0.0])
            .build()
            .expect("a straight hand never fails");
        let minute_hand = Line::new(ORIGIN, [0.0, minute_hand_height, 0.0])
            .build()
            .expect("a straight hand never fails");
        let family = v_group([circle, hour_hand, minute_hand, v_group(ticks)]).map_style_deep(
            move |style| {
                style
                    .stroke(stroke_color, stroke_width, 1.0)
                    .fill(stroke_color, 0.0)
            },
        );
        Self { vmob: family }
    }

    /// The built clock family.
    #[must_use]
    pub fn build(self) -> VMobject {
        self.vmob
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// `DieFace` (drawings.py:687): a rounded square with `value` pips at the
/// Reference's face positions, coalesced toward the center by
/// `dot_coalesce_factor` (the Reference's `space_out_submobjects`).
///
/// Returns `None` outside 1..=6 — the Reference's "DieFace only accepts
/// integer inputs between 1 and 6".
#[must_use]
pub fn die_face(value: u8, side_length: f64, dot_coalesce_factor: f64) -> Option<VMobject> {
    if !(1..=6).contains(&value) {
        return None;
    }
    let square = Rectangle::new()
        .width(side_length)
        .height(side_length)
        .corner_radius(0.15)
        .style(Style::default().stroke(WHITE, 2.0, 1.0).fill(GREY_E, 1.0))
        .build()
        .expect("a die face builds: pure geometry with requested corners");
    let center = square.center_point();
    let face_positions: [&[Vec3]; 6] = [
        &[ORIGIN],
        &[UL, DR],
        &[UL, ORIGIN, DR],
        &[UL, UR, DL, DR],
        &[UL, UR, ORIGIN, DL, DR],
        &[UL, UR, LEFT, RIGHT, DL, DR],
    ];
    let pips: Vec<VMobject> = face_positions[value as usize - 1]
        .iter()
        .map(|direction| {
            let anchor = square.bbox_point(*direction).unwrap_or(center);
            // move_to the face anchor, then the coalesce factor scales each
            // pip's offset from the group center (space_out_submobjects).
            Dot::new()
                .radius(0.08)
                .build()
                .moved_to(anchor)
                .scaled_about(dot_coalesce_factor, center)
        })
        .collect();
    Some(v_group([square, v_group(pips)]))
}

/// `Dartboard` (drawings.py:731): the twenty-sector ring stack — bull ring
/// in `[GREY_B, GREY_E]`, the 25 / triple / double / single rings in
/// alternating `[GREEN_E, RED_E]` — rotated half a sector, two bullseyes,
/// scaled to the class radius 3.
///
/// # Errors
/// [`fmn_geom::GeomError`] from sector construction.
pub fn dartboard() -> Result<VMobject, fmn_geom::GeomError> {
    dartboard_with_palettes(&[GREY_B, GREY_E], &[GREEN_E, RED_E])
}

/// [`dartboard`] with explicit ring palettes.
///
/// # Errors
/// [`fmn_geom::GeomError`] from sector construction.
pub fn dartboard_with_palettes(
    bull_colors: &[Srgb],
    scoring_colors: &[Srgb],
) -> Result<VMobject, fmn_geom::GeomError> {
    const N_SECTORS: usize = 20;
    let angle = TAU / f64::from(N_SECTORS as u32);
    let rings: [(f64, f64); 4] = [(0.0, 1.0), (0.5, 0.55), (0.55, 0.95), (0.95, 1.0)];
    let mut segments: Vec<VMobject> = Vec::new();
    for (ring_index, (inner, outer)) in rings.iter().enumerate() {
        let palette = if ring_index == 0 {
            bull_colors
        } else {
            scoring_colors
        };
        for n in 0..N_SECTORS {
            let color = palette[n % palette.len()];
            segments.push(
                AnnularSector::sector(angle, *outer)
                    .inner_radius(*inner)
                    .start_angle(f64::from(n as u32) * angle)
                    .color(color)
                    .build()?,
            );
        }
    }
    let mut board = v_group(segments).rotated_about(-angle / 2.0, OUT, ORIGIN);
    let bullseyes = v_group([
        Circle::new()
            .radius(0.07)
            .build()
            .map_style(|style| style.fill(GREEN_E, 1.0).stroke(GREEN_E, 0.0, 1.0)),
        Circle::new()
            .radius(0.035)
            .build()
            .map_style(|style| style.fill(RED_E, 1.0).stroke(RED_E, 0.0, 1.0)),
    ]);
    board = v_group([board, bullseyes]);
    Ok(board.scaled_about(3.0, ORIGIN))
}

// ------------------------------------------------ tranche 2 (fm-3kr):
// Speedometer, Piano, Piano3D

/// The needle pivot of a [`speedometer`] build: the arc's center — the
/// origin for the Reference defaults.
#[must_use]
pub fn speedometer_pivot() -> Vec3 {
    ORIGIN
}

/// `Speedometer` (drawings.py:111): the gauge arc, `num_ticks` tick lines
/// with `Integer(10·index)` labels, and the yellow triangular needle
/// pre-rotated to the zero-velocity position. Child order:
/// `[arc, (tick, label)×num_ticks, needle]` — the needle is LAST, so
/// needle wiring can find it positionally.
///
/// The Reference overrides `get_center` to subtract the as-constructed
/// box center (callers read positions relative to the pivot); here the
/// pivot is simply [`speedometer_pivot`] and the geometry is faithful.
///
/// # Errors
/// [`crate::text::TextMobjectError`] from the tick labels or the arc.
pub fn speedometer(book: &fmn_text::FontBook) -> Result<VMobject, crate::text::TextMobjectError> {
    speedometer_with(
        book,
        4.0 * std::f64::consts::PI / 3.0,
        8,
        0.2,
        0.1,
        0.8,
        fmn_core::color::Srgb::from_rgb8(0xFF, 0xFF, 0x00),
    )
}

/// [`speedometer`] with the full constructor surface.
///
/// # Errors
/// As [`speedometer`].
pub fn speedometer_with(
    book: &fmn_text::FontBook,
    arc_angle: f64,
    num_ticks: usize,
    tick_length: f64,
    needle_width: f64,
    needle_height: f64,
    needle_color: Srgb,
) -> Result<VMobject, crate::text::TextMobjectError> {
    let start_angle = std::f64::consts::FRAC_PI_2 + arc_angle / 2.0;
    let end_angle = std::f64::consts::FRAC_PI_2 - arc_angle / 2.0;
    let arc = crate::arc::Arc::new()
        .start_angle(start_angle)
        .angle(-arc_angle)
        .build()
        .map_err(|e| crate::text::TextMobjectError::Geometry {
            what: format!("speedometer arc: {e}"),
        })?;
    let mut members: Vec<VMobject> = Vec::with_capacity(1 + 2 * num_ticks + 1);
    members.push(arc);
    for index in 0..num_ticks {
        let angle = start_angle
            + (end_angle - start_angle) * f64::from(index as u32)
                / f64::from(num_ticks.saturating_sub(1) as u32).max(1.0);
        let vect = [fmn_dmath::cos(angle), fmn_dmath::sin(angle), 0.0];
        let inner = [
            (1.0 - tick_length) * vect[0],
            (1.0 - tick_length) * vect[1],
            0.0,
        ];
        members.push(
            Line::new(vect, inner)
                .build()
                .expect("a straight tick never fails"),
        );
        let label = crate::numbers::Integer::new(f64::from(index as u32) * 10.0)
            .build(book)?
            .into_vmob()
            .with_height(tick_length, true)
            .moved_to([
                (1.0 + tick_length) * vect[0],
                (1.0 + tick_length) * vect[1],
                0.0,
            ]);
        members.push(label);
    }
    // The needle: the LEFT/UP/RIGHT triangle fitted to the needle box,
    // rotated so it points along the zero-velocity start angle.
    let needle = crate::poly::Polygon::new([LEFT, UP, RIGHT])
        .style(
            Style::default()
                .fill(needle_color, 1.0)
                .stroke(needle_color, 0.0, 1.0),
        )
        .build();
    let (w, h) = needle.extent().map_or((1.0, 1.0), |(min, max)| {
        ((max[0] - min[0]).max(1e-9), (max[1] - min[1]).max(1e-9))
    });
    let needle = needle
        .scaled_about(needle_width / w, ORIGIN)
        .scaled_about(needle_height / h, ORIGIN)
        .rotated_about(start_angle - std::f64::consts::FRAC_PI_2, OUT, ORIGIN);
    members.push(needle);
    Ok(v_group(members))
}

/// `move_needle_to_velocity` (drawings.py:180): rotate the needle child of
/// a [`speedometer`] build to the velocity proportion of the gauge
/// (`max_velocity = 10 × (num_ticks − 1)`), about the pivot.
///
/// # Errors
/// [`fmn_mobject::StageError::StaleHandle`] for a dead needle.
pub fn move_needle_to_velocity(
    stage: &mut Stage,
    needle: Mob,
    arc_angle: f64,
    num_ticks: usize,
    velocity: f64,
) -> Result<(), fmn_mobject::StageError> {
    let max_velocity = 10.0 * f64::from(num_ticks.saturating_sub(1) as u32).max(1.0);
    let proportion = (velocity / max_velocity).clamp(0.0, 1.0);
    let start_angle = std::f64::consts::FRAC_PI_2 + arc_angle / 2.0;
    let target = start_angle - arc_angle * proportion;
    let pivot = speedometer_pivot();
    let tip = stage.get_center(needle);
    let current = fmn_dmath::atan2(tip[1] - pivot[1], tip[0] - pivot[0]);
    stage.rotate(needle, target - current, OUT, Some(pivot), None);
    Ok(())
}

/// A built [`piano`]: the keyboard plus which final child indices are the
/// black keys (the Piano3D elevation needs them).
#[derive(Debug, Clone)]
pub struct PianoBuild {
    /// The keyboard, fitted to the total width.
    pub vmob: VMobject,
    /// Child indices of the black keys, in child order.
    pub black_key_indices: Vec<usize>,
}

/// Keep the group and its white/black keys within the 65,536-member
/// family budget, including a pattern that places a black key in every gap.
pub const MAX_PIANO_WHITE_KEYS: usize = 32_768;

/// Invalid keyboard dimensions or a failure to cut the neighboring keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PianoError {
    /// A constructor argument cannot describe a finite keyboard.
    InvalidParameter(&'static str),
    /// Native key geometry or boolean notching failed.
    Geometry(BooleanMobjectError),
}

impl fmt::Display for PianoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameter(name) => write!(f, "invalid Piano {name}"),
            Self::Geometry(error) => write!(f, "Piano key geometry: {error}"),
        }
    }
}

impl std::error::Error for PianoError {}

impl From<BooleanMobjectError> for PianoError {
    fn from(error: BooleanMobjectError) -> Self {
        Self::Geometry(error)
    }
}

impl From<fmn_geom::GeomError> for PianoError {
    fn from(error: fmn_geom::GeomError) -> Self {
        Self::Geometry(error.into())
    }
}

/// `Piano` (drawings.py:593): `n_white_keys` white keys, black keys at the
/// octave positions included in the pattern, each neighboring white key notched
/// by a boolean difference against the enlarged black key — the
/// Reference's `wk.become(Difference(wk, big_bk))` running on the
/// certified Chisel kernel. Keys sort by x, all but the last
/// point-reverse, and the keyboard fits to `total_width`.
///
/// # Errors
/// [`PianoError`] from the boolean notching or the key geometry.
pub fn piano() -> Result<VMobject, PianoError> {
    Ok(piano_with(
        52,
        &[0, 2, 3, 5, 6],
        7,
        (0.15, 1.0),
        (0.1, 0.66),
        0.02,
        WHITE,
        GREY_E,
        13.0,
    )?
    .vmob)
}

/// [`piano`] with the full constructor surface, returning the black-key
/// child indices for the Piano3D elevation.
///
/// # Errors
/// [`PianoError::InvalidParameter`] for zero key counts, a count above
/// [`MAX_PIANO_WHITE_KEYS`], nonpositive or non-finite dimensions/total
/// width, a negative/non-finite gap, or a cutter that erases a white key.
/// [`PianoError::Geometry`] from the boolean notching or the key geometry.
#[allow(clippy::too_many_arguments)]
pub fn piano_with(
    n_white_keys: usize,
    black_pattern: &[usize],
    white_keys_per_octave: usize,
    white_key_dims: (f64, f64),
    black_key_dims: (f64, f64),
    key_buff: f64,
    white_key_color: Srgb,
    black_key_color: Srgb,
    total_width: f64,
) -> Result<PianoBuild, PianoError> {
    if n_white_keys == 0 {
        return Err(PianoError::InvalidParameter(
            "n_white_keys: must be positive",
        ));
    }
    if n_white_keys > MAX_PIANO_WHITE_KEYS {
        return Err(PianoError::InvalidParameter(
            "n_white_keys: family budget exceeded",
        ));
    }
    if white_keys_per_octave == 0 {
        return Err(PianoError::InvalidParameter(
            "white_keys_per_octave: must be positive",
        ));
    }
    for (name, value) in [
        ("white_key_dims width", white_key_dims.0),
        ("white_key_dims height", white_key_dims.1),
        ("black_key_dims width", black_key_dims.0),
        ("black_key_dims height", black_key_dims.1),
        ("total_width", total_width),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(PianoError::InvalidParameter(name));
        }
    }
    if !key_buff.is_finite() || key_buff < 0.0 {
        return Err(PianoError::InvalidParameter(
            "key_buff: must be finite and nonnegative",
        ));
    }
    let keyboard_width = f64::from(n_white_keys as u32) * white_key_dims.0
        + f64::from((n_white_keys - 1) as u32) * key_buff;
    if !keyboard_width.is_finite()
        || !(white_key_dims.0 + key_buff).is_finite()
        || !(black_key_dims.0 + key_buff).is_finite()
        || !(black_key_dims.1 + key_buff).is_finite()
    {
        return Err(PianoError::InvalidParameter(
            "key dimensions overflow the keyboard extent",
        ));
    }
    let white_style = Style::default()
        .fill(white_key_color, 1.0)
        .stroke(white_key_color, 0.0, 1.0);
    let black_style = Style::default()
        .fill(black_key_color, 1.0)
        .stroke(black_key_color, 0.0, 1.0);
    let (ww, wh) = white_key_dims;
    let (bw, bh) = black_key_dims;
    let white_key = crate::poly::Rectangle::new()
        .width(ww)
        .height(wh)
        .style(white_style)
        .build()?;
    let black_key = crate::poly::Rectangle::new()
        .width(bw)
        .height(bh)
        .style(black_style)
        .build()?;
    let mut white_keys: Vec<VMobject> = Vec::with_capacity(n_white_keys);
    for i in 0..n_white_keys {
        let x = f64::from(i as u32) * (ww + key_buff);
        white_keys.push(white_key.clone().moved_to([x + ww / 2.0, 0.0, 0.0]));
    }
    let mut black_keys: Vec<VMobject> = Vec::new();
    for i in 0..n_white_keys.saturating_sub(1) {
        if !black_pattern.contains(&(i % white_keys_per_octave)) {
            continue;
        }
        let top1 = white_keys[i]
            .bbox_point(UP)
            .expect("a white key has an extent");
        let top2 = white_keys[i + 1]
            .bbox_point(UP)
            .expect("a white key has an extent");
        let midpoint = [0.5 * (top1[0] + top2[0]), 0.5 * (top1[1] + top2[1]), 0.0];
        let black = black_key.clone().moved_to_aligned(midpoint, UP);
        // The enlarged cutter, then the notch (Difference(wk, big_bk)).
        let (gw, gh) = black
            .extent()
            .map_or((bw, bh), |(min, max)| (max[0] - min[0], max[1] - min[1]));
        let big_black = black
            .clone()
            .stretched_about((gw + key_buff) / gw, 0, ORIGIN)
            .stretched_about((gh + key_buff) / gh, 1, ORIGIN)
            .moved_to_aligned(midpoint, UP);
        for neighbor in &mut white_keys[i..=i + 1] {
            let notched = crate::boolean_ops::difference(neighbor, &big_black)?.into_mobject();
            if notched.points().is_empty() {
                return Err(PianoError::InvalidParameter(
                    "black_key_dims erase a white key",
                ));
            }
            *neighbor = notched.map_style(move |_| white_style);
        }
        black_keys.push(black);
    }
    // sort_keys: one x-sorted child list, tracking which are black.
    let mut all: Vec<(VMobject, bool)> = white_keys
        .into_iter()
        .map(|key| (key, false))
        .chain(black_keys.into_iter().map(|key| (key, true)))
        .collect();
    all.sort_by(|a, b| {
        a.0.center_point()[0]
            .partial_cmp(&b.0.center_point()[0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // self[:-1].reverse_points(): every key except the LAST in x order.
    let last = all.len().saturating_sub(1);
    let mut keyboard: Vec<VMobject> = Vec::with_capacity(all.len());
    let mut black_key_indices: Vec<usize> = Vec::new();
    for (index, (key, is_black)) in all.into_iter().enumerate() {
        if is_black {
            black_key_indices.push(index);
        }
        keyboard.push(if index == last {
            key
        } else {
            key.reversed_points()
        });
    }
    let keyboard = v_group(keyboard).moved_to(ORIGIN);
    let scale = total_width / keyboard.length_over_dim(0);
    if !scale.is_finite() || scale <= 0.0 || !(keyboard.length_over_dim(1) * scale).is_finite() {
        return Err(PianoError::InvalidParameter(
            "total_width: scaled keyboard must remain finite",
        ));
    }
    Ok(PianoBuild {
        vmob: keyboard.with_width(total_width, false),
        black_key_indices,
    })
}

/// `Piano3D` (drawings.py:657): every 2D key extruded through
/// [`crate::solids::Prismify`], stroked black, shaded `(1.0, 0.2, 0.2)`,
/// depth-tested, with the black keys elevated along OUT and recolored.
///
/// # Errors
/// [`PianoError`] from the 2D piano's key geometry or notching.
pub fn piano_3d() -> Result<VMobject, PianoError> {
    let build = piano_with(
        52,
        &[0, 2, 3, 5, 6],
        7,
        (0.15, 1.0),
        (0.1, 0.66),
        0.001,
        fmn_core::constants::GREY_A,
        GREY_E,
        13.0,
    )?;
    let shade = |mut uniforms: fmn_mobject::Uniforms| {
        uniforms.shading = [1.0, 0.2, 0.2];
        uniforms.depth_test = true;
        uniforms
    };
    let black: std::collections::HashSet<usize> = build.black_key_indices.iter().copied().collect();
    let keys: Vec<VMobject> = build
        .vmob
        .children()
        .iter()
        .enumerate()
        .map(|(index, key)| {
            // Prismify's visible geometry lives in its flat face children;
            // changing only a key or piano group leaves their lighting stale.
            let mut extruded = crate::solids::Prismify::new(key.clone())
                .depth(0.1)
                .build()
                .map_children(|face| face.map_uniforms(shade))
                .map_uniforms(shade);
            if black.contains(&index) {
                extruded = extruded
                    .shifted(fmn_core::constants::OUT.map(|c| c * 0.05))
                    .map_style_deep(|style| style.color(BLACK));
            }
            extruded
        })
        .collect();
    Ok(v_group(keys)
        .map_style_deep(|style| style.stroke(BLACK, 0.25, 1.0))
        .map_uniforms(shade))
}

// ------------------------------------------------ tranche 3 (fm-3kr): the
// asset-backed families under ADR-0020

/// Why an asset-backed drawings family could not be built.
///
/// ADR-0020: the Reference-derived SVG assets (`lightbulb`, `video_icon`,
/// the Bubble bodies, `earth`) are CC BY-NC-SA video-content derivatives and
/// never ship. Every family therefore resolves its body through
/// [`resolve_drawings_svg`], which either reads a user-supplied file from an
/// explicit root or refuses with [`DrawingsAssetError::AssetNotShipped`] —
/// naming the class, the missing file, and the remedy. No silent
/// substitution, no placeholder art.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawingsAssetError {
    /// The default asset is not part of any shipped artifact. Supply your
    /// own legally obtained copy of `asset` through a drawings asset root.
    AssetNotShipped {
        /// The drawings class whose default body was requested.
        class: &'static str,
        /// The Reference's `file_name` for that class's body.
        asset: &'static str,
    },
    /// The user-supplied root could not be read as a regular file there.
    UnreadableAsset {
        /// The joined path attempted.
        path: String,
        /// The underlying refusal, verbatim.
        reason: String,
    },
    /// The user-supplied SVG refused Chisel's hardened document processor;
    /// every [`SvgError`] variant is already a named refusal.
    Svg(SvgError),
    /// A geometry primitive behind a composite family refused construction.
    Geometry(String),
}

impl fmt::Display for DrawingsAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetNotShipped { class, asset } => write!(
                f,
                "{class} does not bundle '{asset}': Reference-derived drawings \
                 assets never ship (CC BY-NC-SA, docs/adr/0020); point \
                 `root` at your own copy of the 3b1b asset"
            ),
            Self::UnreadableAsset { path, reason } => {
                write!(f, "drawings asset root cannot supply {path:?}: {reason}")
            }
            Self::Svg(error) => write!(f, "drawings asset SVG refused: {error}"),
            Self::Geometry(detail) => {
                write!(f, "drawings family geometry refused: {detail}")
            }
        }
    }
}

impl std::error::Error for DrawingsAssetError {}

impl From<SvgError> for DrawingsAssetError {
    fn from(error: SvgError) -> Self {
        Self::Svg(error)
    }
}

/// The one policy chokepoint every asset-backed family resolves through:
/// `root` absent means [`DrawingsAssetError::AssetNotShipped`]; `root`
/// present means read `<root>/<asset>[.svg]` as a regular file and parse it
/// under the hardened document processor's default budgets.
///
/// # Errors
/// [`DrawingsAssetError`] per the policy above.
pub fn resolve_drawings_svg(
    class: &'static str,
    asset: &'static str,
    root: Option<&std::path::Path>,
) -> Result<SvgDocument, DrawingsAssetError> {
    let Some(root) = root else {
        return Err(DrawingsAssetError::AssetNotShipped { class, asset });
    };
    let file = if asset.ends_with(".svg") {
        asset.to_owned()
    } else {
        format!("{asset}.svg")
    };
    let path = root.join(file);
    let metadata =
        std::fs::metadata(&path).map_err(|error| DrawingsAssetError::UnreadableAsset {
            path: path.display().to_string(),
            reason: error.to_string(),
        })?;
    if !metadata.is_file() {
        return Err(DrawingsAssetError::UnreadableAsset {
            path: path.display().to_string(),
            reason: "not a regular file".to_owned(),
        });
    }
    let bytes = std::fs::read(&path).map_err(|error| DrawingsAssetError::UnreadableAsset {
        path: path.display().to_string(),
        reason: error.to_string(),
    })?;
    Ok(SvgDocument::parse(&bytes)?)
}

/// Family width/height from the corner points along the axes.
fn family_extents(vmob: &VMobject) -> (f64, f64) {
    let right = vmob.bbox_point(RIGHT);
    let left = vmob.bbox_point(LEFT);
    let up = vmob.bbox_point(UP);
    let down = vmob.bbox_point(DOWN);
    (
        right.map_or(0.0, |p| p[0]) - left.map_or(0.0, |p| p[0]),
        up.map_or(0.0, |p| p[1]) - down.map_or(0.0, |p| p[1]),
    )
}

/// Mirror a family across its own vertical centre line (the horizontal
/// `flip()` of the Reference; no native flip exists — this is a −1 stretch
/// about the family centre).
fn mirrored_horizontally(vmob: VMobject) -> VMobject {
    let center = vmob.center_point();
    vmob.stretched_about(-1.0, 0, center)
}

/// `Lightbulb`'s body over a user-supplied document (ADR-0020): styled to
/// the Reference defaults — yellow outline, no fill — then sized to
/// `height`.
///
/// Behavior Note: the Reference's `insert_n_curves(25)` refines its fixed
/// 8-curve-per-path import; here curve density follows Chisel's adaptive
/// tolerance at import time, so no refinement pass exists.
#[must_use]
pub fn lightbulb_from_document(document: &SvgDocument) -> VMobject {
    svg_document_mobject(document)
        .map_style_deep(|style| style.fill(YELLOW, 0.0).stroke(YELLOW, 3.0, 1.0))
        .with_height(1.0, false)
}

/// [`Lightbulb`](lightbulb_from_document) resolving its body through
/// [`resolve_drawings_svg`] (`asset = "lightbulb"`).
///
/// # Errors
/// [`DrawingsAssetError`] — by policy when `root` is [`None`].
pub fn lightbulb(root: Option<&std::path::Path>) -> Result<VMobject, DrawingsAssetError> {
    let document = resolve_drawings_svg("Lightbulb", "lightbulb", root)?;
    Ok(lightbulb_from_document(&document))
}

/// `VideoIcon`'s body over a user-supplied document: recolored to the
/// Reference default `BLUE_A` (stroke and fill together, widths preserved)
/// then sized to `width`.
#[must_use]
pub fn video_icon_from_document(document: &SvgDocument) -> VMobject {
    svg_document_mobject(document)
        .map_style_deep(|style| style.color(BLUE_A))
        .with_width(1.2, false)
}

/// [`VideoIcon`](video_icon_from_document) resolving its body through
/// [`resolve_drawings_svg`] (`asset = "video_icon"`).
///
/// # Errors
/// [`DrawingsAssetError`] — by policy when `root` is [`None`].
pub fn video_icon(root: Option<&std::path::Path>) -> Result<VMobject, DrawingsAssetError> {
    let document = resolve_drawings_svg("VideoIcon", "video_icon", root)?;
    Ok(video_icon_from_document(&document))
}

/// `VideoSeries` over a user-supplied VideoIcon document: `num_videos`
/// icons arranged `RIGHT`, set to `width`, then colored along
/// `gradient_colors` exactly as the Reference orders the three steps.
#[must_use]
pub fn video_series_from_document(
    document: &SvgDocument,
    num_videos: usize,
    gradient_colors: &[Srgb],
    width: f64,
) -> VMobject {
    let series = VMobject::arranged(
        std::iter::repeat_with(|| video_icon_from_document(document)).take(num_videos),
        RIGHT,
        MED_SMALL_BUFF,
        ORIGIN,
    )
    .with_width(width, false);
    let colors = color_gradient(gradient_colors, num_videos);
    v_group(
        series
            .children()
            .iter()
            .cloned()
            .zip(colors)
            .map(|(child, color)| child.map_style_deep(move |style| style.color(color))),
    )
}

/// [`VideoSeries`](video_series_from_document) resolving its icon body
/// through [`resolve_drawings_svg`] (`asset = "video_icon"`), with the
/// Reference defaults: eleven videos in a `BLUE_B → BLUE_D` gradient across
/// the frame width minus `MED_LARGE_BUFF`.
///
/// # Errors
/// [`DrawingsAssetError`] — by policy when `root` is [`None`].
pub fn video_series(root: Option<&std::path::Path>) -> Result<VMobject, DrawingsAssetError> {
    let document = resolve_drawings_svg("VideoIcon", "video_icon", root)?;
    Ok(video_series_from_document(
        &document,
        11,
        &[BLUE_B, BLUE_D],
        FRAME_WIDTH - MED_LARGE_BUFF,
    ))
}

/// `Bubble.bubble_center_adjustment_factor` (drawings.py:356).
const BUBBLE_CENTER_ADJUSTMENT_FACTOR: f64 = 0.125;

/// `Bubble.get_body` over a user-supplied document: mirror when `direction`
/// points rightward, stretch to `(content width + min(buff, content
/// height), 1.35 × (content height + buff))`, move onto the content, then
/// drop by the adjustment factor of the *resized* height — the Reference's
/// exact order.
#[must_use]
fn bubble_body_from_document(
    document: &SvgDocument,
    content: &VMobject,
    direction: Vec3,
    buff: f64,
) -> VMobject {
    let mut body = svg_document_mobject(document);
    if direction[0] > 0.0 {
        body = mirrored_horizontally(body);
    }
    let (content_width, content_height) = family_extents(content);
    let target_width = content_width + buff.min(content_height);
    let target_height = 1.35 * (content_height + buff);
    body.with_width(target_width, true)
        .with_height(target_height, true)
        .moved_to(content.center_point())
        .shifted([0.0, -BUBBLE_CENTER_ADJUSTMENT_FACTOR * target_height, 0.0])
        .map_style_deep(|style| style.fill(BLACK, 0.8).stroke(WHITE, 3.0, 1.0))
}

/// One speech/thought bubble over a user-supplied body document. `content`
/// of [`None`] uses the Reference's invisible `Rectangle(3, 2)` filler;
/// otherwise the given mobject is cloned into the group after the body,
/// the Reference's child order.
///
/// # Errors
/// [`DrawingsAssetError::Geometry`] if the filler rectangle refuses.
pub fn bubble_with(
    body_document: &SvgDocument,
    content: Option<&VMobject>,
    direction: Vec3,
    buff: f64,
) -> Result<VMobject, DrawingsAssetError> {
    let owned_content;
    let content = match content {
        Some(given) => given,
        None => {
            owned_content = Rectangle::new()
                .width(3.0)
                .height(2.0)
                .build()
                .map_err(|error| DrawingsAssetError::Geometry(error.to_string()))?
                .map_style_deep(|style| style.fill(BLACK, 0.0).stroke(BLACK, 0.0, 1.0));
            &owned_content
        }
    };
    let body = bubble_body_from_document(body_document, content, direction, buff);
    Ok(v_group([body, content.clone()]))
}

macro_rules! drawings_bubble_family {
    ($(#[$doc:meta])* $name:ident, $class:literal, $asset:literal) => {
        $(#[$doc])*
        ///
        /// # Errors
        /// [`DrawingsAssetError`] — by policy when `root` is [`None`].
        pub fn $name(
            root: Option<&std::path::Path>,
            content: Option<&VMobject>,
            direction: Vec3,
            buff: f64,
        ) -> Result<VMobject, DrawingsAssetError> {
            let document = resolve_drawings_svg($class, $asset, root)?;
            bubble_with(&document, content, direction, buff)
        }
    };
}

drawings_bubble_family!(
    /// `OldSpeechBubble` (`asset = "Bubbles_speech.svg"`).
    old_speech_bubble,
    "OldSpeechBubble",
    "Bubbles_speech.svg"
);
drawings_bubble_family!(
    /// `DoubleSpeechBubble` (`asset = "Bubbles_double_speech.svg"`).
    double_speech_bubble,
    "DoubleSpeechBubble",
    "Bubbles_double_speech.svg"
);

/// `OldThoughtBubble.get_body`'s extra step: the mirrored/stretched body's
/// shapes sorted by ascending y before the group forms.
///
/// # Errors
/// [`DrawingsAssetError`] — by policy when `root` is [`None`].
pub fn old_thought_bubble(
    root: Option<&std::path::Path>,
    content: Option<&VMobject>,
    direction: Vec3,
    buff: f64,
) -> Result<VMobject, DrawingsAssetError> {
    let document = resolve_drawings_svg("OldThoughtBubble", "Bubbles_thought.svg", root)?;
    let owned_content;
    let content = match content {
        Some(given) => given,
        None => {
            owned_content = Rectangle::new()
                .width(3.0)
                .height(2.0)
                .build()
                .map_err(|error| DrawingsAssetError::Geometry(error.to_string()))?
                .map_style_deep(|style| style.fill(BLACK, 0.0).stroke(BLACK, 0.0, 1.0));
            &owned_content
        }
    };
    let mut body = bubble_body_from_document(&document, content, direction, buff);
    let mut ordered: Vec<VMobject> = body.children().to_vec();
    ordered.sort_by(|a, b| a.center_point()[1].total_cmp(&b.center_point()[1]));
    // `map_children` is the only child-replacing hook; drain the sorted
    // clones through it one-for-one.
    let mut supply = ordered.into_iter();
    body = body.map_children(|_| supply.next().expect("one replacement per child"));
    Ok(v_group([body, content.clone()]))
}

/// `OldThoughtBubble.make_green_screen`: fill the thought cloud's last
/// shape (the body's highest child after the y-sort) solid
/// [`GREEN_SCREEN`], returning the reassembled bubble.
#[must_use]
pub fn bubble_make_green_screen(bubble: &VMobject) -> VMobject {
    v_group(bubble.children().iter().enumerate().map(|(index, child)| {
        if index != 0 {
            return child.clone();
        }
        let last = child.children().len().saturating_sub(1);
        let mut position = 0usize;
        child.clone().map_children(move |shape| {
            let is_last = position == last;
            position += 1;
            if is_last {
                shape.map_style_deep(|style| style.fill(GREEN_SCREEN, 1.0))
            } else {
                shape
            }
        })
    }))
}

/// `VectorizedEarth` over a user-supplied document: the imported globe at
/// `height`, backed by a green-outlined, blue-filled circle stretched to
/// the family's extents and placed behind it (the Reference's
/// `circle.replace(self)` + `add_to_back`).
///
/// Behavior Note: as with [`lightbulb_from_document`], the Reference's
/// `insert_n_curves(20)` has no counterpart — import density is adaptive.
///
/// # Errors
/// [`DrawingsAssetError::Geometry`] if the backing circle refuses.
pub fn vectorized_earth_from_document(
    document: &SvgDocument,
) -> Result<VMobject, DrawingsAssetError> {
    let globe = svg_document_mobject(document).with_height(2.0, false);
    let center = globe.center_point();
    let (half_width, half_height) = {
        let (width, height) = family_extents(&globe);
        (width / 2.0, height / 2.0)
    };
    let backdrop = Circle::new()
        .radius(1.0)
        .arc_center(ORIGIN)
        .style(Style::default().fill(BLUE_C, 1.0).stroke(GREEN, 3.0, 1.0))
        .build()
        .stretched_about(half_width.max(half_height), 0, ORIGIN)
        .stretched_about(half_width.max(half_height), 1, ORIGIN)
        .moved_to(center);
    Ok(v_group([backdrop, globe]))
}

/// [`VectorizedEarth`](vectorized_earth_from_document) resolving its body
/// through [`resolve_drawings_svg`] (`asset = "earth"`), Reference-default
/// `height = 2.0` applied inside.
///
/// # Errors
/// [`DrawingsAssetError`] — by policy when `root` is [`None`].
pub fn vectorized_earth(root: Option<&std::path::Path>) -> Result<VMobject, DrawingsAssetError> {
    let document = resolve_drawings_svg("VectorizedEarth", "earth", root)?;
    vectorized_earth_from_document(&document)
}

/// `Laptop` (drawings.py:163) — the last geometry-native census class: a
/// `VCube` body stretched to `body_dimensions` and fitted to `width`, a
/// keyboard of staggered black squares on the OUT face, a screen plate
/// (the Reference copies the keyboard-bearing body, squashes its depth to
/// `screen_thickness`, and mounts a black screen rectangle at
/// `screen_width_to_screen_plate_width`), hinged `open_angle` about the
/// plate's bottom edge, and the black hinge axis across the body's top
/// OUT corners.
///
/// Top-level child order — `[body, screen_plate, axis]` — with the screen
/// as the plate's last child, matching the Reference's attribute surface.
///
/// The screen-plate copy faithfully includes the keyboard (the Reference's
/// `screen_plate = body.copy()` runs after `body.add(keyboard)`); its
/// depth squash therefore carries the keyboard's — replicated, not
/// corrected, because the sequence is the Reference's observable
/// construction and appears in no defect register.
#[must_use]
pub fn laptop() -> VMobject {
    let body_dimensions = [4.0, 3.0, 0.05];
    let screen_thickness = 0.01;
    let keyboard_width_to_body_width = 0.9;
    let keyboard_height_to_body_height = 0.5;
    let screen_width_to_screen_plate_width = 0.9;
    let open_angle = std::f64::consts::FRAC_PI_4;
    let key_style = Style::default().fill(BLACK, 1.0).stroke(BLACK, 0.0, 1.0);

    // Body: VCube(1) stretched per dimension, fitted to width 3, shaded,
    // faces sorted by z with the +z face refilled to the body color.
    let mut body = crate::solids::VCube::new(1.0).build();
    for (dim, factor) in body_dimensions.iter().enumerate() {
        let center = body.center_point();
        body = body.stretched_about(*factor, dim, center);
    }
    body = body.with_width(3.0, true);
    body = body.map_style_deep(|style| style.fill(fmn_core::constants::GREY, 1.0));
    let mut faces: Vec<VMobject> = body.children().to_vec();
    faces.sort_by(|a, b| {
        a.center_point()[2]
            .partial_cmp(&b.center_point()[2])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(top) = faces.last_mut() {
        *top = top
            .clone()
            .map_style(|style| style.fill(fmn_core::constants::GREY_B, 1.0));
    }
    let body_width = body.length_over_dim(0);
    let body_height = body.length_over_dim(1);
    let body = v_group(faces);

    // Keyboard: staggered rows of black squares on the OUT face.
    let mut rows: Vec<VMobject> = Vec::new();
    for y in 0..4 {
        let keys_in_row = 12 - y % 2;
        let row: Vec<VMobject> = (0..keys_in_row)
            .map(|_| crate::poly::Square::new().style(key_style).build())
            .collect();
        rows.push(VMobject::arranged(
            row,
            RIGHT,
            fmn_core::constants::SMALL_BUFF,
            UP,
        ));
    }
    let keyboard = VMobject::arranged(rows, DOWN, fmn_core::constants::MED_SMALL_BUFF, UP)
        .with_width(keyboard_width_to_body_width * body_width, true)
        .with_height(keyboard_height_to_body_height * body_height, true)
        .moved_to([
            body.center_point()[0],
            body.center_point()[1],
            body.center_point()[2]
                + body_dimensions[2] * 0.5
                + 0.1 * fmn_core::constants::SMALL_BUFF,
        ])
        .shifted([0.0, fmn_core::constants::MED_SMALL_BUFF, 0.0]);
    let body = body.with_child(keyboard);

    // Screen plate: the keyboard-bearing body copied, depth squashed to the
    // screen thickness, with the black screen rectangle mounted on OUT.
    let mut screen_plate = body.clone();
    let squash = screen_thickness / body_dimensions[2];
    let plate_center = screen_plate.center_point();
    screen_plate = screen_plate.stretched_about(squash, 2, plate_center);
    let (pw, ph) = screen_plate
        .extent()
        .map_or((1.0, 1.0), |(min, max)| (max[0] - min[0], max[1] - min[1]));
    let plate_center = screen_plate.center_point();
    let screen = crate::poly::Rectangle::new()
        .style(Style::default().fill(BLACK, 1.0).stroke(BLACK, 0.0, 1.0))
        .build()
        .expect("a screen rectangle builds: pure geometry")
        .with_width(pw, true)
        .with_height(ph, true)
        .moved_to(plate_center)
        .scaled_about(screen_width_to_screen_plate_width, plate_center)
        .shifted([
            0.0,
            0.0,
            0.1 * fmn_core::constants::SMALL_BUFF + squash * body_dimensions[2] * 0.5,
        ]);
    let screen_bottom = screen.extent().map_or(0.0, |(min, _)| min[1]);
    let screen_plate = screen_plate
        .with_child(screen)
        .moved_to([
            body.center_point()[0],
            body.center_point()[1] + body_height * 0.5,
            body.center_point()[2],
        ])
        .rotated_about(open_angle, RIGHT, [0.0, screen_bottom, 0.0]);

    // Hinge axis across the body's top OUT corners.
    // Compile-repair (GrayForest, fm-n64 session): `bbox_point` yields an
    // Option; the hinge corners of a built body always exist.
    let axis = {
        let ul = body
            .bbox_point([UP[0] + LEFT[0], UP[1] + LEFT[1], OUT[2]])
            .expect("body UL corner exists");
        let ur = body
            .bbox_point([UP[0] + RIGHT[0], UP[1] + RIGHT[1], OUT[2]])
            .expect("body UR corner exists");
        Line::new(ul, ur)
    }
    .style(Style::default().stroke(BLACK, 2.0, 1.0))
    .build()
    .expect("a straight axis never fails");

    v_group([body, screen_plate, axis])
}

// ------------------------------------------------- geometry-native bubbles

/// The `SpeechBubble` / `ThoughtBubble` filler: an invisible
/// `Rectangle(*filler_shape)` the body surrounds when no content is given.
fn bubble_filler(filler_shape: (f64, f64)) -> Result<VMobject, BooleanMobjectError> {
    Ok(Rectangle::new()
        .width(filler_shape.0)
        .height(filler_shape.1)
        .build()?
        .map_style_deep(|style| style.fill(BLACK, 0.0).stroke(BLACK, 0.0, 1.0)))
}

/// The Reference's `Bubble.__init__` tail: the body takes `fill BLACK 0.8`
/// and `stroke WHITE 3.0` across its family, then the group is
/// `VGroup(body, content)`.
fn bubble_group(body: VMobject, content: VMobject) -> VMobject {
    let body = body.map_style_deep(|style| style.fill(BLACK, 0.8).stroke(WHITE, 3.0, 1.0));
    v_group([body, content])
}

/// Content extent, the surrounding rectangle's corners in the Reference
/// `Rectangle` vertex order (UR, UL, DL, DR), `buff` clear on every side.
fn surrounding_corners(content: &VMobject, buff: f64) -> [Vec3; 4] {
    let (min, max) = content.extent().unwrap_or(([0.0; 3], [0.0; 3]));
    let (x0, x1) = (min[0] - buff, max[0] + buff);
    let (y0, y1) = (min[1] - buff, max[1] + buff);
    let z = 0.5 * (min[2] + max[2]);
    [[x1, y1, z], [x0, y1, z], [x0, y0, z], [x1, y0, z]]
}

fn lerp3(a: Vec3, b: Vec3, alpha: f64) -> Vec3 {
    [
        a[0] + alpha * (b[0] - a[0]),
        a[1] + alpha * (b[1] - a[1]),
        a[2] + alpha * (b[2] - a[2]),
    ]
}

/// `SpeechBubble` (drawings.py:463), geometry-only — no asset involved.
///
/// The body is the content's `SurroundingRectangle(buff)` with
/// `round_corners()` (a quarter of the shortest edge), unioned with a stem
/// triangle whose top spans `stem_top_x_props` of the bottom edge and whose
/// tip hangs `stem_height_to_bubble_height` of the bubble's height below the
/// lower-left corner; a rightward `direction` mirrors it. `content` of
/// [`None`] surrounds the invisible `filler_shape` rectangle. Reference
/// defaults: `buff = MED_SMALL_BUFF`, `filler_shape = (2, 1)`,
/// `stem_height_to_bubble_height = 0.5`, `stem_top_x_props = (0.2, 0.3)`,
/// `direction = LEFT`.
///
/// Behavior Note: the Reference's `insert_n_curves(20)` only densifies the
/// union for later morphing; curve density here is the boolean kernel's.
///
/// # Errors
/// [`BooleanMobjectError`] from the rounded rectangle or the union.
pub fn speech_bubble(
    content: Option<&VMobject>,
    direction: Vec3,
    buff: f64,
    filler_shape: (f64, f64),
    stem_height_to_bubble_height: f64,
    stem_top_x_props: (f64, f64),
) -> Result<VMobject, BooleanMobjectError> {
    let content = match content {
        Some(given) => given.clone(),
        None => bubble_filler(filler_shape)?,
    };
    let corners = surrounding_corners(&content, buff);
    let rect = crate::poly::Polygon::new(corners).round_corners(None)?;
    let (lp, rp) = (corners[2], corners[3]);
    let stem_height = stem_height_to_bubble_height * (corners[0][1] - corners[3][1]);
    let triangle = crate::poly::Polygon::new([
        lerp3(lp, rp, stem_top_x_props.0),
        lerp3(lp, rp, stem_top_x_props.1),
        [lp[0], lp[1] - stem_height, lp[2]],
    ])
    .build();
    let mut body = crate::boolean_ops::union(&[rect, triangle])?.into_mobject();
    if direction[0] > 0.0 {
        body = mirrored_horizontally(body);
    }
    Ok(bubble_group(body, content))
}

/// [`speech_bubble`] with every Reference default, around `content`.
///
/// # Errors
/// As [`speech_bubble`].
pub fn speech_bubble_default(content: Option<&VMobject>) -> Result<VMobject, BooleanMobjectError> {
    speech_bubble(content, LEFT, MED_SMALL_BUFF, (2.0, 1.0), 0.5, (0.2, 0.3))
}

/// The `ThoughtBubble` knobs beyond the shared bubble surface, with the
/// Reference's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct ThoughtBubbleShape {
    /// `bulge_radius` — the cloud's circle radius (`0.35`).
    pub bulge_radius: f64,
    /// `bulge_overlap` — the fraction of a diameter neighbours overlap (`0.25`).
    pub bulge_overlap: f64,
    /// `noise_factor` — jitter of bulge spacing and radius (`0.1`).
    pub noise_factor: f64,
    /// `circle_radii` — the trailing thought circles (`[0.1, 0.15, 0.2]`).
    pub circle_radii: Vec<f64>,
}

impl Default for ThoughtBubbleShape {
    fn default() -> Self {
        Self {
            bulge_radius: 0.35,
            bulge_overlap: 0.25,
            noise_factor: 0.1,
            circle_radii: vec![0.1, 0.15, 0.2],
        }
    }
}

/// `ThoughtBubble` (drawings.py:497), geometry-only — no asset involved.
///
/// Bulge centres walk the content's `SurroundingRectangle(buff)` corner to
/// corner (DL → UL → UR → DR → DL) at `(1 − bulge_overlap)·2·bulge_radius`
/// spacing, each jittered along its edge; every bulge is a circle of
/// jittered radius, and the cloud is their union with the rectangle. The
/// trailing circles are arranged up-right, the middle one nudged down-right,
/// and placed under the cloud's lower-left. A rightward `direction` mirrors
/// the whole body. `content` of [`None`] surrounds the invisible
/// `filler_shape` rectangle. Reference defaults: `buff = SMALL_BUFF`,
/// `filler_shape = (2, 1)`, `direction = LEFT`, and
/// [`ThoughtBubbleShape::default`].
///
/// Randomness is the one RNG (BN-01): pass a stream from the scene's
/// [`fmn_core::rng::RngRoot`] substreams, not an ambient generator. The
/// draws keep the Reference's order — every spacing jitter, then every
/// radius jitter.
///
/// # Errors
/// [`BooleanMobjectError`] from the cloud union.
pub fn thought_bubble(
    content: Option<&VMobject>,
    direction: Vec3,
    buff: f64,
    filler_shape: (f64, f64),
    shape: &ThoughtBubbleShape,
    rng: &mut fmn_core::rng::Pcg64Dxsm,
) -> Result<VMobject, BooleanMobjectError> {
    let content = match content {
        Some(given) => given.clone(),
        None => bubble_filler(filler_shape)?,
    };
    let corners = surrounding_corners(&content, buff);
    let rect = crate::poly::Polygon::new(corners).build();
    let radius = shape.bulge_radius;
    let step = (1.0 - shape.bulge_overlap) * (2.0 * radius);
    let nf = shape.noise_factor;
    // drawings.py walks [DL, UL, UR, DR] as adjacent pairs, closing back.
    let walk = [corners[2], corners[1], corners[0], corners[3]];
    let mut points = Vec::new();
    for index in 0..walk.len() {
        let (c1, c2) = (walk[index], walk[(index + 1) % walk.len()]);
        let distance = fmn_geom::space_ops::get_norm([c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2]]);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n_alphas = (distance / step) as usize + 1;
        for k in 0..n_alphas {
            // np.linspace(0, 1, n): a single sample is 0.
            let alpha = if n_alphas == 1 {
                0.0
            } else {
                k as f64 / (n_alphas - 1) as f64
            };
            let jitter = nf * (step / n_alphas as f64) * (rng.next_f64() - 0.5);
            points.push(lerp3(c1, c2, alpha + jitter));
        }
    }
    let mut operands = Vec::with_capacity(points.len() + 1);
    operands.push(rect);
    for point in points {
        let bulge = radius * (1.0 + nf * rng.next_f64());
        operands.push(Circle::new().radius(bulge).build().moved_to(point));
    }
    let cloud = crate::boolean_ops::union(&operands)?.into_mobject();

    let circ_buff = 0.25 * shape.circle_radii.first().copied().unwrap_or(0.0);
    let circles = VMobject::arranged(
        shape
            .circle_radii
            .iter()
            .map(|r| Circle::new().radius(*r).build()),
        UR,
        circ_buff,
        ORIGIN,
    );
    // `arrange` re-centres the group; then nudge the middle circle.
    let circles = circles.moved_to(ORIGIN);
    let mut position = 0usize;
    let circles = circles.map_children(move |circle| {
        let nudged = if position == 1 {
            circle.shifted([circ_buff * DR[0], circ_buff * DR[1], 0.0])
        } else {
            circle
        };
        position += 1;
        nudged
    });
    let circles = circles.next_to(&cloud, DOWN, 4.0 * circ_buff, LEFT);
    let mut members: Vec<VMobject> = circles.children().to_vec();
    members.push(cloud);
    let mut body = v_group(members);
    if direction[0] > 0.0 {
        body = mirrored_horizontally(body);
    }
    Ok(bubble_group(body, content))
}

/// [`thought_bubble`] with every Reference default, around `content`.
///
/// # Errors
/// As [`thought_bubble`].
pub fn thought_bubble_default(
    content: Option<&VMobject>,
    rng: &mut fmn_core::rng::Pcg64Dxsm,
) -> Result<VMobject, BooleanMobjectError> {
    thought_bubble(
        content,
        LEFT,
        fmn_core::constants::SMALL_BUFF,
        (2.0, 1.0),
        &ThoughtBubbleShape::default(),
        rng,
    )
}
