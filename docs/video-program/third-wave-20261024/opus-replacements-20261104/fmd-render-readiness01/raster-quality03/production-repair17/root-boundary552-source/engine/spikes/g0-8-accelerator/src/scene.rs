//! The preview frame the spike renders.
//!
//! §20.1 spike 8's acceptance is "a Metal-rendered preview frame demonstrably
//! produced from the prototype IR", so the frame has to be worth rendering: a
//! flat test card would prove the plumbing and nothing about the mathematics.
//! This scene deliberately exercises every property the stroke stage owns:
//!
//! - **curvature** — arcs and a Lissajous figure, so the cubic root solve runs
//!   in its three-real-roots branch, not just the degenerate ones;
//! - **the `Line` primitive hint** — an axis grid of exactly-straight paths,
//!   which take the capsule fast path;
//! - **joins** — polylines with sharp interior angles, where the nearest
//!   segment changes across a pixel;
//! - **round caps** — every open end;
//! - **arc-length width tapers** — `set_stroke(width=[0, 6, 0])`-shaped
//!   profiles, whose interpolation is the reason segments carry arc-length
//!   spans at all;
//! - **overlapping transparency in painter order** — the property that makes
//!   per-tile command order load-bearing rather than incidental;
//! - **sub-pixel geometry** — a hairline thinner than the AA band, the regime
//!   where a wrong profile is most visible.
//!
//! Colours come from `fmn_core::constants`' palette, converted to linear light
//! once here, so the frame reads as a FrankenManim frame rather than as a
//! debug pattern.

use crate::ir::{DrawKind, RenderIr, Style, TileGrid};
use crate::sdf::ANTI_ALIAS_WIDTH_PX;
use fmn_core::color::Srgb;
use fmn_core::constants::{
    BLUE, BLUE_B, BLUE_D, BLUE_E, GREEN, GREEN_C, GREY_D, MAROON_B, PURPLE, RED, RED_C, TEAL,
    TEAL_C, WHITE, YELLOW, YELLOW_C,
};
use fmn_geom::quadpath::QuadPath;

/// The preview surface size. A Studio preview frame, not an export frame:
/// §13.5 puts the annex's first production duty at preview latency, where the
/// picture is a fraction of the export resolution.
pub const PREVIEW_WIDTH: u32 = 960;
/// Preview height (16:9 with [`PREVIEW_WIDTH`]).
pub const PREVIEW_HEIGHT: u32 = 540;
/// Tile edge in pixels. 16×16 = 256 threads per threadgroup, comfortably
/// inside Apple silicon's 1024 ceiling and a multiple of the 32-wide SIMD
/// group, so no partial waves.
pub const TILE: u32 = 16;

/// The Reference's default background, `#333333` (`manimlib` camera config).
fn background() -> [f32; 4] {
    linear(Srgb::from_rgb8(0x33, 0x33, 0x33), 1.0)
}

fn linear(c: Srgb, alpha: f64) -> [f32; 4] {
    let l = c.to_linear(alpha);
    [l.r as f32, l.g as f32, l.b as f32, l.a as f32]
}

fn style(color: Srgb, alpha: f64, w0: f32, w1: f32) -> Style {
    let mut st = Style::flat(linear(color, alpha), w0, ANTI_ALIAS_WIDTH_PX as f32);
    st.width_end = w1;
    st.rgba_end = st.rgba;
    st
}

/// Build the preview frame's IR, binned and ready to dispatch.
pub fn preview_frame() -> RenderIr {
    build(PREVIEW_WIDTH, PREVIEW_HEIGHT, TILE)
}

/// The same scene at an arbitrary size — used by the tests to prove the IR is
/// resolution- and tile-independent.
pub fn build(width: u32, height: u32, tile: u32) -> RenderIr {
    let mut ir = RenderIr::new(
        TileGrid {
            width,
            height,
            tile,
        },
        background(),
    );
    let w = width as f64;
    let h = height as f64;

    // ---- the grid: exactly-straight paths, so the Line hint is exercised.
    for i in 1..12 {
        let x = w * i as f64 / 12.0;
        let mut p = QuadPath::default();
        p.start_new_path([x, h * 0.06, 0.0]);
        p.add_line_to([x, h * 0.94, 0.0], false).unwrap();
        ir.compile_path(&p, style(GREY_D, 1.0, 1.0, 1.0), DrawKind::Stroke);
    }
    for i in 1..7 {
        let y = h * i as f64 / 7.0;
        let mut p = QuadPath::default();
        p.start_new_path([w * 0.04, y, 0.0]);
        p.add_line_to([w * 0.96, y, 0.0], false).unwrap();
        ir.compile_path(&p, style(GREY_D, 1.0, 1.0, 1.0), DrawKind::Stroke);
    }

    // ---- a Lissajous figure: dense curvature, every cubic-root branch.
    //
    // Handles sit at the intersection of the sampled points' tangents, which
    // is the quadratic that actually osculates the curve. A midpoint handle
    // would make every segment exactly straight — the `Line` hint would fire
    // and the general solver would never run, which is the one thing this path
    // exists to exercise.
    {
        let mut p = QuadPath::default();
        let n = 96;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let ax = w * 0.36;
        let ay = h * 0.34;
        let at = |k: usize| -> f64 { k as f64 / n as f64 * std::f64::consts::TAU };
        let point = |t: f64| -> [f64; 2] {
            [
                cx + ax * fmn_dmath::sin(3.0 * t),
                cy + ay * fmn_dmath::sin(2.0 * t + 0.7),
            ]
        };
        let tangent = |t: f64| -> [f64; 2] {
            [
                3.0 * ax * fmn_dmath::cos(3.0 * t),
                2.0 * ay * fmn_dmath::cos(2.0 * t + 0.7),
            ]
        };
        let a0 = point(at(0));
        p.start_new_path([a0[0], a0[1], 0.0]);
        for k in 1..=n {
            let t0 = at(k - 1);
            let t1 = at(k);
            let a = point(t0);
            let b = point(t1);
            let handle = tangent_intersection(a, tangent(t0), b, tangent(t1));
            p.add_quadratic_bezier_curve_to([handle[0], handle[1], 0.0], [b[0], b[1], 0.0], false)
                .unwrap();
        }
        ir.compile_path(&p, style(BLUE_B, 1.0, 3.0, 3.0), DrawKind::Stroke);
    }

    // ---- three overlapping translucent arcs: painter order under alpha.
    //
    // Built by fmn-geom's own `QuadPath::arc`, so the arc-density rule under
    // test is BN-09's, not a hand-rolled one — the spike measures the engines,
    // not a private approximation of a circle.
    for (i, color) in [RED_C, GREEN_C, YELLOW_C].into_iter().enumerate() {
        let start = std::f64::consts::PI * (0.15 + 0.10 * i as f64);
        let p = crate::arc(
            start,
            std::f64::consts::PI * 1.4,
            h * 0.26,
            [w * (0.30 + 0.20 * i as f64), h * 0.50, 0.0],
            None,
        );
        ir.compile_path(&p, style(color, 0.55, 12.0, 12.0), DrawKind::Stroke);
    }

    // ---- tapered strokes: the arc-length width interpolation.
    for i in 0..6 {
        let mut p = QuadPath::default();
        let y = h * (0.14 + 0.145 * i as f64);
        p.start_new_path([w * 0.06, y, 0.0]);
        p.add_quadratic_bezier_curve_to([w * 0.16, y - h * 0.07, 0.0], [w * 0.26, y, 0.0], false)
            .unwrap();
        let taper = 2.0 + 2.0 * i as f32;
        ir.compile_path(&p, style(TEAL_C, 0.95, taper, 0.0), DrawKind::Stroke);
    }

    // ---- a sharp-jointed polyline: the nearest segment changes across pixels.
    {
        let mut p = QuadPath::default();
        let y0 = h * 0.80;
        p.start_new_path([w * 0.62, y0, 0.0]);
        for i in 0..7 {
            let x = w * (0.62 + 0.05 * (i + 1) as f64);
            let y = if i % 2 == 0 { y0 - h * 0.11 } else { y0 };
            p.add_line_to([x, y, 0.0], false).unwrap();
        }
        ir.compile_path(&p, style(MAROON_B, 1.0, 7.0, 7.0), DrawKind::Stroke);
    }

    // ---- a hairline thinner than the AA band: the sub-pixel regime.
    {
        let mut p = QuadPath::default();
        p.start_new_path([w * 0.05, h * 0.97, 0.0]);
        p.add_quadratic_bezier_curve_to([w * 0.5, h * 0.90, 0.0], [w * 0.95, h * 0.97, 0.0], false)
            .unwrap();
        ir.compile_path(&p, style(WHITE, 1.0, 0.4, 0.4), DrawKind::Stroke);
    }

    // ---- a heavy opaque underline, drawn last: proves painter order wins.
    {
        let mut p = QuadPath::default();
        p.start_new_path([w * 0.30, h * 0.06, 0.0]);
        p.add_line_to([w * 0.70, h * 0.06, 0.0], false).unwrap();
        ir.compile_path(&p, style(BLUE_D, 1.0, 9.0, 9.0), DrawKind::Stroke);
    }

    ir.bin();
    ir
}

// ------------------------------------------------------------- the fill frame

/// Build the fill frame's IR, binned and ready to dispatch (fm-orn).
pub fn fill_frame() -> RenderIr {
    build_fill(PREVIEW_WIDTH, PREVIEW_HEIGHT, TILE)
}

/// A **fill-only** frame, exercising every property §10.2's fill owns.
///
/// Fill-only on purpose, and for the same reason the stroke frame is
/// stroke-only: each annex kernel refuses the other kind by name, so a mixed
/// frame could not be dispatched without one of the two engines silently
/// skipping half the picture. Composition of the two stages is a W5 question
/// about pass ordering, not a mapping question, and inventing an answer here
/// would pre-empt it.
///
/// What the scene is built to exercise:
///
/// - **doubly-monotone splitting** — circles and blobs, whose pieces need both
///   the horizontal- and vertical-tangent cuts;
/// - **the nonzero winding rule, twice** — an annulus (a hole made by opposing
///   orientation) and a pentagram (a hole that *isn't* one, because the winding
///   in its core is two and the nonzero rule fills it, which is exactly where
///   even-odd would differ);
/// - **the tile carry** — shapes far wider than a tile, so most tiles see no
///   geometry at all and are covered entirely by winding that entered from the
///   left;
/// - **§10.4's interior class** — large convex fills whose middles are whole
///   tiles;
/// - **occlusion** — a layer of small fills under an opaque convex panel, so
///   the pruning pass has something to prune and the measurement is not zero
///   by construction;
/// - **gradients and painter-ordered alpha** — overlapping translucent discs;
/// - **edge-dense tiles** — a cluster of small discs, the glyph-like regime
///   where a fill spends its time.
pub fn build_fill(width: u32, height: u32, tile: u32) -> RenderIr {
    let mut ir = RenderIr::new(
        TileGrid {
            width,
            height,
            tile,
        },
        background(),
    );
    let w = width as f64;
    let h = height as f64;

    let flat = |c: Srgb, alpha: f64| {
        let mut st = Style::flat(linear(c, alpha), 0.0, ANTI_ALIAS_WIDTH_PX as f32);
        st.rgba_end = st.rgba;
        st
    };

    // ---- the occluded layer: small fills that a later opaque panel hides.
    for i in 0..10 {
        let x = w * (0.10 + 0.06 * i as f64);
        let p = polygon(&[
            [x, h * 0.16],
            [x + w * 0.045, h * 0.16],
            [x + w * 0.045, h * 0.40],
            [x, h * 0.40],
        ]);
        ir.compile_path(&p, flat(MAROON_B, 1.0), DrawKind::Fill);
    }

    // ---- the opaque convex panel that hides them.
    {
        let p = polygon(&[
            [w * 0.07, h * 0.12],
            [w * 0.72, h * 0.12],
            [w * 0.72, h * 0.44],
            [w * 0.07, h * 0.44],
        ]);
        let mut st = flat(BLUE_D, 1.0);
        // A gradient across the panel, both ends opaque — so it is still a legal
        // occluder, which is the case worth testing (an alpha-carrying gradient
        // must not be, and that is its own test).
        st.rgba_end = linear(TEAL_C, 1.0);
        st.gradient_axis = [
            (w * 0.07) as f32,
            (h * 0.12) as f32,
            (w * 0.72) as f32,
            (h * 0.44) as f32,
        ];
        ir.compile_path(&p, st, DrawKind::Fill);
    }

    // ---- an annulus: a hole made by opposing orientation.
    //
    // Two subpaths in one path, the inner one wound the other way. Starting the
    // second explicitly is what makes them independent contours rather than one
    // contour with a bridge across the hole.
    {
        let c = [w * 0.20, h * 0.70, 0.0];
        let mut ring = QuadPath::default();
        for (radius, angle) in [
            (h * 0.20, std::f64::consts::TAU),
            (h * 0.11, -std::f64::consts::TAU),
        ] {
            let a = crate::arc(0.0, angle, radius, c, None);
            let [start, _, _] = a.nth_curve_points(0).expect("an arc has curves");
            ring.start_new_path(start);
            for i in 0..a.num_curves() {
                let [_, handle, end] = a.nth_curve_points(i).expect("in range");
                ring.add_quadratic_bezier_curve_to(handle, end, false)
                    .expect("appending to an open subpath");
            }
        }
        ir.compile_path(&ring, flat(YELLOW_C, 1.0), DrawKind::Fill);
    }

    // ---- a lobed blob: the only shape here whose pieces actually need splitting.
    //
    // Worth its own paragraph, because the first draft of this scene had none.
    // `QuadPath::arc` lays a circle out as 16 equal components starting at angle
    // zero, so its extrema — at 0°, 90°, 180°, 270° — land exactly on component
    // *junctions* and every piece is already monotone. A scene made of circles
    // and polygons therefore exercises the monotone table's storage and none of
    // its mathematics, and the GPU comparison would never see a split piece.
    //
    // Handles here sit at the intersection of the sampled tangents (the same
    // construction the stroke frame's Lissajous uses), so the extrema fall
    // wherever the curve puts them.
    {
        let (cx, cy, r) = (w * 0.30, h * 0.68, h * 0.17);
        let n = 12;
        let radius = |t: f64| r * (1.0 + 0.32 * fmn_dmath::sin(3.0 * t + 0.4));
        let d_radius = |t: f64| r * 0.32 * 3.0 * fmn_dmath::cos(3.0 * t + 0.4);
        let point = |t: f64| -> [f64; 2] {
            let rr = radius(t);
            [cx + rr * fmn_dmath::cos(t), cy + rr * fmn_dmath::sin(t)]
        };
        let tangent = |t: f64| -> [f64; 2] {
            let (rr, dr) = (radius(t), d_radius(t));
            let (c, s) = (fmn_dmath::cos(t), fmn_dmath::sin(t));
            [dr * c - rr * s, dr * s + rr * c]
        };
        let at = |k: usize| k as f64 / n as f64 * std::f64::consts::TAU;
        let a0 = point(at(0));
        let mut p = QuadPath::default();
        p.start_new_path([a0[0], a0[1], 0.0]);
        for k in 1..=n {
            let (t0, t1) = (at(k - 1), at(k));
            let (a, b) = (point(t0), point(t1));
            let handle = tangent_intersection(a, tangent(t0), b, tangent(t1));
            p.add_quadratic_bezier_curve_to([handle[0], handle[1], 0.0], [b[0], b[1], 0.0], false)
                .unwrap();
        }
        ir.compile_path(&p, flat(TEAL_C, 0.85), DrawKind::Fill);
    }

    // ---- a pentagram: winding two in the core, which the nonzero rule fills.
    {
        let (cx, cy, r) = (w * 0.50, h * 0.70, h * 0.20);
        let mut pts = Vec::with_capacity(5);
        for k in 0..5 {
            // Every second vertex, which is what makes the path self-intersect.
            let a = std::f64::consts::FRAC_PI_2 + (k as f64) * 4.0 * std::f64::consts::TAU / 10.0;
            pts.push([cx + r * fmn_dmath::cos(a), cy - r * fmn_dmath::sin(a)]);
        }
        ir.compile_path(&polygon(&pts), flat(RED_C, 1.0), DrawKind::Fill);
    }

    // ---- overlapping translucent discs, in painter order.
    for (i, color) in [GREEN_C, BLUE_B, WHITE].into_iter().enumerate() {
        let p = crate::arc(
            0.0,
            std::f64::consts::TAU,
            h * 0.15,
            [
                w * (0.72 + 0.06 * i as f64),
                h * (0.62 + 0.05 * i as f64),
                0.0,
            ],
            None,
        );
        ir.compile_path(&p, flat(color, 0.5), DrawKind::Fill);
    }

    // ---- a cluster of small discs: the edge-dense, glyph-like regime.
    for i in 0..24 {
        let col = i % 8;
        let row = i / 8;
        let p = crate::arc(
            0.0,
            std::f64::consts::TAU,
            h * 0.022,
            [
                w * (0.78 + 0.024 * col as f64),
                h * (0.10 + 0.075 * row as f64),
                0.0,
            ],
            None,
        );
        ir.compile_path(&p, flat(GREY_D, 1.0), DrawKind::Fill);
    }

    // ---- a sliver thinner than a pixel: the sub-pixel coverage regime.
    {
        let p = polygon(&[
            [w * 0.08, h * 0.955],
            [w * 0.92, h * 0.955],
            [w * 0.92, h * 0.9576],
            [w * 0.08, h * 0.9576],
        ]);
        ir.compile_path(&p, flat(WHITE, 1.0), DrawKind::Fill);
    }

    ir.bin();
    ir
}

/// Which G0-2 calibration panel to build ([`calibration`]).
///
/// The four the analytic prototype can express. The captured set has two more
/// — `lighting_3d` and `text_sample` — which need the 3D lighting path and
/// Scribe respectively; neither exists yet, and rendering a stand-in for them
/// would produce a side-by-side that compares nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationPanel {
    /// Fill and stroke colour gradients over opacity compositing.
    GradientFills,
    /// A self-intersecting pentagram filled under the nonzero winding rule —
    /// the panel where the Reference's signed-alpha trick is visible.
    SelfIntersections,
    /// Wide zig-zag strokes through a sharp reversal, for the join comparison.
    JointsAndCaps,
    /// `GlowDot` falloff at three radii and colours.
    Glow,
}

impl CalibrationPanel {
    /// The capture id this panel is compared against, i.e. the basename of the
    /// Reference still in `gallery/reference_captures/`.
    pub fn id(self) -> &'static str {
        match self {
            CalibrationPanel::GradientFills => "gradient_fills",
            CalibrationPanel::SelfIntersections => "self_intersections",
            CalibrationPanel::JointsAndCaps => "joints_and_caps",
            CalibrationPanel::Glow => "glow",
        }
    }

    /// All four, in capture-inventory order.
    pub const ALL: [CalibrationPanel; 4] = [
        CalibrationPanel::GradientFills,
        CalibrationPanel::SelfIntersections,
        CalibrationPanel::JointsAndCaps,
        CalibrationPanel::Glow,
    ];
}

/// Build one G0-2 calibration panel (fm-k77), in **capture pixel coordinates**.
///
/// The geometry below is not re-derived from the Reference's scene units; it is
/// the geometry *measured off the captured stills themselves* (bounding boxes,
/// a least-squares circle fit, the four stroke-band centres). That is the whole
/// point: the two images have to land on the same pixels before a reviewer can
/// judge anything but registration. At 1920x1080 the Reference's mapping is
/// 135 px per scene unit (`FRAME_WIDTH / 1920 = 14.2222/1920`), and stroke
/// width converts at 1.35 px per width unit (`STROKE_WIDTH_CONVERSION = 0.01`
/// scene units, times 135) — both are recorded in the G0-2 note.
pub fn calibration(panel: CalibrationPanel, width: u32, height: u32, tile: u32) -> RenderIr {
    let mut ir = RenderIr::new(
        TileGrid {
            width,
            height,
            tile,
        },
        background(),
    );
    // The measurements are at 1920x1080; scale if asked for another size so the
    // panel stays resolution-independent like every other scene here.
    let k = width as f64 / 1920.0;
    let sx = |x: f64| x * k;
    let sy = |y: f64| y * k;

    let flat = |c: Srgb, alpha: f64, w: f32| {
        let mut st = Style::flat(linear(c, alpha), w, ANTI_ALIAS_WIDTH_PX as f32);
        st.rgba_end = st.rgba;
        st
    };

    match panel {
        CalibrationPanel::GradientFills => {
            // Square: measured bbox cols 516..930, rows 333..746.
            let (cx, cy, half) = (sx(723.0), sy(539.5), sx(207.0));
            let sq = polygon(&[
                [cx - half, cy - half],
                [cx + half, cy - half],
                [cx + half, cy + half],
                [cx - half, cy + half],
            ]);
            // Fill gradient BLUE_E -> YELLOW at opacity 0.8. The Reference runs
            // it along the outline's point order, which for a Square reads as a
            // diagonal across the interior (measured: top-left darkest,
            // bottom-right brightest), so the axis is the diagonal.
            let mut fill = flat(BLUE_E, 0.8, 0.0);
            fill.rgba_end = linear(YELLOW, 0.8);
            fill.gradient_axis = [
                (cx - half) as f32,
                (cy - half) as f32,
                (cx + half) as f32,
                (cy + half) as f32,
            ];
            ir.compile_path(&sq, fill, DrawKind::Fill);
            // Stroke gradient RED -> GREEN, width 6 units = 8.1 px.
            let mut stroke = flat(RED, 1.0, (8.1 * k) as f32);
            stroke.rgba_end = linear(GREEN, 1.0);
            ir.compile_path(&sq, stroke, DrawKind::Stroke);

            // Circle: least-squares fit of the captured edge gave centre
            // (539.49, 1195.86) with R = 205.48 px.
            let (ccx, ccy, r) = (sx(1195.86), sy(539.49), sx(205.48));
            let circle = crate::arc(0.0, std::f64::consts::TAU, r, [ccx, ccy, 0.0], None);
            let mut cfill = flat(PURPLE, 0.5, 0.0);
            cfill.rgba_end = linear(TEAL, 0.5);
            cfill.gradient_axis = [
                (ccx - r) as f32,
                (ccy - r) as f32,
                (ccx + r) as f32,
                (ccy + r) as f32,
            ];
            ir.compile_path(&circle, cfill, DrawKind::Fill);
            ir.compile_path(&circle, flat(RED, 1.0, (5.4 * k) as f32), DrawKind::Stroke);
        }

        CalibrationPanel::SelfIntersections => {
            // Pentagram, vertices taken edge-to-edge so the outline crosses
            // itself: the captured bbox (rows 164..913, cols 565..1354) implies
            // circumradius 414 px about (959.5, 578).
            let (cx, cy, r) = (sx(959.5), sy(578.0), sx(414.0));
            let mut pts = Vec::with_capacity(5);
            for k5 in 0..5 {
                // pi/2 + k*4pi/5, and screen y grows downward.
                let a = std::f64::consts::FRAC_PI_2 + k5 as f64 * 4.0 * std::f64::consts::PI / 5.0;
                pts.push([cx + r * fmn_dmath::cos(a), cy - r * fmn_dmath::sin(a)]);
            }
            let star = polygon(&pts);
            ir.compile_path(&star, flat(BLUE_D, 0.7, 0.0), DrawKind::Fill);
            ir.compile_path(&star, flat(WHITE, 1.0, (5.4 * k) as f32), DrawKind::Stroke);
        }

        CalibrationPanel::JointsAndCaps => {
            // Four identical zig-zags at the captured band centres. Identical
            // on purpose: the Reference draws four DIFFERENT corners here
            // (auto/bevel/miter/no_joint), and a true curve-distance stroke has
            // one corner — the round join the distance field already implies.
            // Rendering four of the same is the finding, stated visually.
            let x0 = sx(737.0);
            let x3 = sx(1254.0);
            let dx = (x3 - x0) / 3.0;
            let amp = sy(104.0);
            for cy_px in [166.0, 415.5, 664.0, 913.0] {
                let cy = sy(cy_px);
                let mut p = QuadPath::default();
                p.start_new_path([x0, cy, 0.0]);
                p.add_line_to([x0 + dx, cy - amp, 0.0], false).unwrap();
                p.add_line_to([x0 + 2.0 * dx, cy + amp, 0.0], false)
                    .unwrap();
                p.add_line_to([x3, cy, 0.0], false).unwrap();
                // Stroke width 20 units = 27 px.
                ir.compile_path(&p, flat(YELLOW, 1.0, (27.0 * k) as f32), DrawKind::Stroke);
            }
        }

        CalibrationPanel::Glow => {
            // GlowDot(LEFT*2, r=1.0, BLUE), (ORIGIN, r=1.5, YELLOW),
            // (RIGHT*2, r=0.75, RED) at 135 px per scene unit about (960, 540).
            for (x, r, c) in [
                (690.0, 135.0, BLUE),
                (960.0, 202.5, YELLOW),
                (1230.0, 101.25, RED),
            ] {
                // A glow is centred on its path's FIRST ANCHOR, but a path with
                // no curve compiles to nothing (`compile_path` refuses zero
                // total arc length), so the anchor needs one real segment to
                // ride on. It contributes no ink — `DrawKind::Glow` never
                // strokes — and the slab is grown by `glow_radius + aa_width`
                // from the path bounds either way.
                let mut p = QuadPath::default();
                p.start_new_path([sx(x), sy(540.0), 0.0]);
                p.add_line_to([sx(x) + 1.0, sy(540.0), 0.0], false).unwrap();
                let mut st = flat(c, 1.0, 0.0);
                st.glow_radius = (r * k) as f32;
                // GlowDot's glow_factor, and DotCloud's wider AA band.
                st.glow_factor = 2.0;
                st.aa_width = 2.0;
                ir.compile_path(&p, st, DrawKind::Glow);
            }
        }
    }

    ir.bin();
    ir
}

/// A closed polygon through `pts`, as straight quadratics.
fn polygon(pts: &[[f64; 2]]) -> QuadPath {
    let mut p = QuadPath::default();
    p.start_new_path([pts[0][0], pts[0][1], 0.0]);
    for q in &pts[1..] {
        p.add_line_to([q[0], q[1], 0.0], false).unwrap();
    }
    p.add_line_to([pts[0][0], pts[0][1], 0.0], false).unwrap();
    p
}

/// How far a control handle may sit from its chord midpoint, as a multiple of
/// the chord length.
///
/// Any real curve fitter needs this clamp, and the spike learned why the hard
/// way: near a Lissajous turning point consecutive tangents are nearly
/// parallel, so their intersection runs away to hundreds of chord lengths. The
/// resulting quadratic is *algebraically* still an interpolant and
/// *numerically* a disaster — its coefficients span orders of magnitude, and
/// the f32 annex and the f64 CPU stop agreeing on a handful of pixels around
/// exactly those control points. Bounding the handle removes the pathology
/// from the content instead of papering over it in the kernel.
const MAX_HANDLE_BOW: f64 = 2.0;

/// Where the tangent at `a` meets the tangent at `b` — the quadratic control
/// point that makes the segment osculate the sampled curve — clamped to
/// [`MAX_HANDLE_BOW`] chord lengths from the chord midpoint.
///
/// Falls back to the chord midpoint when the tangents are parallel (an
/// inflection sampled symmetrically), which is the right answer there: the
/// osculating quadratic genuinely is the straight chord.
fn tangent_intersection(a: [f64; 2], da: [f64; 2], b: [f64; 2], db: [f64; 2]) -> [f64; 2] {
    let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let cross = da[0] * db[1] - da[1] * db[0];
    if cross.abs() < 1e-12 {
        return mid;
    }
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let chord = (dx * dx + dy * dy).sqrt();
    let s = (dx * db[1] - dy * db[0]) / cross;
    let h = [a[0] + da[0] * s, a[1] + da[1] * s];

    let bow = [h[0] - mid[0], h[1] - mid[1]];
    let bow_len = (bow[0] * bow[0] + bow[1] * bow[1]).sqrt();
    let limit = MAX_HANDLE_BOW * chord;
    if bow_len <= limit || bow_len <= 0.0 {
        return h;
    }
    let k = limit / bow_len;
    [mid[0] + bow[0] * k, mid[1] + bow[1] * k]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::PrimitiveHint;

    #[test]
    fn the_preview_frame_is_actually_nontrivial() {
        let ir = preview_frame();
        assert!(ir.paths.len() >= 25, "only {} paths", ir.paths.len());
        assert!(
            ir.segments.len() >= 150,
            "only {} segments",
            ir.segments.len()
        );
        assert!(ir.styles.len() >= 8, "only {} styles", ir.styles.len());
        assert!(
            !ir.tiles.draws.is_empty(),
            "nothing was binned, so nothing would render"
        );
    }

    #[test]
    fn both_primitive_hints_are_exercised() {
        let ir = preview_frame();
        assert!(
            ir.paths.iter().any(|p| p.hint == PrimitiveHint::Line),
            "no straight path — the capsule fast path is untested"
        );
        assert!(
            ir.paths.iter().any(|p| p.hint == PrimitiveHint::General),
            "no curved path — the cubic solve is untested"
        );
    }

    #[test]
    fn tapers_and_translucency_are_both_present() {
        let ir = preview_frame();
        assert!(
            ir.styles.iter().any(|s| s.width_start != s.width_end),
            "no tapered stroke — arc-length width interpolation is untested"
        );
        assert!(
            ir.styles.iter().any(|s| s.rgba[3] < 0.99),
            "no translucent stroke — painter-order compositing is untested"
        );
        assert!(
            ir.styles
                .iter()
                .any(|s| s.width_start < ANTI_ALIAS_WIDTH_PX as f32),
            "no sub-pixel hairline — the thin-stroke regime is untested"
        );
    }

    #[test]
    fn every_binned_draw_indexes_a_real_path() {
        let ir = preview_frame();
        for &d in &ir.tiles.draws {
            assert!((d as usize) < ir.paths.len(), "dangling draw index {d}");
        }
        for w in ir.tiles.offsets.windows(2) {
            assert!(w[0] <= w[1], "CSR offsets are not monotone");
        }
    }

    #[test]
    fn the_scene_is_tile_size_independent_in_its_geometry() {
        // Changing the tiling must change only the command lists, never the
        // geometry that produced them.
        let a = build(320, 180, 8);
        let b = build(320, 180, 20);
        assert_eq!(a.segments, b.segments);
        assert_eq!(a.paths, b.paths);
        assert_eq!(a.styles, b.styles);
        assert_ne!(a.tiles, b.tiles);
    }
}
