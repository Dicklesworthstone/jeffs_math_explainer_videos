//! fm-6l6: the boolean-op mobjects — structural fixtures, honest route
//! recording, degenerate point-set identities, and one bit-locked
//! self-golden per class over the certified Chisel kernel.
//!
//! These tests exercise the WRAPPER contract (family merge, operand-count
//! errors, fold counts, route recording, identities). The kernel's own
//! topology acceptance lives in `fmn-geom`'s boolean suite; nothing here
//! re-proves it.

use fmn_geom::boolean::BooleanRoute;
use fmn_hash::sha256;
use fmn_library::arc::Circle;
use fmn_library::boolean_ops::{
    BooleanBuild, BooleanMobjectError, difference, exclusion, intersection, union,
};
use fmn_library::poly::Square;
use fmn_library::vmobject::VMobject;

/// Overlapping pair: each circle reaches well past the other's centre.
fn left_circle() -> VMobject {
    circle_at(-0.75, 0.0, 1.5)
}

fn right_circle() -> VMobject {
    circle_at(0.75, 0.0, 1.5)
}

fn circle_at(x: f64, y: f64, radius: f64) -> VMobject {
    Circle::new().radius(radius).build().moved_to([x, y, 0.0])
}

fn square_at(x: f64, y: f64) -> VMobject {
    Square::new().build().moved_to([x, y, 0.0])
}

fn contour_count(build: &BooleanBuild) -> usize {
    build
        .mobject()
        .path()
        .expect("boolean result carries a path")
        .subpaths()
        .len()
}

/// The self-golden digest: SHA-256 over the result's points as little-end
/// f64 triples, in path order. Pure function of the inputs — no RNG, no
/// scheduling — so the lock is stable across runs and thread counts.
fn golden_digest(build: &BooleanBuild) -> String {
    let mut bytes = Vec::with_capacity(build.mobject().points().len() * 24);
    for point in build.mobject().points() {
        for component in point {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    sha256(&bytes).to_hex()
}

fn extent_x(build: &BooleanBuild) -> (f64, f64) {
    let (min, max) = build
        .mobject()
        .extent()
        .expect("boolean result has an extent");
    (min[0], max[0])
}

// ------------------------------------------------------- operand counting

#[test]
fn fewer_than_two_operands_is_the_reference_value_error() {
    let one = [left_circle()];
    assert!(matches!(
        union(&one),
        Err(BooleanMobjectError::NeedTwoOperands("Union"))
    ));
    assert!(matches!(
        intersection(&one),
        Err(BooleanMobjectError::NeedTwoOperands("Intersection"))
    ));
    assert!(matches!(
        exclusion(&one),
        Err(BooleanMobjectError::NeedTwoOperands("Exclusion"))
    ));
    let empty: [VMobject; 0] = [];
    assert!(union(&empty).is_err());
}

// ---------------------------------------------------------- union routing

#[test]
fn separated_union_routes_through_the_separated_proof() {
    let a = circle_at(-3.0, 0.0, 1.5);
    let b = circle_at(3.0, 0.0, 1.5);
    let build = union(&[a, b]).expect("separated union");
    assert_eq!(build.routes(), &[BooleanRoute::CurveAwareSeparated]);
    assert_eq!(
        contour_count(&build),
        2,
        "disjoint circles stay two contours"
    );
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-4.5)).abs() < 1e-9);
    assert!((max_x - 4.5).abs() < 1e-9);
}

#[test]
fn overlapping_union_routes_through_the_transversal_proof() {
    let build = union(&[left_circle(), right_circle()]).expect("overlapping union");
    assert_eq!(build.routes(), &[BooleanRoute::CurveAwareTransversal]);
    assert_eq!(contour_count(&build), 1, "overlap merges the outlines");
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-2.25)).abs() < 1e-9);
    assert!((max_x - 2.25).abs() < 1e-9);
}

#[test]
fn corner_touching_squares_route_through_the_certified_fallback() {
    // The two squares share exactly one anchor, the overlap class the
    // transversal proof refuses; the certified flatten-and-clip fallback
    // must take it, and the route must SAY so.
    let a = square_at(0.0, 0.0);
    let b = square_at(2.0, 2.0);
    let build = union(&[a, b]).expect("corner-touching union");
    assert_eq!(build.routes(), &[BooleanRoute::FlattenClip]);
    assert!(build.mobject().points().len() >= 8, "both squares survive");
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-1.0)).abs() < 1e-9);
    assert!((max_x - 3.0).abs() < 1e-9);
}

// ------------------------------------------------------- fold bookkeeping

#[test]
fn three_way_union_folds_one_kernel_call_per_additional_operand() {
    let top = circle_at(0.0, 1.5, 1.5);
    let build = union(&[left_circle(), right_circle(), top]).expect("three-way union");
    assert_eq!(build.routes().len(), 2);
    assert!(
        build
            .routes()
            .iter()
            .all(|route| *route == BooleanRoute::CurveAwareTransversal)
    );
}

#[test]
fn family_geometry_participates_through_children() {
    // A point-less parent carrying two children: the Reference merges
    // family_members_with_points, so both children's geometry joins the op.
    let parent =
        VMobject::new().with_children([circle_at(-0.75, 0.0, 1.5), circle_at(2.5, 0.0, 1.5)]);
    let clipper = circle_at(6.0, 0.0, 0.5);
    let build = union(&[parent, clipper]).expect("family union");
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-2.25)).abs() < 1e-9, "left child present");
    assert!(
        (max_x - 6.5).abs() < 1e-9,
        "right child and clipper present"
    );
}

// ----------------------------------------------- degenerate point sets

#[test]
fn difference_with_an_empty_clip_is_the_subject() {
    let subject = left_circle();
    let build = difference(&subject, &VMobject::new()).expect("empty clip");
    assert!(build.routes().is_empty(), "no kernel call for the identity");
    assert_eq!(contour_count(&build), 1);
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-2.25)).abs() < 1e-9);
    assert!((max_x - 0.75).abs() < 1e-9);
}

#[test]
fn difference_of_an_empty_subject_is_empty() {
    let build = difference(&VMobject::new(), &left_circle()).expect("empty subject");
    assert!(build.routes().is_empty());
    assert_eq!(build.mobject().points().len(), 0);
}

#[test]
fn intersection_with_an_empty_operand_annihilates_without_a_kernel_call() {
    let build = intersection(&[left_circle(), VMobject::new()]).expect("annihilator");
    assert!(build.routes().is_empty());
    assert_eq!(build.mobject().points().len(), 0);
}

#[test]
fn union_ignores_empty_operands() {
    let build = union(&[left_circle(), VMobject::new(), right_circle()]).expect("skip empties");
    assert_eq!(
        build.routes().len(),
        1,
        "one fold over the two real operands"
    );
    assert_eq!(contour_count(&build), 1);
}

// ------------------------------------------------- difference structure

#[test]
fn difference_carves_the_clip_out_of_the_subject() {
    // The square's left edge sits at x = 0.5, inside the subject circle
    // (which spans x in [-2.25, 0.75]), so the difference's right boundary
    // is that straight edge.
    let build = difference(&left_circle(), &square_at(1.5, 0.0)).expect("carve");
    assert_eq!(contour_count(&build), 1);
    let (min_x, max_x) = extent_x(&build);
    assert!((min_x - (-2.25)).abs() < 1e-9);
    assert!((max_x - 0.5).abs() < 1e-9, "carved at the clip's left edge");
}

#[test]
fn exclusion_of_overlapping_circles_leaves_two_lunes() {
    let build = exclusion(&[left_circle(), right_circle()]).expect("xor");
    assert_eq!(build.routes(), &[BooleanRoute::CurveAwareTransversal]);
    assert_eq!(contour_count(&build), 2, "the lens is removed");
}

// ----------------------------------------------------- self-goldens

#[test]
fn self_goldens_lock_each_class_s_canonical_output() {
    let cases: [(&str, String); 4] = [
        (
            "union",
            golden_digest(&union(&[left_circle(), right_circle()]).expect("union")),
        ),
        (
            "difference",
            golden_digest(&difference(&left_circle(), &square_at(1.5, 0.0)).expect("difference")),
        ),
        (
            "intersection",
            golden_digest(&intersection(&[left_circle(), right_circle()]).expect("intersection")),
        ),
        (
            "exclusion",
            golden_digest(&exclusion(&[left_circle(), right_circle()]).expect("exclusion")),
        ),
    ];
    // Deliberate-regeneration workflow: a hash drift is a diff to review,
    // never a silent update. Recompute with PRINT_BOOLEAN_GOLDENS=1.
    let expected: [(&str, &str); 4] = [
        (
            "union",
            "4670ccaae6294f3baf213aa98567843256fd96c6406a2dabf33c914de20f8d2e",
        ),
        (
            "difference",
            "82a361eb029726169cdfd142eca8d087b54a12fb2f60efabb9882caa4af31906",
        ),
        (
            "intersection",
            "cd8bb8b3d20c5b8bff4cd01051400bad7d513c6d24538ea84d2c3781a95b702f",
        ),
        (
            "exclusion",
            "ea6333924d9eaa1372fc40426db765ef6395b774b919b4f134be1f2e3819c639",
        ),
    ];
    if std::env::var("PRINT_BOOLEAN_GOLDENS").is_ok() {
        for (name, digest) in &cases {
            println!("BOOLEAN_GOLDEN {name} {digest}");
        }
    }
    for ((name, actual), (expected_name, expected_hash)) in cases.iter().zip(expected.iter()) {
        assert_eq!(expected_name, name);
        assert_eq!(
            actual, expected_hash,
            "self-golden drift for {name}: rerun with PRINT_BOOLEAN_GOLDENS=1 to review"
        );
    }
}
