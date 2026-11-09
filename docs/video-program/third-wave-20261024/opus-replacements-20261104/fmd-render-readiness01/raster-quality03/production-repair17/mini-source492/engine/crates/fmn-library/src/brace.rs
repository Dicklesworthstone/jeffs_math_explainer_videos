//! The native `Brace` family — a parametric path generator (§11.6, §12.3).
//!
//! The Reference renders `\underbrace{\qquad}` through LaTeX and then
//! *stretches the glyph* to the width it needs. Widening is handled by a hack
//! over six hard-coded submobject indices of that render (stretch the two
//! straight runs, shift the tips); narrowing falls through to
//! `set_width(width, stretch=True)`, which squashes the curl horizontally
//! along with everything else. Neither branch is a shape family — they are
//! repairs applied to one fixed drawing.
//!
//! Here a brace is generated, not stretched. The curl — the two end hooks and
//! the centre point — has its own size, and the straight runs absorb all the
//! width. Every curl dimension is additionally clamped to a fraction of the
//! total width, which is what makes narrow braces degrade gracefully instead
//! of folding through themselves:
//!
//! ```text
//!   cap   ≤ 0.20 · width      (horizontal reach of each end hook)
//!   waist ≤ 0.20 · width      (horizontal reach of the centre point)
//!   ⇒ cap + waist ≤ 0.40 · width < 0.50 · width = the midpoint
//! ```
//!
//! so the left hook can never reach past the centre point, at any width. That
//! is a proof, not a tuning: [`Brace`] is well-formed for every positive
//! width, and `braces_never_self_intersect_at_any_width` holds it to it.
//!
//! Behaviour Note **BN-08**: the shape is ours, so its metrics differ from the
//! LaTeX-derived original. Its scaling behaviour is better defined — the curl
//! keeps its proportions where the Reference's distorted.
//!
//! The construction is adapted from `fmd-math`'s `\overbrace`/`\underbrace`
//! band (`drawn::hbrace`), which solves the same problem for math accents.
//! That module is `pub(crate)` upstream, so the geometry is reproduced here
//! rather than called; keeping the two in the same shape is deliberate, so a
//! brace over a formula and a `Brace` under a mobject look like siblings.

use fmn_core::constants::{DOWN, ORIGIN, PI};
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_geom::space_ops::get_norm;

use crate::style::Style;
use crate::vmobject::VMobject;

/// The Reference's measured depth of a wide brace: `Brace(Square(), DOWN)`
/// is `2.0 × 0.2668` at the default font size (fm-5wq.53, pinned Reference
/// `6199a00d` probed side by side with the portal). The `\underbrace` glyph
/// keeps that depth at every wider width — its `set_initial_width` stretches
/// only the straight runs.
pub const REFERENCE_BRACE_DEPTH: f64 = 0.2668;

/// The size parameter of the curl, in manim units.
///
/// At `em = 1` a wide brace stands `0.05 + 2 × 0.16 = 0.37` units deep; the
/// default scales the whole curl (hooks, centre point and stroke together,
/// so the shape keeps its proportions) to [`REFERENCE_BRACE_DEPTH`] — the
/// kept look (Appendix B), reached by construction rather than by rendering
/// a glyph. An earlier calibration claimed 0.37 matched the Reference; the
/// probe measured 39 % too deep.
pub const DEFAULT_BRACE_EM: f64 = REFERENCE_BRACE_DEPTH / 0.37;

/// The Reference's `Brace(..., buff=0.2)`.
pub const DEFAULT_BRACE_BUFF: f64 = 0.2;

/// `Brace(mobject, direction, buff)` — a brace spanning a width, curling
/// toward whatever it annotates.
///
/// Two ways in: [`Brace::new`] for a bare parametric brace of a given width,
/// and [`Brace::around`] for the Reference's constructor, which measures a
/// mobject and places itself against it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brace {
    width: f64,
    direction: Vec3,
    buff: f64,
    em: f64,
    /// Where the brace's upper-left corner goes, before rotation. `None` for
    /// a bare brace, which builds at the origin.
    anchor: Option<Vec3>,
    /// `LineBrace` first constructs against a horizontal copy of its target,
    /// then restores the result around the original line centre.  Keeping
    /// that final affine step explicit prevents translated lines from being
    /// spuriously rotated around the world origin.
    post_rotation: f64,
    post_rotation_about: Vec3,
    style: Style,
}

impl Default for Brace {
    fn default() -> Self {
        Self::new()
    }
}

impl Brace {
    /// A brace of unit width, curling downward, at the origin.
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 1.0,
            direction: DOWN,
            buff: DEFAULT_BRACE_BUFF,
            em: DEFAULT_BRACE_EM,
            anchor: None,
            post_rotation: 0.0,
            post_rotation_about: ORIGIN,
            // A brace is a filled glyph in the Reference, so it is a filled
            // shape here: fill on, stroke off.
            style: Style::default()
                .fill(fmn_core::constants::WHITE, 1.0)
                .stroke(fmn_core::constants::WHITE, 0.0, 1.0),
        }
    }

    /// `Brace(mobject, direction, buff)`: span `target`'s extent along the
    /// axis perpendicular to `direction`, and sit `buff` away on that side.
    ///
    /// Follows the Reference exactly: rotate the target into the frame where
    /// the brace is horizontal, measure the width between its lower corners,
    /// then rotate the finished brace back. The rotation angle is the
    /// Reference's own `-atan2(dx, dy) + PI` — note the argument order, which
    /// is `arctan2(y, x)` fed `(x, y)`, and is what makes `DOWN` the
    /// unrotated case.
    #[must_use]
    pub fn around(target: &VMobject, direction: Vec3) -> Self {
        let angle = brace_angle(direction);
        let rotated = target
            .clone()
            .rotated_about(-angle, fmn_core::constants::OUT, ORIGIN);
        let (width, anchor) = match (
            rotated.bbox_point([-1.0, -1.0, 0.0]),
            rotated.bbox_point([1.0, -1.0, 0.0]),
        ) {
            (Some(left), Some(right)) => (right[0] - left[0], Some(left)),
            _ => (0.0, None),
        };
        Self {
            width: width.max(0.0),
            direction,
            anchor,
            ..Self::new()
        }
    }

    /// Set the span.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width.max(0.0);
        self
    }

    /// Set the side the brace curls toward.
    #[must_use]
    pub fn direction(mut self, direction: Vec3) -> Self {
        self.direction = direction;
        self
    }

    /// Set the gap between the brace and whatever it annotates.
    #[must_use]
    pub fn buff(mut self, buff: f64) -> Self {
        self.buff = buff;
        self
    }

    /// Set the curl's size parameter. Scales the hooks, the centre point, and
    /// the stroke together; the width is unaffected.
    #[must_use]
    pub fn em(mut self, em: f64) -> Self {
        self.em = em.max(0.0);
        self
    }

    /// Replace the style wholesale.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set fill and stroke colour together (Reference `color=`).
    #[must_use]
    pub fn color(mut self, color: fmn_core::color::Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// How deep the curl stands for this brace's width — the extent along
    /// the direction it curls in.
    #[must_use]
    pub fn depth(&self) -> f64 {
        let m = Metrics::for_width(self.width, self.em);
        m.height
    }

    /// The point of the centre curl, in final placed coordinates — the
    /// Reference's `get_tip()`, which it finds by scanning for the lowest
    /// point of the rendered glyph. Here it is known analytically.
    #[must_use]
    pub fn tip(&self) -> Vec3 {
        let m = Metrics::for_width(self.width, self.em);
        self.place([self.width / 2.0, -m.height, 0.0])
    }

    /// The unit vector from the brace's body toward its tip — the Reference's
    /// `get_direction()`, and the direction `put_at_tip` places into.
    #[must_use]
    pub fn tip_direction(&self) -> Vec3 {
        let n = get_norm(self.direction);
        let direction = if n == 0.0 {
            DOWN
        } else {
            [
                self.direction[0] / n,
                self.direction[1] / n,
                self.direction[2] / n,
            ]
        };
        rotate_z(direction, self.post_rotation)
    }

    /// Place `label` just past the tip, `buff` away (Reference
    /// `put_at_tip`, whose `use_next_to=True` branch this is).
    #[must_use]
    pub fn put_at_tip(&self, label: VMobject, buff: f64) -> VMobject {
        label.next_to_point(self.tip(), self.tip_direction(), buff, ORIGIN)
    }

    /// Local-frame point to final placed coordinates: rotate into the
    /// brace's direction, then offset by the anchor and the buffer.
    fn place(&self, local: Vec3) -> Vec3 {
        let angle = brace_angle(self.direction);
        let shifted = match self.anchor {
            Some(anchor) => [
                local[0] + anchor[0],
                local[1] + anchor[1] - self.buff,
                local[2] + anchor[2],
            ],
            None => local,
        };
        rotate_z_about(
            rotate_z(shifted, angle),
            self.post_rotation,
            self.post_rotation_about,
        )
    }

    /// Build the brace as a filled path.
    #[must_use]
    pub fn build(self) -> VMobject {
        let m = Metrics::for_width(self.width, self.em);
        let path = m.contour(self.width);
        let points: Vec<Vec3> = path.points().iter().map(|p| self.place(*p)).collect();
        VMobject::from_points(points).with_style(self.style)
    }
}

impl From<Brace> for fmn_mobject::Mobject {
    fn from(brace: Brace) -> Self {
        brace.build().into()
    }
}

/// The Reference's placement angle for a brace direction.
///
/// `-atan2(direction[0], direction[1]) + PI`. The argument order is the
/// Reference's: `np.arctan2(y, x)` receives `(x, y)`, which looks like a bug
/// and is load-bearing — it is what makes `DOWN` the zero-rotation case and
/// therefore what every existing scene's brace placement depends on.
fn brace_angle(direction: Vec3) -> f64 {
    -fmn_dmath::atan2(direction[0], direction[1]) + PI
}

fn rotate_z(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = (fmn_dmath::sin(angle), fmn_dmath::cos(angle));
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

fn rotate_z_about(p: Vec3, angle: f64, about: Vec3) -> Vec3 {
    let shifted = [p[0] - about[0], p[1] - about[1], p[2] - about[2]];
    let rotated = rotate_z(shifted, angle);
    [
        rotated[0] + about[0],
        rotated[1] + about[1],
        rotated[2] + about[2],
    ]
}

/// The curl's dimensions at one width — the parametric family itself.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    /// Stroke thickness.
    thickness: f64,
    /// Vertical bend of the hooks and the centre point.
    reach: f64,
    /// Horizontal reach of each end hook.
    cap: f64,
    /// Horizontal reach of the centre point.
    waist: f64,
    /// Total depth of the curl.
    height: f64,
}

impl Metrics {
    /// The clamps are the correctness argument, not a taste knob. Three
    /// invariants make the family well-formed at *every* positive width, and
    /// `braces_never_self_intersect_at_any_width` checks all three:
    ///
    /// 1. `cap + waist ≤ 0.40 w < 0.50 w` — an end hook can never reach the
    ///    centre point, so the straight runs never invert.
    /// 2. `thickness ≤ 0.10 w < cap` — the inner edge's hook (at `x = t`)
    ///    stays left of where the inner run begins (at `x = cap`), and the
    ///    right inner hook (at `x = w − t`) stays inside the span.
    /// 3. `height > 1.2 · thickness` — the inner edge of the centre point
    ///    stays below the runs rather than punching through them.
    ///
    /// Clamping the thickness is what invariant 2 costs, and it is not
    /// cosmetic: without it a brace narrower than the stroke is inside out.
    /// The Reference cannot express this at all — it has one drawing and a
    /// stretch.
    fn for_width(width: f64, em: f64) -> Self {
        let thickness = (0.050 * em).min(width * 0.10);
        let reach = (0.16 * em).min(width * 0.12);
        Self {
            thickness,
            reach,
            cap: (0.30 * em).min(width * 0.20),
            waist: (0.32 * em).min(width * 0.20),
            height: thickness + 2.0 * reach,
        }
    }

    /// One closed contour, tips at `y = 0` and the centre point at
    /// `y = -height`: the `\underbrace` orientation, which is the unrotated
    /// `DOWN` case.
    fn contour(&self, width: f64) -> QuadPath {
        let (t, cap, waist, h) = (self.thickness, self.cap, self.waist, self.height);
        let ys = self.reach;
        let outer = -(ys + t);
        let inner = -ys;
        let mid = width / 2.0;
        let mut path = QuadPath::new();
        path.start_new_path([0.0, 0.0, 0.0]);
        let quad = |handle: [f64; 2], to: [f64; 2], path: &mut QuadPath| {
            let _ = path.add_quadratic_bezier_curve_to(
                [handle[0], handle[1], 0.0],
                [to[0], to[1], 0.0],
                true,
            );
        };
        // Left hook, outer edge: up from the tip into the run.
        quad([0.0, outer], [cap, outer], &mut path);
        let _ = path.add_line_to([mid - waist, outer, 0.0], true);
        // The centre point, outer edge: down and back up.
        quad([mid, outer], [mid, -h], &mut path);
        quad([mid, outer], [mid + waist, outer], &mut path);
        let _ = path.add_line_to([width - cap, outer, 0.0], true);
        // Right hook, outer edge: back up to the tip.
        quad([width, outer], [width, 0.0], &mut path);
        let _ = path.add_line_to([width - t, 0.0, 0.0], true);
        // Right hook, inner edge.
        quad([width - t, inner], [width - cap, inner], &mut path);
        let _ = path.add_line_to([mid + waist, inner, 0.0], true);
        // The centre point, inner edge — shallower, so the point has weight.
        quad([mid + t, inner], [mid, -(h - t * 1.2)], &mut path);
        quad([mid - t, inner], [mid - waist, inner], &mut path);
        let _ = path.add_line_to([cap, inner, 0.0], true);
        // Left hook, inner edge, and close across the tip.
        quad([t, inner], [t, 0.0], &mut path);
        let _ = path.add_line_to([0.0, 0.0, 0.0], true);
        path
    }
}

/// `BraceLabel(obj, text, brace_direction, label_buff)` — a brace with
/// something placed at its tip, as one group.
///
/// The Reference's `BraceLabel` and `BraceText` differ only in which
/// constructor builds the label (`Tex` vs `TexText`); here the label is any
/// already-built mobject, so one type serves both and the caller picks the
/// constructor.
#[derive(Debug, Clone, PartialEq)]
pub struct BraceLabel {
    brace: Brace,
    label: VMobject,
    label_buff: f64,
    label_scale: f64,
}

impl BraceLabel {
    /// Brace `target` on `direction`'s side and put `label` at the tip.
    #[must_use]
    pub fn new(target: &VMobject, label: VMobject, direction: Vec3) -> Self {
        Self {
            brace: Brace::around(target, direction),
            label,
            label_buff: fmn_core::constants::DEFAULT_MOBJECT_TO_MOBJECT_BUFF,
            label_scale: 1.0,
        }
    }

    /// Scale the label before placing it (Reference `label_scale`).
    #[must_use]
    pub fn label_scale(mut self, scale: f64) -> Self {
        self.label_scale = scale;
        self
    }

    /// The gap between tip and label (Reference `label_buff`).
    #[must_use]
    pub fn label_buff(mut self, buff: f64) -> Self {
        self.label_buff = buff;
        self
    }

    /// Set the brace's gap from its target.
    #[must_use]
    pub fn buff(mut self, buff: f64) -> Self {
        self.brace = self.brace.buff(buff);
        self
    }

    /// Build brace and label as a two-child group, brace first.
    #[must_use]
    pub fn build(self) -> VMobject {
        let label = if (self.label_scale - 1.0).abs() > f64::EPSILON {
            let about = self.label.center_point();
            self.label.scaled_about(self.label_scale, about)
        } else {
            self.label
        };
        let placed = self.brace.put_at_tip(label, self.label_buff);
        VMobject::new().with_children([self.brace.build(), placed])
    }
}

impl From<BraceLabel> for fmn_mobject::Mobject {
    fn from(labelled: BraceLabel) -> Self {
        labelled.build().into()
    }
}

/// `LineBrace(line, direction)` — a brace along an arbitrary segment.
///
/// The Reference rotates the line flat, braces it, and rotates back about the
/// line's centre. Doing the same here keeps the orientation semantics the bead
/// asks to preserve: `direction=UP` braces the side the line's normal points
/// to, whatever the line's own angle.
#[must_use]
pub fn line_brace(start: Vec3, end: Vec3, direction: Vec3) -> Brace {
    let angle = fmn_dmath::atan2(end[1] - start[1], end[0] - start[0]);
    let centre = [
        0.5 * (start[0] + end[0]),
        0.5 * (start[1] + end[1]),
        0.5 * (start[2] + end[2]),
    ];
    let length = get_norm([end[0] - start[0], end[1] - start[1], end[2] - start[2]]);
    let flat_start = [centre[0] - length / 2.0, centre[1], centre[2]];
    let flat_end = [centre[0] + length / 2.0, centre[1], centre[2]];
    let flat_target = VMobject::from_points(vec![flat_start, centre, flat_end]);
    let mut brace = Brace::around(&flat_target, direction);
    brace.post_rotation = angle;
    brace.post_rotation_about = centre;
    brace
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::UP;

    /// Every anchor of a built brace, in local (unplaced) coordinates.
    fn local_points(width: f64) -> Vec<Vec3> {
        let m = Metrics::for_width(width, DEFAULT_BRACE_EM);
        m.contour(width).points().to_vec()
    }

    #[test]
    fn a_brace_is_a_closed_shared_anchor_path() {
        let brace = Brace::new().width(3.0).build();
        let points = brace.points();
        assert!(!points.is_empty(), "a brace has geometry");
        assert_eq!(points.len() % 2, 1, "shared-anchor runs have odd length");
        let path = brace.path().expect("the run is a valid path");
        assert!(path.is_closed(), "a brace is a filled closed shape");
    }

    #[test]
    fn braces_never_self_intersect_at_any_width() {
        // The clamp argument in the module docs, exercised across four orders
        // of magnitude — including widths far narrower than the curl's own
        // natural size, which is exactly where the Reference's stretched
        // glyph distorts.
        for width in [
            0.001, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 8.0, 40.0, 200.0,
        ] {
            let m = Metrics::for_width(width, DEFAULT_BRACE_EM);
            // Invariant 1: the hooks cannot reach the centre point.
            assert!(
                m.cap + m.waist < width / 2.0,
                "width {width}: hook (cap {}) reaches the centre point (waist {})",
                m.cap,
                m.waist
            );
            // Invariant 2: the inner edge stays inside the span and in order.
            assert!(
                m.thickness < m.cap,
                "width {width}: stroke {} is thicker than the hook's reach {}, \
                 so the inner edge inverts",
                m.thickness,
                m.cap
            );
            assert!(
                m.thickness < width - m.thickness,
                "width {width}: the two inner hooks cross"
            );
            // Invariant 3: the centre point's inner edge stays below the runs.
            assert!(
                m.height > 1.2 * m.thickness,
                "width {width}: the centre point punches through its own runs"
            );
            assert!(m.height > 0.0, "width {width}: the curl vanished");
            let points = local_points(width);
            for p in &points {
                assert!(
                    p[0] >= -1e-12 && p[0] <= width + 1e-12,
                    "width {width}: point {p:?} escapes the span"
                );
                assert!(
                    p[1] <= 1e-12 && p[1] >= -m.height - 1e-12,
                    "width {width}: point {p:?} escapes the curl's depth"
                );
            }
        }
    }

    /// fm-5wq.53: at the default size a brace under a 2-unit square is the
    /// Reference's 2.0 × 0.2668, its top edge `buff = 0.2` below the target,
    /// so it is centred at y = −1.3334 as the Reference's is.
    #[test]
    fn default_brace_matches_the_reference_extents() {
        let square = crate::poly::Rectangle::new()
            .width(2.0)
            .height(2.0)
            .build()
            .expect("square");
        let brace = Brace::around(&square, DOWN).build();
        let (min, max) = brace.extent().expect("brace has extent");
        assert!(((max[0] - min[0]) - 2.0).abs() < 1e-9, "width");
        assert!(
            ((max[1] - min[1]) - REFERENCE_BRACE_DEPTH).abs() < 1e-9,
            "depth"
        );
        assert!((max[1] - (-1.0 - DEFAULT_BRACE_BUFF)).abs() < 1e-9, "buff");
        assert!((brace.center_point()[1] + 1.3334).abs() < 1e-9);
        // Wider braces keep the depth; the runs absorb the width.
        for width in [3.0, 8.0, 14.0] {
            let depth = Brace::new().width(width).depth();
            assert!(
                (depth - REFERENCE_BRACE_DEPTH).abs() < 1e-9,
                "{width}: {depth}"
            );
        }
    }

    #[test]
    fn the_curl_keeps_its_size_as_the_brace_widens() {
        // The property the Reference lacks: past the clamp, widening moves
        // the straight runs and leaves the hooks and point alone. A stretched
        // glyph would grow all three together.
        let wide = Metrics::for_width(20.0, DEFAULT_BRACE_EM);
        let wider = Metrics::for_width(200.0, DEFAULT_BRACE_EM);
        assert!((wide.cap - wider.cap).abs() < 1e-12);
        assert!((wide.waist - wider.waist).abs() < 1e-12);
        assert!((wide.height - wider.height).abs() < 1e-12);
    }

    #[test]
    fn a_brace_is_symmetric_about_its_midline() {
        let width = 4.0;
        let points = local_points(width);
        let mid = width / 2.0;
        // Mirroring x about the midline must map the point set onto itself.
        for p in &points {
            let mirrored = [2.0 * mid - p[0], p[1], p[2]];
            assert!(
                points.iter().any(|q| {
                    (q[0] - mirrored[0]).abs() < 1e-9
                        && (q[1] - mirrored[1]).abs() < 1e-9
                        && (q[2] - mirrored[2]).abs() < 1e-9
                }),
                "no mirror partner for {p:?}"
            );
        }
    }

    #[test]
    fn the_tip_is_the_deepest_point_of_the_curl() {
        let brace = Brace::new().width(5.0);
        let tip = brace.tip();
        let built = brace.build();
        let lowest = built
            .points()
            .iter()
            .copied()
            .fold(f64::INFINITY, |acc, p| acc.min(p[1]));
        assert!(
            (tip[1] - lowest).abs() < 1e-12,
            "the analytic tip {tip:?} is not the lowest point ({lowest})"
        );
        // The Reference finds this by scanning the rendered glyph for its
        // minimum y; ours is closed-form, and the two must agree.
        assert!((tip[0] - 2.5).abs() < 1e-12, "the tip sits on the midline");
    }

    #[test]
    fn down_is_the_unrotated_case_and_the_others_turn_by_right_angles() {
        // The Reference's argument-swapped atan2 is load-bearing; pin it.
        assert!(brace_angle(DOWN).abs() < 1e-12);
        assert!((brace_angle(UP) - PI).abs() < 1e-12);
        assert!((brace_angle(fmn_core::constants::RIGHT) - PI / 2.0).abs() < 1e-12);
        assert!((brace_angle(fmn_core::constants::LEFT) - 3.0 * PI / 2.0).abs() < 1e-12);
    }

    #[test]
    fn a_brace_around_a_mobject_spans_it_and_clears_it_by_the_buff() {
        let target = crate::poly::Rectangle::new()
            .width(4.0)
            .height(2.0)
            .build()
            .expect("the test target is unrounded");
        let brace = Brace::around(&target, DOWN).buff(0.25);
        assert!(
            (brace.width - 4.0).abs() < 1e-9,
            "the brace spans the target's width, got {}",
            brace.width
        );
        let built = brace.build();
        let (bmin, bmax) = built.extent().expect("the brace has extent");
        let (tmin, _) = target.extent().expect("the target has extent");
        assert!(
            (bmax[1] - (tmin[1] - 0.25)).abs() < 1e-9,
            "the brace's top should sit buff below the target's bottom: \
             top {} vs target bottom {}",
            bmax[1],
            tmin[1]
        );
        assert!(bmin[1] < bmax[1], "the brace has depth");
    }

    #[test]
    fn a_zero_width_brace_degenerates_without_panicking() {
        // A caller bracing an empty group gets width 0. It must produce an
        // empty-but-valid shape rather than a NaN or a panic.
        let brace = Brace::new().width(0.0);
        let m = Metrics::for_width(0.0, DEFAULT_BRACE_EM);
        assert_eq!(m.cap, 0.0);
        assert_eq!(m.waist, 0.0);
        let built = brace.build();
        assert!(
            built
                .points()
                .iter()
                .all(|p| p.iter().all(|c| c.is_finite()))
        );
    }

    #[test]
    fn put_at_tip_places_past_the_tip_along_the_brace_direction() {
        let brace = Brace::new().width(3.0);
        let label = crate::poly::Rectangle::new()
            .width(0.4)
            .height(0.2)
            .build()
            .expect("the test label is unrounded");
        let placed = brace.put_at_tip(label, 0.1);
        let tip = brace.tip();
        let (lmin, _) = placed.extent().expect("the label has extent");
        assert!(
            (lmin[1] - (tip[1] - 0.1 - 0.2)).abs() < 1e-9,
            "the label should clear the tip by buff, got bottom {} tip {}",
            lmin[1],
            tip[1]
        );
    }

    #[test]
    fn a_line_brace_turns_with_its_line() {
        // A 45-degree line braced "up" must produce a brace whose direction
        // is the line's own rotated normal, not the world's UP.
        let brace = line_brace([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], UP);
        let expected = 2.0_f64.sqrt();
        assert!(
            (brace.width - expected).abs() < 1e-9,
            "the brace spans the line's length, got {}",
            brace.width
        );
        let direction = brace.tip_direction();
        assert!(
            direction[0] < 0.0 && direction[1] > 0.0,
            "UP against a 45-degree line points up-left, got {:?}",
            direction
        );
    }

    #[test]
    fn a_translated_line_brace_rotates_about_the_line_centre() {
        let start = [3.0, -2.0, 0.0];
        let end = [5.0, 0.0, 0.0];
        let centre = [4.0, -1.0, 0.0];
        let angle = fmn_dmath::atan2(end[1] - start[1], end[0] - start[0]);
        let tangent = rotate_z([1.0, 0.0, 0.0], angle);
        let normal = rotate_z(UP, angle);
        let buff = 0.125;
        let brace = line_brace(start, end, UP).buff(buff);
        let length = get_norm([end[0] - start[0], end[1] - start[1], 0.0]);
        let depth = brace.depth();
        let built = brace.build();

        let projection = |point: Vec3, axis: Vec3| {
            (point[0] - centre[0]) * axis[0]
                + (point[1] - centre[1]) * axis[1]
                + (point[2] - centre[2]) * axis[2]
        };
        let tangent_min = built
            .points()
            .iter()
            .map(|point| projection(*point, tangent))
            .fold(f64::INFINITY, f64::min);
        let tangent_max = built
            .points()
            .iter()
            .map(|point| projection(*point, tangent))
            .fold(f64::NEG_INFINITY, f64::max);
        let normal_min = built
            .points()
            .iter()
            .map(|point| projection(*point, normal))
            .fold(f64::INFINITY, f64::min);
        let normal_max = built
            .points()
            .iter()
            .map(|point| projection(*point, normal))
            .fold(f64::NEG_INFINITY, f64::max);

        assert!((tangent_min + length / 2.0).abs() < 1e-9);
        assert!((tangent_max - length / 2.0).abs() < 1e-9);
        assert!((normal_min - buff).abs() < 1e-9);
        assert!((normal_max - (buff + depth)).abs() < 1e-9);

        let tip = brace.tip();
        assert!(projection(tip, tangent).abs() < 1e-9);
        assert!((projection(tip, normal) - (buff + depth)).abs() < 1e-9);
    }
}
