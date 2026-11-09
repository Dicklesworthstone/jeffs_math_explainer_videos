//! The geometry-only bubbles (`SpeechBubble`, `ThoughtBubble`): the
//! Reference's construction, as structural facts — body and content in that
//! order, the body enclosing the content, the speech stem hanging below the
//! lower-left (mirrored to the right), the thought cloud's trailing circles,
//! the Reference fill/stroke, and one-RNG determinism.

use fmn_core::constants::{BLACK, MED_SMALL_BUFF, RIGHT, SMALL_BUFF, WHITE};
use fmn_core::rng::Pcg64Dxsm;
use fmn_library::drawings::{
    ThoughtBubbleShape, speech_bubble, speech_bubble_default, thought_bubble,
    thought_bubble_default,
};
use fmn_library::poly::Rectangle;
use fmn_library::vmobject::VMobject;

fn content() -> VMobject {
    Rectangle::new()
        .width(3.0)
        .height(1.0)
        .build()
        .unwrap()
        .shifted([1.0, 0.5, 0.0])
}

fn family_points(vmob: &VMobject) -> Vec<[f64; 3]> {
    let mut out = vmob.points().to_vec();
    for child in vmob.children() {
        out.extend(family_points(child));
    }
    out
}

#[test]
fn speech_bubble_surrounds_its_content_and_hangs_a_stem() {
    let content = content();
    let bubble = speech_bubble_default(Some(&content)).unwrap();
    let [body, kept] = bubble.children() else {
        panic!("a bubble is VGroup(body, content)");
    };
    assert_eq!(kept.points(), content.points());
    let (cmin, cmax) = content.extent().unwrap();
    let (bmin, bmax) = body.extent().unwrap();
    // The rounded rectangle clears the content by `buff` left, right, top.
    assert!(
        (bmax[0] - (cmax[0] + MED_SMALL_BUFF)).abs() < 1e-6,
        "{bmax:?}"
    );
    assert!(
        (bmin[0] - (cmin[0] - MED_SMALL_BUFF)).abs() < 1e-6,
        "{bmin:?}"
    );
    assert!(
        (bmax[1] - (cmax[1] + MED_SMALL_BUFF)).abs() < 1e-6,
        "{bmax:?}"
    );
    // The stem tip hangs half the bubble height below the lower-left corner.
    let rect_height = (cmax[1] - cmin[1]) + 2.0 * MED_SMALL_BUFF;
    let stem_tip = cmin[1] - MED_SMALL_BUFF - 0.5 * rect_height;
    assert!((bmin[1] - stem_tip).abs() < 1e-6, "{bmin:?} vs {stem_tip}");
    let lowest = family_points(body)
        .into_iter()
        .min_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap();
    assert!(
        (lowest[0] - (cmin[0] - MED_SMALL_BUFF)).abs() < 1e-6,
        "{lowest:?}"
    );
    // The Reference's Bubble style on the body.
    assert_eq!(body.style().fill_color, BLACK);
    assert!((body.style().fill_opacity - 0.8).abs() < 1e-12);
    assert_eq!(body.style().stroke_color, WHITE);
    assert!((body.style().stroke_width - 3.0).abs() < 1e-12);

    // A rightward bubble mirrors the stem to the lower-right.
    let right = speech_bubble(
        Some(&content),
        RIGHT,
        MED_SMALL_BUFF,
        (2.0, 1.0),
        0.5,
        (0.2, 0.3),
    )
    .unwrap();
    let lowest = family_points(&right.children()[0])
        .into_iter()
        .min_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap();
    assert!(
        (lowest[0] - (cmax[0] + MED_SMALL_BUFF)).abs() < 1e-6,
        "{lowest:?}"
    );
}

#[test]
fn speech_bubble_without_content_surrounds_an_invisible_filler() {
    let bubble = speech_bubble_default(None).unwrap();
    let filler = &bubble.children()[1];
    let (min, max) = filler.extent().unwrap();
    assert!(((max[0] - min[0]) - 2.0).abs() < 1e-9);
    assert!(((max[1] - min[1]) - 1.0).abs() < 1e-9);
    assert_eq!(filler.style().fill_opacity, 0.0);
    assert_eq!(filler.style().stroke_width, 0.0);
}

#[test]
fn thought_bubble_is_a_cloud_with_trailing_circles_from_the_one_rng() {
    let content = content();
    let bubble = thought_bubble_default(Some(&content), &mut Pcg64Dxsm::from_seed(7)).unwrap();
    let body = &bubble.children()[0];
    // Three trailing circles, then the cloud — the Reference's child order.
    assert_eq!(body.children().len(), 4);
    let cloud = &body.children()[3];
    let (cmin, cmax) = content.extent().unwrap();
    let (min, max) = cloud.extent().unwrap();
    // The cloud's bulges stand out past the surrounding rectangle.
    assert!(min[0] < cmin[0] - SMALL_BUFF && max[0] > cmax[0] + SMALL_BUFF);
    assert!(min[1] < cmin[1] - SMALL_BUFF && max[1] > cmax[1] + SMALL_BUFF);
    // Circles trail below the cloud, ascending in radius up-right.
    let circles: Vec<_> = body.children()[..3]
        .iter()
        .map(|circle| circle.extent().unwrap())
        .collect();
    for (lo, hi) in &circles {
        assert!(hi[1] < min[1], "a trailing circle overlaps the cloud");
        assert!(hi[0] - lo[0] > 0.0);
    }
    let widths: Vec<f64> = circles.iter().map(|(lo, hi)| hi[0] - lo[0]).collect();
    assert!(widths[0] < widths[1] && widths[1] < widths[2], "{widths:?}");
    assert!(circles[0].0[0] < circles[2].0[0]);

    // One RNG: a seed reproduces the cloud exactly; another seed differs.
    let again = thought_bubble_default(Some(&content), &mut Pcg64Dxsm::from_seed(7)).unwrap();
    assert_eq!(family_points(&bubble), family_points(&again));
    let other = thought_bubble_default(Some(&content), &mut Pcg64Dxsm::from_seed(8)).unwrap();
    assert_ne!(family_points(&bubble), family_points(&other));

    // A rightward bubble mirrors the trailing circles to the right.
    let right = thought_bubble(
        Some(&content),
        RIGHT,
        SMALL_BUFF,
        (2.0, 1.0),
        &ThoughtBubbleShape::default(),
        &mut Pcg64Dxsm::from_seed(7),
    )
    .unwrap();
    let left_tail = body.children()[0].extent().unwrap().0[0];
    let right_tail = right.children()[0].children()[0].extent().unwrap().1[0];
    assert!(
        left_tail < 1.0 && right_tail > 1.0,
        "{left_tail} {right_tail}"
    );
}

#[test]
fn vmobject_from_svg_path_keeps_raw_path_coordinates() {
    use fmn_library::vmobject_from_svg_path;
    // Two subpaths: a right triangle and a quadratic, in user space (y down).
    let vmob = vmobject_from_svg_path("M 0 0 L 4 0 L 4 3 Z M 10 10 Q 12 14 14 10").unwrap();
    let points = vmob.points();
    assert_eq!(points.first().copied(), Some([0.0, 0.0, 0.0]));
    assert!(points.contains(&[4.0, 3.0, 0.0]), "{points:?}");
    assert!(points.contains(&[12.0, 14.0, 0.0]), "{points:?}");
    assert_eq!(points.last().copied(), Some([14.0, 10.0, 0.0]));
    // No fit, flip or recentring: the extent is the path's own.
    let (min, max) = vmob.extent().unwrap();
    assert_eq!((min[0], min[1], max[0], max[1]), (0.0, 0.0, 14.0, 14.0));
    // Cubics reduce to quadratics within the converter's tolerance.
    let cubic = vmobject_from_svg_path("M 0 0 C 0 10 10 10 10 0").unwrap();
    assert_eq!(cubic.points().len() % 2, 1);
    assert_eq!(cubic.points().last().copied(), Some([10.0, 0.0, 0.0]));
    // Markup in the string cannot escape the attribute; garbage refuses.
    assert!(vmobject_from_svg_path("M 0 0 L 1 1\"/><script/>").is_err());
    assert!(vmobject_from_svg_path("M 0 0 L nope").is_err());
    assert!(vmobject_from_svg_path("").unwrap().points().is_empty());
}
