//! The Arc lineage (§12.1): `Arc`, `ArcBetweenPoints`, `CurvedArrow`,
//! `CurvedDoubleArrow`, `Circle`, `Dot`, `SmallDot`, `Ellipse`,
//! `AnnularSector`, `Sector`, and `Annulus`.
//!
//! # One arc-density rule (BN-09)
//!
//! The Reference decides how many quadratic components trace an arc in
//! three mutually inconsistent places — `int(15·|θ|/TAU)+1` in
//! `Arc.__init__`, `ceil(8·|θ|/TAU)` in `add_arc_to`, and a flat `8` in
//! `quadratic_bezier_points_for_arc` — so the same quarter arc is 4, 2, or
//! 8 components depending on which code path happened to build it. Every
//! arc here goes through `fmn_geom::bezier::arc_n_components`:
//! `max(1, ceil(16·|θ|/TAU))`, which agrees with the `Arc` convention (the
//! finest of the three) at every common angle and is never coarser than
//! any of them. An explicit `n_components` is honoured when it satisfies
//! Chisel's checked component budget; invalid counts are typed refusals.
//! Behaviour Note BN-09 carries the migration guidance.

use fmn_core::color::Srgb;
use fmn_core::constants::{
    BLACK, DEFAULT_LIGHT_COLOR, DEFAULT_MOBJECT_COLOR, ORIGIN, OUT, RED, TAU,
};
use fmn_core::types::Vec3;
use fmn_geom::{GeomError, QuadPath, bezier, space_ops};
use fmn_mobject::{Mobject, ShapeTag};

use crate::poly::{ArrowTip, arc_between_points};
use crate::style::Style;
use crate::tip::{TipEnd, attach_tip};
use crate::vmobject::VMobject;

/// The Reference's `DEFAULT_DOT_RADIUS`.
pub const DEFAULT_DOT_RADIUS: f64 = 0.08;
/// The Reference's `DEFAULT_SMALL_DOT_RADIUS`.
pub const DEFAULT_SMALL_DOT_RADIUS: f64 = 0.04;

/// `Arc(start_angle, angle, radius, n_components, arc_center)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    start_angle: f64,
    angle: f64,
    radius: f64,
    n_components: Option<usize>,
    arc_center: Vec3,
    style: Style,
}

impl Default for Arc {
    fn default() -> Self {
        Self::new()
    }
}

impl Arc {
    /// The Reference's default arc: a quarter turn of unit radius at the
    /// origin.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_angle: 0.0,
            angle: TAU / 4.0,
            radius: 1.0,
            n_components: None,
            arc_center: ORIGIN,
            style: Style::default(),
        }
    }

    /// Angle of the first anchor, measured from `+x`, counter-clockwise.
    #[must_use]
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Subtended angle, signed.
    #[must_use]
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Radius.
    #[must_use]
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// Override the arc-density rule for this arc (BN-09 honours valid
    /// explicit counts exactly).
    #[must_use]
    pub fn n_components(mut self, n: usize) -> Self {
        self.n_components = Some(n);
        self
    }

    /// Centre of curvature.
    #[must_use]
    pub fn arc_center(mut self, center: Vec3) -> Self {
        self.arc_center = center;
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The number of quadratic components this arc will use.
    pub fn component_count(&self) -> Result<usize, GeomError> {
        match self.n_components {
            Some(count) => bezier::arc_point_count(self.angle, count).map(|_| count),
            None => bezier::arc_n_components(self.angle),
        }
    }

    /// Build the detached mobject.
    ///
    /// # Errors
    /// The typed arc-component refusal produced before allocation.
    pub fn build(self) -> Result<VMobject, GeomError> {
        let path = QuadPath::try_arc(
            self.start_angle,
            self.angle,
            self.radius,
            self.arc_center,
            self.n_components,
        )?;
        Ok(VMobject::from_path(&path)
            .with_style(self.style)
            .with_shape(ShapeTag::Arc {
                center: self.arc_center,
                radius: self.radius,
                start_angle: self.start_angle,
                angle: self.angle,
            }))
    }
}

impl TryFrom<Arc> for VMobject {
    type Error = GeomError;

    fn try_from(arc: Arc) -> Result<Self, Self::Error> {
        arc.build()
    }
}

impl TryFrom<Arc> for Mobject {
    type Error = GeomError;

    fn try_from(arc: Arc) -> Result<Self, Self::Error> {
        arc.build().map(Into::into)
    }
}

/// `ArcBetweenPoints(start, end, angle)`: an arc through two points,
/// subtending `angle` at its centre. A zero angle is the straight segment,
/// as in the Reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcBetweenPoints {
    start: Vec3,
    end: Vec3,
    angle: f64,
    n_components: Option<usize>,
    style: Style,
}

impl ArcBetweenPoints {
    /// An arc from `start` to `end` subtending `TAU/4`.
    #[must_use]
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            start,
            end,
            angle: TAU / 4.0,
            n_components: None,
            style: Style::default(),
        }
    }

    /// Subtended angle.
    #[must_use]
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Override the arc-density rule.
    #[must_use]
    pub fn n_components(mut self, n: usize) -> Self {
        self.n_components = Some(n);
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the detached mobject.
    ///
    /// # Errors
    /// The typed arc-component refusal produced before allocation.
    pub fn build(self) -> Result<VMobject, GeomError> {
        Ok(VMobject::from_points(arc_between_points(
            self.start,
            self.end,
            self.angle,
            self.n_components,
        )?)
        .with_style(self.style))
    }
}

impl TryFrom<ArcBetweenPoints> for VMobject {
    type Error = GeomError;

    fn try_from(arc: ArcBetweenPoints) -> Result<Self, Self::Error> {
        arc.build()
    }
}

impl TryFrom<ArcBetweenPoints> for Mobject {
    type Error = GeomError;

    fn try_from(arc: ArcBetweenPoints) -> Result<Self, Self::Error> {
        arc.build().map(Into::into)
    }
}

/// `CurvedArrow(start_point, end_point, angle)`: an [`ArcBetweenPoints`]
/// with a tip at its end.
///
/// # Errors
/// The typed arc-component refusal produced before allocation.
pub fn curved_arrow(
    start: Vec3,
    end: Vec3,
    angle: f64,
    style: Style,
) -> Result<VMobject, GeomError> {
    let arc = ArcBetweenPoints::new(start, end)
        .angle(angle)
        .style(style)
        .build()?;
    Ok(attach_tip(
        arc,
        ArrowTip::new().color(style.stroke_color),
        TipEnd::End,
    ))
}

/// `CurvedDoubleArrow`: a [`curved_arrow`] with a tip at both ends.
///
/// # Errors
/// The typed arc-component refusal produced before allocation.
pub fn curved_double_arrow(
    start: Vec3,
    end: Vec3,
    angle: f64,
    style: Style,
) -> Result<VMobject, GeomError> {
    let once = curved_arrow(start, end, angle, style)?;
    Ok(attach_tip(
        once,
        ArrowTip::new().color(style.stroke_color),
        TipEnd::Start,
    ))
}

/// `Circle(radius, arc_center)`: an [`Arc`] of a full turn, stroked red by
/// default as in the Reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    arc: Arc,
}

impl Default for Circle {
    fn default() -> Self {
        Self::new()
    }
}

impl Circle {
    /// A unit circle at the origin.
    #[must_use]
    pub fn new() -> Self {
        Self {
            arc: Arc::new().angle(TAU).style(Style::default().stroke(
                RED,
                Style::default().stroke_width,
                1.0,
            )),
        }
    }

    /// Radius.
    #[must_use]
    pub fn radius(mut self, radius: f64) -> Self {
        self.arc = self.arc.radius(radius);
        self
    }

    /// Centre.
    #[must_use]
    pub fn arc_center(mut self, center: Vec3) -> Self {
        self.arc = self.arc.arc_center(center);
        self
    }

    /// Angle of the first anchor.
    #[must_use]
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.arc = self.arc.start_angle(angle);
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.arc = self.arc.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.arc = self.arc.style(style);
        self
    }

    /// Build the detached mobject.
    #[must_use]
    pub fn build(self) -> VMobject {
        let center = self.arc.arc_center;
        let radius = self.arc.radius;
        self.arc
            .build()
            .expect("Circle's fixed full-turn component request is valid")
            .with_shape(ShapeTag::Circle { center, radius })
    }

    /// Build with an explicit component count: the Reference's
    /// `Circle(n_components=...)`, which reaches `Arc`. Unlike the default
    /// density, an explicit count can be invalid.
    ///
    /// # Errors
    /// As [`Arc::build`] for an invalid `n_components`.
    pub fn build_with_components(self, n_components: usize) -> Result<VMobject, GeomError> {
        let center = self.arc.arc_center;
        let radius = self.arc.radius;
        Ok(self
            .arc
            .n_components(n_components)
            .build()?
            .with_shape(ShapeTag::Circle { center, radius }))
    }
}

impl From<Circle> for Mobject {
    fn from(c: Circle) -> Self {
        c.build().into()
    }
}

/// `Dot(point, radius)`: a filled circle with no stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dot {
    center: Vec3,
    radius: f64,
    style: Style,
}

impl Default for Dot {
    fn default() -> Self {
        Self::new()
    }
}

impl Dot {
    /// A dot at the origin, at the Reference's default radius.
    #[must_use]
    pub fn new() -> Self {
        Self {
            center: ORIGIN,
            radius: DEFAULT_DOT_RADIUS,
            style: Style::default()
                .fill(DEFAULT_MOBJECT_COLOR, 1.0)
                .stroke(BLACK, 0.0, 1.0),
        }
    }

    /// `SmallDot`: the same at [`DEFAULT_SMALL_DOT_RADIUS`].
    #[must_use]
    pub fn small() -> Self {
        Self::new().radius(DEFAULT_SMALL_DOT_RADIUS)
    }

    /// Centre.
    #[must_use]
    pub fn point(mut self, point: Vec3) -> Self {
        self.center = point;
        self
    }

    /// Radius.
    #[must_use]
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// Set the fill colour (a dot is a filled shape).
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.fill(color, self.style.fill_opacity);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the detached mobject.
    #[must_use]
    pub fn build(self) -> VMobject {
        Arc::new()
            .angle(TAU)
            .radius(self.radius)
            .arc_center(self.center)
            .style(self.style)
            .build()
            .expect("Dot's fixed full-turn component request is valid")
            .with_shape(ShapeTag::Dot {
                center: self.center,
                radius: self.radius,
            })
    }
}

impl From<Dot> for Mobject {
    fn from(d: Dot) -> Self {
        d.build().into()
    }
}

/// `Ellipse(width, height)`: a circle stretched to the given extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    width: f64,
    height: f64,
    center: Vec3,
    start_angle: f64,
    style: Style,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self::new()
    }
}

impl Ellipse {
    /// The Reference's default ellipse, 2 wide and 1 tall.
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 2.0,
            height: 1.0,
            center: ORIGIN,
            start_angle: 0.0,
            style: Style::default().stroke(RED, Style::default().stroke_width, 1.0),
        }
    }

    /// Full width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Full height.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Centre.
    #[must_use]
    pub fn arc_center(mut self, center: Vec3) -> Self {
        self.center = center;
        self
    }

    /// Angle of the inherited circle's first anchor. It remains observable
    /// after the axis stretch through path traversal and point queries.
    #[must_use]
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the detached mobject.
    ///
    /// Stretching is not a shape-preserving transform, so the circle's tag
    /// does not survive it — an ellipse is not a circle, and claiming the
    /// circle kernel for one would draw the wrong picture.
    #[must_use]
    pub fn build(self) -> VMobject {
        Circle::new()
            .arc_center(self.center)
            .start_angle(self.start_angle)
            .style(self.style)
            .build()
            .with_width(self.width, true)
            .with_height(self.height, true)
            .with_shape(ShapeTag::General)
    }
}

impl From<Ellipse> for Mobject {
    fn from(e: Ellipse) -> Self {
        e.build().into()
    }
}

/// `AnnularSector(angle, start_angle, inner_radius, outer_radius,
/// arc_center)`: the region between two concentric arcs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnnularSector {
    angle: f64,
    start_angle: f64,
    inner_radius: f64,
    outer_radius: f64,
    center: Vec3,
    style: Style,
}

impl Default for AnnularSector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnularSector {
    /// The Reference's defaults: a quarter turn between radii 1 and 2,
    /// filled and unstroked.
    #[must_use]
    pub fn new() -> Self {
        Self {
            angle: TAU / 4.0,
            start_angle: 0.0,
            inner_radius: 1.0,
            outer_radius: 2.0,
            center: ORIGIN,
            style: Style::default()
                .fill(DEFAULT_LIGHT_COLOR, 1.0)
                .stroke(BLACK, 0.0, 1.0),
        }
    }

    /// `Sector(angle, radius)`: an annular sector with no hole.
    #[must_use]
    pub fn sector(angle: f64, radius: f64) -> Self {
        Self::new()
            .angle(angle)
            .inner_radius(0.0)
            .outer_radius(radius)
    }
}

/// Top-level `Sector(angle, radius)` constructor matching the Python class.
#[must_use]
pub fn sector(angle: f64, radius: f64) -> AnnularSector {
    AnnularSector::sector(angle, radius)
}

/// Top-level `SmallDot(point)` constructor matching the Python class.
#[must_use]
pub fn small_dot(point: Vec3) -> Dot {
    Dot::small().point(point)
}

impl AnnularSector {
    /// Subtended angle.
    #[must_use]
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Angle of the first anchor.
    #[must_use]
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Inner radius.
    #[must_use]
    pub fn inner_radius(mut self, radius: f64) -> Self {
        self.inner_radius = radius;
        self
    }

    /// Outer radius.
    #[must_use]
    pub fn outer_radius(mut self, radius: f64) -> Self {
        self.outer_radius = radius;
        self
    }

    /// Centre.
    #[must_use]
    pub fn arc_center(mut self, center: Vec3) -> Self {
        self.center = center;
        self
    }

    /// Set the fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.fill(color, self.style.fill_opacity);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the detached mobject.
    ///
    /// # Errors
    /// The typed arc-component refusal produced before allocation.
    pub fn build(self) -> Result<VMobject, GeomError> {
        let arc_of = |radius: f64| -> Result<Vec<Vec3>, GeomError> {
            Ok(
                QuadPath::try_arc(self.start_angle, self.angle, radius, self.center, None)?
                    .points()
                    .to_vec(),
            )
        };
        let inner = arc_of(self.inner_radius)?;
        let outer = arc_of(self.outer_radius)?;

        let mut path = QuadPath::new();
        let mut reversed = inner.clone();
        reversed.reverse();
        let _ = path.set_points(reversed);
        if let Some(&first) = outer.first() {
            let _ = path.add_line_to(first, true);
        }
        let _ = path.add_subpath(&outer);
        if let Some(&last) = inner.last() {
            let _ = path.add_line_to(last, true);
        }
        Ok(VMobject::from_path(&path).with_style(self.style))
    }
}

impl TryFrom<AnnularSector> for VMobject {
    type Error = GeomError;

    fn try_from(sector: AnnularSector) -> Result<Self, Self::Error> {
        sector.build()
    }
}

impl TryFrom<AnnularSector> for Mobject {
    type Error = GeomError;

    fn try_from(sector: AnnularSector) -> Result<Self, Self::Error> {
        sector.build().map(Into::into)
    }
}

/// `Annulus(inner_radius, outer_radius, center)`: a full ring, drawn as an
/// outer circle and a counter-wound inner one so the hole is a hole under
/// the nonzero winding rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Annulus {
    inner_radius: f64,
    outer_radius: f64,
    center: Vec3,
    style: Style,
}

impl Default for Annulus {
    fn default() -> Self {
        Self::new()
    }
}

impl Annulus {
    /// The Reference's defaults: radii 1 and 2, filled and unstroked.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner_radius: 1.0,
            outer_radius: 2.0,
            center: ORIGIN,
            style: Style::default()
                .fill(DEFAULT_LIGHT_COLOR, 1.0)
                .stroke(BLACK, 0.0, 1.0),
        }
    }

    /// Inner radius.
    #[must_use]
    pub fn inner_radius(mut self, radius: f64) -> Self {
        self.inner_radius = radius;
        self
    }

    /// Outer radius.
    #[must_use]
    pub fn outer_radius(mut self, radius: f64) -> Self {
        self.outer_radius = radius;
        self
    }

    /// Centre.
    #[must_use]
    pub fn center(mut self, center: Vec3) -> Self {
        self.center = center;
        self
    }

    /// Set the fill colour.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.fill(color, self.style.fill_opacity);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the detached mobject.
    #[must_use]
    pub fn build(self) -> VMobject {
        let outer: Vec<Vec3> = QuadPath::try_arc(0.0, TAU, self.outer_radius, ORIGIN, None)
            .expect("Annulus's fixed full-turn outer arc is valid")
            .points()
            .to_vec();
        let inner: Vec<Vec3> = QuadPath::try_arc(0.0, -TAU, self.inner_radius, ORIGIN, None)
            .expect("Annulus's fixed full-turn inner arc is valid")
            .points()
            .to_vec();
        let mut path = QuadPath::new();
        let _ = path.add_subpath(&outer);
        let _ = path.add_subpath(&inner);
        VMobject::from_path(&path)
            .with_style(self.style)
            .shifted(self.center)
    }
}

impl From<Annulus> for Mobject {
    fn from(a: Annulus) -> Self {
        a.build().into()
    }
}

/// Reference `Arc.get_arc_center`: the intersection of the normals at the
/// first two anchors.
#[must_use]
pub fn arc_center_of(points: &[Vec3]) -> Option<Vec3> {
    let (&a1, &h, &a2) = (points.first()?, points.get(1)?, points.get(2)?);
    let t1 = sub(h, a1);
    let t2 = sub(h, a2);
    let n1 = space_ops::rotate_vector(t1, TAU / 4.0, OUT);
    let n2 = space_ops::rotate_vector(t2, TAU / 4.0, OUT);
    Some(space_ops::find_intersection(
        a1,
        n1,
        a2,
        n2,
        space_ops::DEFAULT_INTERSECTION_THRESHOLD,
    ))
}

/// Reference `Arc.get_start_angle`: the angle of the first anchor about
/// the arc centre, in `[0, TAU)`.
#[must_use]
pub fn start_angle_of(points: &[Vec3]) -> Option<f64> {
    let center = arc_center_of(points)?;
    Some(space_ops::angle_of_vector(sub(*points.first()?, center)).rem_euclid(TAU))
}

/// Reference `Arc.get_stop_angle`.
#[must_use]
pub fn stop_angle_of(points: &[Vec3]) -> Option<f64> {
    let center = arc_center_of(points)?;
    Some(space_ops::angle_of_vector(sub(*points.last()?, center)).rem_euclid(TAU))
}

/// Reference `Circle.get_radius`: the distance from the centre of the
/// bounding box to the first anchor.
#[must_use]
pub fn radius_of(shape: &VMobject) -> f64 {
    let Some(&start) = shape.points().first() else {
        return 0.0;
    };
    space_ops::get_norm(sub(start, shape.center_point()))
}

/// Reference `Circle.point_at_angle`: the point at the given angle,
/// measured from the circle's own start angle.
///
/// The Reference walks `point_from_proportion`, which now means **true**
/// arc length (BN-03) — on a circle the two agree, and on anything else
/// ours is the one that means what it says.
#[must_use]
pub fn point_at_angle(shape: &VMobject, angle: f64) -> Option<Vec3> {
    let path = shape.path().ok()?;
    let start = start_angle_of(shape.points())?;
    path.point_from_proportion(((angle - start).rem_euclid(TAU)) / TAU)
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_vec(a: Vec3, b: Vec3) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9)
    }

    #[test]
    fn arc_density_is_one_rule() {
        // BN-09: 16 components for a full turn, and agreement with the
        // Reference's Arc convention at the common angles.
        assert_eq!(Arc::new().angle(TAU).component_count(), Ok(16));
        assert_eq!(Arc::new().angle(PI).component_count(), Ok(8));
        assert_eq!(Arc::new().angle(TAU / 4.0).component_count(), Ok(4));
        assert_eq!(Arc::new().angle(-TAU / 4.0).component_count(), Ok(4));
        assert_eq!(Arc::new().angle(1e-9).component_count(), Ok(1));
        // An explicit count is honoured verbatim.
        assert_eq!(
            Arc::new().angle(TAU).n_components(3).component_count(),
            Ok(3)
        );
        let arc = Arc::new()
            .angle(TAU)
            .n_components(3)
            .build()
            .expect("three components are valid");
        assert_eq!(arc.points().len(), 7);
    }

    #[test]
    fn arc_anchors_sit_on_the_circle() {
        let arc = Arc::new()
            .start_angle(0.3)
            .angle(1.7)
            .radius(2.5)
            .arc_center([1.0, -2.0, 0.0])
            .build()
            .expect("the fixture arc is valid");
        let points = arc.points();
        for anchor in points.iter().step_by(2) {
            let r = space_ops::get_norm(sub(*anchor, [1.0, -2.0, 0.0]));
            assert!(close(r, 2.5), "anchor at radius {r}");
        }
        assert!(close(start_angle_of(points).unwrap(), 0.3));
        assert!(close(stop_angle_of(points).unwrap(), 2.0));
        assert!(close_vec(arc_center_of(points).unwrap(), [1.0, -2.0, 0.0]));
    }

    #[test]
    fn circle_is_closed_red_and_tagged() {
        let circle = Circle::new().radius(3.0).build();
        assert_eq!(circle.points().len(), 33, "16 components, shared anchors");
        assert!(close_vec(circle.points()[0], [3.0, 0.0, 0.0]));
        assert!(
            close(
                space_ops::get_norm(sub(*circle.points().last().unwrap(), circle.points()[0])),
                0.0
            ),
            "a circle closes on itself"
        );
        assert_eq!(circle.style().stroke_color, RED);
        assert!(matches!(circle.shape(), ShapeTag::Circle { radius, .. } if close(radius, 3.0)));
        assert!(close(radius_of(&circle), 3.0));
    }

    #[test]
    fn circle_honours_an_explicit_component_count() {
        // The Reference's Circle(n_components=...) reaches Arc; 2**14 is a
        // real corpus request (lies.py).
        for count in [3, 32, 1 << 14] {
            let circle = Circle::new()
                .radius(2.0)
                .build_with_components(count)
                .expect("a valid explicit count");
            assert_eq!(circle.points().len(), 2 * count + 1);
            assert!(
                matches!(circle.shape(), ShapeTag::Circle { radius, .. } if close(radius, 2.0))
            );
            for anchor in circle.points().iter().step_by(2) {
                assert!(close(space_ops::get_norm(*anchor), 2.0));
            }
        }
        assert_eq!(
            Circle::new().build_with_components(0),
            Err(GeomError::ZeroArcComponents)
        );
        assert!(Circle::new().build_with_components(usize::MAX / 4).is_err());
    }

    #[test]
    fn circle_point_at_angle_walks_the_circle() {
        let circle = Circle::new().radius(2.0).build();
        for (angle, expected) in [
            (0.0, [2.0, 0.0, 0.0]),
            (PI / 2.0, [0.0, 2.0, 0.0]),
            (PI, [-2.0, 0.0, 0.0]),
        ] {
            let p = point_at_angle(&circle, angle).unwrap();
            assert!(
                (0..3).all(|k| (p[k] - expected[k]).abs() < 1e-6),
                "angle {angle}: {p:?} vs {expected:?}"
            );
        }
    }

    #[test]
    fn dot_is_a_filled_unstroked_circle() {
        let dot = Dot::new().point([1.0, 2.0, 0.0]).build();
        assert!(close(dot.length_over_dim(0), 2.0 * DEFAULT_DOT_RADIUS));
        assert!(close_vec(dot.center_point(), [1.0, 2.0, 0.0]));
        assert!(close(dot.style().fill_opacity, 1.0));
        assert!(close(dot.style().stroke_width, 0.0));
        assert!(matches!(dot.shape(), ShapeTag::Dot { .. }));
        let small = Dot::small().build();
        assert!(close(
            small.length_over_dim(0),
            2.0 * DEFAULT_SMALL_DOT_RADIUS
        ));
    }

    #[test]
    fn ellipse_has_its_own_extents_and_no_circle_tag() {
        let ellipse = Ellipse::new()
            .width(4.0)
            .height(1.0)
            .start_angle(PI / 2.0)
            .build();
        assert!(close(ellipse.length_over_dim(0), 4.0));
        assert!(close(ellipse.length_over_dim(1), 1.0));
        assert!(close_vec(ellipse.points()[0], [0.0, 0.5, 0.0]));
        assert_eq!(
            ellipse.shape(),
            ShapeTag::General,
            "a stretched circle is not a circle"
        );
    }

    #[test]
    fn arc_between_points_meets_its_ends() {
        let (start, end) = ([0.0, 0.0, 0.0], [1.0, 2.0, 0.0]);
        for angle in [TAU / 4.0, -TAU / 12.0, TAU / 2.0] {
            let arc = ArcBetweenPoints::new(start, end)
                .angle(angle)
                .build()
                .expect("the fixture angle is valid");
            assert!(close_vec(arc.points()[0], start), "angle {angle}");
            assert!(
                close_vec(*arc.points().last().unwrap(), end),
                "angle {angle}"
            );
        }
        // A zero angle degenerates to the straight segment.
        let straight = ArcBetweenPoints::new(start, end)
            .angle(0.0)
            .build()
            .expect("a zero-angle segment uses one valid component");
        assert!(close_vec(straight.points()[0], start));
        assert!(close_vec(*straight.points().last().unwrap(), end));
        assert_eq!(straight.points().len(), 3);
    }

    #[test]
    fn annular_sector_spans_both_radii() {
        let sector = AnnularSector::new()
            .inner_radius(1.0)
            .outer_radius(2.0)
            .angle(TAU / 4.0)
            .build()
            .expect("the fixture sector is valid");
        let radii: Vec<f64> = sector
            .points()
            .iter()
            .step_by(2)
            .map(|p| space_ops::get_norm(*p))
            .collect();
        let min = radii.iter().copied().fold(f64::INFINITY, f64::min);
        let max = radii.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(close(min, 1.0), "inner radius {min}");
        assert!(close(max, 2.0), "outer radius {max}");
        // Sector: the hole closes to a point at the centre.
        let plain = AnnularSector::sector(TAU / 4.0, 2.0)
            .build()
            .expect("the fixture sector is valid");
        assert!(
            plain
                .points()
                .iter()
                .any(|p| space_ops::get_norm(*p) < 1e-9)
        );
    }

    #[test]
    fn annulus_winds_its_hole_the_other_way() {
        let ring = Annulus::new().inner_radius(1.0).outer_radius(2.0).build();
        let path = ring.path().unwrap();
        assert_eq!(path.subpath_end_indices().len(), 2, "two rings");
        // The subpaths wind opposite ways, which is what makes the hole a
        // hole under the nonzero rule.
        let subpaths = path.subpaths();
        let winding = |pts: &[Vec3]| {
            let anchors: Vec<Vec3> = pts.iter().step_by(2).copied().collect();
            space_ops::get_winding_number(&anchors)
        };
        let outer = winding(subpaths[0]);
        let inner = winding(subpaths[1]);
        assert!(outer * inner < 0.0, "windings {outer} and {inner}");
        assert!(close(ring.length_over_dim(0), 4.0));
    }

    #[test]
    fn a_shifted_arc_family_keeps_its_centre() {
        let ring = Annulus::new().center([2.0, 0.0, 0.0]).build();
        assert!(close_vec(ring.center_point(), [2.0, 0.0, 0.0]));
    }

    #[test]
    fn degenerate_arcs_are_defined() {
        let flat = Arc::new()
            .angle(0.0)
            .build()
            .expect("the default density admits a zero-angle arc");
        assert!(!flat.points().is_empty());
        let zero_radius = Circle::new().radius(0.0).build();
        assert!(close(zero_radius.length_over_dim(0), 0.0));
        assert!(close(radius_of(&zero_radius), 0.0));
    }

    #[test]
    fn public_arc_builders_propagate_component_refusals() {
        assert!(matches!(
            Arc::new().n_components(0).build(),
            Err(GeomError::ZeroArcComponents)
        ));
        assert!(matches!(
            Arc::new().n_components(usize::MAX).build(),
            Err(GeomError::ArcComponentOverflow { count: usize::MAX })
        ));
        assert!(matches!(
            Arc::new().angle(f64::NAN).build(),
            Err(GeomError::NonFiniteArcAngle)
        ));
        assert!(matches!(
            Arc::new().angle(f64::INFINITY).build(),
            Err(GeomError::NonFiniteArcAngle)
        ));
        assert!(
            Arc::new()
                .n_components(bezier::MAX_ARC_COMPONENTS)
                .build()
                .is_ok()
        );
        assert!(matches!(
            Arc::new()
                .n_components(bezier::MAX_ARC_COMPONENTS + 1)
                .build(),
            Err(GeomError::ArcComponentsAboveBudget { .. })
        ));
        assert!(matches!(
            ArcBetweenPoints::new([0.0; 3], [1.0, 0.0, 0.0])
                .angle(0.0)
                .n_components(0)
                .build(),
            Err(GeomError::ZeroArcComponents)
        ));
        assert!(matches!(
            AnnularSector::new().angle(f64::NAN).build(),
            Err(GeomError::NonFiniteArcAngle)
        ));
    }
}
