//! Retained per-record stroke paint at true-arc stations.
//!
//! Profiles are immutable and shared by prepared draws. Their storage is owned
//! by the retained style, not by a pixel worker or an exported RecordBuffer.
use std::sync::Arc;

/// Maximum knots admitted by one stroke profile.
pub const MAX_STROKE_PROFILE_KNOTS: usize = 1 << 20;

/// One stroke record located on the drawn path's normalized true arc length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeKnot {
    /// Normalized true arc length, in the closed interval `[0, 1]`.
    pub s: f64,
    /// Full stroke width in the same units as [`crate::Style::stroke_width`].
    pub width: f32,
    /// Linear-light, straight-alpha stroke color.
    pub rgba: [f32; 4],
}

/// Invalid profile input, rejected before it enters a retained style table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeProfileError {
    /// A profile needs at least one knot and must fit the resource ceiling.
    KnotCount,
    /// Stations must be finite, nondecreasing, and in `[0, 1]`.
    Stations,
    /// Widths and color components must be finite.
    Paint,
}

impl std::fmt::Display for StrokeProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::KnotCount => "stroke profile has an empty or oversized knot table",
            Self::Stations => "stroke profile stations must be finite, ordered and normalized",
            Self::Paint => "stroke profile paint must be finite",
        })
    }
}
impl std::error::Error for StrokeProfileError {}

/// Piecewise-linear stroke width and color over immutable true-arc stations.
///
/// Coincident stations are allowed at a subpath break. Sampling is
/// right-continuous there; callers evaluating an outgoing endpoint can request
/// its one-sided parameter with [`StrokeProfile::endpoint_parameter`]. Negative
/// finite widths retain the existing renderer's zero-coverage interpretation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrokeProfile {
    knots: Vec<StrokeKnot>,
    maximum_width: f32,
    visible_alpha: bool,
    constant_width: bool,
    uniform_paint: bool,
}

/// `a` and `b` are one finite value that is not negative zero — the condition
/// under which `a + (b - a) * t` is exactly `a` for every `t >= 0`
/// (`b - a` is `+0`, and adding a zero of either sign leaves a nonzero or
/// positive-zero `a` unchanged).
fn interpolation_is_exactly_constant(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() && a.is_finite() && !(a == 0.0 && a.is_sign_negative())
}

impl StrokeProfile {
    /// Validate and retain a profile without retaining any writable source view.
    ///
    /// # Errors
    /// Rejects empty/oversized tables, invalid stations, and nonfinite paint.
    pub fn new(knots: Vec<StrokeKnot>) -> Result<Self, StrokeProfileError> {
        if knots.is_empty() || knots.len() > MAX_STROKE_PROFILE_KNOTS {
            return Err(StrokeProfileError::KnotCount);
        }
        let mut previous = 0.0;
        let mut maximum_width = 0.0f32;
        let mut visible_alpha = false;
        let first_width = knots[0].width;
        let first_rgba = knots[0].rgba;
        let mut constant_width = true;
        let mut uniform_paint = true;
        for knot in &knots {
            if !knot.s.is_finite() || !(previous..=1.0).contains(&knot.s) {
                return Err(StrokeProfileError::Stations);
            }
            if !knot.width.is_finite() || knot.rgba.iter().any(|v| !v.is_finite()) {
                return Err(StrokeProfileError::Paint);
            }
            previous = knot.s;
            maximum_width = maximum_width.max(knot.width);
            visible_alpha |= knot.rgba[3] > 0.0;
            constant_width &= knot.width == first_width;
            uniform_paint &= interpolation_is_exactly_constant(first_width, knot.width)
                && (0..4).all(|i| interpolation_is_exactly_constant(first_rgba[i], knot.rgba[i]));
        }
        Ok(Self {
            knots,
            maximum_width,
            visible_alpha,
            constant_width,
            uniform_paint,
        })
    }

    /// The immutable station table, in source path order.
    #[must_use]
    pub fn knots(&self) -> &[StrokeKnot] {
        &self.knots
    }

    /// Largest nonnegative full width, including interior extrema.
    #[must_use]
    pub fn maximum_width(&self) -> f32 {
        self.maximum_width
    }

    /// Whether any station has positive alpha.
    #[must_use]
    pub fn has_visible_alpha(&self) -> bool {
        self.visible_alpha
    }

    /// Whether width is constant over the entire table, not merely its ends.
    #[must_use]
    pub fn has_constant_width(&self) -> bool {
        self.constant_width
    }

    /// Whether every station evaluates to exactly the same width and colour
    /// bits, so a shader may skip computing the station at all.
    #[must_use]
    pub fn has_uniform_paint(&self) -> bool {
        self.uniform_paint
    }

    fn interval(&self, s: f64) -> (StrokeKnot, StrokeKnot, f32) {
        let s = s.clamp(0.0, 1.0);
        let index = self.knots.partition_point(|knot| knot.s <= s);
        if index == 0 {
            return (self.knots[0], self.knots[0], 0.0);
        }
        let left = self.knots[index - 1];
        let Some(&right) = self.knots.get(index) else {
            return (left, left, 0.0);
        };
        let alpha = ((s - left.s) / (right.s - left.s)).clamp(0.0, 1.0) as f32;
        (left, right, alpha)
    }

    /// Full width at a normalized true-arc station.
    #[must_use]
    pub fn width_at(&self, s: f64) -> f32 {
        let (left, right, alpha) = self.interval(s);
        (f64::from(left.width)
            + (f64::from(right.width) - f64::from(left.width)) * f64::from(alpha)) as f32
    }

    /// Linear-light straight-alpha color at a normalized true-arc station.
    #[must_use]
    pub fn rgba_at(&self, s: f64) -> [f32; 4] {
        let (left, right, alpha) = self.interval(s);
        std::array::from_fn(|i| {
            (f64::from(left.rgba[i])
                + (f64::from(right.rgba[i]) - f64::from(left.rgba[i])) * f64::from(alpha))
                as f32
        })
    }

    /// Preserve an outgoing subpath's endpoint at a discontinuous station.
    ///
    /// One representable step selects the correct interval; interpolation back
    /// to f32 still reaches the stored endpoint. Ordinary continuous stations,
    /// including a path's first and last station, are left exactly unchanged.
    #[must_use]
    pub fn endpoint_parameter(&self, s: f64, at_end: bool) -> f64 {
        if !at_end {
            return s;
        }
        let first = self.knots.partition_point(|knot| knot.s < s);
        let last = self.knots.partition_point(|knot| knot.s <= s);
        if last > first + 1 { s.next_down() } else { s }
    }
}

impl crate::Style {
    /// Full width at a station, honoring an optional retained record profile.
    #[must_use]
    pub fn stroke_width_at(&self, s: f64) -> f32 {
        if let Some(profile) = &self.stroke_profile {
            return profile.width_at(s);
        }
        let s = s.clamp(0.0, 1.0) as f32;
        self.stroke_width + (self.stroke_width_end - self.stroke_width) * s
    }

    /// Linear-light stroke color, honoring an optional retained record profile.
    #[must_use]
    pub fn stroke_color_at(&self, s: f64) -> [f32; 4] {
        if let Some(profile) = &self.stroke_profile {
            return profile.rgba_at(s);
        }
        let s = s.clamp(0.0, 1.0) as f32;
        std::array::from_fn(|i| {
            self.stroke_rgba[i] + (self.stroke_rgba_end[i] - self.stroke_rgba[i]) * s
        })
    }

    /// Conservative width bound used by culling and both camera pipelines.
    #[must_use]
    pub fn maximum_stroke_width(&self) -> f32 {
        self.stroke_profile.as_ref().map_or_else(
            || self.stroke_width.max(self.stroke_width_end).max(0.0),
            |profile| profile.maximum_width(),
        )
    }

    /// Whether any part of the stroke can contribute a visible fragment.
    #[must_use]
    pub fn draws_stroke(&self) -> bool {
        self.maximum_stroke_width() > 0.0
            && self.stroke_profile.as_ref().map_or_else(
                || self.stroke_rgba[3] > 0.0 || self.stroke_rgba_end[3] > 0.0,
                |profile| profile.has_visible_alpha(),
            )
    }

    /// Whether the constant-width fast path is valid for the full stroke.
    #[must_use]
    pub fn has_constant_stroke_width(&self) -> bool {
        self.stroke_profile.as_ref().map_or_else(
            || self.stroke_width == self.stroke_width_end,
            |profile| profile.has_constant_width(),
        )
    }

    /// Whether the arc-length station of a stroke sample can change its width
    /// or colour. False only when [`Self::stroke_width_at`] and
    /// [`Self::stroke_color_at`] are exactly — bit for bit — the same at every
    /// station: one finite, non-negative-zero value at both ends (or at every
    /// profile knot). Shaders use it to skip the per-pixel arc-length solve;
    /// any doubt keeps the station.
    #[must_use]
    pub fn stroke_station_is_observable(&self) -> bool {
        match &self.stroke_profile {
            Some(profile) => !profile.has_uniform_paint(),
            None => {
                !(interpolation_is_exactly_constant(self.stroke_width, self.stroke_width_end)
                    && (0..4).all(|i| {
                        interpolation_is_exactly_constant(
                            self.stroke_rgba[i],
                            self.stroke_rgba_end[i],
                        )
                    }))
            }
        }
    }

    /// One-sided color/width parameter for an outgoing subpath endpoint.
    #[must_use]
    pub fn stroke_endpoint_parameter(&self, s: f64, at_end: bool) -> f64 {
        self.stroke_profile
            .as_ref()
            .map_or(s, |profile| profile.endpoint_parameter(s, at_end))
    }

    /// Retain an already validated profile while keeping preparation clones cheap.
    #[must_use]
    pub fn with_stroke_profile(mut self, profile: StrokeProfile) -> Self {
        self.stroke_profile = Some(Arc::new(profile));
        self
    }
}

/// Whether any record's stroke paint differs from the first.
pub(crate) fn has_varying_paint(stage: &fmn_mobject::Stage, mob: fmn_mobject::Mob) -> bool {
    let Some(entry) = stage.get(mob) else {
        return false;
    };
    let buffer = &entry.buffer;
    let count = buffer.len();
    if count == 0 {
        return false;
    }
    let paint = |index: usize| {
        let width = buffer
            .read(index, "stroke_width")
            .and_then(|v| v.first().copied())
            .unwrap_or(0.0);
        let color = buffer
            .read(index, "stroke_rgba")
            .and_then(|v| <[f32; 4]>::try_from(v.as_slice()).ok())
            .unwrap_or([0.0; 4]);
        (width, color)
    };
    let first = paint(0);
    (1..count).any(|i| paint(i) != first)
}

/// Derive record knots through Chisel's path layout and true arc lengths.
/// Uniform columns deliberately do not read/measure geometry.
pub(crate) fn from_records(
    stage: &fmn_mobject::Stage,
    mob: fmn_mobject::Mob,
    decode: impl Fn([f32; 4]) -> [f32; 4],
) -> Result<Option<StrokeProfile>, crate::plan::SyncError> {
    use crate::plan::SyncError;
    let Some(entry) = stage.get(mob) else {
        return Ok(None);
    };
    let buffer = &entry.buffer;
    let count = buffer.len();
    if count == 0 {
        return Ok(None);
    }
    let paint = |index: usize| {
        let width = buffer
            .read(index, "stroke_width")
            .and_then(|v| v.first().copied())
            .unwrap_or(0.0);
        let color = buffer
            .read(index, "stroke_rgba")
            .and_then(|v| <[f32; 4]>::try_from(v.as_slice()).ok())
            .unwrap_or([0.0; 4]);
        (width, color)
    };
    let first = paint(0);
    if (1..count).all(|i| paint(i) == first) {
        return Ok(None);
    }
    if count > MAX_STROKE_PROFILE_KNOTS {
        return Err(SyncError::LimitExceeded {
            resource: "stroke profile knots",
            requested: count as u64,
            limit: MAX_STROKE_PROFILE_KNOTS as u64,
        });
    }
    let stations = record_stations(stage, mob)?;
    if stations.is_empty() {
        return Ok(None);
    }
    let mut knots = Vec::new();
    for (index, s) in stations {
        let (width, rgba) = paint(index);
        let knot = StrokeKnot {
            s,
            width,
            rgba: decode(rgba),
        };
        if knots.last() != Some(&knot) {
            knots.push(knot);
        }
    }
    StrokeProfile::new(knots)
        .map(Some)
        .map_err(|source| SyncError::InvalidStrokeProfile { mob, source })
}

/// Geometry-only station derivation shared by fill and stroke profiles.
/// Shared anchors occur once; subpath breaks retain their two source indices.
pub(crate) fn record_stations(
    stage: &fmn_mobject::Stage,
    mob: fmn_mobject::Mob,
) -> Result<Vec<(usize, f64)>, crate::plan::SyncError> {
    use crate::plan::SyncError;
    use fmn_geom::{arclength::ArcLengthTable, quadpath::QuadPath};
    let Some(points) = stage.get_object_points(mob) else {
        return Ok(Vec::new());
    };
    let origin = points.first().copied().unwrap_or([0.0; 3]);
    let local: Vec<_> = points
        .iter()
        .map(|p| std::array::from_fn(|i| p[i] - origin[i]))
        .collect();
    let path = QuadPath::from_points(local)
        .map_err(|source| SyncError::InvalidGeometry { mob, source })?;
    let original = ArcLengthTable::for_path(&path);
    let breaks = path.subpath_end_indices();
    let Some(placement) = stage.placement(mob) else {
        return Ok(Vec::new());
    };
    let unchanged = crate::table::retains_normalized_arc_length(placement);
    let mut curves = Vec::new();
    let mut total = 0.0;
    for (i, &length) in original.curve_lengths().iter().enumerate() {
        if length <= 0.0 || breaks.contains(&(2 * i)) {
            continue;
        }
        let length = if unchanged {
            length
        } else {
            let [a, b, c] = path.nth_curve_points(i).expect("measured path curve");
            fmn_geom::arclength::quadratic_arc_length(
                placement.apply_vector(a),
                placement.apply_vector(b),
                placement.apply_vector(c),
            )
        };
        total += length;
        curves.push((i, length));
    }
    if !total.is_finite() {
        return Err(SyncError::InvalidStrokeProfile {
            mob,
            source: StrokeProfileError::Stations,
        });
    }
    if total <= 0.0 {
        return Ok(Vec::new());
    }
    let mut stations = Vec::new();
    let mut distance = 0.0;
    for (index, length) in curves {
        for (offset, fraction) in [(0, 0.0), (1, 0.5), (2, 1.0)] {
            let station = (
                2 * index + offset,
                ((distance + fraction * length) / total).clamp(0.0, 1.0),
            );
            if stations.last() != Some(&station) {
                stations.push(station);
            }
        }
        distance += length;
    }
    Ok(stations)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn knot(s: f64, width: f32, alpha: f32) -> StrokeKnot {
        StrokeKnot {
            s,
            width,
            rgba: [0.25, 0.5, 0.75, alpha],
        }
    }
    #[test]
    fn interior_width_and_alpha_survive_zero_ends() {
        let profile = StrokeProfile::new(vec![
            knot(0.0, 0.0, 0.0),
            knot(0.5, 8.0, 1.0),
            knot(1.0, 0.0, 0.0),
        ])
        .unwrap();
        let style = crate::Style::default().with_stroke_profile(profile);
        assert!(style.draws_stroke());
        assert!(!style.has_constant_stroke_width());
        assert_eq!(style.maximum_stroke_width(), 8.0);
        assert_eq!(style.stroke_width_at(0.25), 4.0);
        assert_eq!(style.stroke_color_at(0.25)[3], 0.5);
        assert_eq!(style.stroke_width_at(0.5), 8.0);
    }
    #[test]
    fn coincident_stations_have_explicit_one_sided_limits() {
        let profile = StrokeProfile::new(vec![
            knot(0.0, 2.0, 1.0),
            knot(0.5, 2.0, 1.0),
            knot(0.5, 6.0, 0.5),
            knot(1.0, 6.0, 0.5),
        ])
        .unwrap();
        assert_eq!(profile.width_at(0.5), 6.0);
        assert_eq!(profile.width_at(profile.endpoint_parameter(0.5, true)), 2.0);
        assert_eq!(profile.endpoint_parameter(1.0, true), 1.0);
    }
    #[test]
    fn malformed_profiles_are_refused() {
        assert_eq!(
            StrokeProfile::new(vec![]),
            Err(StrokeProfileError::KnotCount)
        );
        for s in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert_eq!(
                StrokeProfile::new(vec![knot(s, 1.0, 1.0)]),
                Err(StrokeProfileError::Stations)
            );
        }
        assert_eq!(
            StrokeProfile::new(vec![knot(0.7, 1.0, 1.0), knot(0.2, 1.0, 1.0)]),
            Err(StrokeProfileError::Stations)
        );
        assert_eq!(
            StrokeProfile::new(vec![knot(0.0, f32::NAN, 1.0)]),
            Err(StrokeProfileError::Paint)
        );
        assert_eq!(
            StrokeProfile::new(vec![knot(0.0, 1.0, f32::INFINITY)]),
            Err(StrokeProfileError::Paint)
        );
    }
    #[test]
    fn legacy_endpoint_styles_keep_their_exact_arithmetic() {
        let style = crate::Style {
            stroke_width: 2.0,
            stroke_width_end: 10.0,
            stroke_rgba: [0.0; 4],
            stroke_rgba_end: [1.0; 4],
            ..Default::default()
        };
        assert_eq!(style.stroke_width_at(0.25), 4.0);
        assert_eq!(style.stroke_color_at(0.25), [0.25; 4]);
    }
}
