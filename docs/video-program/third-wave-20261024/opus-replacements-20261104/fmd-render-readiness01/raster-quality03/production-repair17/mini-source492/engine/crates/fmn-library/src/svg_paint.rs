//! Renderable SVG paint layers over Chisel's boolean and arc-length authorities.
//!
//! The document parser remains the only XML/style parser. Even-odd filled sets
//! are normalized through its bounded boolean kernel; stroke paths retain their
//! original curves. Dash placement uses true arc length and restarts at moves.
use fmn_geom::{
    ArcLengthTable, BooleanError, BooleanLimits, BooleanOperation, BooleanOptions, FillRule,
    GeomError, QuadPath, path_boolean,
};

use super::{SvgDocument, SvgError, SvgShape, shape_style};
use crate::{VMobject, style::Style, vmobject::v_group};
use fmn_core::color::Srgb;

/// Aggregate limit on native points emitted by one document import.
pub const MAX_SVG_PAINT_POINTS: usize = 1_048_576;
/// Aggregate limit on visible dash pieces emitted by one document import.
pub const MAX_SVG_DASH_PIECES: usize = 4096;
const MAX_DASH_STEPS: usize = 131_072;

/// Constructor overrides, applied before fill/stroke layers are separated.
#[derive(Clone, Copy, Debug, Default)]
pub struct SvgPaintOverrides {
    /// Replaces the resolved fill RGB, not its opacity.
    pub fill_color: Option<Srgb>,
    /// Replaces the resolved stroke RGB, not its opacity.
    pub stroke_color: Option<Srgb>,
    /// Replaces the flattened fill alpha.
    pub fill_opacity: Option<f64>,
    /// Replaces the flattened stroke alpha.
    pub stroke_opacity: Option<f64>,
    /// Replaces stroke width in native manim units.
    pub stroke_width: Option<f64>,
}

/// A document that cannot be prepared without losing SVG paint semantics.
#[derive(Debug)]
pub enum SvgPaintError {
    /// Chisel's original document diagnostic.
    Document(SvgError),
    /// Malformed native path geometry.
    Geometry(GeomError),
    /// Bounded even-odd normalization refused the input.
    Boolean(BooleanError),
    /// Invalid or unrepresentable paint parameters.
    Invalid(&'static str),
    /// Aggregate work or output budget exhausted.
    Limit(&'static str),
}
impl std::fmt::Display for SvgPaintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(e) => e.fmt(f),
            Self::Geometry(e) => e.fmt(f),
            Self::Boolean(e) => e.fmt(f),
            Self::Invalid(message) => write!(f, "SVG paint: {message}"),
            Self::Limit(message) => write!(f, "SVG paint budget exceeded: {message}"),
        }
    }
}
impl std::error::Error for SvgPaintError {}
impl From<GeomError> for SvgPaintError {
    fn from(error: GeomError) -> Self {
        Self::Geometry(error)
    }
}
impl From<BooleanError> for SvgPaintError {
    fn from(error: BooleanError) -> Self {
        Self::Boolean(error)
    }
}

struct Budget {
    points: usize,
    pieces: usize,
    steps: usize,
    boolean: BooleanLimits,
}
impl Budget {
    fn points(&mut self, count: usize) -> Result<(), SvgPaintError> {
        self.points = self
            .points
            .checked_add(count)
            .filter(|&n| n <= MAX_SVG_PAINT_POINTS)
            .ok_or(SvgPaintError::Limit("native points"))?;
        Ok(())
    }
}

impl SvgPaintOverrides {
    fn apply(self, mut style: Style) -> Result<Style, SvgPaintError> {
        for color in [self.fill_color, self.stroke_color].into_iter().flatten() {
            if [color.r, color.g, color.b]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err(SvgPaintError::Invalid(
                    "RGB components must be finite and in [0, 1]",
                ));
            }
        }
        for alpha in [self.fill_opacity, self.stroke_opacity]
            .into_iter()
            .flatten()
        {
            if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
                return Err(SvgPaintError::Invalid(
                    "opacities must be finite and in [0, 1]",
                ));
            }
        }
        if let Some(width) = self.stroke_width {
            if !width.is_finite() || !(0.0..=f64::from(f32::MAX)).contains(&width) {
                return Err(SvgPaintError::Invalid(
                    "stroke width must be nonnegative and f32-representable",
                ));
            }
            style.stroke_width = width;
        }
        if let Some(color) = self.fill_color {
            style.fill_color = color;
        }
        if let Some(color) = self.stroke_color {
            style.stroke_color = color;
        }
        if let Some(alpha) = self.fill_opacity {
            style.fill_opacity = alpha;
        }
        if let Some(alpha) = self.stroke_opacity {
            style.stroke_opacity = alpha;
        }
        Ok(style)
    }
}

/// Prepare a resolved SVG as renderable native paint layers.
///
/// Keeps one top-level child per pointful authored shape. Ordinary solid,
/// nonzero shapes keep their original single path. An even-odd or dashed shape
/// owns distinct fill/stroke children, so filling a dashed path never closes
/// its dashes and normalization never deletes coincident stroked contours.
/// Even-odd fill uses the existing boolean tolerance in viewport user units.
/// Native round caps and fixed native miter limits remain the renderer's look.
///
/// # Errors
/// Returns typed parse/geometry/work refusals before publishing any live state.
pub fn svg_document_with_paints(
    document: &SvgDocument,
    overrides: SvgPaintOverrides,
) -> Result<VMobject, SvgPaintError> {
    // Validate even an empty document before doing any geometry work.
    overrides.apply(Style::default())?;
    let mut budget = Budget {
        points: 0,
        pieces: 0,
        steps: 0,
        boolean: BooleanLimits::default(),
    };
    let mut shapes = Vec::new();
    for shape in &document.shapes {
        if shape.path.has_points() {
            shapes.push(prepare_shape(shape, overrides, &mut budget)?);
        }
    }
    Ok(v_group(shapes))
}

/// Parse and prepare SVG paint layers through the same bounded native importer.
///
/// # Errors
/// See [`svg_document_with_paints`] and [`SvgDocument::parse`].
pub fn svg_mobject_with_paints(
    bytes: &[u8],
    overrides: SvgPaintOverrides,
) -> Result<VMobject, SvgPaintError> {
    let document = SvgDocument::parse(bytes).map_err(SvgPaintError::Document)?;
    svg_document_with_paints(&document, overrides)
}

fn prepare_shape(
    shape: &SvgShape,
    overrides: SvgPaintOverrides,
    budget: &mut Budget,
) -> Result<VMobject, SvgPaintError> {
    let style = overrides.apply(shape_style(&shape.style))?;
    let even_odd = shape.style.fill_rule == FillRule::EvenOdd;
    let dashed = shape
        .style
        .stroke_dasharray
        .iter()
        .any(|&value| value != 0.0);
    if !even_odd && !dashed {
        budget.points(shape.path.points().len())?;
        // Preserve existing ordinary-solid import bits and native stroke look.
        return Ok(VMobject::from_path(&shape.path).with_style(style));
    }
    let mut layers = Vec::new();
    // Retain both paint roles even when a constructor makes one invisible.
    // Later styling must reveal the same filled set/dashes, not a solid path.
    {
        let fill = if even_odd {
            let result = path_boolean(
                &shape.path,
                &QuadPath::new(),
                BooleanOperation::Union,
                BooleanOptions {
                    subject_fill_rule: FillRule::EvenOdd,
                    limits: budget.boolean,
                    ..Default::default()
                },
            )?;
            let used = result.stats;
            budget.boolean.max_flattened_segments = budget
                .boolean
                .max_flattened_segments
                .saturating_sub(used.flattened_segments);
            budget.boolean.max_pair_tests = budget
                .boolean
                .max_pair_tests
                .saturating_sub(used.pair_tests);
            budget.boolean.max_intersections = budget
                .boolean
                .max_intersections
                .saturating_sub(used.intersections);
            budget.boolean.max_classification_tests = budget
                .boolean
                .max_classification_tests
                .saturating_sub(used.classification_tests);
            budget.boolean.max_output_segments = budget
                .boolean
                .max_output_segments
                .saturating_sub(used.output_segments);
            result.path
        } else {
            shape.path.clone()
        };
        budget.points(fill.points().len())?;
        layers.push(VMobject::from_path(&fill).with_style(Style {
            stroke_opacity: 0.0,
            stroke_width: 0.0,
            ..style
        }));
    }
    {
        let stroke_style = Style {
            fill_opacity: 0.0,
            ..style
        };
        if dashed {
            for path in dash_paths(shape, budget)? {
                layers.push(VMobject::from_path(&path).with_style(stroke_style));
            }
        } else {
            budget.points(shape.path.points().len())?;
            layers.push(VMobject::from_path(&shape.path).with_style(stroke_style));
        }
    }
    Ok(v_group(layers).with_style(style))
}

fn dash_paths(shape: &SvgShape, budget: &mut Budget) -> Result<Vec<QuadPath>, SvgPaintError> {
    let raw = &shape.style.stroke_dasharray;
    if raw.len() > MAX_SVG_DASH_PIECES
        || raw.iter().any(|v| !v.is_finite() || *v < 0.0)
        || !shape.style.stroke_dashoffset.is_finite()
    {
        return Err(SvgPaintError::Invalid(
            "dash pattern must be bounded, finite and nonnegative",
        ));
    }
    let mut pattern = raw.clone();
    if pattern.len() % 2 == 1 {
        pattern.extend_from_slice(raw);
    }
    let period: f64 = pattern.iter().sum();
    if !period.is_finite() || period <= 0.0 {
        return Err(SvgPaintError::Invalid(
            "dash period must be positive and finite",
        ));
    }
    if pattern.iter().step_by(2).any(|&v| v == 0.0) {
        return Err(SvgPaintError::Invalid(
            "zero-length painted dashes require dot-cap support; use positive dash lengths",
        ));
    }
    let phase = shape.style.stroke_dashoffset.rem_euclid(period);
    let mut output = Vec::new();
    for points in shape.path.subpaths() {
        let path = QuadPath::from_points(points.to_vec())?;
        let table = ArcLengthTable::for_path(&path);
        let length = table.total();
        if !length.is_finite() {
            return Err(SvgPaintError::Invalid("nonfinite stroke length"));
        }
        if length == 0.0 {
            continue;
        }
        let mut intervals: Vec<(f64, f64)> = Vec::new();
        let mut position = -phase;
        let mut index: usize = 0;
        while position < length {
            budget.steps += 1;
            if budget.steps > MAX_DASH_STEPS {
                return Err(SvgPaintError::Limit("dash steps"));
            }
            let step = pattern[index % pattern.len()];
            let next = position + step;
            if step > 0.0 && next <= position {
                return Err(SvgPaintError::Invalid(
                    "dash length is below coordinate precision",
                ));
            }
            if index.is_multiple_of(2) && next > 0.0 {
                let interval = (position.max(0.0), next.min(length));
                if let Some(last) = intervals.last_mut().filter(|last| last.1 == interval.0) {
                    last.1 = interval.1;
                } else {
                    if intervals.len() + budget.pieces >= MAX_SVG_DASH_PIECES {
                        return Err(SvgPaintError::Limit("dash pieces"));
                    }
                    intervals.push(interval);
                }
            }
            position = next;
            index += 1;
        }
        let mut pieces = Vec::with_capacity(intervals.len());
        for &(a, b) in &intervals {
            let locate = |distance: f64| {
                table
                    .curve_and_t_at(&path, distance / length)
                    .ok_or(SvgPaintError::Invalid(
                        "dash distance could not be resolved",
                    ))
            };
            let (first, start_t) = locate(a)?;
            let (last, end_t) = locate(b)?;
            let source = &path.points()[2 * first..2 * last + 3];
            // partial_points intentionally retains collapsed animation flanks.
            // Limit it to the touched curves, then drop those flanks. Do not
            // allocate a full source-path copy for every tiny painted dash.
            // Reserve the upper bound before either extraction allocation.
            budget.points(source.len())?;
            let curves = (last - first + 1) as f64;
            let (points, i1, i4) = QuadPath::partial_points(
                source,
                start_t / curves,
                ((last - first) as f64 + end_t) / curves,
            )
            .ok_or(SvgPaintError::Invalid(
                "dash interval could not be extracted",
            ))?;
            pieces.push(QuadPath::from_points(points[i1..i4].to_vec())?);
        }
        // A painted dash crossing a closed contour's seam has a join, not two
        // new end caps. Stitch only that seam, retaining the native curves.
        if pieces.len() > 1
            && points.first() == points.last()
            && intervals.first().is_some_and(|p| p.0 == 0.0)
            && intervals.last().is_some_and(|p| p.1 == length)
        {
            let first = pieces.remove(0);
            let last = pieces.pop().expect("at least two pieces");
            let mut joined = last.points().to_vec();
            joined.extend_from_slice(&first.points()[1..]);
            pieces.insert(0, QuadPath::from_points(joined)?);
        }
        budget.pieces += pieces.len();
        output.extend(pieces);
    }
    Ok(output)
}
