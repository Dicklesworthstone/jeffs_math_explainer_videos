//! Immutable record colors on the same true-arc stations as retained strokes.
//!
//! Unlike an endpoint ramp, a profile can have interior extrema and transparent
//! ends. Interior evaluation interpolates *colors*, not a scalar parameter that
//! would erase non-affine boundary data. No writable record view is retained.

/// Maximum records admitted by a single fill profile.
pub const MAX_FILL_PROFILE_KNOTS: usize = 1 << 20;

/// Linear-light, straight-alpha color at a normalized true-arc station.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillKnot {
    /// Finite, nondecreasing station in `[0, 1]`.
    pub s: f64,
    /// Finite linear-light RGB and straight alpha.
    pub rgba: [f32; 4],
}

/// A rejected fill profile never enters the retained style table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillProfileError {
    /// Empty or oversized station table.
    KnotCount,
    /// Nonfinite, unordered, or unnormalized stations.
    Stations,
    /// Nonfinite color or alpha.
    Paint,
}
impl std::fmt::Display for FillProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::KnotCount => "fill profile has an empty or oversized knot table",
            Self::Stations => "fill profile stations must be finite, ordered and normalized",
            Self::Paint => "fill profile paint must be finite",
        })
    }
}
impl std::error::Error for FillProfileError {}

/// Piecewise-linear boundary paint. Coincident stations encode subpath breaks.
#[derive(Debug, Clone, PartialEq)]
pub struct FillProfile {
    knots: Vec<FillKnot>,
    bounds: [[f32; 4]; 2],
    flat: bool,
    opaque: bool,
}
impl FillProfile {
    /// Validate the entire immutable table before publication.
    ///
    /// # Errors
    /// Rejects invalid station counts, ordering, normalization and nonfinite paint.
    pub fn new(knots: Vec<FillKnot>) -> Result<Self, FillProfileError> {
        if knots.is_empty() || knots.len() > MAX_FILL_PROFILE_KNOTS {
            return Err(FillProfileError::KnotCount);
        }
        let first = knots[0].rgba;
        let mut bounds = [first; 2];
        let mut previous = 0.0;
        let mut flat = true;
        let mut opaque = true;
        for knot in &knots {
            if !knot.s.is_finite() || !(previous..=1.0).contains(&knot.s) {
                return Err(FillProfileError::Stations);
            }
            if knot.rgba.iter().any(|x| !x.is_finite()) {
                return Err(FillProfileError::Paint);
            }
            previous = knot.s;
            for (i, &value) in knot.rgba.iter().enumerate() {
                bounds[0][i] = bounds[0][i].min(value);
                bounds[1][i] = bounds[1][i].max(value);
                flat &= value.to_bits() == first[i].to_bits();
            }
            opaque &= knot.rgba[3] == 1.0;
        }
        Ok(Self {
            knots,
            bounds,
            flat,
            opaque,
        })
    }
    /// Source-ordered knots, shared by all prepared draws of this style.
    #[must_use]
    pub fn knots(&self) -> &[FillKnot] {
        &self.knots
    }
    /// Whether any record can contribute visible fill.
    #[must_use]
    pub fn has_visible_alpha(&self) -> bool {
        self.bounds[1][3] > 0.0
    }
    /// Whether every record is fully opaque, not just the endpoints.
    #[must_use]
    pub fn is_opaque(&self) -> bool {
        self.opaque
    }
    /// Whether all components are bitwise constant.
    #[must_use]
    pub fn is_flat(&self) -> bool {
        self.flat
    }
    /// Componentwise extrema, also bounding the degenerate/nonconvex field.
    #[must_use]
    pub fn bounds(&self) -> [[f32; 4]; 2] {
        self.bounds
    }
    /// Evaluate straight-alpha boundary color, right-continuous at a break.
    #[must_use]
    pub fn rgba_at(&self, s: f64) -> [f32; 4] {
        let s = s.clamp(0.0, 1.0);
        let index = self.knots.partition_point(|k| k.s <= s);
        if index == 0 {
            return self.knots[0].rgba;
        }
        let a = self.knots[index - 1];
        let Some(b) = self.knots.get(index) else {
            return a.rgba;
        };
        let t = ((s - a.s) / (b.s - a.s)).clamp(0.0, 1.0);
        std::array::from_fn(|i| {
            (f64::from(a.rgba[i]) + (f64::from(b.rgba[i]) - f64::from(a.rgba[i])) * t) as f32
        })
    }
    /// Select the outgoing side when two contours share a normalized station.
    #[must_use]
    pub fn endpoint_parameter(&self, s: f64, at_end: bool) -> f64 {
        if !at_end {
            return s;
        }
        let first = self.knots.partition_point(|k| k.s < s);
        let last = self.knots.partition_point(|k| k.s <= s);
        if last > first + 1 { s.next_down() } else { s }
    }
}

impl crate::Style {
    /// Retain an immutable profile instead of the legacy endpoint ramp.
    #[must_use]
    pub fn with_fill_profile(mut self, profile: FillProfile) -> Self {
        self.fill_profile = Some(std::sync::Arc::new(profile));
        self
    }
    /// Whether fill can contribute anywhere, including interior alpha extrema.
    #[must_use]
    pub fn draws_fill(&self) -> bool {
        self.fill_profile.as_ref().map_or_else(
            || self.fill_rgba[3] > 0.0 || self.fill_rgba_end[3] > 0.0,
            |p| p.has_visible_alpha(),
        )
    }
    /// Conservative opacity predicate for painter-order-safe occlusion pruning.
    #[must_use]
    pub fn has_opaque_fill(&self) -> bool {
        self.fill_profile.as_ref().map_or_else(
            || self.fill_rgba[3] == 1.0 && self.fill_rgba_end[3] == 1.0,
            |p| p.is_opaque(),
        )
    }
    /// Select the outgoing color at a disconnected contour endpoint.
    #[must_use]
    pub fn fill_endpoint_parameter(&self, s: f64, at_end: bool) -> f64 {
        self.fill_profile
            .as_ref()
            .map_or(s, |profile| profile.endpoint_parameter(s, at_end))
    }
    /// Total retained knots in both independent paint columns.
    #[must_use]
    pub fn profile_knots(&self) -> usize {
        self.stroke_profile.as_ref().map_or(0, |p| p.knots().len())
            + self.fill_profile.as_ref().map_or(0, |p| p.knots().len())
    }
}

/// Whether any record's fill paint differs from the first.
pub(crate) fn has_varying_paint(stage: &fmn_mobject::Stage, mob: fmn_mobject::Mob) -> bool {
    let Some(entry) = stage.get(mob) else {
        return false;
    };
    let count = entry.buffer.len();
    if count == 0 {
        return false;
    }
    let paint = |i| {
        entry
            .buffer
            .read(i, "fill_rgba")
            .and_then(|v| <[f32; 4]>::try_from(v.as_slice()).ok())
            .unwrap_or([0.0; 4])
    };
    let first = paint(0);
    (1..count).any(|i| paint(i) != first)
}

pub(crate) fn from_records(
    stage: &fmn_mobject::Stage,
    mob: fmn_mobject::Mob,
    decode: impl Fn([f32; 4]) -> [f32; 4],
) -> Result<Option<FillProfile>, crate::plan::SyncError> {
    use crate::plan::SyncError;
    let Some(entry) = stage.get(mob) else {
        return Ok(None);
    };
    let count = entry.buffer.len();
    if count == 0 {
        return Ok(None);
    }
    let paint = |i| {
        entry
            .buffer
            .read(i, "fill_rgba")
            .and_then(|v| <[f32; 4]>::try_from(v.as_slice()).ok())
            .unwrap_or([0.0; 4])
    };
    let first = paint(0);
    if (1..count).all(|i| paint(i) == first) {
        return Ok(None);
    }
    if count > MAX_FILL_PROFILE_KNOTS {
        return Err(SyncError::LimitExceeded {
            resource: "fill profile knots",
            requested: count as u64,
            limit: MAX_FILL_PROFILE_KNOTS as u64,
        });
    }
    // Validate even connector records; a nonfinite authored lane must not be
    // silently discarded by a zero-length curve or a subsequent color decode.
    if (0..count).any(|i| paint(i).iter().any(|v| !v.is_finite())) {
        return Err(SyncError::InvalidFillProfile {
            mob,
            source: FillProfileError::Paint,
        });
    }
    let stations = crate::stroke_profile::record_stations(stage, mob)?;
    if stations.is_empty() {
        return Ok(None);
    }
    let mut knots = Vec::new();
    for (index, s) in stations {
        let knot = FillKnot {
            s,
            rgba: decode(paint(index)),
        };
        // Do not erase an interior station just because its paint equals its
        // neighbor: boundary geometry is independent of style edits.
        knots.push(knot);
    }
    FillProfile::new(knots)
        .map(Some)
        .map_err(|source| SyncError::InvalidFillProfile { mob, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn k(s: f64, rgba: [f32; 4]) -> FillKnot {
        FillKnot { s, rgba }
    }
    #[test]
    fn interior_color_and_alpha_are_not_replaced_by_endpoints() {
        let p = FillProfile::new(vec![
            k(0., [0.; 4]),
            k(0.5, [0., 0., 1., 1.]),
            k(1., [0.; 4]),
        ])
        .unwrap();
        assert!(p.has_visible_alpha());
        assert!(!p.is_flat());
        assert!(!p.is_opaque());
        assert_eq!(p.rgba_at(0.25), [0., 0., 0.5, 0.5]);
        assert_eq!(p.rgba_at(0.5), [0., 0., 1., 1.]);
        assert_eq!(p.rgba_at(-1.), [0.; 4]);
        assert_eq!(p.rgba_at(2.), [0.; 4]);
    }
    #[test]
    fn breaks_have_two_distinct_one_sided_colors() {
        let red = [1., 0., 0., 1.];
        let blue = [0., 0., 1., 1.];
        let p = FillProfile::new(vec![k(0., red), k(0.5, red), k(0.5, blue), k(1., blue)]).unwrap();
        assert_eq!(p.rgba_at(0.5), blue);
        assert_eq!(p.rgba_at(p.endpoint_parameter(0.5, true)), red);
        assert!(p.is_opaque());
    }
    #[test]
    fn malformed_profiles_refuse_before_retention() {
        assert_eq!(FillProfile::new(vec![]), Err(FillProfileError::KnotCount));
        for s in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert_eq!(
                FillProfile::new(vec![k(s, [0.; 4])]),
                Err(FillProfileError::Stations)
            );
        }
        assert_eq!(
            FillProfile::new(vec![k(0.8, [0.; 4]), k(0.2, [0.; 4])]),
            Err(FillProfileError::Stations)
        );
        for v in [f32::NAN, f32::INFINITY] {
            assert_eq!(
                FillProfile::new(vec![k(0., [v; 4])]),
                Err(FillProfileError::Paint)
            );
        }
    }
}
