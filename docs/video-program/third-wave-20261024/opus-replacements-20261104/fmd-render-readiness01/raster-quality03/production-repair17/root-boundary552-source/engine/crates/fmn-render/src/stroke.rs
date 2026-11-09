//! §10.3's strokes: **true curve-distance**, round caps, arc-length width and
//! colour — replacing the Reference's ≤32-segment polyline ribbons.
//!
//! > True curve-distance strokes: exact/high-accuracy signed distance to the
//! > quadratic within conservative slabs; **round caps** on open ends;
//! > round/bevel/miter joins with a real miter limit plus a smooth "auto" join
//! > tuned in the look study; width and colour interpolated by **arc length**;
//! > `flat_stroke` vs camera-facing 3D construction both supported; the smoothstep
//! > AA profile at the familiar ~1.5 px weight.
//!
//! ## The stroke is one expression
//!
//! Everything above except the joins falls out of a single quantity — the
//! **signed excess**
//!
//! ```text
//! excess(p) = min over segments of ( distance(p, segment) − half_width(s) )
//! ```
//!
//! where `s` is the *arc length* at the nearest point. Coverage is that excess
//! through G0-2's measured antialiasing profile, and nothing else is needed:
//!
//! - **Round caps come free.** The distance to an open end is radial, so the
//!   level set is a semicircle without anyone asking for one. The Reference's
//!   butt caps were a ribbon-pipeline artifact; round is the deliberate choice.
//! - **Round joins come free.** Two segments sharing an anchor contribute two
//!   distance fields whose minimum is exactly a round join — no notch, no gap, no
//!   bookkeeping. G0-2's finding L6 is that this collapses the Reference's four
//!   joint types to one, and its calibration capture renders four identical rows
//!   on purpose.
//! - **Width by arc length is a change of what `s` means**, not a change of
//!   mechanism. Because the excess is taken *per segment* before the minimum, a
//!   stroke whose width varies is a union of variable-radius tubes rather than one
//!   tube of averaged radius — which is what makes a reparameterization of the
//!   curve leave the stroke identical, the metamorphic law §10.3's acceptance
//!   names.
//!
//! ## What is not free
//!
//! The `Miter`, `Bevel` and `NoJoint` overrides are real geometry the distance
//! field does not produce, and they land with [`JoinWedge`] rather than by
//! bending the distance field into shapes it does not have. Their ruling — which
//! numeric code draws which corner, and why the Reference's *names* cannot be
//! trusted — is ADR-0012.
//!
//! ## `flat_stroke` and camera-facing construction
//!
//! This module owns the affine distance kernel. Under its [`ScreenMap`], a width
//! measured in the object plane and one measured in screen pixels differ only
//! by the uniform scale, so both constructions intentionally collapse to the
//! same expression here. Perspective vectors take the camera-bound route in
//! [`crate::three_d::ThreeDJob`]: it derives separate flat and billboard
//! half-widths from world-space curve controls, then feeds the resulting
//! screen-space widths back into this distance kernel.
//! [`crate::engine::FrameJob`] rejects camera-dependent style fields so they
//! cannot silently render through the affine route.
//!
//! ## What the slabs are, and why there is no slab table
//!
//! §10.3 says "within conservative slabs", and the IR already retains what a slab
//! is made of: a segment's control-polygon hull contains its curve. The
//! width-dependent expansion is deliberately **not** retained, because the width
//! is a property of the *style* and two occurrences of one interned outline can
//! carry different widths — a retained slab would be invalidated by a restyle,
//! which is exactly the coupling §10.8's revision axes exist to avoid. So
//! [`segment_slab`] computes it per draw from retained data, and it is cheap: a
//! hull and an add.
//!
//! ## Prepared rejection and the SIMD ruling
//!
//! fm-4wt.3 found that the engine used only the union slab: every pixel inside
//! that union still ran the cubic nearest-point solve and both arc-length
//! evaluations for every segment. `PreparedStroke` now derives each segment's
//! style/placement-dependent slab and geometry-only total arc length once per
//! draw. The scalar kernel keeps segment order, rejects only provably
//! zero-coverage segments, and remains bit-identical to the unculled oracle
//! wherever coverage can change.
//!
//! On the committed 512×256 curved-chain benchmark, 8/32/64 segments fell from
//! 86.63/373.44/770.93 ms to 14.84/28.94/45.95 ms: 5.8×/12.9×/16.8× from
//! eliminating work. A governed x86-64-v3 `f64x4` prototype then evaluated four
//! slab admissions together while retaining scalar f64 cubic solves in original
//! order. Three SIMD sweeps measured 14.60–14.84/28.32–29.46/43.79–44.58 ms
//! versus scalar sweeps of 14.64–15.02/27.96–28.80/43.43–45.43 ms. That is
//! noise, not a repeatable tier speedup, so the packing route is deliberately
//! not retained.
//! The ill-conditioned distance solve remains f64 exactly as G0-8 ruled.

use crate::bin::ScreenMap;
use crate::table::{Segment, Style};

/// Convert a stroke width from manim's width units to screen pixels.
///
/// `STROKE_WIDTH_CONVERSION = 0.01` scene units per width unit
/// (`stroke/vert.glsl:21`), then scene units to pixels. G0-2 derived the pair: at
/// 1920×1080 with the default frame there are 135 px per scene unit, so one width
/// unit is 1.35 px and `DEFAULT_STROKE_WIDTH = 4.0` is 5.4 px.
///
/// The one conversion for every width in the renderer — §10.2's
/// `fill_border_width` included, since the Reference feeds that field into this
/// same stroke-width slot.
#[must_use]
pub fn width_px(width_units: f32, map: ScreenMap) -> f64 {
    f64::from(width_units) * fmn_core::constants::STROKE_WIDTH_CONVERSION * map.scale.abs()
}

/// Coverage from a signed excess, through G0-2's **measured** antialiasing
/// profile.
///
/// `stroke/frag.glsl` is `smoothstep(0.5, −0.5, |d|/aaw − hw/aaw)`, i.e.
/// `t = clamp(½ − excess/aaw, 0, 1)` then the Hermite `t²(3 − 2t)`, with
/// `anti_alias_width = 1.5` px (`vectorized_mobject.py:96`). G0-2 finding L1
/// measured that band at **1.560 px** with RMS 0.0031 against a declared 1.5 —
/// inside the capture's own 8-bit quantization — and ratified keeping both the
/// width and the curve.
///
/// The profile lives here because the stroke is where it was measured and where
/// it is used most; §10.4's adaptive-AA bead (fm-gmr) may relocate it, and
/// §10.2's inner border already calls it rather than restating it.
#[must_use]
pub fn aa_coverage(excess_px: f64, aa_width_px: f64) -> f64 {
    let aa = if aa_width_px > 0.0 { aa_width_px } else { 1e-8 };
    let t = (0.5 - excess_px / aa).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The half-width in pixels at normalized arc length `s`.
///
/// The IR's [`Style`] carries the two ends of the per-point `stroke_width`
/// column, which is how the Reference expresses a width ramp along a path, so the
/// ramp is linear in `s`. Interpolating the *width* and halving is not the same
/// as interpolating two half-widths, but it is the same to floating point and it
/// is the order the Reference uses.
#[must_use]
pub fn half_width_px(style: &Style, map: ScreenMap, s: f64) -> f64 {
    let w = style.stroke_width_at(s);
    0.5 * width_px(w, map)
}

/// The stroke colour at normalized arc length `s`.
///
/// Componentwise in the linear-light straight-alpha space the table stores
/// (§6.3, BN-04) — a colour interpolation happens where colour interpolation is
/// defined, not in an encoded space.
#[must_use]
pub fn stroke_rgba_at(style: &Style, s: f64) -> [f32; 4] {
    style.stroke_color_at(s)
}

/// One segment's conservative screen-space slab: the curve's hull, grown by the
/// widest half-width the ramp can reach plus the antialiasing band.
///
/// Conservative in the safe direction — a slab that is too large costs a distance
/// evaluation, and there is no input for which one that is too small is anything
/// but a clipped stroke. It uses the **whole ramp's** maximum rather than the
/// width at this segment's own arc-length span, because the nearest point to a
/// pixel inside this slab can lie on a neighbouring segment.
#[must_use]
pub fn segment_slab(seg: &Segment, style: &Style, map: ScreenMap, translate: [f64; 2]) -> [f64; 4] {
    let to_px = |p: fmn_core::types::Vec3| {
        let q = map.to_pixel(p[0], p[1]);
        [q[0] + translate[0], q[1] + translate[1]]
    };
    let pts = [to_px(seg.p0), to_px(seg.p1), to_px(seg.p2)];
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for q in pts {
        for k in 0..2 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    let pad = max_stroke_reach_px(style, map);
    [lo[0] - pad, lo[1] - pad, hi[0] + pad, hi[1] + pad]
}

/// Maximum screen-space reach beyond the control-polygon hull.
///
/// Round and bevelled strokes reach one half-width beyond the centreline. An
/// admitted miter may reach [`MITER_LIMIT`] half-widths, and the binning and
/// engine slabs must include that tip before they are allowed to reject a
/// pixel. The antialiasing band is added in either case.
#[must_use]
pub fn max_stroke_reach_px(style: &Style, map: ScreenMap) -> f64 {
    let widest = 0.5 * width_px(style.maximum_stroke_width(), map).max(0.0);
    let geometric = if style.joint_type == fmn_mobject::JointType::Miter {
        MITER_LIMIT * widest
    } else {
        widest
    };
    geometric + f64::from(style.anti_alias_width).max(0.0)
}

/// One segment's style/placement-dependent stroke preparation.
///
/// `pub(crate)` so the frame arena ([`crate::arena`]) can hold a typed pool
/// of them; the engine's per-frame path bump-allocates there instead of
/// allocating a fresh `Vec` per draw (PG-6).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedSegment {
    slab: [f64; 4],
    total_arc_length: f64,
    line: bool,
    /// Whether the arc-length station of the nearest point is observable:
    /// false for a uniform stroke ([`Style::stroke_station_is_observable`]),
    /// whose width and colour are exactly the same at every station, so the
    /// per-pixel arc-length solve can be skipped without changing a bit.
    station: bool,
    /// For a uniform curved stroke, the squared object-space distance from the
    /// segment's control triangle at and beyond which the segment provably
    /// leaves the pixel's coverage unchanged ([`uniform_cull_distance`]).
    cull_distance_squared: Option<f64>,
    /// The screen-space strip around the segment's chord outside which no
    /// point can receive coverage from it ([`ChordStrip`]); `None` for a
    /// degenerate chord, which keeps the slab bound alone.
    strip: Option<ChordStrip>,
}

/// A conservative screen-space strip around a segment's chord line.
///
/// Every point of a quadratic lies within half its control point's distance
/// from the chord line, and a stroke's coverage — round, bevelled or mitred,
/// plus the AA band — reaches at most [`max_stroke_reach_px`] beyond the
/// curve; a mitre tip sits within that reach of the joint, which lies on the
/// chord line. So a point farther than `reach + deviation` from the chord
/// line (widened for rounding) gets zero coverage from this segment and
/// cannot be the corner of any join it takes part in.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ChordStrip {
    /// A point on the chord line (the start anchor), in pixels.
    origin: [f64; 2],
    /// The chord's unit normal.
    normal: [f64; 2],
    /// The strip's half-width, in pixels.
    half_width: f64,
}

impl ChordStrip {
    fn of(segment: &Segment, map: ScreenMap, translate: [f64; 2], reach: f64) -> Option<Self> {
        let to_px = |p: fmn_core::types::Vec3| {
            let q = map.to_pixel(p[0], p[1]);
            [q[0] + translate[0], q[1] + translate[1]]
        };
        let [a, c, b] = [to_px(segment.p0), to_px(segment.p1), to_px(segment.p2)];
        let chord = [b[0] - a[0], b[1] - a[1]];
        let length = norm2(chord);
        if !(length > 1e-9 && length.is_finite() && reach.is_finite()) {
            return None;
        }
        let normal = [-chord[1] / length, chord[0] / length];
        let deviation = dot2(normal, [c[0] - a[0], c[1] - a[1]]).abs();
        let magnitude = [a, b, c]
            .iter()
            .flat_map(|q| [q[0].abs(), q[1].abs()])
            .fold(0.0f64, f64::max);
        let half_width = reach.max(0.0) + deviation + 1e-6 + 1e-9 * magnitude;
        half_width.is_finite().then_some(Self {
            origin: a,
            normal,
            half_width,
        })
    }

    /// The x-interval of the strip over the pixel row `[y0, y0 + 1]`, or
    /// `None` when the chord is too close to horizontal to bound `x`.
    fn row_interval(&self, y0: f64) -> Option<(f64, f64)> {
        let [nx, ny] = self.normal;
        if nx.abs() < 1e-6 {
            return None;
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for y in [y0, y0 + 1.0] {
            for offset in [-self.half_width, self.half_width] {
                let x = self.origin[0] + (offset - ny * (y - self.origin[1])) / nx;
                lo = lo.min(x);
                hi = hi.max(x);
            }
        }
        (lo.is_finite() && hi.is_finite()).then_some((lo, hi))
    }
}

/// The shortest stroke [`segments_are_x_sorted`] accepts. Shorter strokes keep
/// the linear scan, whose cost is already bounded by this constant.
const STROKE_X_INDEX_MIN_SEGMENTS: usize = 32;

/// May [`PreparedStroke::x_candidates`] search these segments by x?
///
/// True for a long stroke whose slabs are all finite and whose left and right
/// slab edges both never decrease in shape order, which makes the segments a
/// point can be admitted by a contiguous run. The aggregate `slab` must be the
/// one [`PreparedStroke::prepare_into`] returned for `segments`.
///
/// The slabs are the whole proof, so the geometry that produced them does not
/// matter. Finiteness keeps the aggregate a true bound: a `NaN` edge fails both
/// of the slab test's comparisons, so its segment passes that test, and
/// `min`/`max` drop the edge from the aggregate.
#[must_use]
pub(crate) fn segments_are_x_sorted(segments: &[PreparedSegment], slab: [f64; 4]) -> bool {
    let finite = |edges: &[f64; 4]| edges.iter().all(|edge| edge.is_finite());
    segments.len() >= STROKE_X_INDEX_MIN_SEGMENTS
        && finite(&slab)
        && segments.iter().all(|prepared| finite(&prepared.slab))
        && segments
            .windows(2)
            .all(|pair| pair[0].slab[0] <= pair[1].slab[0] && pair[0].slab[2] <= pair[1].slab[2])
}

/// Per-draw stroke data whose inputs are fixed before a tile is touched.
///
/// The retained [`Segment`] remains geometry-only. These values additionally
/// depend on style, placement, and camera scale, so they belong to the
/// per-frame draw derivation: one conservative slab and one total arc length
/// per segment. A pixel outside a segment slab cannot receive coverage from
/// that segment, which lets the engine avoid its cubic solve without changing a
/// visible bit.
///
/// The struct is a *view*: the segment storage lives in the caller's buffer —
/// the frame arena's typed pool in the engine, a plain `Vec` in tests — so the
/// per-frame path allocates nothing once the arena is warm.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedStroke<'a> {
    segments: &'a [PreparedSegment],
    slab: [f64; 4],
    /// [`segments_are_x_sorted`] accepted `segments`.
    x_sorted: bool,
}

impl<'a> PreparedStroke<'a> {
    /// The view without the x index: the scan the index is tested against.
    #[cfg(test)]
    pub(crate) fn from_parts(segments: &'a [PreparedSegment], slab: [f64; 4]) -> Self {
        Self::from_parts_indexed(segments, slab, false)
    }

    /// The view over caller-owned prepared segments and their aggregate slab,
    /// with the verdict of [`segments_are_x_sorted`] on those same two.
    pub(crate) fn from_parts_indexed(
        segments: &'a [PreparedSegment],
        slab: [f64; 4],
        x_sorted: bool,
    ) -> Self {
        Self {
            segments,
            slab,
            x_sorted,
        }
    }

    /// Derive every segment's slab and total arc length into `out`, returning
    /// the aggregate slab. Same loop, same order as the retained derivation —
    /// only the destination is the caller's.
    pub(crate) fn prepare_into(
        out: &mut impl crate::arena::Sink<PreparedSegment>,
        segments: &[Segment],
        style: &Style,
        map: ScreenMap,
        translate: [f64; 2],
        straight_segments: bool,
    ) -> [f64; 4] {
        let mut slab = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        let station = style.stroke_station_is_observable();
        let cull = if station || straight_segments {
            None
        } else {
            uniform_cull_distance(style, map)
        };
        let reach = max_stroke_reach_px(style, map);
        for segment in segments {
            let segment_slab = segment_slab(segment, style, map, translate);
            slab[0] = slab[0].min(segment_slab[0]);
            slab[1] = slab[1].min(segment_slab[1]);
            slab[2] = slab[2].max(segment_slab[2]);
            slab[3] = slab[3].max(segment_slab[3]);
            out.put(PreparedSegment {
                slab: segment_slab,
                total_arc_length: if station {
                    fmn_geom::arclength::quadratic_arc_length(segment.p0, segment.p1, segment.p2)
                } else {
                    0.0
                },
                line: straight_segments,
                station,
                cull_distance_squared: cull
                    .and_then(|distance| segment_cull_distance_squared(segment, distance)),
                strip: ChordStrip::of(segment, map, translate, reach),
            });
        }
        slab
    }

    #[must_use]
    pub(crate) fn slab(&self) -> [f64; 4] {
        self.slab
    }

    /// The x-interval of the pixel row `[y0, y0 + 1]` outside which no point
    /// receives coverage from this stroke: the hull, over segments whose slab
    /// meets the row, of each segment's slab x-range cut down to its
    /// [`ChordStrip`]. `None` when no segment meets the row.
    ///
    /// Shading only the pixels whose cell meets this interval changes no
    /// bit: every skipped point is beyond every segment's reach, so its
    /// coverage — and any subcell probe of its cell — is exactly zero.
    #[must_use]
    pub(crate) fn row_span(&self, y0: f64) -> Option<(f64, f64)> {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for prepared in self.segments {
            let slab = prepared.slab;
            if slab[3] < y0 || slab[1] > y0 + 1.0 {
                continue;
            }
            let (mut a, mut b) = (slab[0], slab[2]);
            if let Some((strip_lo, strip_hi)) =
                prepared.strip.and_then(|strip| strip.row_interval(y0))
            {
                a = a.max(strip_lo);
                b = b.min(strip_hi);
            }
            if a <= b {
                lo = lo.min(a);
                hi = hi.max(b);
            }
        }
        (lo <= hi).then_some((lo, hi))
    }

    /// The segments [`Self::nearest`] has to visit for `p`, in shape order.
    ///
    /// Everything outside the returned range fails the slab test, so skipping
    /// it changes no state. For a stroke [`segments_are_x_sorted`] accepted, a
    /// `p` outside the aggregate's y-range is outside every segment's, and the
    /// rest of the test's x half is a contiguous run: `slab[2] < x` holds on a
    /// prefix and `slab[0] <= x` on a prefix. The run is visited in its
    /// original order, so the earliest segment still wins a tie.
    ///
    /// A `NaN` coordinate fails both comparisons of the slab test and is
    /// admitted by every segment, so it keeps the whole range.
    fn x_candidates(&self, p: [f64; 2]) -> std::ops::Range<usize> {
        if !(self.x_sorted && p[0].is_finite() && p[1].is_finite()) {
            return 0..self.segments.len();
        }
        if p[1] < self.slab[1] || p[1] > self.slab[3] {
            return 0..0;
        }
        let first = self
            .segments
            .partition_point(|prepared| prepared.slab[2] < p[0]);
        let count = self.segments[first..].partition_point(|prepared| prepared.slab[0] <= p[0]);
        first..first + count
    }

    fn nearest(
        &self,
        segments: &[Segment],
        style: &Style,
        map: ScreenMap,
        translate: [f64; 2],
        p: [f64; 2],
    ) -> Option<(f64, f64)> {
        let scale = map.scale;
        if scale == 0.0 || segments.is_empty() || segments.len() != self.segments.len() {
            return None;
        }
        let candidates = self.x_candidates(p);
        if candidates.is_empty() {
            return None;
        }
        let [ox, oy] = map.to_object(p, translate);
        let obj = [ox, oy, 0.0];
        let mut best = f64::INFINITY;
        let mut best_s = 0.0;
        let mut admitted = false;
        for (segment, prepared) in segments[candidates.clone()]
            .iter()
            .zip(self.segments[candidates].iter())
        {
            if p[0] < prepared.slab[0]
                || p[0] > prepared.slab[2]
                || p[1] < prepared.slab[1]
                || p[1] > prepared.slab[3]
            {
                continue;
            }
            admitted = true;
            let (excess, s) = if prepared.line {
                line_segment_excess_and_s(segment, style, map, obj)
            } else if prepared.station {
                segment_excess_and_s(segment, prepared.total_arc_length, style, map, obj)
            } else if prepared
                .cull_distance_squared
                .is_some_and(|threshold| hull_distance_squared(segment, [ox, oy]) >= threshold)
            {
                (zero_coverage_excess(style), segment.s0)
            } else {
                uniform_segment_excess(segment, style, map, obj)
            };
            if excess < best {
                best = excess;
                best_s = s;
            }
        }
        admitted.then_some((best, best_s))
    }

    #[must_use]
    pub(crate) fn shade(
        &self,
        segments: &[Segment],
        joins: &[JoinWedge],
        style: &Style,
        map: ScreenMap,
        translate: [f64; 2],
        p: [f64; 2],
    ) -> (f64, f64) {
        if style.maximum_stroke_width() <= 0.0 {
            return (0.0, 0.0);
        }
        match self.nearest(segments, style, map, translate, p) {
            Some((round, s)) => (
                aa_coverage(
                    apply_joins(round, joins, style.joint_type, p),
                    f64::from(style.anti_alias_width),
                ),
                s,
            ),
            None => (0.0, 0.0),
        }
    }
}

/// Maximum screen-space Hausdorff error admitted by the line fast path.
///
/// One quarter of a milli-pixel is far below the raw half-float surface's
/// useful spatial resolution, while remaining explicit and invariant under
/// scene scale. Semantic `Line`/`Polyline` hints bypass the numeric test because
/// they are the authoritative construction identity; writable point mutation
/// invalidates that hint before the frame job is built.
#[cfg(any(feature = "metal", test))]
pub(crate) const LINE_APPROXIMATION_MAX_ERROR_PX: f64 = 1.0 / 4096.0;

/// Whether a segment may use capsule distance without a visible geometry move.
#[cfg(any(feature = "metal", test))]
pub(crate) fn line_approximation_admitted(
    segment: &Segment,
    map: ScreenMap,
    semantic_line: bool,
) -> bool {
    if semantic_line {
        return true;
    }
    let chord = [segment.p2[0] - segment.p0[0], segment.p2[1] - segment.p0[1]];
    let handle = [segment.p1[0] - segment.p0[0], segment.p1[1] - segment.p0[1]];
    let chord_squared = chord[0] * chord[0] + chord[1] * chord[1];
    if chord_squared == 0.0 {
        return handle[0] == 0.0 && handle[1] == 0.0;
    }
    let projection = (handle[0] * chord[0] + handle[1] * chord[1]) / chord_squared;
    if !(0.0..=1.0).contains(&projection) {
        return false;
    }
    let twice_area = (chord[0] * handle[1] - chord[1] * handle[0]).abs();
    let maximum_object_error = 0.5 * twice_area / chord_squared.sqrt();
    maximum_object_error * map.scale.abs() <= LINE_APPROXIMATION_MAX_ERROR_PX
}

fn line_segment_excess_and_s(
    segment: &Segment,
    style: &Style,
    map: ScreenMap,
    object_point: fmn_core::types::Vec3,
) -> (f64, f64) {
    let chord = [
        segment.p2[0] - segment.p0[0],
        segment.p2[1] - segment.p0[1],
        segment.p2[2] - segment.p0[2],
    ];
    let delta = [
        object_point[0] - segment.p0[0],
        object_point[1] - segment.p0[1],
        object_point[2] - segment.p0[2],
    ];
    let denominator = dot3(chord, chord);
    let t = if denominator > 0.0 {
        (dot3(delta, chord) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let nearest_delta = [
        delta[0] - chord[0] * t,
        delta[1] - chord[1] * t,
        delta[2] - chord[2] * t,
    ];
    let s = segment.s0 + (segment.s1 - segment.s0) * t;
    let s = style.stroke_endpoint_parameter(s, t == 1.0);
    let distance = dot3(nearest_delta, nearest_delta).sqrt();
    (distance * map.scale.abs() - half_width_px(style, map, s), s)
}

fn dot3(a: fmn_core::types::Vec3, b: fmn_core::types::Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The stroke's **signed excess** at a screen point, in pixels: negative inside
/// the stroke, zero on its silhouette, positive outside.
///
/// The single quantity the stroke is made of. Queried in **object space** and
/// scaled, rather than by projecting the segments: the map is a uniform scale plus
/// a translation, so it preserves which point is nearest and multiplies the
/// distance by the scale — and doing it this way means the renderer holds no
/// second copy of the geometry (D-04).
///
/// The excess is formed **per segment before the minimum**, which is what makes a
/// variable-width stroke a union of variable-radius tubes rather than one tube of
/// some averaged radius. Taking `min(distance) − half_width(s_of_that_min)`
/// instead would use one segment's width at another segment's nearest point and
/// would give a different, wrong silhouette wherever the ramp is steep.
///
/// `None` for a path with no segments or a degenerate map.
#[must_use]
pub fn stroke_excess_px(
    segments: &[Segment],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> Option<f64> {
    stroke_nearest(segments, style, map, translate, p).map(|(excess, _)| excess)
}

/// [`stroke_excess_px`], plus the **arc-length coordinate of the winning
/// segment's nearest point**.
///
/// The coordinate is not a diagnostic: [`stroke_rgba_at`] and
/// [`half_width_px`] are both functions of it, and §10.3 says width and colour
/// interpolate by the *same* arc length. Nothing produced it until an engine
/// needed to shade a pixel, so the ramp existed with no way to evaluate it —
/// which is why this is the primitive and [`stroke_excess_px`] is the
/// projection, rather than the loop being written twice and kept in agreement
/// by hand.
///
/// Ties go to the **earliest segment in the shape's own segment order**, which
/// is a property of the geometry rather than of the sync that compiled it
/// (ADR-0013): `excess < best` keeps the incumbent, so the winner is
/// order-determined only where two segments are exactly equidistant, and that
/// order is the outline's.
///
/// `None` for a path with no segments or a degenerate map.
#[must_use]
pub fn stroke_nearest(
    segments: &[Segment],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> Option<(f64, f64)> {
    let scale = map.scale;
    if scale == 0.0 || segments.is_empty() {
        return None;
    }
    let [ox, oy] = map.to_object(p, translate);
    let obj = [ox, oy, 0.0];
    let mut best = f64::INFINITY;
    let mut best_s = 0.0;
    for g in segments {
        let total = fmn_geom::arclength::quadratic_arc_length(g.p0, g.p1, g.p2);
        let (excess, s) = segment_excess_and_s(g, total, style, map, obj);
        if excess < best {
            best = excess;
            best_s = s;
        }
    }
    Some((best, best_s))
}

/// The smallest excess [`aa_coverage`] maps to exactly zero coverage: half the
/// AA band, through the same band substitution `aa_coverage` makes.
/// `excess / aa >= 0.5` holds for every `excess` at or above it, because the
/// halving is exact and correctly rounded division is monotone.
fn zero_coverage_excess(style: &Style) -> f64 {
    let aa_width_px = f64::from(style.anti_alias_width);
    let aa = if aa_width_px > 0.0 { aa_width_px } else { 1e-8 };
    0.5 * aa
}

/// The object-space distance at and beyond which a segment of a **uniform**
/// stroke cannot change a pixel's shade.
///
/// Such a segment's excess is at least [`zero_coverage_excess`], so it either
/// loses the minimum to a nearer segment or leaves a minimum that `aa_coverage`
/// sends to zero. [`apply_joins`] is a chain of `min`/`max` steps whose result
/// for any input at or above that excess is either at or above it too (zero
/// coverage) or an input-independent wedge value — so the excess can be replaced
/// by any value at or above the threshold without moving a bit. The station is
/// unobservable (one exact width and colour), so the replaced station cannot
/// move a bit either.
///
/// `None` (never cull) for a degenerate map or a non-finite bound.
fn uniform_cull_distance(style: &Style, map: ScreenMap) -> Option<f64> {
    let scale = map.scale.abs();
    if !(scale > 0.0 && scale.is_finite()) {
        return None;
    }
    let distance =
        ((zero_coverage_excess(style) + half_width_px(style, map, 0.0)) / scale).max(0.0);
    distance.is_finite().then_some(distance)
}

/// [`uniform_cull_distance`] widened for one segment's rounding, and squared.
///
/// The computed nearest distance can undercut the exact distance to the hull by
/// a few units in the last place of the coordinates involved. The widening —
/// one part in 10⁹ of the distance plus 10⁻⁹ of the coordinate magnitude, many
/// orders above that rounding — keeps the lower bound strictly conservative.
fn segment_cull_distance_squared(segment: &Segment, distance: f64) -> Option<f64> {
    let magnitude = [segment.p0, segment.p1, segment.p2]
        .iter()
        .flat_map(|p| [p[0].abs(), p[1].abs()])
        .fold(0.0f64, f64::max);
    let widened = distance * (1.0 + 1e-9) + 1e-9 * (1.0 + magnitude);
    let squared = widened * widened;
    squared.is_finite().then_some(squared)
}

/// Squared planar distance from `q` to a segment's control triangle: zero on or
/// inside it, and a degenerate (collinear) triangle measures as its edges.
///
/// A quadratic Bézier lies in its control triangle, and dropping `z` can only
/// shorten a distance, so this is a lower bound on the distance
/// [`fmn_geom::distance::nearest_on_quadratic`] measures. Any `NaN` reads as
/// inside (zero), which never culls.
fn hull_distance_squared(segment: &Segment, q: [f64; 2]) -> f64 {
    let a = [segment.p0[0], segment.p0[1]];
    let b = [segment.p1[0], segment.p1[1]];
    let c = [segment.p2[0], segment.p2[1]];
    let cross =
        |o: [f64; 2], u: [f64; 2]| (u[0] - o[0]) * (q[1] - o[1]) - (u[1] - o[1]) * (q[0] - o[0]);
    let sides = [cross(a, b), cross(b, c), cross(c, a)];
    let negative = sides.iter().any(|side| *side < 0.0);
    let positive = sides.iter().any(|side| *side > 0.0);
    if !(negative && positive) {
        return 0.0;
    }
    edge_distance_squared(a, b, q)
        .min(edge_distance_squared(b, c, q))
        .min(edge_distance_squared(c, a, q))
}

fn edge_distance_squared(a: [f64; 2], b: [f64; 2], q: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let aq = [q[0] - a[0], q[1] - a[1]];
    let length_squared = dot2(ab, ab);
    let t = if length_squared > 0.0 {
        (dot2(aq, ab) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = [aq[0] - t * ab[0], aq[1] - t * ab[1]];
    dot2(d, d)
}

/// [`segment_excess_and_s`] for a stroke whose station is unobservable: the
/// same nearest-point solve and the same excess, without the arc-length
/// station. The width and colour are exactly equal at every station of such a
/// stroke, so the returned station (the segment's start) shades identically.
fn uniform_segment_excess(
    segment: &Segment,
    style: &Style,
    map: ScreenMap,
    object_point: fmn_core::types::Vec3,
) -> (f64, f64) {
    let near =
        fmn_geom::distance::nearest_on_quadratic(segment.p0, segment.p1, segment.p2, object_point);
    let s = segment.s0;
    let excess = near.distance * map.scale.abs() - half_width_px(style, map, s);
    (excess, s)
}

fn segment_excess_and_s(
    segment: &Segment,
    total_arc_length: f64,
    style: &Style,
    map: ScreenMap,
    object_point: fmn_core::types::Vec3,
) -> (f64, f64) {
    let near =
        fmn_geom::distance::nearest_on_quadratic(segment.p0, segment.p1, segment.p2, object_point);
    // Arc length at the nearest point, not the parameter: the ramp is
    // arc-length-parameterized (BN-03), and `t` and `s` differ by exactly the
    // amount that note exists to talk about.
    let frac = if total_arc_length > 0.0 {
        let sub =
            fmn_geom::bezier::partial_quadratic(&[segment.p0, segment.p1, segment.p2], 0.0, near.t);
        (fmn_geom::arclength::quadratic_arc_length(sub[0], sub[1], sub[2]) / total_arc_length)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let s = segment.s0 + (segment.s1 - segment.s0) * frac;
    let s = style.stroke_endpoint_parameter(s, near.t == 1.0);
    let excess = near.distance * map.scale.abs() - half_width_px(style, map, s);
    (excess, s)
}

/// The stroke's coverage **and** its ramp coordinate at a screen point — the
/// engine's per-pixel stroke shading call.
///
/// [`stroke_coverage_with_joins`] is this with the coordinate dropped. Both
/// numbers come from one pass over the segments, because computing the coverage
/// and then re-solving for the colour would evaluate every nearest-point solve
/// twice and could disagree with itself on a tie.
#[must_use]
pub fn stroke_shade(
    segments: &[Segment],
    joins: &[JoinWedge],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> (f64, f64) {
    if style.maximum_stroke_width() <= 0.0 {
        return (0.0, 0.0);
    }
    match stroke_nearest(segments, style, map, translate, p) {
        Some((round, s)) => (
            aa_coverage(
                apply_joins(round, joins, style.joint_type, p),
                f64::from(style.anti_alias_width),
            ),
            s,
        ),
        None => (0.0, 0.0),
    }
}

/// The stroke's coverage at a screen point, in `[0, 1]`.
///
/// [`stroke_excess_px`] through [`aa_coverage`]. Round caps and round joins are
/// properties of the excess, so they need no argument and cannot be switched off
/// by accident; the explicit joint overrides are [`JoinWedge`]'s.
#[must_use]
pub fn stroke_coverage(
    segments: &[Segment],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> f64 {
    // A zero-width ramp draws nothing, and it is worth short-circuiting rather
    // than letting the AA band paint a hairline where the author asked for
    // nothing at all.
    if style.maximum_stroke_width() <= 0.0 {
        return 0.0;
    }
    match stroke_excess_px(segments, style, map, translate, p) {
        Some(excess) => aa_coverage(excess, f64::from(style.anti_alias_width)),
        None => 0.0,
    }
}

// ------------------------------------------------------------------- the joins

/// The Reference's **de facto** miter limit, in half-widths.
///
/// There is no `miter_limit` constant in the Reference to copy. G0-2 finding L6
/// derived it: `auto` crosses over at `cos θ = −0.8` (θ = 143.13°), so its worst
/// admissible extension is `|shift| = tan(71.565°) = 3.0`, i.e.
/// `√(1 + 9) = 3.1623` half-widths of miter length. That is the number §10.3 asked
/// this bead to adopt.
pub const MITER_LIMIT: f64 = 3.162_277_660_168_379_5;

/// One interior anchor's join geometry, in screen pixels.
///
/// The **outer** wedge at a corner: the region whose nearest point on the path is
/// the shared anchor itself. Everything a non-round join needs is here, and
/// nothing a round join needs is — a round join is what the distance field
/// already produces, so [`join_wedges`] returns nothing for it and the round path
/// is untouched by construction rather than by care.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JoinWedge {
    /// The shared anchor.
    pub anchor: [f64; 2],
    /// Unit tangent arriving at the anchor.
    pub t_in: [f64; 2],
    /// Unit tangent leaving it.
    pub t_out: [f64; 2],
    /// Half-width in pixels at this anchor.
    pub half_width: f64,
    /// Unit bisector pointing **into** the wedge.
    pub bisector: [f64; 2],
    /// Outward normal of the incoming segment's edge, pointing into the wedge.
    pub n_in: [f64; 2],
    /// Outward normal of the outgoing segment's edge.
    pub n_out: [f64; 2],
}

fn dot2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn norm2(a: [f64; 2]) -> f64 {
    dot2(a, a).sqrt()
}

fn unit2(a: [f64; 2]) -> Option<[f64; 2]> {
    let n = norm2(a);
    if n <= 0.0 {
        None
    } else {
        Some([a[0] / n, a[1] / n])
    }
}

impl JoinWedge {
    /// Is this point in the outer wedge — i.e. is the anchor its nearest point on
    /// the path?
    ///
    /// Inclusive on both half-planes on purpose: the anchor itself must belong to
    /// its own wedge, or a pixel centre landing exactly on a corner would fall
    /// through every region.
    #[must_use]
    pub fn contains(&self, p: [f64; 2]) -> bool {
        let d = [p[0] - self.anchor[0], p[1] - self.anchor[1]];
        dot2(d, self.t_in) >= 0.0 && dot2(d, self.t_out) <= 0.0
    }

    /// The miter length in half-widths: `1 / cos φ`, with `φ` the half-angle
    /// between an edge normal and the bisector.
    ///
    /// `∞` for a reversal (a 180° cusp), where the two edges are antiparallel and
    /// no finite point is on both offset lines — which is precisely the case
    /// [`MITER_LIMIT`] exists to catch.
    #[must_use]
    pub fn miter_ratio(&self) -> f64 {
        let c = dot2(self.n_in, self.bisector);
        if c <= 0.0 { f64::INFINITY } else { 1.0 / c }
    }

    /// The bevel's signed excess: distance past the chord joining the two offset
    /// points.
    ///
    /// The chord sits at `half_width · cos φ` from the anchor along the bisector,
    /// because it passes through `anchor + half_width · n_in`.
    #[must_use]
    pub fn bevel_excess(&self, p: [f64; 2]) -> f64 {
        let d = [p[0] - self.anchor[0], p[1] - self.anchor[1]];
        dot2(d, self.bisector) - self.half_width * dot2(self.n_in, self.bisector)
    }

    /// The miter's signed excess: the intersection of the two offset half-planes.
    #[must_use]
    pub fn miter_excess(&self, p: [f64; 2]) -> f64 {
        let d = [p[0] - self.anchor[0], p[1] - self.anchor[1]];
        (dot2(d, self.n_in) - self.half_width).max(dot2(d, self.n_out) - self.half_width)
    }
}

/// The join wedges a path needs for the given joint type.
///
/// Empty for [`fmn_mobject::JointType::Auto`] and
/// [`fmn_mobject::JointType::NoJoint`], both of which render
/// the round join the distance field already produces (ADR-0012). Empty is not a
/// shortcut: it is what makes the round path bit-identical whether or not the
/// join machinery exists.
///
/// `subpath_starts` is [`crate::table::Shape::subpath_starts`], relative to this
/// slice. Joins are formed only *within* a subpath, and only between segments
/// that actually share an anchor — plus the wrap join of a closed subpath, which
/// is a corner like any other and would otherwise be the one place a closed
/// outline showed a round join under a miter setting.
#[must_use]
pub fn join_wedges(
    segments: &[Segment],
    subpath_starts: &[u32],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
) -> Vec<JoinWedge> {
    let mut out = Vec::new();
    let mut pairs = crate::arena::Pool::default();
    join_wedges_into(
        &mut out,
        &mut pairs,
        segments,
        subpath_starts,
        style,
        map,
        translate,
    );
    out
}

/// [`join_wedges`] into caller-owned storage — the engine's per-frame path.
///
/// `out` receives the wedges in exactly the order [`join_wedges`] produces
/// them; `pairs` is per-subpath corner-index scratch, cleared on every use,
/// so the arena's pooled copy allocates only until the widest subpath has
/// been seen. Identical arithmetic; only the destinations differ.
pub(crate) fn join_wedges_into(
    out: &mut impl crate::arena::Sink<JoinWedge>,
    pairs: &mut crate::arena::Pool<(usize, usize)>,
    segments: &[Segment],
    subpath_starts: &[u32],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
) {
    if matches!(
        style.joint_type,
        fmn_mobject::JointType::Auto | fmn_mobject::JointType::NoJoint
    ) {
        return;
    }
    let to_px = |p: fmn_core::types::Vec3| {
        let q = map.to_pixel(p[0], p[1]);
        [q[0] + translate[0], q[1] + translate[1]]
    };
    // A quadratic's tangents are `2(p1 − p0)` and `2(p2 − p1)`; a coincident
    // handle leaves one of them zero, so the chord stands in.
    let tan_in = |g: &Segment| {
        let a = to_px(g.p1);
        let b = to_px(g.p2);
        unit2([b[0] - a[0], b[1] - a[1]]).or_else(|| {
            let z = to_px(g.p0);
            unit2([b[0] - z[0], b[1] - z[1]])
        })
    };
    let tan_out = |g: &Segment| {
        let a = to_px(g.p0);
        let b = to_px(g.p1);
        unit2([b[0] - a[0], b[1] - a[1]]).or_else(|| {
            let z = to_px(g.p2);
            unit2([z[0] - a[0], z[1] - a[1]])
        })
    };

    for (k, &start) in subpath_starts.iter().enumerate() {
        let end = subpath_starts
            .get(k + 1)
            .copied()
            .unwrap_or(segments.len() as u32);
        let (a, b) = (
            (start as usize).min(segments.len()),
            (end as usize).min(segments.len()),
        );
        if b <= a {
            continue;
        }
        let sub = &segments[a..b];
        // Interior corners, then the wrap corner if the subpath closes.
        pairs.clear();
        for i in 0..sub.len().saturating_sub(1) {
            pairs.put((i, i + 1));
        }
        let first = sub[0];
        let last = sub[sub.len() - 1];
        if sub.len() > 1 && to_px(last.p2) == to_px(first.p0) {
            pairs.put((sub.len() - 1, 0));
        }
        for &(i, j) in pairs.iter() {
            let (Some(t_in), Some(t_out)) = (tan_in(&sub[i]), tan_out(&sub[j])) else {
                continue;
            };
            // The wedge's bisector points along `t_in − t_out`. A smooth join
            // leaves that vector zero and has no wedge to draw.
            let Some(bisector) = unit2([t_in[0] - t_out[0], t_in[1] - t_out[1]]) else {
                continue;
            };
            let pick = |t: [f64; 2]| {
                let cand = [-t[1], t[0]];
                if dot2(cand, bisector) > 0.0 {
                    cand
                } else {
                    [t[1], -t[0]]
                }
            };
            out.put(JoinWedge {
                anchor: to_px(sub[i].p2),
                t_in,
                t_out,
                half_width: half_width_px(style, map, sub[i].s1),
                bisector,
                n_in: pick(t_in),
                n_out: pick(t_out),
            });
        }
    }
}

/// Apply the joint override to a round excess at one point.
///
/// The round join is the baseline and the override *edits* it, which is why this
/// takes an excess rather than producing one:
///
/// - **Bevel** trims. Its region is a subset of the round join's, so the edit is
///   `max` with the chord's half-plane — coverage can only be removed, and only
///   inside the wedge.
/// - **Miter** extends. Its region contains the round join's near the tip, so the
///   edit is `min` with the two offset half-planes. Over [`MITER_LIMIT`] it falls
///   back to the bevel, which is the classical behaviour and the one G0-2's
///   measured limit was derived for.
///
/// Which numeric code selects which corner is ADR-0012's ruling: **our names mean
/// what they say**, so `Bevel` cuts flat and `Miter` comes to a point. The
/// Reference's constants are swapped relative to the geometry its own shader
/// produces (G0-2 L6), and reproducing a misnamed constant is not compatibility.
#[must_use]
pub fn apply_joins(
    round_excess: f64,
    joins: &[JoinWedge],
    joint: fmn_mobject::JointType,
    p: [f64; 2],
) -> f64 {
    use fmn_mobject::JointType;
    if joins.is_empty() {
        return round_excess;
    }
    let mut excess = round_excess;
    for w in joins {
        if !w.contains(p) {
            continue;
        }
        match joint {
            JointType::Bevel => excess = excess.max(w.bevel_excess(p)),
            JointType::Miter => {
                if w.miter_ratio() > MITER_LIMIT {
                    excess = excess.max(w.bevel_excess(p));
                } else {
                    excess = excess.min(w.miter_excess(p));
                }
            }
            JointType::Auto | JointType::NoJoint => {}
        }
    }
    excess
}

/// The stroke's coverage at a screen point, with the joint override applied.
///
/// [`stroke_coverage`] is this with no joins, and for
/// [`JointType::Auto`](fmn_mobject::JointType::Auto) the two are the same number:
/// [`join_wedges`] returns nothing, so nothing is edited.
#[must_use]
pub fn stroke_coverage_with_joins(
    segments: &[Segment],
    joins: &[JoinWedge],
    style: &Style,
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> f64 {
    stroke_shade(segments, joins, style, map, translate, p).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hint::Hint;
    use crate::table::{compile_shape, shape_digest};
    use fmn_core::types::Vec3;
    use fmn_geom::quadpath::QuadPath;

    fn unit() -> ScreenMap {
        ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        }
    }

    fn segs_of(path: &QuadPath) -> Vec<Segment> {
        compile_shape(shape_digest(path.points()), path, Hint::General, 0)
            .expect("fixture fits retained table widths")
            .1
    }

    /// Own the prepared-segment backing for a [`PreparedStroke`] view.
    fn prepared(
        segments: &[Segment],
        style: &Style,
        map: ScreenMap,
        translate: [f64; 2],
        straight_segments: bool,
    ) -> (Vec<PreparedSegment>, [f64; 4]) {
        let mut backing = Vec::new();
        let slab = PreparedStroke::prepare_into(
            &mut backing,
            segments,
            style,
            map,
            translate,
            straight_segments,
        );
        (backing, slab)
    }

    /// A style whose stroke is `width` units wide, constant, with the calibrated
    /// AA band.
    fn flat_stroke_style(width: f32) -> Style {
        Style {
            stroke_width: width,
            stroke_width_end: width,
            stroke_rgba: [1.0, 1.0, 1.0, 1.0],
            stroke_rgba_end: [1.0, 1.0, 1.0, 1.0],
            anti_alias_width: 1.5,
            ..Style::default()
        }
    }

    /// A horizontal line from `(0,0)` to `(len,0)`, as one straight quadratic.
    fn line(len: f64) -> QuadPath {
        let mut p = QuadPath::default();
        p.start_new_path([0.0, 0.0, 0.0]);
        p.add_line_to([len, 0.0, 0.0], false).unwrap();
        p
    }

    #[test]
    fn the_line_route_is_bounded_and_rejects_curves_and_overshoots() {
        let segment = |p0, p1, p2| Segment {
            p0,
            p1,
            p2,
            s0: 0.0,
            s1: 1.0,
        };
        let map = ScreenMap {
            scale: 60.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        let quantized_line = segment(
            [0.0, 0.0, 0.0],
            [0.009_999_990_463_256_836, 0.199_999_928_474_426_27, 0.0],
            [0.019_999_980_926_513_672, 0.399_999_976_158_142_1, 0.0],
        );
        assert!(line_approximation_admitted(&quantized_line, map, false));
        assert!(
            line_approximation_admitted(
                &segment([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0],),
                map,
                true,
            ),
            "a valid semantic line hint is authoritative"
        );
        assert!(!line_approximation_admitted(
            &segment([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0],),
            map,
            false,
        ));
        assert!(!line_approximation_admitted(
            &segment([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 0.0, 0.0],),
            map,
            false,
        ));
    }

    #[test]
    fn a_quantized_polyline_uses_capsule_distance() {
        let segments = [
            Segment {
                p0: [0.0, 0.0, 0.0],
                p1: [0.009_999_990_463_256_836, 0.199_999_928_474_426_27, 0.0],
                p2: [0.019_999_980_926_513_672, 0.399_999_976_158_142_1, 0.0],
                s0: 0.0,
                s1: 0.5,
            },
            Segment {
                p0: [0.019_999_980_926_513_672, 0.399_999_976_158_142_1, 0.0],
                p1: [0.029_999_971_389_770_508, 0.199_999_928_474_426_27, 0.0],
                p2: [0.040_000_021_457_672_12, 0.0, 0.0],
                s0: 0.5,
                s1: 0.95,
            },
        ];
        let map = ScreenMap {
            scale: 60.0,
            origin: [160.0, 90.0],
            y_up: false,
        };
        let style = flat_stroke_style(14.0);
        let translate = [40.8, -84.0];
        let (backing, slab) = prepared(&segments, &style, map, translate, true);
        let prepared = PreparedStroke::from_parts(&backing, slab);
        let (coverage, _) = prepared.shade(&segments, &[], &style, map, translate, [206.5, 10.5]);
        assert!(coverage > 0.99, "{coverage}");
    }

    /// GH #1: a `set_points_as_corners` segment with no semantic line hint
    /// takes the curve route, and its f32-rounded midpoint handle left the
    /// nearest-point cubic ill-conditioned: the interior of a segment longer
    /// than the stroke is wide read as uncovered. Width 7 at 1280×720's
    /// 90 px/unit, sampled along the whole centerline, must be solid ink —
    /// through both the unprepared and the prepared (engine) kernels.
    #[test]
    fn a_rounded_corner_polyline_segment_is_covered_along_its_whole_length() {
        let segments = [Segment {
            p0: [0.440_766_543_149_948_1, 1.937_205_195_426_941, 0.0],
            p1: [0.459_930_300_712_585_45, 1.850_354_909_896_850_6, 0.0],
            p2: [0.479_094_088_077_545_17, 1.763_504_743_576_049_8, 0.0],
            s0: 0.0,
            s1: 1.0,
        }];
        let map = ScreenMap::y_up(90.0, [640.0, 360.0]);
        let style = flat_stroke_style(7.0);
        let translate = [0.0, 0.0];
        let (backing, slab) = prepared(&segments, &style, map, translate, false);
        let prepared = PreparedStroke::from_parts(&backing, slab);
        let [a, _, b] = [segments[0].p0, segments[0].p1, segments[0].p2];
        for k in 0..=64 {
            let t = f64::from(k) / 64.0;
            let object = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            let screen = map.to_pixel(object[0], object[1]);
            let direct = stroke_coverage(&segments, &style, map, translate, screen);
            assert_eq!(direct, 1.0, "centerline t={t} uncovered: {direct}");
            let (engine, _) = prepared.shade(&segments, &[], &style, map, translate, screen);
            assert_eq!(engine, 1.0, "prepared centerline t={t} uncovered: {engine}");
        }
    }

    #[test]
    fn the_width_conversion_is_the_references_own() {
        let map = ScreenMap {
            scale: 135.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        assert!((width_px(1.0, map) - 1.35).abs() < 1e-12);
        assert!(
            (width_px(4.0, map) - 5.4).abs() < 1e-12,
            "DEFAULT_STROKE_WIDTH"
        );
        assert_eq!(width_px(0.0, map), 0.0);
        // And it is the same function §10.2's inner border uses.
        assert_eq!(width_px(0.5, map), crate::fill::border_width_px(0.5, map));
    }

    #[test]
    fn the_aa_profile_is_the_measured_smoothstep() {
        // G0-2 L1, pinned at the points that identify the curve rather than
        // merely sample it.
        let aa = 1.5;
        assert_eq!(aa_coverage(-aa, aa), 1.0, "well inside");
        assert!(
            (aa_coverage(0.0, aa) - 0.5).abs() < 1e-12,
            "on the silhouette"
        );
        assert_eq!(aa_coverage(aa, aa), 0.0, "beyond the band");
        // Hermite, not linear: at a quarter of the band the linear answer would
        // be 0.75 and the Hermite is 0.84375.
        let quarter = aa_coverage(-0.25 * aa, aa);
        assert!((quarter - 0.843_75).abs() < 1e-12, "{quarter}");
        // Monotone, and symmetric about the silhouette.
        let mut last = 1.0;
        for k in 0..=20 {
            let e = -aa + 2.0 * aa * f64::from(k) / 20.0;
            let c = aa_coverage(e, aa);
            assert!(c <= last + 1e-12, "not monotone at {e}");
            last = c;
        }
        for k in 1..=10 {
            let e = aa * f64::from(k) / 20.0;
            assert!(
                (aa_coverage(-e, aa) + aa_coverage(e, aa) - 1.0).abs() < 1e-12,
                "not symmetric at {e}"
            );
        }
    }

    #[test]
    fn a_straight_stroke_has_the_width_it_was_asked_for() {
        // The simplest silhouette check: a 100-unit-wide stroke at scale 1 is
        // 1 px of half-width, so coverage is 1/2 exactly one pixel off the axis.
        let path = line(40.0);
        let segs = segs_of(&path);
        let style = flat_stroke_style(200.0); // 2 px wide, 1 px half-width
        let map = unit();
        assert!((half_width_px(&style, map, 0.0) - 1.0).abs() < 1e-12);

        // On the axis: fully covered.
        assert_eq!(
            stroke_coverage(&segs, &style, map, [0.0, 0.0], [20.0, 0.0]),
            1.0
        );
        // On the silhouette: half.
        let edge = stroke_coverage(&segs, &style, map, [0.0, 0.0], [20.0, 1.0]);
        assert!((edge - 0.5).abs() < 1e-9, "{edge}");
        // Beyond the band: nothing.
        assert_eq!(
            stroke_coverage(&segs, &style, map, [0.0, 0.0], [20.0, 1.0 + 1.5]),
            0.0
        );
    }

    #[test]
    fn open_ends_are_round_caps_and_not_butt_caps() {
        // The distance to an open end is radial, so the level set is a semicircle
        // with nobody asking for one. A butt cap would read zero coverage
        // immediately past the endpoint; a round cap reads the same value at the
        // same distance in every direction.
        let path = line(40.0);
        let segs = segs_of(&path);
        let style = flat_stroke_style(400.0); // 4 px wide, 2 px half-width
        let map = unit();
        let end = [40.0, 0.0];

        // Straight out past the end, at half the half-width: covered.
        assert_eq!(
            stroke_coverage(&segs, &style, map, [0.0, 0.0], [end[0] + 1.0, end[1]]),
            1.0,
            "a butt cap would be empty here"
        );
        // The cap is a circle: every direction at radius 2 sits on the silhouette.
        for k in 0..12 {
            let a = std::f64::consts::TAU * f64::from(k) / 12.0;
            // Only the outward half is the cap; the inward half is the stroke body.
            if a.cos() <= 0.0 {
                continue;
            }
            let p = [end[0] + 2.0 * a.cos(), end[1] + 2.0 * a.sin()];
            let c = stroke_coverage(&segs, &style, map, [0.0, 0.0], p);
            assert!(
                (c - 0.5).abs() < 1e-9,
                "cap is not circular at angle {}: {c}",
                a.to_degrees()
            );
        }
    }

    #[test]
    fn a_corner_is_a_round_join_with_no_notch_and_no_gap() {
        // G0-2 L6: two segments sharing an anchor contribute two distance fields
        // whose minimum is exactly a round join. The failure modes this excludes
        // are a NOTCH (coverage dipping at the outer corner) and a GAP (coverage
        // dipping anywhere on the join arc).
        let mut p = QuadPath::default();
        p.start_new_path([0.0, 20.0, 0.0]);
        p.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([20.0, 0.0, 0.0], false).unwrap();
        let segs = segs_of(&p);
        let style = flat_stroke_style(600.0); // 6 px wide, 3 px half-width
        let map = unit();

        // The outer corner is the third quadrant from the vertex. Sweep the join
        // arc at 3 px: every sample must sit on the silhouette, so the outer
        // boundary is a circular arc rather than a mitre or a cut.
        for k in 0..=16 {
            let a = std::f64::consts::PI + std::f64::consts::FRAC_PI_2 * f64::from(k) / 16.0;
            let q = [3.0 * a.cos(), 3.0 * a.sin()];
            let c = stroke_coverage(&segs, &style, map, [0.0, 0.0], q);
            assert!(
                (c - 0.5).abs() < 1e-9,
                "join is not round at {} deg: {c}",
                a.to_degrees()
            );
        }
        // And just inside that arc there is no gap.
        for k in 0..=16 {
            let a = std::f64::consts::PI + std::f64::consts::FRAC_PI_2 * f64::from(k) / 16.0;
            let q = [2.0 * a.cos(), 2.0 * a.sin()];
            assert_eq!(
                stroke_coverage(&segs, &style, map, [0.0, 0.0], q),
                1.0,
                "gap in the join at {} deg",
                a.to_degrees()
            );
        }
    }

    #[test]
    fn reparameterizing_a_curve_leaves_the_stroke_identical() {
        // §10.3's metamorphic law, and the reason width interpolates by ARC
        // LENGTH. A de Casteljau split changes the point array and the
        // parameterization; it does not change the curve, so it must not change
        // one pixel of the stroke. A parameter-space width ramp fails this.
        let mut path = QuadPath::default();
        path.start_new_path([0.0, 0.0, 0.0]);
        path.add_quadratic_bezier_curve_to([15.0, 25.0, 0.0], [30.0, 0.0, 0.0], false)
            .unwrap();

        let mut split = QuadPath::default();
        for i in 0..path.num_curves() {
            let [p0, p1, p2] = path.nth_curve_points(i).unwrap();
            let mid = |a: Vec3, b: Vec3| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, 0.0];
            let q0 = mid(p0, p1);
            let q1 = mid(p1, p2);
            let r = mid(q0, q1);
            if i == 0 {
                split.start_new_path(p0);
            }
            split.add_quadratic_bezier_curve_to(q0, r, false).unwrap();
            split.add_quadratic_bezier_curve_to(q1, p2, false).unwrap();
        }

        let a = segs_of(&path);
        let b = segs_of(&split);
        assert!(b.len() > a.len(), "the subdivision must add segments");

        // A steep width ramp, so a parameter-space interpolation would be
        // visibly different rather than subtly.
        let style = Style {
            stroke_width: 100.0,
            stroke_width_end: 800.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        let map = unit();
        let mut worst = 0.0f64;
        for y in -6..30 {
            for x in -6..36 {
                let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                let ca = stroke_coverage(&a, &style, map, [0.0, 0.0], p);
                let cb = stroke_coverage(&b, &style, map, [0.0, 0.0], p);
                worst = worst.max((ca - cb).abs());
            }
        }
        assert!(
            worst < 1e-6,
            "stroke drift under reparameterization: {worst}"
        );
    }

    #[test]
    fn the_width_ramp_is_taken_per_segment_before_the_minimum() {
        // The ordering claim in `stroke_excess_px`. On a path whose width ramps
        // steeply, a pixel near the seam between two segments is nearest to one of
        // them, and the silhouette there must use THAT segment's width. Computing
        // min(distance) first and then looking up a width would use the wrong
        // one.
        let mut p = QuadPath::default();
        p.start_new_path([0.0, 0.0, 0.0]);
        p.add_line_to([20.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([40.0, 0.0, 0.0], false).unwrap();
        let segs = segs_of(&p);
        assert_eq!(segs.len(), 2);
        let style = Style {
            stroke_width: 200.0,     // 2 px wide at s=0 -> half-width 1
            stroke_width_end: 800.0, // 8 px wide at s=1 -> half-width 4
            anti_alias_width: 1.5,
            ..Style::default()
        };
        let map = unit();
        // At the start the half-width is 1; at the end it is 4; at the midpoint
        // (the shared anchor, s = 1/2) it is 2.5. Each is checked on its own
        // silhouette.
        for (x, hw) in [(0.0f64, 1.0f64), (20.0, 2.5), (40.0, 4.0)] {
            let c = stroke_coverage(&segs, &style, map, [0.0, 0.0], [x, hw]);
            assert!(
                (c - 0.5).abs() < 0.05,
                "at x={x} the silhouette should sit at y={hw}, got coverage {c}"
            );
        }
    }

    #[test]
    fn the_colour_ramp_is_linear_in_arc_length() {
        let style = Style {
            stroke_rgba: [0.0, 0.25, 1.0, 0.5],
            stroke_rgba_end: [1.0, 0.75, 0.0, 1.0],
            ..Style::default()
        };
        assert_eq!(stroke_rgba_at(&style, 0.0), style.stroke_rgba);
        assert_eq!(stroke_rgba_at(&style, 1.0), style.stroke_rgba_end);
        let mid = stroke_rgba_at(&style, 0.5);
        for (k, ((got, a), b)) in mid
            .iter()
            .zip(&style.stroke_rgba)
            .zip(&style.stroke_rgba_end)
            .enumerate()
        {
            assert!((got - 0.5 * (a + b)).abs() < 1e-6, "channel {k}");
        }
        // Clamped, not extrapolated.
        assert_eq!(stroke_rgba_at(&style, -2.0), style.stroke_rgba);
        assert_eq!(stroke_rgba_at(&style, 3.0), style.stroke_rgba_end);
    }

    #[test]
    fn the_slab_contains_the_stroke_it_bounds() {
        // Conservative means conservative: every pixel with any coverage must be
        // inside the slab, or the rejection would clip a stroke. Checked by
        // sweeping a window larger than the slab.
        let mut p = QuadPath::default();
        p.start_new_path([2.0, 3.0, 0.0]);
        p.add_quadratic_bezier_curve_to([10.0, 18.0, 0.0], [24.0, 5.0, 0.0], false)
            .unwrap();
        let segs = segs_of(&p);
        let style = flat_stroke_style(500.0); // 5 px wide
        let map = ScreenMap {
            scale: 2.0,
            origin: [5.0, 7.0],
            y_up: false,
        };
        let translate = [11.0, -3.0];
        let slab = segment_slab(&segs[0], &style, map, translate);
        for y in -30..90 {
            for x in -30..90 {
                let q = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                if stroke_coverage(&segs, &style, map, translate, q) <= 0.0 {
                    continue;
                }
                assert!(
                    q[0] >= slab[0] && q[0] <= slab[2] && q[1] >= slab[1] && q[1] <= slab[3],
                    "covered pixel {q:?} outside slab {slab:?}"
                );
            }
        }
    }

    #[test]
    fn the_miter_tip_is_inside_the_conservative_slab() {
        // A shallow V whose admitted miter reaches more than one half-width
        // beyond the hull along y. Growing only by the round body clips this
        // point in both the binner and the engine's explicit slab reject.
        let mut path = QuadPath::default();
        path.start_new_path([-20.0, 40.0, 0.0]);
        path.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        path.add_line_to([20.0, 40.0, 0.0], false).unwrap();
        let (shape, segments) = compile_shape(shape_digest(path.points()), &path, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let style = Style {
            joint_type: JointType::Miter,
            ..flat_stroke_style(2000.0)
        };
        let joins = join_wedges(&segments, &shape.subpath_starts, &style, unit(), [0.0, 0.0]);
        assert_eq!(joins.len(), 1);
        let join = joins[0];
        assert!(
            (1.0..MITER_LIMIT).contains(&join.miter_ratio()),
            "fixture must exercise an admitted extension: {}",
            join.miter_ratio()
        );
        let probe_distance = 0.9 * join.miter_ratio() * join.half_width;
        let probe = [
            join.anchor[0] + probe_distance * join.bisector[0],
            join.anchor[1] + probe_distance * join.bisector[1],
        ];
        assert_eq!(
            stroke_coverage_with_joins(&segments, &joins, &style, unit(), [0.0, 0.0], probe),
            1.0,
            "fixture probe must be inside the miter"
        );
        let round_pad = join.half_width + f64::from(style.anti_alias_width);
        assert!(
            probe[1] < -round_pad,
            "the probe must expose the old half-width-only slab"
        );

        let slab = segments
            .iter()
            .map(|segment| segment_slab(segment, &style, unit(), [0.0, 0.0]))
            .fold(
                [
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ],
                |mut union, segment| {
                    union[0] = union[0].min(segment[0]);
                    union[1] = union[1].min(segment[1]);
                    union[2] = union[2].max(segment[2]);
                    union[3] = union[3].max(segment[3]);
                    union
                },
            );
        assert!(
            probe[0] >= slab[0]
                && probe[0] <= slab[2]
                && probe[1] >= slab[1]
                && probe[1] <= slab[3],
            "covered miter point {probe:?} outside slab {slab:?}"
        );
    }

    /// A uniform stroke skips the per-pixel arc-length station; its coverage
    /// and shaded colour must still equal the full scalar evaluation bit for
    /// bit. A ramped stroke keeps the station.
    #[test]
    fn uniform_strokes_skip_the_station_without_changing_a_bit() {
        let mut path = QuadPath::default();
        path.start_new_path([-28.0, -3.0, 0.0]);
        path.add_quadratic_bezier_curve_to([-20.0, 24.0, 0.0], [-10.0, 1.0, 0.0], false)
            .unwrap();
        path.add_quadratic_bezier_curve_to([0.0, -22.0, 0.0], [11.0, 4.0, 0.0], false)
            .unwrap();
        path.add_quadratic_bezier_curve_to([26.0, 4.0, 0.0], [18.0, -2.0, 0.0], false)
            .unwrap();
        let (shape, segments) = compile_shape(shape_digest(path.points()), &path, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let translate = [13.0, -9.0];
        let map = ScreenMap {
            scale: 1.75,
            origin: [7.0, -4.0],
            y_up: false,
        };
        let uniform = Style {
            stroke_rgba: [0.25, 0.5, 1.0, 0.75],
            stroke_rgba_end: [0.25, 0.5, 1.0, 0.75],
            ..flat_stroke_style(500.0)
        };
        assert!(!uniform.stroke_station_is_observable());
        for ramped in [
            Style {
                stroke_width_end: 900.0,
                ..uniform.clone()
            },
            Style {
                stroke_rgba_end: [0.25, 0.5, 0.0, 0.75],
                ..uniform.clone()
            },
            Style {
                stroke_rgba: [-0.0, 0.5, 1.0, 0.75],
                stroke_rgba_end: [-0.0, 0.5, 1.0, 0.75],
                ..uniform.clone()
            },
        ] {
            assert!(ramped.stroke_station_is_observable(), "{ramped:?}");
        }
        // Thick, hairline and negative (zero-coverage) widths; the default and
        // a degenerate AA band; every joint. The control-triangle cull must
        // fire on the lattice, and nothing it skips may move a bit.
        let mut culled = 0;
        for (width, anti_alias_width, min_covered) in [
            (500.0, 1.5, 500),
            (20.0, 1.5, 100),
            (500.0, 0.0, 500),
            (-50.0, 1.5, 0),
        ] {
            for joint_type in [JointType::Auto, JointType::Bevel, JointType::Miter] {
                let style = Style {
                    joint_type,
                    stroke_width: width,
                    stroke_width_end: width,
                    anti_alias_width,
                    ..uniform.clone()
                };
                let joins = join_wedges(&segments, &shape.subpath_starts, &style, map, translate);
                let (backing, slab) = prepared(&segments, &style, map, translate, false);
                assert!(backing.iter().all(|segment| !segment.station));
                let prepared = PreparedStroke::from_parts(&backing, slab);
                let mut covered = 0;
                for y in -80..80 {
                    for x in -80..120 {
                        let point = [f64::from(x) + 0.25, f64::from(y) + 0.75];
                        let scalar = stroke_shade(&segments, &joins, &style, map, translate, point);
                        let fast = prepared.shade(&segments, &joins, &style, map, translate, point);
                        assert_eq!(
                            fast.0.to_bits(),
                            scalar.0.to_bits(),
                            "coverage at {point:?}, {style:?}"
                        );
                        if scalar.0 > 0.0 {
                            covered += 1;
                            let colour = |s| stroke_rgba_at(&style, s).map(f32::to_bits);
                            assert_eq!(colour(fast.1), colour(scalar.1), "colour at {point:?}");
                            assert_eq!(
                                half_width_px(&style, map, fast.1).to_bits(),
                                half_width_px(&style, map, scalar.1).to_bits()
                            );
                        }
                        let [ox, oy] = map.to_object(point, translate);
                        culled += segments
                            .iter()
                            .zip(&backing)
                            .filter(|(segment, prepared)| {
                                point[0] >= prepared.slab[0]
                                    && point[0] <= prepared.slab[2]
                                    && point[1] >= prepared.slab[1]
                                    && point[1] <= prepared.slab[3]
                                    && prepared.cull_distance_squared.is_some_and(|threshold| {
                                        hull_distance_squared(segment, [ox, oy]) >= threshold
                                    })
                            })
                            .count();
                    }
                }
                assert!(
                    covered >= min_covered,
                    "the grid exercises the stroke: {covered} at width {width}"
                );
            }
        }
        assert!(
            culled > 1000,
            "the lattice exercises the hull cull: {culled}"
        );
    }

    #[test]
    fn every_covered_point_lies_in_its_rows_span() {
        let mut chain = QuadPath::default();
        chain.start_new_path([-28.0, -3.0, 0.0]);
        chain
            .add_quadratic_bezier_curve_to([-20.0, 24.0, 0.0], [-10.0, 1.0, 0.0], false)
            .unwrap();
        chain
            .add_quadratic_bezier_curve_to([0.0, -22.0, 0.0], [11.0, 4.0, 0.0], false)
            .unwrap();
        chain
            .add_quadratic_bezier_curve_to([26.0, 4.0, 0.0], [18.0, -2.0, 0.0], false)
            .unwrap();
        let mut diagonal = QuadPath::default();
        diagonal.start_new_path([-30.0, -20.0, 0.0]);
        diagonal
            .add_quadratic_bezier_curve_to([0.0, 0.5, 0.0], [30.0, 21.0, 0.0], false)
            .unwrap();
        diagonal
            .add_quadratic_bezier_curve_to([31.0, 0.0, 0.0], [32.0, -20.0, 0.0], false)
            .unwrap();
        let translate = [13.0, -9.0];
        let map = ScreenMap {
            scale: 1.75,
            origin: [7.0, -4.0],
            y_up: true,
        };
        let mut covered_points = 0;
        for path in [&chain, &diagonal] {
            let (shape, segments) =
                compile_shape(shape_digest(path.points()), path, Hint::General, 0)
                    .expect("fixture fits retained table widths");
            for (width, width_end) in [(500.0, 500.0), (40.0, 40.0), (200.0, 900.0)] {
                for joint_type in [JointType::Auto, JointType::Bevel, JointType::Miter] {
                    let style = Style {
                        joint_type,
                        stroke_width: width,
                        stroke_width_end: width_end,
                        ..flat_stroke_style(width)
                    };
                    let joins =
                        join_wedges(&segments, &shape.subpath_starts, &style, map, translate);
                    let (backing, slab) = prepared(&segments, &style, map, translate, false);
                    let prepared = PreparedStroke::from_parts(&backing, slab);
                    for y in -90..90 {
                        let span = prepared.row_span(f64::from(y));
                        for x in -90..140 {
                            for (dx, dy) in [
                                (0.5, 0.5),
                                (0.25, 0.25),
                                (0.75, 0.25),
                                (0.25, 0.75),
                                (0.75, 0.75),
                            ] {
                                let point = [f64::from(x) + dx, f64::from(y) + dy];
                                let coverage = prepared
                                    .shade(&segments, &joins, &style, map, translate, point)
                                    .0;
                                if coverage <= 0.0 {
                                    continue;
                                }
                                covered_points += 1;
                                let (lo, hi) = span.unwrap_or_else(|| {
                                    panic!("covered {point:?} in a row with no span ({style:?})")
                                });
                                assert!(
                                    f64::from(x) + 1.0 >= lo && f64::from(x) <= hi,
                                    "covered {point:?} outside the row span [{lo}, {hi}] ({joint_type:?}, {width})"
                                );
                            }
                        }
                    }
                }
            }
        }
        assert!(
            covered_points > 10_000,
            "the lattice exercises strokes: {covered_points}"
        );
    }

    #[test]
    fn the_control_triangle_bounds_the_curve_distance_from_below() {
        let segment = Segment {
            p0: [-3.0, -1.0, 0.0],
            p1: [0.5, 4.0, 2.0],
            p2: [4.0, -0.5, -1.0],
            s0: 0.0,
            s1: 1.0,
        };
        let degenerate = Segment {
            p0: [0.0, 0.0, 0.0],
            p1: [1.0, 1.0, 0.0],
            p2: [2.0, 2.0, 0.0],
            s0: 0.0,
            s1: 1.0,
        };
        let mut outside = 0;
        for subject in [segment, degenerate] {
            for y in -40..40 {
                for x in -40..40 {
                    let q = [f64::from(x) * 0.23, f64::from(y) * 0.19];
                    let bound = hull_distance_squared(&subject, q);
                    let near = fmn_geom::distance::nearest_on_quadratic(
                        subject.p0,
                        subject.p1,
                        subject.p2,
                        [q[0], q[1], 0.0],
                    );
                    assert!(
                        bound.sqrt() <= near.distance + 1e-12,
                        "{q:?}: hull {} > curve {}",
                        bound.sqrt(),
                        near.distance
                    );
                    outside += usize::from(bound > 0.0);
                }
            }
        }
        assert!(outside > 1000);
        let nan = [f64::NAN, 0.0];
        assert_eq!(hull_distance_squared(&segment, nan), 0.0);
    }

    #[test]
    fn prepared_stroke_rejection_is_bit_exact_where_coverage_can_change() {
        let mut path = QuadPath::default();
        path.start_new_path([-28.0, -3.0, 0.0]);
        path.add_quadratic_bezier_curve_to([-20.0, 24.0, 0.0], [-10.0, 1.0, 0.0], false)
            .unwrap();
        path.add_quadratic_bezier_curve_to([0.0, -22.0, 0.0], [11.0, 4.0, 0.0], false)
            .unwrap();
        path.add_quadratic_bezier_curve_to([26.0, 4.0, 0.0], [18.0, -2.0, 0.0], false)
            .unwrap();
        path.add_quadratic_bezier_curve_to([24.0, -2.0 + 1e-12, 0.0], [30.0, -2.0, 0.0], false)
            .unwrap();
        let (shape, segments) = compile_shape(shape_digest(path.points()), &path, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let translate = [13.0, -9.0];

        for map in [
            ScreenMap {
                scale: 1.75,
                origin: [7.0, -4.0],
                y_up: false,
            },
            ScreenMap {
                scale: -1.75,
                origin: [7.0, -4.0],
                y_up: false,
            },
        ] {
            for joint_type in [JointType::Auto, JointType::Bevel, JointType::Miter] {
                let style = Style {
                    stroke_width: 300.0,
                    stroke_width_end: 900.0,
                    joint_type,
                    ..flat_stroke_style(300.0)
                };
                let joins = join_wedges(&segments, &shape.subpath_starts, &style, map, translate);
                let (backing, slab) = prepared(&segments, &style, map, translate, false);
                let prepared = PreparedStroke::from_parts(&backing, slab);
                for y in -80..80 {
                    for x in -80..120 {
                        let point = [f64::from(x) + 0.25, f64::from(y) + 0.75];
                        let scalar = stroke_shade(&segments, &joins, &style, map, translate, point);
                        let culled =
                            prepared.shade(&segments, &joins, &style, map, translate, point);
                        assert_eq!(
                            culled.0.to_bits(),
                            scalar.0.to_bits(),
                            "{joint_type:?} scale {} coverage at {point:?}",
                            map.scale
                        );
                        if scalar.0 > 0.0 {
                            assert_eq!(
                                culled.1.to_bits(),
                                scalar.1.to_bits(),
                                "{joint_type:?} scale {} ramp coordinate at {point:?}",
                                map.scale
                            );
                        }
                    }
                }

                let first_slab = prepared.segments[0].slab;
                let near_first = [
                    0.5 * (first_slab[0] + first_slab[2]),
                    0.5 * (first_slab[1] + first_slab[3]),
                ];
                assert!(
                    prepared
                        .segments
                        .iter()
                        .filter(|segment| {
                            near_first[0] >= segment.slab[0]
                                && near_first[0] <= segment.slab[2]
                                && near_first[1] >= segment.slab[1]
                                && near_first[1] <= segment.slab[3]
                        })
                        .count()
                        < segments.len(),
                    "the corpus must exercise a rejected segment"
                );
            }
        }
    }

    /// The outline of one blended plate strip along `y`: `quads` straight
    /// quadratics between two end caps and two zero-length segments.
    fn plate_strip(left: f64, y: f64, cell: f64, quads: usize) -> Vec<Segment> {
        let right = left + cell * quads as f64;
        let mut xs = vec![[left - 2.0 * cell, left - cell, left], [left, left, left]];
        xs.extend((0..quads).map(|k| {
            let a = left + cell * k as f64;
            [a, a + 0.5 * cell, a + cell]
        }));
        xs.extend([
            [right, right, right],
            [right, right + cell, right + 2.0 * cell],
        ]);
        let count = xs.len() as f64;
        xs.iter()
            .enumerate()
            .map(|(k, x)| Segment {
                p0: [x[0], y, 0.0],
                p1: [x[1], y, 0.0],
                p2: [x[2], y, 0.0],
                s0: k as f64 / count,
                s1: (k + 1) as f64 / count,
            })
            .collect()
    }

    /// Every coordinate where a slab test can change its answer, and one ulp
    /// either side of it.
    fn slab_edge_points(backing: &[PreparedSegment], slab: [f64; 4]) -> Vec<[f64; 2]> {
        let middle = [0.5 * (slab[0] + slab[2]), 0.5 * (slab[1] + slab[3])];
        let around = |edge: f64| [edge.next_down(), edge, edge.next_up()];
        let mut points = Vec::new();
        for prepared in backing {
            for edge in [prepared.slab[0], prepared.slab[2]] {
                points.extend(around(edge).map(|x| [x, middle[1]]));
            }
        }
        for edge in [slab[1], slab[3]] {
            points.extend(around(edge).map(|y| [middle[0], y]));
        }
        points
    }

    /// `nearest` and `shade` through the index, bit for bit against the scan.
    /// Returns whether the scan admitted any segment.
    fn assert_index_matches_scan(
        index: PreparedStroke<'_>,
        scan: PreparedStroke<'_>,
        segments: &[Segment],
        style: &Style,
        map: ScreenMap,
        translate: [f64; 2],
        p: [f64; 2],
    ) -> bool {
        let bits = |found: Option<(f64, f64)>| found.map(|(a, b)| (a.to_bits(), b.to_bits()));
        let expected = scan.nearest(segments, style, map, translate, p);
        assert_eq!(
            bits(index.nearest(segments, style, map, translate, p)),
            bits(expected),
            "nearest at {p:?}"
        );
        let shaded = index.shade(segments, &[], style, map, translate, p);
        let reference = scan.shade(segments, &[], style, map, translate, p);
        assert_eq!(
            (shaded.0.to_bits(), shaded.1.to_bits()),
            (reference.0.to_bits(), reference.1.to_bits()),
            "shade at {p:?}"
        );
        expected.is_some()
    }

    #[test]
    fn the_x_index_is_bit_exact_on_a_plate_strip() {
        let segments = plate_strip(-1.0, 0.25, 0.25, 64);
        let translate = [3.0, -2.0];
        for map in [
            ScreenMap::y_up(4.5, [32.0, 18.0]),
            ScreenMap::y_up(270.0, [1920.0, 1080.0]),
            ScreenMap {
                scale: 1.75,
                origin: [7.0, -4.0],
                y_up: false,
            },
        ] {
            // Each route of the kernel: capsule, station, and uniform curve,
            // with and without the film's zero AA band.
            for (straight, paint_end, aa) in [
                (true, [0.0, 1.0, 0.5, 1.0], 0.0),
                (true, [1.0, 1.0, 1.0, 1.0], 1.5),
                (false, [0.0, 1.0, 0.5, 1.0], 1.5),
                (false, [1.0, 1.0, 1.0, 1.0], 0.0),
            ] {
                let style = Style {
                    stroke_rgba_end: paint_end,
                    anti_alias_width: aa,
                    ..flat_stroke_style(0.885)
                };
                let (backing, slab) = prepared(&segments, &style, map, translate, straight);
                assert!(
                    segments_are_x_sorted(&backing, slab),
                    "the fixture must take the index"
                );
                let index = PreparedStroke::from_parts_indexed(&backing, slab, true);
                let scan = PreparedStroke::from_parts(&backing, slab);
                let middle = [0.5 * (slab[0] + slab[2]), 0.5 * (slab[1] + slab[3])];
                assert!(
                    index.x_candidates(middle).len() < segments.len() / 4,
                    "the index must narrow the search at {middle:?}"
                );
                let mut admitted = 0u32;
                for p in slab_edge_points(&backing, slab) {
                    let found = assert_index_matches_scan(
                        index, scan, &segments, &style, map, translate, p,
                    );
                    admitted += u32::from(found);
                }
                let (width, height) = (slab[2] - slab[0], slab[3] - slab[1]);
                for j in -4..=52 {
                    for i in -4..=204 {
                        let p = [
                            slab[0] + width * f64::from(i) / 200.0,
                            slab[1] + height * f64::from(j) / 48.0,
                        ];
                        let found = assert_index_matches_scan(
                            index, scan, &segments, &style, map, translate, p,
                        );
                        admitted += u32::from(found);
                    }
                }
                assert!(admitted > 0, "the corpus must exercise an admitted segment");
            }
        }
    }

    #[test]
    fn the_x_index_keeps_the_earliest_of_tied_segments() {
        // Two copies of every cell, so each point below has two exactly equal
        // nearest segments whose stations differ. The scan keeps the first.
        let cells = 20;
        let half = 0.5 / f64::from(cells);
        let mut segments = Vec::new();
        for k in 0..cells {
            let a = f64::from(k);
            for copy in 0..2 {
                let s0 = a / f64::from(cells) + half * f64::from(copy);
                segments.push(Segment {
                    p0: [a, 0.0, 0.0],
                    p1: [a + 0.5, 0.0, 0.0],
                    p2: [a + 1.0, 0.0, 0.0],
                    s0,
                    s1: s0 + half,
                });
            }
        }
        let map = unit();
        let translate = [0.0, 0.0];
        let style = Style {
            stroke_rgba_end: [0.0, 1.0, 0.5, 1.0],
            ..flat_stroke_style(200.0)
        };
        let (backing, slab) = prepared(&segments, &style, map, translate, true);
        assert!(segments_are_x_sorted(&backing, slab));
        let index = PreparedStroke::from_parts_indexed(&backing, slab, true);
        let scan = PreparedStroke::from_parts(&backing, slab);
        for k in 0..cells {
            let p = [f64::from(k) + 0.5, 0.25];
            assert!(assert_index_matches_scan(
                index, scan, &segments, &style, map, translate, p
            ));
            let (_, s) = index
                .nearest(&segments, &style, map, translate, p)
                .expect("an admitted point");
            let first = &segments[2 * k as usize];
            assert_eq!(
                s.to_bits(),
                (first.s0 + (first.s1 - first.s0) * 0.5).to_bits(),
                "the earlier copy of cell {k} must win"
            );
        }
    }

    #[test]
    fn the_x_index_refuses_what_it_cannot_prove() {
        let map = unit();
        let style = flat_stroke_style(100.0);
        let sorted = |segments: &[Segment]| {
            let (backing, slab) = prepared(segments, &style, map, [0.0, 0.0], false);
            segments_are_x_sorted(&backing, slab)
        };
        let strip = plate_strip(0.0, 1.0, 0.25, 32);
        assert!(sorted(&strip), "the positive control");
        assert!(sorted(&strip[..STROKE_X_INDEX_MIN_SEGMENTS]));
        assert!(!sorted(&strip[..STROKE_X_INDEX_MIN_SEGMENTS - 1]));
        assert!(!sorted(&[]));

        let reversed: Vec<Segment> = strip.iter().rev().copied().collect();
        assert!(!sorted(&reversed), "reversed");
        let mut swapped = strip.clone();
        swapped.swap(10, 20);
        assert!(!sorted(&swapped), "one segment out of place");

        let mirror = ScreenMap {
            scale: -1.75,
            origin: [0.0, 0.0],
            y_up: false,
        };
        let (backing, slab) = prepared(&strip, &style, mirror, [0.0, 0.0], false);
        assert!(!segments_are_x_sorted(&backing, slab), "mirrored");

        let mut poisoned = strip.clone();
        poisoned[7].p0 = [f64::NAN; 3];
        poisoned[7].p1 = [f64::NAN; 3];
        poisoned[7].p2 = [f64::NAN; 3];
        assert!(!sorted(&poisoned), "a NaN segment");
        let mut infinite = strip.clone();
        infinite[7].p2[0] = f64::INFINITY;
        assert!(!sorted(&infinite), "an unbounded segment");
    }

    #[test]
    fn the_x_index_leaves_non_finite_queries_to_the_scan() {
        let segments = plate_strip(0.0, 1.0, 0.25, 40);
        let map = unit();
        let translate = [0.0, 0.0];
        let style = flat_stroke_style(100.0);
        let (backing, slab) = prepared(&segments, &style, map, translate, true);
        assert!(segments_are_x_sorted(&backing, slab));
        let index = PreparedStroke::from_parts_indexed(&backing, slab, true);
        let scan = PreparedStroke::from_parts(&backing, slab);
        let middle = [0.5 * (slab[0] + slab[2]), 0.5 * (slab[1] + slab[3])];
        for p in [
            [f64::NAN, middle[1]],
            [middle[0], f64::NAN],
            [f64::NAN, f64::NAN],
            [f64::INFINITY, middle[1]],
            [f64::NEG_INFINITY, middle[1]],
            [middle[0], f64::INFINITY],
            [middle[0], f64::NEG_INFINITY],
        ] {
            assert_index_matches_scan(index, scan, &segments, &style, map, translate, p);
        }
        assert!(
            index
                .nearest(&segments, &style, map, translate, [f64::NAN, middle[1]])
                .is_some(),
            "a NaN coordinate is admitted by every segment"
        );
    }

    #[test]
    fn the_degenerate_corpus_produces_no_stroke_rather_than_a_panic() {
        let map = unit();
        let style = flat_stroke_style(200.0);
        // No segments.
        assert_eq!(
            stroke_excess_px(&[], &style, map, [0.0, 0.0], [1.0, 1.0]),
            None
        );
        assert_eq!(
            stroke_coverage(&[], &style, map, [0.0, 0.0], [1.0, 1.0]),
            0.0
        );
        let (backing, slab) = prepared(&[], &style, map, [0.0, 0.0], false);
        assert_eq!(
            PreparedStroke::from_parts(&backing, slab).shade(
                &[],
                &[],
                &style,
                map,
                [0.0, 0.0],
                [1.0, 1.0]
            ),
            (0.0, 0.0)
        );
        // A degenerate map has no pixels to measure in.
        let path = line(10.0);
        let segs = segs_of(&path);
        let degenerate_map = ScreenMap {
            scale: 0.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        assert_eq!(
            stroke_excess_px(&segs, &style, degenerate_map, [0.0, 0.0], [1.0, 1.0]),
            None
        );
        let (backing, slab) = prepared(&segs, &style, degenerate_map, [0.0, 0.0], false);
        assert_eq!(
            PreparedStroke::from_parts(&backing, slab).shade(
                &segs,
                &[],
                &style,
                degenerate_map,
                [0.0, 0.0],
                [1.0, 1.0]
            ),
            (0.0, 0.0)
        );
        // A zero width draws nothing at all, not a hairline from the AA band.
        let zero = Style {
            stroke_width: 0.0,
            stroke_width_end: 0.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        assert_eq!(
            stroke_coverage(&segs, &zero, map, [0.0, 0.0], [5.0, 0.0]),
            0.0
        );
        // A cusp: the handle beyond an endpoint, so the distance solve's roots
        // collide. It must still produce a finite, in-range coverage.
        let mut cusp = QuadPath::default();
        cusp.start_new_path([0.0, 0.0, 0.0]);
        cusp.add_quadratic_bezier_curve_to([20.0, 0.0, 0.0], [0.0, 0.0, 0.0], true)
            .unwrap();
        let cusp_segs = segs_of(&cusp);
        let (cusp_backing, cusp_slab) = prepared(&cusp_segs, &style, map, [0.0, 0.0], false);
        let prepared_cusp = PreparedStroke::from_parts(&cusp_backing, cusp_slab);
        for x in 0..12 {
            let point = [f64::from(x), 0.5];
            let c = stroke_coverage(&cusp_segs, &style, map, [0.0, 0.0], point);
            assert!(
                (0.0..=1.0).contains(&c) && c.is_finite(),
                "cusp at x={x}: {c}"
            );
            assert_eq!(
                prepared_cusp
                    .shade(&cusp_segs, &[], &style, map, [0.0, 0.0], point)
                    .0
                    .to_bits(),
                c.to_bits(),
                "prepared cusp at x={x}"
            );
        }
    }

    // ------------------------------------------------------------- the joins

    use fmn_mobject::JointType;

    /// A right-angle corner: in along +x to the origin, out along +y.
    fn corner() -> (Vec<Segment>, Vec<u32>) {
        let mut p = QuadPath::default();
        p.start_new_path([-20.0, 0.0, 0.0]);
        p.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([0.0, 20.0, 0.0], false).unwrap();
        let (shape, segs) = compile_shape(shape_digest(p.points()), &p, Hint::General, 0)
            .expect("fixture fits retained table widths");
        (segs, shape.subpath_starts)
    }

    #[test]
    fn round_and_no_joint_produce_no_wedges_at_all() {
        // What makes the round path bit-identical whether or not the join
        // machinery exists: for the two round settings there is nothing to edit.
        let (segs, starts) = corner();
        for joint in [JointType::Auto, JointType::NoJoint] {
            let style = Style {
                joint_type: joint,
                ..flat_stroke_style(600.0)
            };
            let joins = join_wedges(&segs, &starts, &style, unit(), [0.0, 0.0]);
            assert!(joins.is_empty(), "{joint:?} must need no wedge");
            // And the coverage is identical to the join-free path, pixel for
            // pixel, not merely close.
            for y in -8..8 {
                for x in -8..8 {
                    let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                    assert_eq!(
                        stroke_coverage_with_joins(&segs, &joins, &style, unit(), [0.0, 0.0], p),
                        stroke_coverage(&segs, &style, unit(), [0.0, 0.0], p),
                        "{joint:?} at {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_wedge_geometry_is_the_corner_it_describes() {
        // The right-angle corner, checked against hand arithmetic: the wedge is
        // the fourth quadrant, the bisector points into it, the offset normals are
        // the two edges' outward normals, and the miter ratio is sec(45 deg).
        let (segs, starts) = corner();
        let style = Style {
            joint_type: JointType::Miter,
            ..flat_stroke_style(600.0) // 6 px wide -> 3 px half-width
        };
        let joins = join_wedges(&segs, &starts, &style, unit(), [0.0, 0.0]);
        assert_eq!(joins.len(), 1, "one interior corner");
        let w = joins[0];
        assert_eq!(w.anchor, [0.0, 0.0]);
        assert!((w.half_width - 3.0).abs() < 1e-12);
        assert!((w.t_in[0] - 1.0).abs() < 1e-12 && w.t_in[1].abs() < 1e-12);
        assert!(w.t_out[0].abs() < 1e-12 && (w.t_out[1] - 1.0).abs() < 1e-12);
        let r = 0.5f64.sqrt();
        assert!((w.bisector[0] - r).abs() < 1e-12 && (w.bisector[1] + r).abs() < 1e-12);
        assert!(w.n_in[0].abs() < 1e-12 && (w.n_in[1] + 1.0).abs() < 1e-12);
        assert!((w.n_out[0] - 1.0).abs() < 1e-12 && w.n_out[1].abs() < 1e-12);
        assert!(
            (w.miter_ratio() - std::f64::consts::SQRT_2).abs() < 1e-12,
            "sec(45 deg): {}",
            w.miter_ratio()
        );

        // The wedge is the outer quadrant and nothing else.
        assert!(w.contains([1.0, -1.0]), "outer");
        assert!(!w.contains([-1.0, 1.0]), "inner");
        assert!(!w.contains([1.0, 1.0]), "ahead of the outgoing segment");
        assert!(!w.contains([-1.0, -1.0]), "behind the incoming one");
        assert!(
            w.contains([0.0, 0.0]),
            "the anchor belongs to its own wedge"
        );
    }

    #[test]
    fn bevel_trims_the_round_corner_and_miter_extends_it() {
        // The two edits, at the one point that tells them apart: the corner tip,
        // straight out along the bisector.
        // A 20 px stroke, so the three silhouettes — bevel chord at 0.707 hw,
        // round arc at 1.0 hw, miter tip at 1.414 hw — are each further apart than
        // the 1.5 px antialiasing band. At hw = 3 they overlap and every probe
        // reads a blend of two joins, which is how this test first failed.
        let (segs, starts) = corner();
        let hw = 10.0;
        let mk = |joint| Style {
            joint_type: joint,
            ..flat_stroke_style(2000.0)
        };
        let map = unit();
        let tip = |k: f64| {
            let r = 0.5f64.sqrt();
            [k * hw * r, -k * hw * r]
        };

        let round_style = mk(JointType::Auto);
        let round_joins = join_wedges(&segs, &starts, &round_style, map, [0.0, 0.0]);
        let bevel_style = mk(JointType::Bevel);
        let bevel_joins = join_wedges(&segs, &starts, &bevel_style, map, [0.0, 0.0]);
        let miter_style = mk(JointType::Miter);
        let miter_joins = join_wedges(&segs, &starts, &miter_style, map, [0.0, 0.0]);
        assert_eq!(bevel_joins.len(), 1);
        assert_eq!(miter_joins.len(), 1);

        let cov = |style: &Style, joins: &[JoinWedge], p| {
            stroke_coverage_with_joins(&segs, joins, style, map, [0.0, 0.0], p)
        };

        // Half a half-width in: inside all three, so the joins agree where they
        // are supposed to.
        let deep = tip(0.5);
        assert_eq!(cov(&round_style, &round_joins, deep), 1.0);
        assert_eq!(cov(&bevel_style, &bevel_joins, deep), 1.0);
        assert_eq!(cov(&miter_style, &miter_joins, deep), 1.0);

        // At 0.9 of a half-width: inside the round join and the miter, and OUTSIDE
        // the bevel, whose chord sits at hw*cos(45 deg) = 0.707 hw. That is the
        // trim.
        let inner = tip(0.9);
        assert_eq!(cov(&round_style, &round_joins, inner), 1.0);
        assert_eq!(cov(&miter_style, &miter_joins, inner), 1.0);
        assert_eq!(
            cov(&bevel_style, &bevel_joins, inner),
            0.0,
            "bevel must have trimmed the round corner"
        );

        // At 1.2 half-widths the round join has ended and the miter has not: the
        // miter tip reaches sqrt(2) = 1.414 hw. That is the extension.
        let outer = tip(1.2);
        assert_eq!(cov(&round_style, &round_joins, outer), 0.0);
        assert_eq!(cov(&bevel_style, &bevel_joins, outer), 0.0);
        assert_eq!(
            cov(&miter_style, &miter_joins, outer),
            1.0,
            "the miter must reach past the round join"
        );

        // The miter's own tip is its silhouette.
        let point = tip(std::f64::consts::SQRT_2);
        let at_tip = cov(&miter_style, &miter_joins, point);
        assert!((at_tip - 0.5).abs() < 1e-9, "miter tip coverage {at_tip}");
        assert_eq!(
            cov(&miter_style, &miter_joins, tip(1.9)),
            0.0,
            "and past it"
        );
    }

    #[test]
    fn a_join_edit_never_leaves_its_own_wedge() {
        // The containment guarantee. Away from the corner all three joint types
        // must agree pixel for pixel, or a joint setting would be changing the
        // stroke's body.
        let (segs, starts) = corner();
        let map = unit();
        let styles: Vec<Style> = [JointType::Auto, JointType::Bevel, JointType::Miter]
            .into_iter()
            .map(|joint| Style {
                joint_type: joint,
                ..flat_stroke_style(600.0)
            })
            .collect();
        let joins: Vec<Vec<JoinWedge>> = styles
            .iter()
            .map(|s| join_wedges(&segs, &starts, s, map, [0.0, 0.0]))
            .collect();
        for y in -24..24 {
            for x in -24..24 {
                let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                // Outside the outer quadrant of the corner, nothing may differ.
                if joins[1][0].contains(p) {
                    continue;
                }
                let base =
                    stroke_coverage_with_joins(&segs, &joins[0], &styles[0], map, [0.0, 0.0], p);
                for k in 1..3 {
                    assert_eq!(
                        stroke_coverage_with_joins(
                            &segs,
                            &joins[k],
                            &styles[k],
                            map,
                            [0.0, 0.0],
                            p
                        ),
                        base,
                        "{:?} changed the body at {p:?}",
                        styles[k].joint_type
                    );
                }
            }
        }
    }

    #[test]
    fn a_reversal_exceeds_the_miter_limit_and_falls_back_to_bevel() {
        // A 180-degree turn has no finite miter point, so the limit is not a
        // tuning parameter here — it is the only thing standing between the
        // request and a spike of infinite length.
        let mut p = QuadPath::default();
        p.start_new_path([-20.0, 0.0, 0.0]);
        p.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([-20.0, 0.0, 0.0], true).unwrap();
        let (shape, segs) = compile_shape(shape_digest(p.points()), &p, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let map = unit();
        let style = Style {
            joint_type: JointType::Miter,
            ..flat_stroke_style(600.0)
        };
        let joins = join_wedges(&segs, &shape.subpath_starts, &style, map, [0.0, 0.0]);
        if let Some(w) = joins.first() {
            assert!(
                w.miter_ratio() > MITER_LIMIT,
                "a reversal must exceed the limit: {}",
                w.miter_ratio()
            );
        }
        // Whatever the degenerate geometry, nothing may extend past the limit.
        for k in 1..=12 {
            let x = 3.0 * f64::from(k) / 4.0;
            let c = stroke_coverage_with_joins(&segs, &joins, &style, map, [0.0, 0.0], [x, 0.0]);
            assert!((0.0..=1.0).contains(&c) && c.is_finite());
            if x > MITER_LIMIT * 3.0 + 1.5 {
                assert_eq!(c, 0.0, "the limit did not hold at x={x}");
            }
        }
    }

    #[test]
    fn the_miter_limit_is_the_measured_one() {
        // G0-2 L6's derivation, reproduced from both ends.
        //
        // The Reference's `auto` join crosses over at `cos_angle = -0.8`, where
        // `cos_angle` is the cosine of the angle between the two TANGENTS. The
        // interior angle at the vertex is its supplement, so `cos psi = +0.8`, and
        // the classical miter length is `1 / sin(psi/2)`:
        //   psi = acos(0.8) = 36.87 deg, sin(psi/2) = 0.31623, 1/that = 3.16228.
        // The shader's own arithmetic agrees by a different route: the worst
        // admissible |shift| is tan(psi_turn/2) = tan(71.565 deg) = 3 half-widths
        // along the tangent, one half-width off it, so the tip is sqrt(1+9) away.
        assert!((MITER_LIMIT - 10.0f64.sqrt()).abs() < 1e-15);
        let psi = 0.8f64.acos();
        let predicted = 1.0 / (psi / 2.0).sin();
        assert!(
            (MITER_LIMIT - predicted).abs() < 1e-9,
            "{MITER_LIMIT} vs {predicted}"
        );
        // And it is the ratio this module's own wedge reports for that angle: a
        // corner whose tangents meet at cos = -0.8 must sit exactly at the limit.
        let turn = (-0.8f64).acos();
        let mut p = QuadPath::default();
        p.start_new_path([-20.0, 0.0, 0.0]);
        p.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([20.0 * turn.cos(), 20.0 * turn.sin(), 0.0], false)
            .unwrap();
        let (shape, segs) = compile_shape(shape_digest(p.points()), &p, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let style = Style {
            joint_type: JointType::Miter,
            ..flat_stroke_style(600.0)
        };
        let joins = join_wedges(&segs, &shape.subpath_starts, &style, unit(), [0.0, 0.0]);
        assert_eq!(joins.len(), 1);
        assert!(
            (joins[0].miter_ratio() - MITER_LIMIT).abs() < 1e-9,
            "the wedge reports {} at the crossover angle",
            joins[0].miter_ratio()
        );
    }

    #[test]
    fn a_smooth_join_needs_no_wedge_and_a_closed_path_gets_its_wrap_corner() {
        // Two segments meeting tangentially have no corner to draw, and a wedge
        // there would be a zero-area region with an ill-defined bisector.
        let mut smooth = QuadPath::default();
        smooth.start_new_path([0.0, 0.0, 0.0]);
        smooth.add_line_to([10.0, 0.0, 0.0], false).unwrap();
        smooth.add_line_to([20.0, 0.0, 0.0], false).unwrap();
        let (sh, sg) = compile_shape(shape_digest(smooth.points()), &smooth, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let style = Style {
            joint_type: JointType::Miter,
            ..flat_stroke_style(400.0)
        };
        assert!(
            join_wedges(&sg, &sh.subpath_starts, &style, unit(), [0.0, 0.0]).is_empty(),
            "a tangential join has no wedge"
        );

        // A closed triangle has three corners, and the third is the wrap — the one
        // place a closed outline would otherwise keep a round join under a miter
        // setting.
        let mut tri = QuadPath::default();
        tri.start_new_path([0.0, 0.0, 0.0]);
        tri.add_line_to([20.0, 0.0, 0.0], false).unwrap();
        tri.add_line_to([10.0, 16.0, 0.0], false).unwrap();
        tri.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        let (sh, sg) = compile_shape(shape_digest(tri.points()), &tri, Hint::General, 0)
            .expect("fixture fits retained table widths");
        assert_eq!(sg.len(), 3);
        assert_eq!(
            join_wedges(&sg, &sh.subpath_starts, &style, unit(), [0.0, 0.0]).len(),
            3,
            "two interior corners plus the wrap"
        );
    }

    #[test]
    fn a_stroke_follows_its_occurrence() {
        let path = line(20.0);
        let segs = segs_of(&path);
        let style = flat_stroke_style(300.0);
        let map = ScreenMap {
            scale: 3.0,
            origin: [4.0, 9.0],
            y_up: false,
        };
        let here = stroke_coverage(&segs, &style, map, [0.0, 0.0], [4.0 + 30.0, 9.0]);
        let there = stroke_coverage(&segs, &style, map, [60.0, -21.0], [4.0 + 90.0, -12.0]);
        assert_eq!(here, there);
        assert_eq!(here, 1.0, "on the axis, so fully covered");
    }
}
