//! Shape matchers and the de-TeX'd marks (§12.3, Appendix A
//! `mobject/shape_matchers` and `mobject/svg/drawings`).
//!
//! Five classes that wrap or annotate another mobject —
//! [`SurroundingRectangle`], [`background_rectangle`], [`cross`],
//! [`underline`] — plus the two marks the Reference draws with the `pifont`
//! LaTeX package, [`checkmark`] and [`exmark`].
//!
//! # Why the marks are drawn rather than set
//!
//! The bead asked for Checkmark/Exmark "from bundled glyphs (no pifont)". The
//! bundled faces were checked rather than assumed, and they do not support
//! that plan:
//!
//! | Codepoint | Character | Computer Modern | IBM Plex Sans | Noto Sans Math |
//! |---|---|---|---|---|
//! | U+2713 | ✓ CHECK MARK | no | **yes** | **yes** |
//! | U+2717 | ✗ BALLOT X | no | no | no |
//! | U+2714 / U+2718 / U+2715 | heavy variants | no | no | no |
//!
//! So a glyph exists for the check and **none exists for the cross**, in any
//! bundled face. The three ways out were: substitute U+00D7 (present, but a
//! thin multiplication sign — the wrong weight and the wrong shape for
//! `\ding{55}`), bundle a dingbat face (a new font in the closure, for two
//! glyphs), or draw both.
//!
//! Both are drawn. Drawing only the cross would leave a matched pair
//! mismatched — the Reference's `\ding{51}`/`\ding{55}` come from one font and
//! read as siblings, and a Plex check beside a hand-drawn cross would not. It
//! also follows the precedent already set for delimiters in ADR-0005, where
//! the drawn path is the mainline rather than a fallback, for the same reason:
//! the authored glyph is not there.
//!
//! Recorded in Behaviour Note **BN-08**.

use fmn_core::color::Srgb;
use fmn_core::constants::{DL, DR, SMALL_BUFF, UL, UR};
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_mobject::ShapeTag;

use crate::style::Style;
use crate::vmobject::VMobject;

/// `SurroundingRectangle(mobject, buff=SMALL_BUFF, color=YELLOW)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurroundingRectangle {
    extent: Option<(Vec3, Vec3)>,
    buff: f64,
    style: Style,
}

impl SurroundingRectangle {
    /// A rectangle enclosing `target`, `buff` clear of it on every side.
    ///
    /// An empty target yields an empty rectangle rather than a degenerate one
    /// at the origin: there is nothing to surround, and a zero-size box drawn
    /// at `(0, 0)` would be a silent lie about where the caller's content is.
    #[must_use]
    pub fn new(target: &VMobject) -> Self {
        Self::from_extent(target.extent())
    }

    /// Build from an already-authoritative family extent.
    ///
    /// Marionette's live [`fmn_mobject::Stage`] owns family bounding boxes;
    /// front doors must not copy a bound family into a detached [`VMobject`]
    /// merely to feed this matcher.  `None` retains the same empty-target
    /// semantics as [`Self::new`].
    #[must_use]
    pub fn from_extent(extent: Option<(Vec3, Vec3)>) -> Self {
        Self {
            extent,
            buff: SMALL_BUFF,
            style: Style::default()
                .stroke(fmn_core::constants::YELLOW, DEFAULT_STROKE, 1.0)
                .fill(fmn_core::constants::YELLOW, 0.0),
        }
    }

    /// Set the clearance (Reference `set_buff`).
    #[must_use]
    pub fn buff(mut self, buff: f64) -> Self {
        self.buff = buff;
        self
    }

    /// Re-target an existing rectangle (Reference `surround`).
    #[must_use]
    pub fn surround(mut self, target: &VMobject) -> Self {
        self.extent = target.extent();
        self
    }

    /// Set stroke and fill colour together.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style wholesale.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Build the rectangle.
    #[must_use]
    pub fn build(self) -> VMobject {
        let Some((min, max)) = self.extent else {
            return VMobject::new().with_style(self.style);
        };
        let w = (max[0] - min[0]) + 2.0 * self.buff;
        let h = (max[1] - min[1]) + 2.0 * self.buff;
        let centre = [
            0.5 * (min[0] + max[0]),
            0.5 * (min[1] + max[1]),
            0.5 * (min[2] + max[2]),
        ];
        crate::poly::Rectangle::new()
            .width(w)
            .height(h)
            .style(self.style)
            .build()
            .expect("a surrounding rectangle never requests rounded corners")
            .shifted(centre)
            .with_shape(ShapeTag::Rect {
                center: centre,
                width: w,
                height: h,
            })
    }
}

impl From<SurroundingRectangle> for fmn_mobject::Mobject {
    fn from(rect: SurroundingRectangle) -> Self {
        rect.build().into()
    }
}

/// The Reference's default stroke width for these classes.
const DEFAULT_STROKE: f64 = 4.0;

/// `BackgroundRectangle(mobject, color=BLACK, fill_opacity=0.75, buff=0)`:
/// an opaque plate behind content, with no stroke at all.
#[must_use]
pub fn background_rectangle(target: &VMobject, color: Srgb, fill_opacity: f64) -> VMobject {
    SurroundingRectangle::new(target)
        .buff(0.0)
        .style(
            Style::default()
                .fill(color, fill_opacity)
                .stroke(color, 0.0, 0.0),
        )
        .build()
}

/// `Cross(mobject, stroke_color=RED, stroke_width=[0, 6, 0])`: two strokes
/// through the target's box, tapering to nothing at all four ends.
///
/// The Reference builds two `Line`s, inserts twenty curves into each so the
/// width profile has somewhere to land, and stretches the pair onto the
/// target. The taper is the whole visual character of the class — a uniform
/// stroke reads as a plus sign drawn with a ruler — so it is reproduced here
/// through [`VMobject::with_stroke_profile`].
#[must_use]
pub fn cross(target: &VMobject, color: Srgb, width: f64) -> VMobject {
    let Some((min, max)) = target.extent() else {
        return VMobject::new();
    };
    let corner = |d: Vec3| {
        [
            if d[0] > 0.0 { max[0] } else { min[0] },
            if d[1] > 0.0 { max[1] } else { min[1] },
            0.5 * (min[2] + max[2]),
        ]
    };
    let stroke = Style::default().stroke(color, width, 1.0).fill(color, 0.0);
    let arm = |from: Vec3, to: Vec3| {
        // Twenty curves, matching the Reference's `insert_n_curves(20)`: the
        // taper is sampled per point, so a two-point line would have nowhere
        // to put the middle of the profile.
        let mut path = QuadPath::new();
        path.start_new_path(from);
        for i in 1..=20 {
            let t = f64::from(i) / 20.0;
            let at = |a: f64, b: f64| a + (b - a) * t;
            let mid_t = t - 0.5 / 20.0;
            let mid = |a: f64, b: f64| a + (b - a) * mid_t;
            let _ = path.add_quadratic_bezier_curve_to(
                [
                    mid(from[0], to[0]),
                    mid(from[1], to[1]),
                    mid(from[2], to[2]),
                ],
                [at(from[0], to[0]), at(from[1], to[1]), at(from[2], to[2])],
                true,
            );
        }
        VMobject::from_path(&path)
            .with_style(stroke)
            .with_stroke_profile(vec![0.0, width, 0.0])
    };
    VMobject::new().with_children([arm(corner(UL), corner(DR)), arm(corner(UR), corner(DL))])
}

/// `Underline(mobject, buff=SMALL_BUFF, stroke_width=[0, 3, 3, 0],
/// stretch_factor=1.2)`: a rule under the target, fading in at both ends.
#[must_use]
pub fn underline(target: &VMobject, color: Srgb, buff: f64, stretch_factor: f64) -> VMobject {
    let Some((min, max)) = target.extent() else {
        return VMobject::new();
    };
    let half = 0.5 * (max[0] - min[0]) * stretch_factor;
    let centre_x = 0.5 * (min[0] + max[0]);
    let y = min[1] - buff;
    let z = 0.5 * (min[2] + max[2]);
    let mut path = QuadPath::new();
    path.start_new_path([centre_x - half, y, z]);
    for i in 1..=8 {
        let t = f64::from(i) / 8.0;
        let x = centre_x - half + 2.0 * half * t;
        let mid_x = centre_x - half + 2.0 * half * (t - 0.0625);
        let _ = path.add_quadratic_bezier_curve_to([mid_x, y, z], [x, y, z], true);
    }
    VMobject::from_path(&path)
        .with_style(Style::default().stroke(color, 3.0, 1.0).fill(color, 0.0))
        // Flat in the middle, gone at both ends — four control values, not
        // three: the Reference's underline holds full weight across the
        // interior rather than peaking at one point like `Cross` does.
        .with_stroke_profile(vec![0.0, 3.0, 3.0, 0.0])
}

/// The Reference's `Checkmark()` box at its default `font_size = 48`:
/// `\ding{51}`'s ink is 0.3345 × 0.3504 units (fm-5wq.52, pinned Reference
/// `6199a00d` probed side by side with the portal).
pub const CHECKMARK_SIZE: (f64, f64) = (0.3345, 0.3504);

/// The Reference's `Exmark()` box at `font_size = 48`: `\ding{55}`'s ink is
/// 0.279 × 0.3549 units.
pub const EXMARK_SIZE: (f64, f64) = (0.279, 0.3549);

/// A closed polygon over unit-box `outline` points, mapped onto a
/// `size`-box centred on the origin and filled.
fn drawn_mark(outline: &[[f64; 3]], size: (f64, f64), color: Srgb) -> VMobject {
    let place = |p: [f64; 3]| [p[0] * size.0, p[1] * size.1, 0.0];
    let mut path = QuadPath::new();
    path.start_new_path(place(outline[0]));
    for point in &outline[1..] {
        let _ = path.add_line_to(place(*point), true);
    }
    let _ = path.add_line_to(place(outline[0]), true);
    VMobject::from_path(&path).with_style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0))
}

/// `Checkmark` — the Reference's `\ding{51}`, drawn (see the module docs).
///
/// Two strokes meeting at the low point: a short descent to the left and a
/// long rise to the right, filling exactly [`CHECKMARK_SIZE`] centred on the
/// origin — the glyph's size at the default font size, so a portal
/// `font_size` scales from the Reference's own box.
#[must_use]
pub fn checkmark(color: Srgb) -> VMobject {
    // Both tips are cut vertically, so the shape reaches x = ±0.5 on a flat
    // edge rather than at a needle point; the bottom vertex and the top-right
    // tip carry y = ∓0.5. That is what makes the outline exactly fill its
    // box, and therefore what makes the pair's sizes exact.
    let outline: [[f64; 3]; 6] = [
        [-0.5, 0.14, 0.0],   // left tip, upper corner
        [-0.10, -0.24, 0.0], // inner elbow
        [0.5, 0.5, 0.0],     // right tip, upper corner
        [0.5, 0.36, 0.0],    // right tip, lower corner
        [-0.14, -0.5, 0.0],  // bottom vertex, outer
        [-0.5, -0.02, 0.0],  // left tip, lower corner
    ];
    drawn_mark(&outline, CHECKMARK_SIZE, color)
}

/// `Exmark` — the Reference's `\ding{55}`, drawn (see the module docs).
///
/// Two crossed bars of equal weight, drawn like [`checkmark`] so the pair
/// reads as siblings the way the Reference's two dingbats do, filling
/// exactly [`EXMARK_SIZE`] centred on the origin.
#[must_use]
pub fn exmark(color: Srgb) -> VMobject {
    let t = 0.12;
    // One closed contour tracing the whole cross, corner to corner.
    let outline: [[f64; 3]; 12] = [
        [-0.5 + t, -0.5, 0.0],
        [0.0, -t * 0.7, 0.0],
        [0.5 - t, -0.5, 0.0],
        [0.5, -0.5 + t, 0.0],
        [t * 0.7, 0.0, 0.0],
        [0.5, 0.5 - t, 0.0],
        [0.5 - t, 0.5, 0.0],
        [0.0, t * 0.7, 0.0],
        [-0.5 + t, 0.5, 0.0],
        [-0.5, 0.5 - t, 0.0],
        [-t * 0.7, 0.0, 0.0],
        [-0.5, -0.5 + t, 0.0],
    ];
    drawn_mark(&outline, EXMARK_SIZE, color)
}

// -------------------------------------------------- flash conveniences
// (fm-jh7: the Reference's FlashAround/FlashUnder end to end — the
// geometry tier owns the path types, so the value-side pipeline lives
// here and Choreo's wiring consumes the finished handle)

fn processed_flash_path(path: &VMobject, stroke_width: f64, color: Srgb) -> VMobject {
    let mut quad = path
        .path()
        .expect("a surrounding path is always a valid QuadPath");
    quad.insert_n_curves(100)
        .expect("resampling a four-curve rectangle or a nine-curve underline never fails");
    let points = quad.points_without_null_curves(1e-9);
    VMobject::from_points(points).with_style(
        Style::default()
            .stroke(color, stroke_width, 1.0)
            .fill(color, 0.0),
    )
}

/// `FlashAround(mobject, buff=SMALL_BUFF, stroke_width=4.0, color=YELLOW)`
/// (indication.py:255), end to end: build the surrounding rectangle over
/// `target`'s extent, run the Reference's resample-and-strip pipeline,
/// style it, add it to the stage, and return the [`fmn_anim::indication::VShowPassingFlash`]
/// with the Reference defaults (`time_width = 1.0`, `taper_width = 0.0`,
/// `remover`).
#[must_use]
pub fn flash_around(
    stage: &mut fmn_mobject::Stage,
    target: &VMobject,
    buff: f64,
    stroke_width: f64,
    color: Srgb,
) -> fmn_anim::indication::VShowPassingFlash {
    let path = processed_flash_path(
        &SurroundingRectangle::new(target)
            .buff(buff)
            .color(color)
            .build(),
        stroke_width,
        color,
    );
    let mob = stage.add(path);
    fmn_anim::indication::flash_around(mob, 1.0, 0.0)
}

/// `FlashUnder(mobject, buff=SMALL_BUFF, stroke_width=4.0, color=YELLOW)`
/// (indication.py:279): [`flash_around`] over an [`underline`] path (the
/// Reference's `FlashUnder` overrides only `get_path`).
#[must_use]
pub fn flash_under(
    stage: &mut fmn_mobject::Stage,
    target: &VMobject,
    buff: f64,
    stroke_width: f64,
    color: Srgb,
) -> fmn_anim::indication::VShowPassingFlash {
    let path = processed_flash_path(&underline(target, color, buff, 1.2), stroke_width, color);
    let mob = stage.add(path);
    fmn_anim::indication::flash_around(mob, 1.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_anim::Animation as _;
    use fmn_core::constants::{GREEN, RED, YELLOW};

    fn box_of(w: f64, h: f64) -> VMobject {
        crate::poly::Rectangle::new()
            .width(w)
            .height(h)
            .build()
            .expect("the test box is unrounded")
    }

    #[test]
    fn a_surrounding_rectangle_clears_its_target_on_every_side() {
        let target = box_of(2.0, 1.0).shifted([0.5, -0.25, 0.0]);
        let rect = SurroundingRectangle::new(&target).buff(0.1).build();
        let (rmin, rmax) = rect.extent().expect("the rectangle has extent");
        let (tmin, tmax) = target.extent().expect("the target has extent");
        for dim in 0..2 {
            assert!(
                (rmin[dim] - (tmin[dim] - 0.1)).abs() < 1e-9,
                "dim {dim}: min {} vs expected {}",
                rmin[dim],
                tmin[dim] - 0.1
            );
            assert!(
                (rmax[dim] - (tmax[dim] + 0.1)).abs() < 1e-9,
                "dim {dim}: max {} vs expected {}",
                rmax[dim],
                tmax[dim] + 0.1
            );
        }
    }

    #[test]
    fn an_authoritative_family_extent_uses_the_same_matcher_path() {
        let extent = Some(([-2.0, -1.5, 0.0], [3.0, 0.5, 0.0]));
        let rect = SurroundingRectangle::from_extent(extent).buff(0.25).build();
        let (min, max) = rect.extent().expect("the rectangle has extent");
        assert_eq!(min, [-2.25, -1.75, 0.0]);
        assert_eq!(max, [3.25, 0.75, 0.0]);

        assert!(
            SurroundingRectangle::from_extent(None)
                .build()
                .points()
                .is_empty(),
            "an empty live family must not fabricate a box at the origin"
        );
    }

    #[test]
    fn a_surrounding_rectangle_keeps_its_primitive_hint() {
        let rect = SurroundingRectangle::new(&box_of(2.0, 1.0)).build();
        assert!(
            matches!(rect.shape(), ShapeTag::Rect { .. }),
            "the renderer's rectangle fast path needs the tag, got {:?}",
            rect.shape()
        );
    }

    #[test]
    fn surround_retargets_without_disturbing_the_buff() {
        let first = SurroundingRectangle::new(&box_of(1.0, 1.0)).buff(0.3);
        let moved = first.surround(&box_of(4.0, 2.0));
        let (min, max) = moved.build().extent().expect("extent");
        assert!((max[0] - min[0] - (4.0 + 0.6)).abs() < 1e-9);
        assert!((max[1] - min[1] - (2.0 + 0.6)).abs() < 1e-9);
    }

    #[test]
    fn empty_surrounds_have_no_geometry_and_can_be_retargeted() {
        let empty = VMobject::new();
        let initial = SurroundingRectangle::new(&empty).build();
        assert!(initial.points().is_empty());
        assert!(initial.children().is_empty());

        let cleared = SurroundingRectangle::new(&box_of(2.0, 1.0))
            .surround(&empty)
            .build();
        assert!(cleared.points().is_empty());

        let restored = SurroundingRectangle::new(&empty)
            .buff(0.3)
            .surround(&box_of(4.0, 2.0))
            .build();
        let (min, max) = restored.extent().expect("retargeted extent");
        assert!((max[0] - min[0] - (4.0 + 0.6)).abs() < 1e-9);
        assert!((max[1] - min[1] - (2.0 + 0.6)).abs() < 1e-9);
    }

    #[test]
    fn a_background_rectangle_has_fill_and_no_stroke() {
        let plate = background_rectangle(&box_of(2.0, 1.0), fmn_core::constants::BLACK, 0.75);
        let style = plate.style();
        assert!((style.fill_opacity - 0.75).abs() < 1e-12);
        assert_eq!(style.stroke_width, 0.0, "a plate must not outline itself");
        assert_eq!(style.stroke_opacity, 0.0);
        let (pmin, pmax) = plate.extent().expect("extent");
        assert!(
            (pmax[0] - pmin[0] - 2.0).abs() < 1e-9,
            "buff is zero, so the plate matches the target exactly"
        );
    }

    #[test]
    fn a_cross_spans_its_targets_diagonals_and_tapers_to_nothing() {
        let target = box_of(2.0, 1.0);
        let mark = cross(&target, RED, 6.0);
        assert_eq!(mark.children().len(), 2, "a cross has two arms");
        let (cmin, cmax) = mark.extent().expect("extent");
        let (tmin, tmax) = target.extent().expect("extent");
        for dim in 0..2 {
            assert!((cmin[dim] - tmin[dim]).abs() < 1e-9, "dim {dim} min");
            assert!((cmax[dim] - tmax[dim]).abs() < 1e-9, "dim {dim} max");
        }
        for arm in mark.children() {
            let profile = arm.stroke_profile().expect("each arm tapers");
            assert_eq!(profile.first(), Some(&0.0), "an arm starts at nothing");
            assert_eq!(profile.last(), Some(&0.0), "an arm ends at nothing");
            assert!(
                arm.points().len() > 20,
                "the taper needs points to land on, got {}",
                arm.points().len()
            );
        }
    }

    #[test]
    fn an_underline_sits_below_its_target_and_overhangs_it() {
        let target = box_of(2.0, 1.0);
        let rule = underline(&target, fmn_core::constants::WHITE, 0.1, 1.2);
        let (rmin, rmax) = rule.extent().expect("extent");
        let (tmin, _) = target.extent().expect("extent");
        assert!(
            (rmax[1] - (tmin[1] - 0.1)).abs() < 1e-9,
            "the rule sits buff below the target"
        );
        assert!(
            (rmax[0] - rmin[0] - 2.4).abs() < 1e-9,
            "stretch_factor 1.2 overhangs a 2.0-wide target to 2.4, got {}",
            rmax[0] - rmin[0]
        );
        let profile = rule.stroke_profile().expect("the rule tapers");
        assert_eq!(profile, [0.0, 3.0, 3.0, 0.0], "flat across the interior");
    }

    #[test]
    fn both_marks_are_closed_shapes_at_the_reference_glyph_size() {
        // The pair has to read as siblings — matched weight — which is the
        // whole reason both are drawn rather than one being set from IBM
        // Plex's U+2713. Their boxes are the Reference's glyph boxes at the
        // default font size (fm-5wq.52: they were 3x too large).
        for (name, mark, size) in [
            ("check", checkmark(GREEN), CHECKMARK_SIZE),
            ("ex", exmark(RED), EXMARK_SIZE),
        ] {
            let path = mark.path().expect("a built mark is a valid path");
            assert!(path.is_closed(), "{name} must be a closed filled shape");
            let (min, max) = mark.extent().expect("a built mark has an extent");
            assert!(
                (max[0] - min[0] - size.0).abs() < 1e-9,
                "{name} width {}",
                max[0] - min[0]
            );
            assert!(
                (max[1] - min[1] - size.1).abs() < 1e-9,
                "{name} height {}",
                max[1] - min[1]
            );
            assert!(mark.center_point()[0].abs() < 1e-9 && mark.center_point()[1].abs() < 1e-9);
            let style = mark.style();
            assert!(
                (style.fill_opacity - 1.0).abs() < 1e-12,
                "{name} is a filled glyph shape, not an outline"
            );
        }
    }

    #[test]
    fn the_marks_carry_the_references_default_colours() {
        assert_eq!(checkmark(GREEN).style().fill_color, GREEN);
        assert_eq!(exmark(RED).style().fill_color, RED);
    }

    #[test]
    fn matchers_on_an_empty_target_have_no_geometry() {
        // fm-sjl: styling a point-less mobject is a documented no-op, and
        // these classes are the ones most likely to meet one (a caller
        // surrounds a group before filling it).
        let empty = VMobject::new();
        for (name, built) in [
            ("surround", SurroundingRectangle::new(&empty).build()),
            ("plate", background_rectangle(&empty, RED, 0.5)),
            ("cross", cross(&empty, RED, 6.0)),
            ("underline", underline(&empty, RED, 0.1, 1.2)),
        ] {
            assert!(built.points().is_empty(), "{name} invented target geometry");
            assert!(
                built.children().is_empty(),
                "{name} invented child geometry"
            );
        }
    }

    #[test]
    fn a_stroke_profile_resizes_onto_the_point_run() {
        // The profile describes the shape of the taper, not one width per
        // point, so it must survive a change of path resolution.
        let coarse = cross(&box_of(1.0, 1.0), RED, 6.0);
        let arm = &coarse.children()[0];
        assert_eq!(arm.stroke_profile(), Some([0.0, 6.0, 0.0].as_slice()));
        let mob: fmn_mobject::Mobject = arm.clone().into();
        let widths: Vec<f32> = (0..mob.buffer.len())
            .filter_map(|i| mob.buffer.read(i, "stroke_width").map(|v| v[0]))
            .collect();
        assert_eq!(widths.len(), arm.points().len());
        assert!(widths[0].abs() < 1e-6, "starts at nothing");
        assert!(
            widths[widths.len() - 1].abs() < 1e-6,
            "ends at nothing, got {}",
            widths[widths.len() - 1]
        );
        let peak = widths.iter().copied().fold(0.0_f32, f32::max);
        assert!(
            (peak - 6.0).abs() < 1e-4,
            "peaks at the declared width, got {peak}"
        );
    }

    #[test]
    fn flash_around_and_flash_under_build_dense_processed_sweeps() {
        let mut stage = fmn_mobject::Stage::new();
        let target = box_of(2.0, 1.0);

        let around = flash_around(&mut stage, &target, SMALL_BUFF, 4.0, YELLOW);
        assert_eq!(around.state().config.name, "FlashAround");
        assert!(around.state().config.remover);
        let around_points = stage
            .get(around.state().mobject())
            .expect("the sweep was added")
            .buffer
            .len();
        assert!(around_points >= 100, "resampled dense, got {around_points}");

        let under = flash_under(&mut stage, &target, SMALL_BUFF, 4.0, YELLOW);
        assert_eq!(under.state().config.name, "FlashAround");
        let under_points = stage
            .get(under.state().mobject())
            .expect("the sweep was added")
            .buffer
            .len();
        assert!(under_points >= 100, "resampled dense, got {under_points}");

        // The two sweeps ride different handles — each convenience added its
        // own processed path.
        assert_ne!(around.state().mobject(), under.state().mobject());
    }
}
