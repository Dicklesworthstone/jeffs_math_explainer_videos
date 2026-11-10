//! Primitive hints (§10.8): routing semantic shapes off the general quadratic
//! solver, and the rule that keeps that an optimization rather than a second
//! semantics.
//!
//! §10.8 enumerates the routing exhaustively:
//!
//! > line/polyline → capsule-distance kernel; circle/arc → direct arc kernel;
//! > rectangle/rounded rectangle → specialized coverage; dot → radial kernel;
//! > image → sampler kernel; nearly-linear quadratic → line fast path under an
//! > explicit screen-space error bound; glyph → cached path instance. Mutation
//! > through `set_points` or a writable live view invalidates the hint back to
//! > the general path.
//!
//! ## The rule that matters more than the routing
//!
//! **Correctness never depends on a hint.** A kernel selected by a hint must
//! produce the same answer the general path would, so dropping every hint can
//! only cost speed. That is what makes the invalidation rule safe to be
//! conservative about, and it is asserted directly in G0-8's spike — the `Line`
//! hint and the general quadratic solver are held to the same pixels.
//!
//! Two invalidation sources, and they are different in kind:
//!
//! - **A write demotes the hint.** Marionette already implements this by
//!   *revision comparison* rather than a dirty flag: `Stage::primitive_hint`
//!   returns the tag only while the point revision it was stamped at is still
//!   current, so all four write channels (positional transform, raw record
//!   write, `set_points`, a writable live view) demote it without any of them
//!   knowing the rule exists. This module consumes that answer; it does not
//!   re-derive it.
//! - **A screen-space bound demotes a hint the tag never claimed.** The
//!   `nearly-linear quadratic → line fast path` route is not a durable class
//!   identity, it is a per-frame numeric judgement, and it is the one hint that
//!   can be *wrong* rather than merely absent. [`Hint::for_curve`] applies it
//!   under an explicit bound (§10.8's words), computed against the actual screen
//!   scale rather than a tolerance someone picked once.
//!
//! ## What is deliberately absent
//!
//! `Image` and `Glyph` have no producer in the geometry tier's vocabulary
//! — that is the `ShapeTag` contract — so this enum stops there too. Images
//! exist (`fmn_library::ImageMobject` produces a retained `ImageQuad`), but
//! they route through `RenderPrimitive` and its own sampler kernel rather
//! than through this geometric-hint selector. Glyphs are still full vector
//! outlines in Scribe; once they are interned as cached path instances, an
//! `Image`-/`Glyph`-style hint may be added without dead branches because
//! there will be producers that can construct them.

use fmn_geom::quadpath::QuadPath;
use fmn_mobject::{Mob, ShapeTag, Stage};

/// A compiled path's kernel routing.
///
/// Distinct from `ShapeTag`, which is Marionette's *durable class identity*:
/// a `Circle` stays a `Circle` in the object model after someone drags a point,
/// while its hint is gone. Keeping the two types apart is what stops a render
/// kernel reading a radius that no longer describes the points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Hint {
    /// The general quadratic machinery. Always correct, never fastest.
    #[default]
    General,
    /// Every curve is straight: point-to-capsule distance, no root solve.
    Line,
    /// A run of straight segments meeting at corners.
    Polyline {
        /// Whether the last vertex repeats the first.
        closed: bool,
    },
    /// A circular arc: distance is `|r - |p - c||`, one subtraction.
    Arc {
        /// Centre of curvature, object space.
        center: [f64; 3],
        /// Radius.
        radius: f64,
        /// Angle of the first anchor from `+x`.
        start_angle: f64,
        /// Subtended angle, signed.
        angle: f64,
    },
    /// A full circle — an [`Hint::Arc`] of `TAU` with no angular test at all.
    Circle {
        /// Centre.
        center: [f64; 3],
        /// Radius.
        radius: f64,
    },
    /// A filled disc: the radial kernel.
    Dot {
        /// Centre.
        center: [f64; 3],
        /// Radius.
        radius: f64,
    },
    /// An axis-aligned rectangle: coverage is two clamped intervals.
    Rect {
        /// Centre.
        center: [f64; 3],
        /// Full width.
        width: f64,
        /// Full height.
        height: f64,
    },
    /// A rectangle with circular corners.
    RoundedRect {
        /// Centre.
        center: [f64; 3],
        /// Full width.
        width: f64,
        /// Full height.
        height: f64,
        /// Corner radius.
        corner_radius: f64,
    },
}

impl Hint {
    /// A short stable name, for traces and IR golden snapshots.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Hint::General => "general",
            Hint::Line => "line",
            Hint::Polyline { .. } => "polyline",
            Hint::Arc { .. } => "arc",
            Hint::Circle { .. } => "circle",
            Hint::Dot { .. } => "dot",
            Hint::Rect { .. } => "rect",
            Hint::RoundedRect { .. } => "rounded_rect",
        }
    }

    /// Does this hint route off the general solver at all?
    #[must_use]
    pub fn is_general(self) -> bool {
        matches!(self, Hint::General)
    }

    /// Shift this hint's geometric payload by `delta`.
    ///
    /// Hints carry absolute object-space geometry — a circle's centre, a
    /// rectangle's centre — while a compiled outline is stored **shape-local**
    /// so that instancing can place it (see `crate::table::Shape`). The two must
    /// agree, so a hint travelling with an interned outline is localized the
    /// same way and a consumer adds the instance's offset back.
    ///
    /// Getting this wrong is silent and wrong in the worst way: the hint selects
    /// a kernel, so a circle kernel handed the *first* instance's centre draws
    /// every later copy in the wrong place while the general path would have
    /// been right.
    #[must_use]
    pub fn translated(self, delta: [f64; 3]) -> Hint {
        let shift = |c: [f64; 3]| [c[0] + delta[0], c[1] + delta[1], c[2] + delta[2]];
        match self {
            Hint::General | Hint::Line | Hint::Polyline { .. } => self,
            Hint::Arc {
                center,
                radius,
                start_angle,
                angle,
            } => Hint::Arc {
                center: shift(center),
                radius,
                start_angle,
                angle,
            },
            Hint::Circle { center, radius } => Hint::Circle {
                center: shift(center),
                radius,
            },
            Hint::Dot { center, radius } => Hint::Dot {
                center: shift(center),
                radius,
            },
            Hint::Rect {
                center,
                width,
                height,
            } => Hint::Rect {
                center: shift(center),
                width,
                height,
            },
            Hint::RoundedRect {
                center,
                width,
                height,
                corner_radius,
            } => Hint::RoundedRect {
                center: shift(center),
                width,
                height,
                corner_radius,
            },
        }
    }

    /// The hint for `mob`'s current points, as Marionette reports them.
    ///
    /// Returns [`Hint::General`] whenever the shape tag has been demoted, which
    /// is Marionette's answer and not a judgement made here — the four write
    /// channels are its business, and duplicating the rule would give two places
    /// for it to be wrong.
    #[must_use]
    pub fn of(stage: &Stage, mob: Mob) -> Hint {
        stage.primitive_hint(mob).map_or(Hint::General, Hint::from)
    }
}

impl From<ShapeTag> for Hint {
    /// Translate a live shape tag into a kernel route.
    ///
    /// `Line` with a nonzero `path_arc` is **not** a line: the Reference's
    /// `Line` family carries a curved variant, and routing it to the capsule
    /// kernel would straighten a visible arc. It becomes [`Hint::General`],
    /// because there is no arc kernel that can be given a centre this tag does
    /// not carry.
    fn from(tag: ShapeTag) -> Hint {
        match tag {
            ShapeTag::General => Hint::General,
            ShapeTag::Line { path_arc, .. } => {
                if path_arc == 0.0 {
                    Hint::Line
                } else {
                    Hint::General
                }
            }
            ShapeTag::Polyline { closed, .. } => Hint::Polyline { closed },
            ShapeTag::Arc {
                center,
                radius,
                start_angle,
                angle,
            } => Hint::Arc {
                center,
                radius,
                start_angle,
                angle,
            },
            ShapeTag::Circle { center, radius } => Hint::Circle { center, radius },
            ShapeTag::Dot { center, radius } => Hint::Dot { center, radius },
            ShapeTag::Rect {
                center,
                width,
                height,
            } => Hint::Rect {
                center,
                width,
                height,
            },
            ShapeTag::RoundedRect {
                center,
                width,
                height,
                corner_radius,
            } => Hint::RoundedRect {
                center,
                width,
                height,
                corner_radius,
            },
        }
    }
}

/// The screen-space error a curve may show and still take the line fast path.
///
/// A quarter of a pixel: comfortably inside the ~1.5 px AA band the kept look
/// runs on (§1.5), so a curve that qualifies cannot move an antialiased edge by
/// a level it could be judged on. It is a **screen-space** bound, which is the
/// whole point — the same object-space curve qualifies when the camera is far
/// and does not when it is close, and a tolerance chosen in object space would
/// be a different picture at every zoom.
pub const NEARLY_LINEAR_PX: f64 = 0.25;

impl Hint {
    /// §10.8's "nearly-linear quadratic → line fast path under an explicit
    /// screen-space error bound".
    ///
    /// `scale` is object units → pixels. A quadratic's greatest deviation from
    /// its chord is at `t = ½` and equals **half** the handle's offset from the
    /// chord midpoint — exact, not a bound, because a quadratic is a parabola
    /// and `B(½) = ¼p₀ + ½p₁ + ¼p₂` sits exactly halfway between the chord
    /// midpoint and the handle. So the test is one subtraction and a comparison,
    /// with no subdivision and no iteration.
    ///
    /// Returns [`Hint::Line`] or [`Hint::General`]; it never upgrades to a
    /// richer hint, because being *nearly* straight says nothing about being a
    /// circle.
    #[must_use]
    pub fn for_curve(a0: [f64; 3], handle: [f64; 3], a1: [f64; 3], scale: f64) -> Hint {
        let mid = [
            0.5 * (a0[0] + a1[0]),
            0.5 * (a0[1] + a1[1]),
            0.5 * (a0[2] + a1[2]),
        ];
        let dev = [
            0.5 * (handle[0] - mid[0]),
            0.5 * (handle[1] - mid[1]),
            0.5 * (handle[2] - mid[2]),
        ];
        let sagitta = (dev[0] * dev[0] + dev[1] * dev[1] + dev[2] * dev[2]).sqrt();
        if sagitta * scale <= NEARLY_LINEAR_PX {
            Hint::Line
        } else {
            Hint::General
        }
    }

    /// The whole-path form of [`Hint::for_curve`]: [`Hint::Line`] only if
    /// **every** curve qualifies.
    ///
    /// One curve over the bound disqualifies the path, because the hint selects
    /// a kernel for the path and a per-curve mixture would need the general
    /// solver present anyway.
    #[must_use]
    pub fn for_path(path: &QuadPath, scale: f64) -> Hint {
        let n = path.num_curves();
        if n == 0 {
            return Hint::General;
        }
        for i in 0..n {
            let Some([a0, h, a1]) = path.nth_curve_points(i) else {
                return Hint::General;
            };
            if Hint::for_curve(a0, h, a1, scale).is_general() {
                return Hint::General;
            }
        }
        Hint::Line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};

    fn vmobject(points: &[[f64; 3]]) -> Mobject {
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
        for (i, p) in points.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
        }
        Mobject::from_buffer(buffer)
    }

    fn staged_circle() -> (Stage, Mob) {
        let mut stage = Stage::new();
        let mob = stage.add(vmobject(&[
            [1.0, 0.0, 0.0],
            [1.0, 0.5, 0.0],
            [0.0, 1.0, 0.0],
            [-0.5, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
        ]));
        stage.set_shape(
            mob,
            ShapeTag::Circle {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
            },
        );
        (stage, mob)
    }

    #[test]
    fn a_live_tag_becomes_its_kernel_route() {
        let (stage, mob) = staged_circle();
        assert_eq!(
            Hint::of(&stage, mob),
            Hint::Circle {
                center: [0.0, 0.0, 0.0],
                radius: 1.0
            }
        );
    }

    #[test]
    fn writing_a_point_demotes_the_hint_to_the_general_path() {
        // The §10.8 invalidation rule. It is Marionette's revision comparison
        // doing the work — this test exists to prove fmn-render *consumes* that
        // answer rather than caching a stale one.
        let (mut stage, mob) = staged_circle();
        assert!(!Hint::of(&stage, mob).is_general());
        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .write(2, "point", &[0.0, 9.0, 0.0]);
        assert_eq!(
            Hint::of(&stage, mob),
            Hint::General,
            "a write through the record buffer must demote the hint"
        );
    }

    #[test]
    fn a_writable_live_view_demotes_the_hint() {
        // The fourth write channel, and the one most likely to be forgotten:
        // §8.2's exported NumPy view can mutate points without any Stage method
        // being called.
        let (mut stage, mob) = staged_circle();
        let view = stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .export_field_view("point", true)
            .expect("point field");
        assert_eq!(
            Hint::of(&stage, mob),
            Hint::General,
            "a zero-copy writer need not call RecordView::write"
        );
        assert!(view.write(0, "point", &[5.0, 0.0, 0.0]), "the view writes");
        drop(view);
        assert_eq!(Hint::of(&stage, mob), Hint::General);
    }

    #[test]
    fn a_recolour_does_not_demote_the_hint() {
        // The complement, and the reason the rule is keyed on the *point*
        // revision rather than the buffer's global one: a circle that changes
        // colour is still a circle, and re-deriving its geometry would be the
        // exact waste §10.8 exists to prevent.
        let (mut stage, mob) = staged_circle();
        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .write(0, "stroke_rgba", &[1.0, 0.0, 0.0, 1.0]);
        assert!(!Hint::of(&stage, mob).is_general());
    }

    #[test]
    fn a_curved_line_tag_is_not_a_line() {
        // The Reference's Line family carries `path_arc`; routing a curved one
        // to the capsule kernel would straighten a visible arc.
        assert_eq!(
            Hint::from(ShapeTag::Line {
                start: [0.0; 3],
                end: [1.0, 0.0, 0.0],
                path_arc: 0.0,
                buff: 0.0
            }),
            Hint::Line
        );
        assert_eq!(
            Hint::from(ShapeTag::Line {
                start: [0.0; 3],
                end: [1.0, 0.0, 0.0],
                path_arc: 0.5,
                buff: 0.0
            }),
            Hint::General
        );
    }

    #[test]
    fn an_untagged_mobject_is_general() {
        let mut stage = Stage::new();
        let mob = stage.add(vmobject(&[[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]));
        assert_eq!(Hint::of(&stage, mob), Hint::General);
    }

    #[test]
    fn the_nearly_linear_bound_is_measured_in_pixels_not_object_units() {
        // The same curve, two camera scales. This is the whole reason the bound
        // takes a scale: an object-space tolerance is a different picture at
        // every zoom.
        let a0 = [0.0, 0.0, 0.0];
        let a1 = [1.0, 0.0, 0.0];
        // Sagitta = half the handle's offset from the chord midpoint = 0.01.
        let handle = [0.5, 0.02, 0.0];

        assert_eq!(Hint::for_curve(a0, handle, a1, 10.0), Hint::Line, "0.1 px");
        assert_eq!(
            Hint::for_curve(a0, handle, a1, 1000.0),
            Hint::General,
            "10 px of bow is not a line at any zoom"
        );
        // And the boundary itself: sagitta * scale == the bound exactly.
        let scale = NEARLY_LINEAR_PX / 0.01;
        assert_eq!(
            Hint::for_curve(a0, handle, a1, scale),
            Hint::Line,
            "the bound is inclusive"
        );
    }

    #[test]
    fn an_exactly_straight_curve_qualifies_at_any_scale() {
        let a0 = [0.0, 0.0, 0.0];
        let a1 = [4.0, 2.0, 0.0];
        let handle = [2.0, 1.0, 0.0];
        for scale in [1.0, 1e3, 1e9] {
            assert_eq!(Hint::for_curve(a0, handle, a1, scale), Hint::Line);
        }
    }

    #[test]
    fn the_sagitta_is_exact_for_a_quadratic() {
        // Not a bound — the claim in the doc comment is that a quadratic's
        // greatest deviation from its chord is exactly half the handle's offset
        // from the chord midpoint, attained at t = 1/2. If that is ever wrong
        // the fast path is unsound, so it is checked against the curve itself.
        let (a0, handle, a1) = ([0.0, 0.0, 0.0], [0.5, 1.0, 0.0], [1.0, 0.0, 0.0]);
        let mut worst: f64 = 0.0;
        for i in 0..=1000 {
            let t = i as f64 / 1000.0;
            let u = 1.0 - t;
            let p = [
                u * u * a0[0] + 2.0 * u * t * handle[0] + t * t * a1[0],
                u * u * a0[1] + 2.0 * u * t * handle[1] + t * t * a1[1],
            ];
            // Distance from the chord, which here is the x-axis.
            worst = worst.max(p[1].abs());
        }
        let predicted = 0.5 * (handle[1] - 0.5 * (a0[1] + a1[1])).abs();
        assert!(
            (worst - predicted).abs() < 1e-9,
            "sampled {worst} vs predicted {predicted}"
        );
    }

    #[test]
    fn a_path_is_a_line_only_if_every_curve_is() {
        let mut straight = QuadPath::default();
        straight.start_new_path([0.0, 0.0, 0.0]);
        straight.add_line_to([1.0, 0.0, 0.0], false).unwrap();
        straight.add_line_to([2.0, 1.0, 0.0], false).unwrap();
        assert_eq!(Hint::for_path(&straight, 100.0), Hint::Line);

        let mut mixed = QuadPath::default();
        mixed.start_new_path([0.0, 0.0, 0.0]);
        mixed.add_line_to([1.0, 0.0, 0.0], false).unwrap();
        mixed
            .add_quadratic_bezier_curve_to([1.5, 1.0, 0.0], [2.0, 0.0, 0.0], false)
            .unwrap();
        assert_eq!(
            Hint::for_path(&mixed, 100.0),
            Hint::General,
            "one bowed curve disqualifies the path"
        );

        assert_eq!(Hint::for_path(&QuadPath::default(), 100.0), Hint::General);
    }
}
