//! `VMobject` and its variants: the detached builder every library class
//! is made of (§12, Appendix A `mobject/types/vectorized_mobject`).
//!
//! Per G0-1's ratified §15.1 surface, a library class is a **value**:
//! chained by-value setters producing a plain struct that
//! `Stage::add(impl Into<Mobject>)` moves into the arena. [`VMobject`] is
//! that value for the vectorized family — points in the shared-anchor
//! layout, a [`Style`], the uniform inventory, the semantic
//! [`ShapeTag`] (§10.8), and detached children.
//!
//! Everything above it in this crate builds one of these and hands it to
//! `Stage::add`; nothing in the library reaches into the arena to finish a
//! half-constructed object.

use fmn_core::color::{Srgb, color_gradient};
use fmn_core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS, OUT};
use fmn_core::types::Vec3;
use fmn_geom::{GeomError, QuadPath, space_ops};
use fmn_mobject::stage::{Mob, Stage};
use fmn_mobject::uniforms::{JointType, Uniforms};
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, RenderPrimitive, ShapeTag};

use crate::style::Style;

/// numpy `sign`: `-1`, `0` or `1` (`NaN` stays `NaN`).
fn numpy_sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value * 0.0
    }
}

/// Maximum number of dash children one construction may publish.
///
/// This matches the library's other explicit geometry multiplicity caps:
/// large enough for authored scenes, small enough that a scalar parameter
/// cannot turn one mobject into an effectively unbounded family.
pub const MAX_DASHES: usize = 4_096;

/// A dashed-path configuration refused before arc-length work or iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashError {
    /// The source path refused its arc configuration before allocation.
    Geometry(GeomError),
    /// Dash length must be positive and finite.
    InvalidDashLength,
    /// The drawn fraction of a period must be finite and in `(0, 1]`.
    InvalidPositiveSpaceRatio,
    /// Pattern offsets must be finite.
    InvalidDashOffset,
    /// The source path did not have a finite true length.
    NonFiniteSourceLength,
    /// Deriving a count overflowed finite host-representable arithmetic.
    DashCountOverflow,
    /// The requested or derived child count exceeds [`MAX_DASHES`].
    TooManyDashes {
        /// Requested number of dash children.
        requested: usize,
        /// Declared maximum.
        max: usize,
    },
}

impl std::fmt::Display for DashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Geometry(error) => write!(f, "dash source geometry failed: {error}"),
            Self::InvalidDashLength => write!(f, "dash length must be positive and finite"),
            Self::InvalidPositiveSpaceRatio => {
                write!(f, "positive-space ratio must be finite and in (0, 1]")
            }
            Self::InvalidDashOffset => write!(f, "dash offset must be finite"),
            Self::NonFiniteSourceLength => write!(f, "source path length must be finite"),
            Self::DashCountOverflow => write!(f, "derived dash count overflowed"),
            Self::TooManyDashes { requested, max } => {
                write!(f, "requested {requested} dashes, above the {max} cap")
            }
        }
    }
}

impl std::error::Error for DashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GeomError> for DashError {
    fn from(error: GeomError) -> Self {
        Self::Geometry(error)
    }
}

pub(crate) fn validate_dash_length(length: f64) -> Result<(), DashError> {
    if length.is_finite() && length > 0.0 {
        Ok(())
    } else {
        Err(DashError::InvalidDashLength)
    }
}

pub(crate) fn validate_positive_space_ratio(ratio: f64) -> Result<(), DashError> {
    if ratio.is_finite() && ratio > 0.0 && ratio <= 1.0 {
        Ok(())
    } else {
        Err(DashError::InvalidPositiveSpaceRatio)
    }
}

pub(crate) fn validate_dash_count(count: usize) -> Result<(), DashError> {
    if count <= MAX_DASHES {
        Ok(())
    } else {
        Err(DashError::TooManyDashes {
            requested: count,
            max: MAX_DASHES,
        })
    }
}

/// A detached vectorized mobject.
#[derive(Debug, Clone, PartialEq)]
pub struct VMobject {
    points: Vec<Vec3>,
    style: Style,
    uniforms: Uniforms,
    shape: ShapeTag,
    z_index: i32,
    /// Per-point stroke widths, when the class wants a taper rather than a
    /// uniform stroke. Resized onto the point run at conversion time, the
    /// way the Reference's `set_stroke(width=[...])` does.
    stroke_profile: Option<Vec<f64>>,
    submobjects: Vec<VMobject>,
}

impl Default for VMobject {
    fn default() -> Self {
        Self::new()
    }
}

impl VMobject {
    /// An empty vectorized mobject: no points, default style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            style: Style::default(),
            uniforms: Uniforms::default(),
            shape: ShapeTag::General,
            z_index: 0,
            stroke_profile: None,
            submobjects: Vec::new(),
        }
    }

    /// Set the scene-list sort key (§8.5): higher draws later, ties keep
    /// insertion order. Only a top-level member's value orders anything —
    /// see `fmn_mobject::order`.
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// A vectorized mobject over an explicit shared-anchor point run.
    #[must_use]
    pub fn from_points(points: Vec<Vec3>) -> Self {
        Self::new().with_points(points)
    }

    /// A vectorized mobject over a built [`QuadPath`] — the usual route,
    /// since every geometry class draws its path with the Chisel API.
    #[must_use]
    pub fn from_path(path: &QuadPath) -> Self {
        Self::new().with_points(path.points().to_vec())
    }

    /// Replace the point run.
    #[must_use]
    pub fn with_points(mut self, points: Vec<Vec3>) -> Self {
        self.points = points;
        self
    }

    /// Replace the whole style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Amend the style in place (`vmob.map_style(|s| s.color(RED))`).
    #[must_use]
    pub fn map_style(mut self, f: impl FnOnce(Style) -> Style) -> Self {
        self.style = f(self.style);
        self
    }

    /// Amend the style of the whole family: self and every descendant,
    /// the Reference's `set_fill`/`set_stroke`/`set_color` family
    /// propagation. The style-recursive sibling of [`VMobject::map_points`].
    #[must_use]
    pub fn map_style_deep(mut self, f: impl Fn(Style) -> Style + Copy) -> Self {
        self.style = f(self.style);
        self.submobjects = self
            .submobjects
            .into_iter()
            .map(|child| child.map_style_deep(f))
            .collect();
        self
    }

    /// The Reference's `color=`: stroke and fill together.
    #[must_use]
    pub fn with_color(self, color: Srgb) -> Self {
        self.map_style(|s| s.color(color))
    }

    /// Tag the semantic shape this builder produced (§10.8).
    #[must_use]
    pub fn with_shape(mut self, shape: ShapeTag) -> Self {
        self.shape = shape;
        self
    }

    /// Replace the uniform inventory.
    #[must_use]
    pub fn with_uniforms(mut self, uniforms: Uniforms) -> Self {
        self.uniforms = uniforms;
        self
    }

    /// Transform the uniform inventory in place (group constructors
    /// overriding one or two fields, e.g. `VGroup3D`).
    #[must_use]
    pub fn map_uniforms(mut self, f: impl FnOnce(Uniforms) -> Uniforms) -> Self {
        self.uniforms = f(self.uniforms);
        self
    }

    /// Read the uniform inventory.
    #[must_use]
    pub fn uniforms(&self) -> Uniforms {
        self.uniforms
    }

    /// Set the joint type uniform (`joint_type=`).
    #[must_use]
    pub fn with_joint_type(mut self, joint_type: JointType) -> Self {
        self.uniforms.joint_type = joint_type;
        self
    }

    /// Set the `stroke_behind` uniform.
    #[must_use]
    pub fn with_stroke_behind(mut self, behind: bool) -> Self {
        self.uniforms.stroke_behind = behind;
        self
    }

    /// Set the `flat_stroke` uniform.
    #[must_use]
    pub fn with_flat_stroke(mut self, flat: bool) -> Self {
        self.uniforms.flat_stroke = flat;
        self
    }

    /// Taper the stroke along the path (Reference `set_stroke(width=[...])`).
    ///
    /// The list is resized onto the point run by linear interpolation at
    /// conversion time, so it describes the *shape* of the taper rather than
    /// one width per point: `[0.0, 6.0, 0.0]` is "nothing at the ends, six in
    /// the middle" whatever the path's resolution. `Cross` and `Underline`
    /// are defined by exactly this, and it overrides the style's uniform
    /// width.
    #[must_use]
    pub fn with_stroke_profile(mut self, widths: impl Into<Vec<f64>>) -> Self {
        let widths = widths.into();
        self.stroke_profile = (!widths.is_empty()).then_some(widths);
        self
    }

    /// The stroke taper, if this mobject has one.
    #[must_use]
    pub fn stroke_profile(&self) -> Option<&[f64]> {
        self.stroke_profile.as_deref()
    }

    /// Append a detached child (`VMobject.add`).
    #[must_use]
    pub fn with_child(mut self, child: VMobject) -> Self {
        self.submobjects.push(child);
        self
    }

    /// Append several detached children.
    #[must_use]
    pub fn with_children(mut self, children: impl IntoIterator<Item = VMobject>) -> Self {
        self.submobjects.extend(children);
        self
    }

    /// Replace the detached children by mapping each one (the detached
    /// hook for the Reference's family operations — `set_color` and
    /// friends recurse into submobjects; compose with [`VMobject::map_style`] /
    /// [`VMobject::map_points`] to do the same here).
    #[must_use]
    pub fn map_children(mut self, mut f: impl FnMut(VMobject) -> VMobject) -> Self {
        self.submobjects = self.submobjects.into_iter().map(&mut f).collect();
        self
    }

    /// The point run.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The style.
    #[must_use]
    pub fn style(&self) -> Style {
        self.style
    }

    /// The semantic shape tag.
    #[must_use]
    pub fn shape(&self) -> ShapeTag {
        self.shape
    }

    /// The detached children.
    #[must_use]
    pub fn children(&self) -> &[VMobject] {
        &self.submobjects
    }

    /// Read the point run back as a [`QuadPath`] — the builder-side
    /// equivalent of reading the arena's records, for classes that
    /// measure their own geometry while still detached (tips, dashes).
    ///
    /// # Errors
    /// [`fmn_geom::GeomError::EvenPointCount`] if the run is not in the
    /// shared-anchor layout.
    pub fn path(&self) -> Result<QuadPath, fmn_geom::GeomError> {
        QuadPath::from_points(self.points.clone())
    }

    /// Apply a point map (the detached form of `apply_points_function`
    /// with no pivot) to this mobject and its detached children.
    ///
    /// Any semantic shape tag is dropped: the caller is by definition no
    /// longer building the shape the tag names. A class that *knows* the
    /// map preserves its shape re-tags afterwards.
    #[must_use]
    pub fn map_points(self, f: impl Fn(Vec3) -> Vec3 + Copy) -> Self {
        self.map_points_and_shapes(f, |_| ShapeTag::General)
    }

    /// Apply one point map while deriving every detached family member's
    /// semantic tag through the same transform.
    ///
    /// This is intentionally private: only the affine operations below can
    /// prove how all of a tag's geometric payload moves. Arbitrary public
    /// point maps continue to route through [`Self::map_points`] and demote.
    fn map_points_and_shapes(
        mut self,
        point_map: impl Fn(Vec3) -> Vec3 + Copy,
        shape_map: impl Fn(ShapeTag) -> ShapeTag + Copy,
    ) -> Self {
        for p in &mut self.points {
            *p = point_map(*p);
        }
        self.submobjects = self
            .submobjects
            .into_iter()
            .map(|child| child.map_points_and_shapes(point_map, shape_map))
            .collect();
        self.shape = shape_map(self.shape);
        self
    }

    /// `reverse_points`: reverse the shared-anchor point run (anchors and
    /// handles swap roles, winding flips), recursively over children.
    #[must_use]
    pub fn reversed_points(mut self) -> Self {
        self.points.reverse();
        self.submobjects = self
            .submobjects
            .into_iter()
            .map(VMobject::reversed_points)
            .collect();
        self
    }

    /// Shift every point (including children's).
    #[must_use]
    pub fn shifted(self, offset: Vec3) -> Self {
        self.map_points_and_shapes(
            |p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]],
            |shape| shifted_tag(shape, offset),
        )
    }

    /// The `(min, max)` corners of the family's points, or `None` when
    /// there are none — the detached form of the bounding box the
    /// positional API works against.
    #[must_use]
    pub fn extent(&self) -> Option<(Vec3, Vec3)> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut any = false;
        self.visit_points(&mut |p| {
            any = true;
            for k in 0..3 {
                min[k] = min[k].min(p[k]);
                max[k] = max[k].max(p[k]);
            }
        });
        any.then_some((min, max))
    }

    fn visit_points(&self, f: &mut impl FnMut(Vec3)) {
        for p in &self.points {
            f(*p);
        }
        for child in &self.submobjects {
            child.visit_points(f);
        }
    }

    /// The family's extent along one axis (Reference `length_over_dim`).
    #[must_use]
    pub fn length_over_dim(&self, dim: usize) -> f64 {
        self.extent().map_or(0.0, |(min, max)| max[dim] - min[dim])
    }

    /// The centre of the family's extent (Reference `get_center`).
    #[must_use]
    pub fn center_point(&self) -> Vec3 {
        self.extent().map_or([0.0; 3], |(min, max)| {
            [
                0.5 * (min[0] + max[0]),
                0.5 * (min[1] + max[1]),
                0.5 * (min[2] + max[2]),
            ]
        })
    }

    /// Scale about a pivot (Reference `scale(factor, about_point=…)`).
    #[must_use]
    pub fn scaled_about(self, factor: f64, about: Vec3) -> Self {
        self.map_points_and_shapes(
            move |p| scale_point_about(p, factor, about),
            move |shape| scaled_tag(shape, factor, about),
        )
    }

    /// Stretch one axis about a pivot (Reference `stretch`).
    #[must_use]
    pub fn stretched_about(self, factor: f64, dim: usize, about: Vec3) -> Self {
        self.map_points(move |mut p| {
            p[dim] = about[dim] + (p[dim] - about[dim]) * factor;
            p
        })
    }

    /// Reference `rescale_to_fit`: resize along `dim` to `length`, about
    /// the family's own centre. A zero extent along that axis is left
    /// alone (the Reference's early return), never divided by.
    #[must_use]
    pub fn rescaled_to_fit(self, length: f64, dim: usize, stretch: bool) -> Self {
        let old = self.length_over_dim(dim);
        if old == 0.0 {
            return self;
        }
        let about = self.center_point();
        let factor = length / old;
        if stretch {
            self.stretched_about(factor, dim, about)
        } else {
            self.scaled_about(factor, about)
        }
    }

    /// Reference `set_width(width, stretch=…)`.
    #[must_use]
    pub fn with_width(self, width: f64, stretch: bool) -> Self {
        self.rescaled_to_fit(width, 0, stretch)
    }

    /// Reference `set_height(height, stretch=…)`.
    #[must_use]
    pub fn with_height(self, height: f64, stretch: bool) -> Self {
        self.rescaled_to_fit(height, 1, stretch)
    }

    /// Rotate about a pivot and axis (Reference `rotate`).
    #[must_use]
    pub fn rotated_about(self, angle: f64, axis: Vec3, about: Vec3) -> Self {
        let m = fmn_geom::rotation_matrix(angle, axis);
        self.map_points_and_shapes(
            move |p| rotate_point_about(p, m, about),
            move |shape| rotated_tag(shape, angle, axis, about, m),
        )
    }

    /// Move the family's centre to `point` (Reference `move_to`).
    #[must_use]
    pub fn moved_to(self, point: Vec3) -> Self {
        let c = self.center_point();
        self.shifted([point[0] - c[0], point[1] - c[1], point[2] - c[2]])
    }

    /// The critical point of the family's box in a direction — the detached
    /// twin of `Stage::get_bounding_box_point`, and the primitive the rest of
    /// the detached positional API is built on.
    ///
    /// Component-wise: a positive direction picks the box's maximum, a
    /// negative one its minimum, and a zero one the midpoint. That is the
    /// Reference's rule exactly, and it is why `UL` names a corner while `UP`
    /// names an edge centre.
    ///
    /// `None` when the family has no points at all — a caller composing an
    /// empty group has nothing to align to, and inventing the origin would
    /// silently place things at the wrong spot.
    #[must_use]
    pub fn bbox_point(&self, direction: Vec3) -> Option<Vec3> {
        let (min, max) = self.extent()?;
        let pick = |d: f64, lo: f64, hi: f64| {
            if d > 0.0 {
                hi
            } else if d < 0.0 {
                lo
            } else {
                0.5 * (lo + hi)
            }
        };
        Some([
            pick(direction[0], min[0], max[0]),
            pick(direction[1], min[1], max[1]),
            pick(direction[2], min[2], max[2]),
        ])
    }

    /// Move so the family's critical point at `aligned_edge` lands on
    /// `point` (Reference `move_to(point, aligned_edge=…)`).
    ///
    /// [`moved_to`](Self::moved_to) is this with `aligned_edge = ORIGIN`.
    /// The aligned form is what grid layout needs: a matrix cell is placed by
    /// a shared corner so that differently-sized entries line up, not by
    /// each one's own centre.
    #[must_use]
    pub fn moved_to_aligned(self, point: Vec3, aligned_edge: Vec3) -> Self {
        match self.bbox_point(aligned_edge) {
            Some(p) => self.shifted([point[0] - p[0], point[1] - p[1], point[2] - p[2]]),
            None => self,
        }
    }

    /// Align a side or corner against the default frame's border, inset by
    /// `buff` (Reference `align_on_border`, the engine of `to_edge` and
    /// `to_corner`): only the axes `direction` points along move.
    #[must_use]
    pub fn aligned_on_border(self, direction: Vec3, buff: f64) -> Self {
        let Some(point) = self.bbox_point(direction) else {
            return self;
        };
        let radius = [FRAME_X_RADIUS, FRAME_Y_RADIUS, 0.0];
        let shift: Vec3 = std::array::from_fn(|k| {
            let sign = numpy_sign(direction[k]);
            (sign * radius[k] - point[k] - buff * direction[k]) * sign.abs()
        });
        self.shifted(shift)
    }

    /// Reference `to_edge(edge, buff)`.
    #[must_use]
    pub fn to_edge(self, edge: Vec3, buff: f64) -> Self {
        self.aligned_on_border(edge, buff)
    }

    /// Reference `shift_onto_screen(buff=…)`: for each of `UP`, `DOWN`,
    /// `LEFT`, `RIGHT` in that order, an edge centre beyond the default
    /// frame's half-extent less `buff` is pulled back with `to_edge`.
    #[must_use]
    pub fn shifted_onto_screen(mut self, buff: f64) -> Self {
        for (vect, radius) in [
            ([0.0, 1.0, 0.0], FRAME_Y_RADIUS),
            ([0.0, -1.0, 0.0], FRAME_Y_RADIUS),
            ([-1.0, 0.0, 0.0], FRAME_X_RADIUS),
            ([1.0, 0.0, 0.0], FRAME_X_RADIUS),
        ] {
            let Some(edge) = self.bbox_point(vect) else {
                return self;
            };
            let reach = edge[0] * vect[0] + edge[1] * vect[1] + edge[2] * vect[2];
            if reach > radius - buff {
                self = self.to_edge(vect, buff);
            }
        }
        self
    }

    /// Place next to a point, on the `direction` side, `buff` away, aligned
    /// along `aligned_edge` (Reference `next_to` with `coor_mask = 1`).
    #[must_use]
    pub fn next_to_point(
        self,
        target: Vec3,
        direction: Vec3,
        buff: f64,
        aligned_edge: Vec3,
    ) -> Self {
        let anchor = [
            aligned_edge[0] - direction[0],
            aligned_edge[1] - direction[1],
            aligned_edge[2] - direction[2],
        ];
        match self.bbox_point(anchor) {
            Some(p) => self.shifted([
                target[0] - p[0] + direction[0] * buff,
                target[1] - p[1] + direction[1] * buff,
                target[2] - p[2] + direction[2] * buff,
            ]),
            None => self,
        }
    }

    /// Place next to another detached mobject (Reference `next_to`).
    ///
    /// The target's anchor is its critical point at `aligned_edge +
    /// direction`, and ours is at `aligned_edge - direction` — the same pair
    /// the arena's `Stage::next_to` uses, so a class laid out detached and
    /// one laid out on the Stage agree.
    #[must_use]
    pub fn next_to(self, target: &Self, direction: Vec3, buff: f64, aligned_edge: Vec3) -> Self {
        let corner = [
            aligned_edge[0] + direction[0],
            aligned_edge[1] + direction[1],
            aligned_edge[2] + direction[2],
        ];
        match target.bbox_point(corner) {
            Some(point) => self.next_to_point(point, direction, buff, aligned_edge),
            None => self,
        }
    }

    /// Align to another mobject along the nonzero components of `direction`
    /// (Reference `align_to`), leaving the other axes alone.
    #[must_use]
    pub fn aligned_to(self, target: &Self, direction: Vec3) -> Self {
        let (Some(there), Some(here)) = (target.bbox_point(direction), self.bbox_point(direction))
        else {
            return self;
        };
        let mut offset = [0.0; 3];
        for dim in 0..3 {
            if direction[dim] != 0.0 {
                offset[dim] = there[dim] - here[dim];
            }
        }
        self.shifted(offset)
    }

    /// Lay a sequence out end to end along `direction`, `buff` apart,
    /// aligned along `aligned_edge` (Reference `arrange`), returning them as
    /// the children of one group.
    ///
    /// The Reference re-centres the whole arrangement afterwards; that is
    /// left to the caller here, because a caller who is about to `next_to`
    /// the group would only be undoing it.
    #[must_use]
    pub fn arranged(
        items: impl IntoIterator<Item = Self>,
        direction: Vec3,
        buff: f64,
        aligned_edge: Vec3,
    ) -> Self {
        let mut placed: Vec<Self> = Vec::new();
        for item in items {
            match placed.last() {
                None => placed.push(item),
                Some(previous) => {
                    let next = item.next_to(previous, direction, buff, aligned_edge);
                    placed.push(next);
                }
            }
        }
        Self::new().with_children(placed)
    }

    /// Reference `put_start_and_end_on` for a detached builder: scale,
    /// turn, and shift so the first point lands on `start` and the last on
    /// `end`.
    ///
    /// Returns `self` unchanged when the current endpoints coincide —
    /// there is no such transform, and the Reference raises here. Classes
    /// that can rebuild instead (`Line`) do that rather than call this.
    #[must_use]
    pub fn put_start_and_end_on(self, start: Vec3, end: Vec3) -> Self {
        let (Some(&curr_start), Some(&curr_end)) = (self.points.first(), self.points.last()) else {
            return self;
        };
        let curr_vect = sub(curr_end, curr_start);
        if curr_vect == [0.0; 3] {
            return self;
        }
        let target_vect = sub(end, start);
        let scale = space_ops::get_norm(target_vect) / space_ops::get_norm(curr_vect);
        let scaled = self.scaled_about(scale, curr_start);
        let center = scaled.center_point();
        let turned = scaled.rotated_about(
            space_ops::angle_of_vector(target_vect) - space_ops::angle_of_vector(curr_vect),
            OUT,
            center,
        );
        let curr_xy = space_ops::get_norm([curr_vect[0], curr_vect[1], 0.0]);
        let target_xy = space_ops::get_norm([target_vect[0], target_vect[1], 0.0]);
        let center = turned.center_point();
        let tilted = turned.rotated_about(
            space_ops::angle_of_vector([curr_xy, curr_vect[2], 0.0])
                - space_ops::angle_of_vector([target_xy, target_vect[2], 0.0]),
            [-target_vect[1], target_vect[0], 0.0],
            center,
        );
        let now_start = tilted.points.first().copied().unwrap_or(start);
        tilted.shifted(sub(start, now_start))
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Translate a tag's geometric payload.
fn shifted_tag(tag: ShapeTag, offset: Vec3) -> ShapeTag {
    if !finite_vec3(offset) {
        return ShapeTag::General;
    }
    let moved = |p: Vec3| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]];
    match tag {
        ShapeTag::General | ShapeTag::Polyline { .. } => tag,
        ShapeTag::Line {
            start,
            end,
            path_arc,
            buff,
        } => ShapeTag::Line {
            start: moved(start),
            end: moved(end),
            path_arc,
            buff,
        },
        ShapeTag::Arc {
            center,
            radius,
            start_angle,
            angle,
        } => ShapeTag::Arc {
            center: moved(center),
            radius,
            start_angle,
            angle,
        },
        ShapeTag::Circle { center, radius } => ShapeTag::Circle {
            center: moved(center),
            radius,
        },
        ShapeTag::Dot { center, radius } => ShapeTag::Dot {
            center: moved(center),
            radius,
        },
        ShapeTag::Rect {
            center,
            width,
            height,
        } => ShapeTag::Rect {
            center: moved(center),
            width,
            height,
        },
        ShapeTag::RoundedRect {
            center,
            width,
            height,
            corner_radius,
        } => ShapeTag::RoundedRect {
            center: moved(center),
            width,
            height,
            corner_radius,
        },
    }
}

fn scaled_tag(tag: ShapeTag, factor: f64, about: Vec3) -> ShapeTag {
    if !factor.is_finite() || !finite_vec3(about) {
        return ShapeTag::General;
    }
    let scaled = |point| scale_point_about(point, factor, about);
    let magnitude = factor.abs();
    match tag {
        ShapeTag::General => ShapeTag::General,
        ShapeTag::Line {
            start,
            end,
            path_arc,
            buff,
        } => ShapeTag::Line {
            start: scaled(start),
            end: scaled(end),
            path_arc,
            buff: buff * magnitude,
        },
        ShapeTag::Polyline { .. } => tag,
        ShapeTag::Arc {
            center,
            radius,
            start_angle,
            angle,
        } => ShapeTag::Arc {
            center: scaled(center),
            radius: radius * magnitude,
            start_angle: start_angle
                + if factor < 0.0 {
                    std::f64::consts::PI
                } else {
                    0.0
                },
            angle,
        },
        ShapeTag::Circle { center, radius } => ShapeTag::Circle {
            center: scaled(center),
            radius: radius * magnitude,
        },
        ShapeTag::Dot { center, radius } => ShapeTag::Dot {
            center: scaled(center),
            radius: radius * magnitude,
        },
        ShapeTag::Rect {
            center,
            width,
            height,
        } => ShapeTag::Rect {
            center: scaled(center),
            width: width * magnitude,
            height: height * magnitude,
        },
        ShapeTag::RoundedRect {
            center,
            width,
            height,
            corner_radius,
        } => ShapeTag::RoundedRect {
            center: scaled(center),
            width: width * magnitude,
            height: height * magnitude,
            corner_radius: corner_radius * magnitude,
        },
    }
}

fn rotated_tag(
    tag: ShapeTag,
    angle: f64,
    axis: Vec3,
    about: Vec3,
    matrix: [[f64; 3]; 3],
) -> ShapeTag {
    if !angle.is_finite() || !finite_vec3(axis) || !finite_vec3(about) {
        return ShapeTag::General;
    }
    if axis == [0.0; 3] || angle == 0.0 {
        return tag;
    }
    if axis[0] != 0.0 || axis[1] != 0.0 || axis[2] == 0.0 {
        return ShapeTag::General;
    }

    let rotated = |point| rotate_point_about(point, matrix, about);
    let signed_angle = if axis[2].is_sign_negative() {
        -angle
    } else {
        angle
    };
    match tag {
        ShapeTag::General => ShapeTag::General,
        ShapeTag::Line {
            start,
            end,
            path_arc,
            buff,
        } => ShapeTag::Line {
            start: rotated(start),
            end: rotated(end),
            path_arc,
            buff,
        },
        ShapeTag::Polyline { .. } => tag,
        ShapeTag::Arc {
            center,
            radius,
            start_angle,
            angle,
        } => ShapeTag::Arc {
            center: rotated(center),
            radius,
            start_angle: start_angle + signed_angle,
            angle,
        },
        ShapeTag::Circle { center, radius } => ShapeTag::Circle {
            center: rotated(center),
            radius,
        },
        ShapeTag::Dot { center, radius } => ShapeTag::Dot {
            center: rotated(center),
            radius,
        },
        // These tags encode axis alignment and carry no orientation field.
        ShapeTag::Rect { .. } | ShapeTag::RoundedRect { .. } => ShapeTag::General,
    }
}

fn scale_point_about(point: Vec3, factor: f64, about: Vec3) -> Vec3 {
    [
        about[0] + (point[0] - about[0]) * factor,
        about[1] + (point[1] - about[1]) * factor,
        about[2] + (point[2] - about[2]) * factor,
    ]
}

fn rotate_point_about(point: Vec3, matrix: [[f64; 3]; 3], about: Vec3) -> Vec3 {
    let v = [
        point[0] - about[0],
        point[1] - about[1],
        point[2] - about[2],
    ];
    [
        about[0] + matrix[0][0] * v[0] + matrix[0][1] * v[1] + matrix[0][2] * v[2],
        about[1] + matrix[1][0] * v[0] + matrix[1][1] * v[1] + matrix[1][2] * v[2],
        about[2] + matrix[2][0] * v[0] + matrix[2][1] * v[1] + matrix[2][2] * v[2],
    ]
}

fn finite_vec3(value: Vec3) -> bool {
    value.into_iter().all(f64::is_finite)
}

impl From<VMobject> for Mobject {
    fn from(v: VMobject) -> Self {
        let VMobject {
            points,
            style,
            uniforms,
            shape,
            z_index,
            stroke_profile,
            submobjects,
        } = v;
        let pointless = points.is_empty();
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len())
            .expect("record sizing bounded by the point list");
        #[allow(clippy::cast_possible_truncation)]
        let flat: Vec<f32> = points
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("point", 0, &flat);
        // Joint angles are a function of the path, so they are written
        // with it rather than left for a later refresh to remember.
        if let Ok(path) = QuadPath::from_points(points) {
            #[allow(clippy::cast_possible_truncation)]
            let angles: Vec<f32> = path.joint_angles().iter().map(|a| *a as f32).collect();
            buffer.write_range("joint_angle", 0, &angles);
        }
        style.write(&mut buffer);
        // A point-free node keeps its constructor style as the Reference's
        // `_data_defaults` row: the portal's color getters answer from it
        // and a later resize seeds records with it. With points present the
        // per-record lanes above already carry the style.
        let defaults_row = if pointless {
            Some(vec![
                (
                    "fill_rgba".to_string(),
                    vec![
                        style.fill_color.r as f32,
                        style.fill_color.g as f32,
                        style.fill_color.b as f32,
                        style.fill_opacity as f32,
                    ],
                ),
                (
                    "stroke_rgba".to_string(),
                    vec![
                        style.stroke_color.r as f32,
                        style.stroke_color.g as f32,
                        style.stroke_color.b as f32,
                        style.stroke_opacity as f32,
                    ],
                ),
                ("stroke_width".to_string(), vec![style.stroke_width as f32]),
                (
                    "fill_border_width".to_string(),
                    vec![style.fill_border_width as f32],
                ),
            ])
        } else {
            None
        };
        // A taper overrides the uniform width the style just wrote — the
        // Reference's `set_stroke(width=[0, 6, 0])`, which resizes the list
        // onto the point run by linear interpolation.
        if let Some(profile) = stroke_profile {
            #[allow(clippy::cast_possible_truncation)]
            let widths: Vec<f32> = resize_with_interpolation(&profile, buffer.len())
                .into_iter()
                .map(|w| w as f32)
                .collect();
            buffer.write_range("stroke_width", 0, &widths);
        }

        Mobject {
            buffer,
            uniforms,
            shape,
            render_primitive: RenderPrimitive::Vector,
            image: None,
            z_index,
            defaults_row,
            submobjects: submobjects.into_iter().map(Mobject::from).collect(),
        }
    }
}

/// `Group`: a heterogeneous mobject holding any children (Appendix A
/// `types/mobject`).
///
/// Unlike [`v_group`], this accepts the broader [`Mobject`] family and is
/// the native analogue of `manimlib.mobject.mobject.Group`.
#[must_use]
pub fn group(children: impl IntoIterator<Item = Mobject>) -> Mobject {
    Mobject::group(children.into_iter().collect())
}

/// `VGroup`: a vectorized mobject with no geometry of its own, holding
/// children (Appendix A `types/vectorized_mobject`).
///
/// The Reference's `VGroup` refuses non-VMobject members; here that is the
/// type system's job, since [`VMobject`] is the only thing it accepts.
#[must_use]
pub fn v_group(children: impl IntoIterator<Item = VMobject>) -> VMobject {
    VMobject::new().with_children(children)
}

/// `VectorizedPoint`: a single location that behaves like a mobject.
///
/// The pinned Reference stores exactly one point. One is also a valid
/// shared-anchor run (an anchor with zero curves), so the native value keeps
/// the same observable record topology rather than inventing a null curve.
#[must_use]
pub fn vectorized_point(location: Vec3) -> VMobject {
    VMobject::from_points(vec![location])
        .map_style(|s| s.stroke(s.stroke_color, 0.0, 0.0).fill(s.fill_color, 0.0))
}

/// `CurvesAsSubmobjects`: one child per Bézier curve of the source.
///
/// Used by the passing-flash animations, which reveal a path curve by
/// curve. Style is inherited by every piece.
#[must_use]
pub fn curves_as_submobjects(source: &VMobject) -> VMobject {
    let mut group = VMobject::new().with_style(source.style());
    if let Ok(path) = source.path() {
        for tuple in path.bezier_tuples() {
            group =
                group.with_child(VMobject::from_points(tuple.to_vec()).with_style(source.style()));
        }
    }
    group
}

/// `DashedVMobject`: a source path cut into `num_dashes` dashes.
///
/// **The dashes are placed by true arc length** (BN-03's dash corollary):
/// the Reference walks `pointwise_become_partial` in *curve-index* space,
/// so on a path whose curves differ in length — every arc, every smoothed
/// curve — its dashes bunch up where the curves are short. Ours cut at
/// equal true-length proportions, so a dashed circle has evenly spaced
/// dashes, which is what the name always promised.
///
/// `positive_space_ratio` is the fraction of each period that is dash;
/// `dash_offset` shifts the pattern along the path, both as in the
/// Reference.
///
/// # Errors
///
/// Returns [`DashError`] when the requested child count exceeds
/// [`MAX_DASHES`], the drawn fraction is outside `(0, 1]`, or the offset is
/// non-finite. Validation happens before arc-length work or child iteration.
pub fn dashed_vmobject(
    source: &VMobject,
    num_dashes: usize,
    positive_space_ratio: f64,
    dash_offset: f64,
) -> Result<VMobject, DashError> {
    let mut group = VMobject::new().with_style(source.style());
    let Ok(path) = source.path() else {
        return Ok(group);
    };
    for (a, b) in dash_curve_intervals(source, num_dashes, positive_space_ratio, dash_offset)? {
        if let Some((points, _, _)) = QuadPath::partial_points(path.points(), a, b) {
            group = group.with_child(VMobject::from_points(points).with_style(source.style()));
        }
    }
    Ok(group)
}

/// Curve-index intervals for a true-arclength dash pattern.
///
/// The pinned Reference normalizes dash starts by
/// `1 - full_period + dash_length`; that placement rule is kept. The only
/// deliberate change is BN-03: those starts and ends are interpreted in true
/// length space, then converted to the curve-index proportions consumed by
/// [`QuadPath::partial_points`] and Marionette's partial-reveal operation.
/// Returning the intervals separately lets bindings preserve the source's
/// complete record data while the native geometry authority chooses the cut.
///
/// # Errors
///
/// Returns [`DashError`] for an excessive count, an invalid drawn ratio, or
/// a non-finite offset. Non-positive counts are represented by the caller as
/// an empty pattern; the Rust surface accepts zero directly.
pub fn dash_curve_intervals(
    source: &VMobject,
    num_dashes: usize,
    positive_space_ratio: f64,
    dash_offset: f64,
) -> Result<Vec<(f64, f64)>, DashError> {
    validate_dash_count(num_dashes)?;
    if !dash_offset.is_finite() {
        return Err(DashError::InvalidDashOffset);
    }
    if num_dashes == 0 {
        return Ok(Vec::new());
    }
    validate_positive_space_ratio(positive_space_ratio)?;

    let Ok(path) = source.path() else {
        return Ok(Vec::new());
    };
    let n_curves = path.num_curves();
    if n_curves == 0 {
        return Ok(Vec::new());
    }
    let table = fmn_geom::ArcLengthTable::for_path(&path);
    let index_alpha = |alpha: f64| -> f64 {
        let clamped = alpha.clamp(0.0, 1.0);
        match table.curve_and_t_at(&path, clamped) {
            Some((curve, t)) => (curve as f64 + t) / n_curves as f64,
            None => clamped,
        }
    };

    let n = num_dashes as f64;
    let full_period = 1.0 / n;
    let dash_length = full_period * positive_space_ratio;
    let start_denominator = 1.0 - full_period + dash_length;
    let mut intervals = Vec::with_capacity(num_dashes);
    for i in 0..num_dashes {
        let start = (i as f64 * full_period) / start_denominator + dash_offset * full_period;
        let start = start.rem_euclid(1.0);
        let end = (start + dash_length).min(1.0);
        let (a, b) = (index_alpha(start), index_alpha(end));
        if b > a {
            intervals.push((a, b));
        }
    }
    Ok(intervals)
}

/// `VHighlight`: full-family copies of a source with gradient-colored,
/// progressively wider strokes — the Reference's cheap outline glow.
///
/// The returned order matches the pinned Reference: the first child has the
/// largest addition and the final gradient color, while the last child has
/// the smallest addition and first color. Fill opacity is cleared throughout
/// every copied family; existing stroke opacity is preserved.
#[must_use]
pub fn v_highlight(
    source: &VMobject,
    n_layers: usize,
    max_stroke_addition: f64,
    color_bounds: [Srgb; 2],
) -> VMobject {
    let mut group = VMobject::new();
    if n_layers == 0 {
        return group;
    }
    let colors = color_gradient(&color_bounds, n_layers);
    for child_index in 0..n_layers {
        let reverse_index = n_layers - 1 - child_index;
        let addition = max_stroke_addition * (reverse_index + 1) as f64 / n_layers as f64;
        let color = colors[reverse_index];
        let child = source.clone().map_style_deep(|style| {
            style
                .stroke(color, style.stroke_width + addition, style.stroke_opacity)
                .fill(style.fill_color, 0.0)
        });
        group = group.with_child(child);
    }
    group
}

/// Add a built [`VMobject`] to a stage and return its handle — sugar for
/// `stage.add(vmob)` that reads the way the README's examples do.
pub fn add_to(stage: &mut Stage, vmob: VMobject) -> Mob {
    stage.add(vmob)
}

/// Resample `values` onto `length` evenly spaced positions by linear
/// interpolation — the Reference's `resize_with_interpolation`, which is how
/// a short stroke-width list becomes one width per point.
///
/// A single value fills; an empty list yields nothing to write.
#[must_use]
fn resize_with_interpolation(values: &[f64], length: usize) -> Vec<f64> {
    if values.is_empty() || length == 0 {
        return Vec::new();
    }
    if values.len() == 1 {
        return vec![values[0]; length];
    }
    (0..length)
        .map(|i| {
            if length == 1 {
                return values[0];
            }
            #[allow(clippy::cast_precision_loss)]
            let alpha = i as f64 / (length - 1) as f64;
            #[allow(clippy::cast_precision_loss)]
            let scaled = alpha * (values.len() - 1) as f64;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let lo = (scaled.floor() as usize).min(values.len() - 1);
            let hi = (lo + 1).min(values.len() - 1);
            #[allow(clippy::cast_precision_loss)]
            let t = scaled - lo as f64;
            values[lo] + (values[hi] - values[lo]) * t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{Arc, Circle};
    use crate::line::Line;
    use crate::poly::Rectangle;
    use crate::style::VStyle;
    use fmn_core::constants::{RED, TAU};

    fn assert_points_close(actual: &[Vec3], expected: &[Vec3], context: &str) {
        assert_eq!(actual.len(), expected.len(), "{context}: point count");
        for (point_index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            for axis in 0..3 {
                assert!(
                    (actual[axis] - expected[axis]).abs() <= 2.0e-6,
                    "{context}: point {point_index} axis {axis}: {} != {}",
                    actual[axis],
                    expected[axis]
                );
            }
        }
    }

    #[test]
    fn a_built_vmobject_carries_points_style_uniforms_and_shape() {
        let mut stage = Stage::new();
        let mob = stage.add(
            VMobject::from_points(vec![[0.0; 3], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]])
                .with_color(RED)
                .with_joint_type(JointType::Bevel)
                .with_shape(ShapeTag::Polyline {
                    vertices: 2,
                    closed: false,
                }),
        );
        assert_eq!(stage.get_points(mob).unwrap().len(), 3);
        // Colours round-trip through f32 records (§6.1), so this is an
        // f32-tolerance comparison, not a bit-for-bit one.
        let stroke = stage.get_stroke_color(mob).unwrap();
        assert!(
            (stroke.r - RED.r).abs() < 1e-6
                && (stroke.g - RED.g).abs() < 1e-6
                && (stroke.b - RED.b).abs() < 1e-6,
            "{stroke:?}"
        );
        assert_eq!(
            stage.get(mob).unwrap().uniforms().joint_type,
            JointType::Bevel
        );
        assert!(matches!(
            stage.primitive_hint(mob),
            Some(ShapeTag::Polyline { vertices: 2, .. })
        ));
    }

    #[test]
    fn joint_angles_are_written_with_the_path() {
        let mut stage = Stage::new();
        let mob = stage.add(VMobject::from_points(vec![
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
        ]));
        let angles = stage
            .get(mob)
            .unwrap()
            .buffer
            .read_column("joint_angle")
            .unwrap();
        assert_eq!(angles.len(), 5);
        // The corner at index 2 turns; the ends do not.
        assert!(angles[2].abs() > 1e-3, "corner joint angle {}", angles[2]);
        assert!(angles[0].abs() < 1e-6);
    }

    #[test]
    fn children_enter_the_arena_as_family() {
        let mut stage = Stage::new();
        let group = stage.add(v_group([
            VMobject::from_points(vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]),
            VMobject::from_points(vec![[0.0; 3], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]),
        ]));
        assert_eq!(stage.get(group).unwrap().submobjects().len(), 2);
        assert_eq!(stage.family(group).len(), 3);
    }

    #[test]
    fn vectorized_point_is_a_degenerate_path_at_its_location() {
        let mut stage = Stage::new();
        let mob = stage.add(vectorized_point([1.0, 2.0, 3.0]));
        assert_eq!(
            stage.get(mob).expect("live point").buffer.len(),
            1,
            "the pinned Reference exposes one location record"
        );
        assert_eq!(stage.get_start(mob), Some([1.0, 2.0, 3.0]));
        assert_eq!(stage.get_end(mob), Some([1.0, 2.0, 3.0]));
        let bbox = stage.get_bounding_box(mob);
        assert_eq!(bbox.width(), 0.0);
        assert_eq!(bbox.height(), 0.0);
        assert!(!stage.has_stroke(mob), "a point draws nothing by itself");
    }

    #[test]
    fn curves_as_submobjects_splits_by_curve() {
        let circle = Circle::new().build();
        let pieces = curves_as_submobjects(&circle);
        assert_eq!(
            pieces.children().len(),
            circle.path().unwrap().num_curves(),
            "one child per Bezier"
        );
        for piece in pieces.children() {
            assert_eq!(piece.points().len(), 3);
        }
    }

    #[test]
    fn dashes_are_evenly_spaced_by_true_length() {
        // On a circle every curve is the same length, so index-space and
        // length-space agree and the dashes are evenly spaced either way.
        let circle = Circle::new().radius(1.0).build();
        let dashed = dashed_vmobject(&circle, 8, 0.5, 0.0).expect("valid dash pattern");
        assert_eq!(dashed.children().len(), 8);
        let lengths: Vec<f64> = dashed
            .children()
            .iter()
            .map(|d| d.path().unwrap().get_arc_length())
            .collect();
        let first = lengths[0];
        for l in &lengths {
            assert!((l - first).abs() < 1e-6, "dash lengths differ: {lengths:?}");
        }
        // Half of the circumference is drawn, at ratio 0.5.
        let total: f64 = lengths.iter().sum();
        assert!(
            (total - TAU / 2.0).abs() < 1e-3,
            "drawn length {total} vs {}",
            TAU / 2.0
        );
    }

    #[test]
    fn dashes_on_an_uneven_path_still_measure_equal() {
        // A path whose curves differ wildly in length: index-space dashes
        // would come out uneven, true-length dashes do not (BN-03).
        let points = vec![
            [0.0, 0.0, 0.0],
            [0.05, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [6.0, 0.0, 0.0],
        ];
        let source = VMobject::from_points(points);
        let dashed = dashed_vmobject(&source, 4, 0.5, 0.0).expect("valid dash pattern");
        assert_eq!(dashed.children().len(), 4);
        let lengths: Vec<f64> = dashed
            .children()
            .iter()
            .map(|d| d.path().unwrap().get_arc_length())
            .collect();
        let first = lengths[0];
        for l in &lengths {
            assert!((l - first).abs() < 1e-6, "dash lengths differ: {lengths:?}");
        }
    }

    #[test]
    fn dash_intervals_keep_reference_start_normalization_in_length_space() {
        let source = VMobject::from_points(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let intervals = dash_curve_intervals(&source, 4, 0.5, 0.0).expect("valid dash intervals");
        assert_eq!(intervals.len(), 4);
        let starts: Vec<f64> = intervals.iter().map(|(start, _)| *start).collect();
        let ends: Vec<f64> = intervals.iter().map(|(_, end)| *end).collect();
        let expected_starts = [0.0, 2.0 / 7.0, 4.0 / 7.0, 6.0 / 7.0];
        for (actual, expected) in starts.iter().zip(expected_starts) {
            assert!((actual - expected).abs() < 1e-12, "{starts:?}");
        }
        for (start, end) in starts.iter().zip(ends) {
            assert!((end - start - 0.125).abs() < 1e-12, "{intervals:?}");
        }
    }

    #[test]
    fn zero_dashes_and_empty_sources_are_defined() {
        let circle = Circle::new().build();
        assert!(
            dashed_vmobject(&circle, 0, 0.5, 0.0)
                .expect("zero is a defined empty pattern")
                .children()
                .is_empty()
        );
        let empty = VMobject::new();
        assert!(
            dashed_vmobject(&empty, 4, 0.5, 0.0)
                .expect("an empty source is a defined empty pattern")
                .children()
                .is_empty()
        );
        assert!(curves_as_submobjects(&empty).children().is_empty());
        assert!(
            v_highlight(&empty, 0, 5.0, [RED, RED])
                .children()
                .is_empty()
        );
    }

    #[test]
    fn dash_contract_checks_parameters_and_the_exact_count_boundary() {
        let empty = VMobject::new();
        assert!(
            dashed_vmobject(&empty, MAX_DASHES, 0.5, 0.0)
                .expect("the declared boundary is admitted")
                .children()
                .is_empty()
        );
        assert_eq!(
            dashed_vmobject(&empty, MAX_DASHES + 1, 0.5, 0.0),
            Err(DashError::TooManyDashes {
                requested: MAX_DASHES + 1,
                max: MAX_DASHES,
            })
        );
        assert_eq!(
            dashed_vmobject(&empty, usize::MAX, 0.5, 0.0),
            Err(DashError::TooManyDashes {
                requested: usize::MAX,
                max: MAX_DASHES,
            })
        );
        for ratio in [0.0, -1.0, 1.1, f64::NAN, f64::INFINITY] {
            assert_eq!(
                dashed_vmobject(&empty, 1, ratio, 0.0),
                Err(DashError::InvalidPositiveSpaceRatio)
            );
        }
        for offset in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                dashed_vmobject(&empty, 1, 0.5, offset),
                Err(DashError::InvalidDashOffset)
            );
        }
    }

    #[test]
    fn highlight_layers_clone_the_family_and_add_gradient_strokes() {
        let child = Circle::new()
            .radius(0.25)
            .build()
            .map_style(|style| style.stroke_width(2.0));
        let circle = Circle::new()
            .build()
            .map_style(|style| style.stroke(RED, 3.0, 0.4).fill(RED, 0.75))
            .with_child(child);
        let glow = v_highlight(&circle, 5, 5.0, [RED, fmn_core::constants::BLUE]);
        assert_eq!(glow.children().len(), 5);
        let widths: Vec<f64> = glow
            .children()
            .iter()
            .map(|c| c.style().stroke_width)
            .collect();
        assert_eq!(widths, vec![8.0, 7.0, 6.0, 5.0, 4.0]);
        let opacities: Vec<f64> = glow
            .children()
            .iter()
            .map(|c| c.style().stroke_opacity)
            .collect();
        assert_eq!(opacities, vec![0.4; 5]);
        assert!(
            glow.children().iter().all(|copy| {
                copy.style().fill_opacity == 0.0
                    && copy.children().len() == 1
                    && copy.children()[0].style().fill_opacity == 0.0
            }),
            "every full-family copy clears fill opacity"
        );
        assert_eq!(
            glow.children()[0].style().stroke_color,
            fmn_core::constants::BLUE
        );
        assert_eq!(glow.children()[4].style().stroke_color, RED);
        assert_eq!(
            glow.children()[0].children()[0].style().stroke_width,
            7.0,
            "the same width addition reaches descendants"
        );
    }

    #[test]
    fn detached_positional_operations_preserve_or_demote_hints_by_contract() {
        let circle = Circle::new()
            .radius(2.0)
            .arc_center([0.5, -0.25, 0.0])
            .build();
        let target = Circle::new()
            .radius(0.5)
            .arc_center([4.0, 1.0, 0.0])
            .build();

        let preserved = [
            ("shift", circle.clone().shifted([1.0, 2.0, 0.0])),
            (
                "uniform scale",
                circle.clone().scaled_about(1.5, [1.0, 0.0, 0.0]),
            ),
            (
                "z rotation",
                circle.clone().rotated_about(0.7, [0.0, 0.0, 2.0], [0.0; 3]),
            ),
            ("move_to", circle.clone().moved_to([3.0, -2.0, 0.0])),
            (
                "aligned move_to",
                circle
                    .clone()
                    .moved_to_aligned([3.0, -2.0, 0.0], [1.0, 1.0, 0.0]),
            ),
            (
                "next_to point",
                circle
                    .clone()
                    .next_to_point([3.0, 2.0, 0.0], [1.0, 0.0, 0.0], 0.2, [0.0; 3]),
            ),
            (
                "next_to mobject",
                circle
                    .clone()
                    .next_to(&target, [1.0, 0.0, 0.0], 0.2, [0.0; 3]),
            ),
            (
                "align_to",
                circle.clone().aligned_to(&target, [1.0, 0.0, 0.0]),
            ),
            (
                "rescale_to_fit",
                circle.clone().rescaled_to_fit(5.0, 0, false),
            ),
            ("set_width", circle.clone().with_width(5.0, false)),
            ("set_height", circle.clone().with_height(5.0, false)),
        ];
        for (operation, result) in preserved {
            assert!(
                matches!(result.shape(), ShapeTag::Circle { .. }),
                "{operation} unexpectedly demoted {:?}",
                result.shape()
            );
        }

        let demoted = [
            (
                "arbitrary point map",
                circle.clone().map_points(|p| [p[0], p[1] * 0.5, p[2]]),
            ),
            ("stretch", circle.clone().stretched_about(0.5, 1, [0.0; 3])),
            (
                "stretch rescale_to_fit",
                circle.clone().rescaled_to_fit(5.0, 0, true),
            ),
            ("stretch set_width", circle.clone().with_width(5.0, true)),
            ("stretch set_height", circle.clone().with_height(5.0, true)),
            (
                "out-of-plane rotation",
                circle.clone().rotated_about(0.7, [1.0, 0.0, 0.0], [0.0; 3]),
            ),
            (
                "non-finite shift",
                circle.clone().shifted([f64::NAN, 0.0, 0.0]),
            ),
        ];
        for (operation, result) in demoted {
            assert_eq!(
                result.shape(),
                ShapeTag::General,
                "{operation} retained a false primitive hint"
            );
        }

        let rectangle = Rectangle::new().build().expect("rectangle");
        assert_eq!(
            rectangle
                .rotated_about(0.5, [0.0, 0.0, 1.0], [0.0; 3])
                .shape(),
            ShapeTag::General,
            "an axis-aligned rectangle tag cannot encode orientation"
        );
    }

    #[test]
    fn shape_preserving_maps_update_every_detached_family_member() {
        let family = VMobject::new().with_child(
            Circle::new()
                .radius(1.0)
                .arc_center([1.0, 0.0, 0.0])
                .build(),
        );
        let transformed = family
            .scaled_about(2.0, [0.0; 3])
            .rotated_about(TAU / 4.0, [0.0, 0.0, 1.0], [0.0; 3])
            .shifted([3.0, 1.0, 0.0]);

        assert!(matches!(transformed.shape(), ShapeTag::General));
        assert!(
            matches!(
                transformed.children()[0].shape(),
                ShapeTag::Circle { center, radius }
                    if (center[0] - 3.0).abs() < 1.0e-12
                        && (center[1] - 3.0).abs() < 1.0e-12
                        && center[2].abs() < 1.0e-12
                        && (radius - 2.0).abs() < 1.0e-12
            ),
            "child hint did not follow its points: {:?}",
            transformed.children()[0].shape()
        );
    }

    #[test]
    fn preserved_payload_tags_rebuild_the_transformed_points() {
        let transformed_arc = Arc::new()
            .start_angle(0.3)
            .angle(1.2)
            .radius(1.7)
            .arc_center([0.2, -0.4, 0.0])
            .build()
            .expect("arc")
            .shifted([0.8, -0.3, 0.0])
            .scaled_about(-1.5, [0.5, 0.2, 0.0])
            .rotated_about(0.6, [0.0, 0.0, -2.0], [-0.2, 0.1, 0.0]);

        let arc_tag = transformed_arc.shape();
        assert!(
            matches!(arc_tag, ShapeTag::Arc { .. }),
            "shape-preserving transforms demoted the arc: {arc_tag:?}"
        );
        let ShapeTag::Arc {
            center,
            radius,
            start_angle,
            angle,
        } = arc_tag
        else {
            return;
        };
        let rebuilt_arc = Arc::new()
            .start_angle(start_angle)
            .angle(angle)
            .radius(radius)
            .arc_center(center)
            .build()
            .expect("rebuilt arc");
        assert_points_close(
            transformed_arc.points(),
            rebuilt_arc.points(),
            "rebuilt arc",
        );

        let transformed_line = Line::new([-2.0, -0.5, 0.0], [1.5, 1.0, 0.0])
            .path_arc(0.8)
            .buff(0.15)
            .build()
            .expect("line")
            .scaled_about(1.75, [0.2, -0.1, 0.0])
            .rotated_about(0.4, [0.0, 0.0, 1.0], [0.3, 0.7, 0.0])
            .shifted([-0.6, 0.9, 0.0]);
        let line_tag = transformed_line.shape();
        assert!(
            matches!(line_tag, ShapeTag::Line { .. }),
            "shape-preserving transforms demoted the line: {line_tag:?}"
        );
        let ShapeTag::Line {
            start,
            end,
            path_arc,
            buff,
        } = line_tag
        else {
            return;
        };
        let rebuilt_line = Line::new(start, end)
            .path_arc(path_arc)
            .buff(buff)
            .build()
            .expect("rebuilt line");
        assert_points_close(
            transformed_line.points(),
            rebuilt_line.points(),
            "rebuilt line",
        );

        let transformed_rect = Rectangle::new()
            .width(3.0)
            .height(1.5)
            .build()
            .expect("rectangle")
            .scaled_about(1.25, [0.4, -0.2, 0.0])
            .shifted([0.7, 1.1, 0.0]);
        let rect_tag = transformed_rect.shape();
        assert!(
            matches!(rect_tag, ShapeTag::Rect { .. }),
            "shape-preserving transforms demoted the rectangle: {rect_tag:?}"
        );
        let ShapeTag::Rect {
            center,
            width,
            height,
        } = rect_tag
        else {
            return;
        };
        let rebuilt_rect = Rectangle::new()
            .width(width)
            .height(height)
            .build()
            .expect("rebuilt rectangle")
            .shifted(center);
        assert_points_close(
            transformed_rect.points(),
            rebuilt_rect.points(),
            "rebuilt rectangle",
        );
    }
}
