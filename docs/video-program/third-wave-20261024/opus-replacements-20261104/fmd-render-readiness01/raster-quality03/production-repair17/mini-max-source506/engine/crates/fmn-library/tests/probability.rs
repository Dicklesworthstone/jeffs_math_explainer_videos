//! fm-n64 tranche 3: `SampleSpace` — p-list completion, division stacking
//! order, brace-and-label seating, and a bit-locked self-golden.

use fmn_core::constants::{DOWN, LEFT, RIGHT, UP};
use fmn_hash::sha256;
use fmn_text::FontBook;

use fmn_library::probability::{SampleSpace, vertical_default_colors};
use fmn_library::vmobject::VMobject;

fn fail(message: String) -> ! {
    std::panic::panic_any(message)
}

fn book() -> FontBook {
    FontBook::bundled().unwrap_or_else(|error| fail(format!("bundled fonts: {error}")))
}

#[test]
fn complete_p_list_pads_with_the_remainder() {
    let space = SampleSpace::new();
    assert_eq!(
        space
            .complete_p_list(&[0.3, 0.2])
            .unwrap_or_else(|error| fail(format!("{error}"))),
        vec![0.3, 0.2, 0.5]
    );
    // An exact list stays exact — no spurious zero band.
    assert_eq!(
        space
            .complete_p_list(&[0.25, 0.75])
            .unwrap_or_else(|error| fail(format!("{error}"))),
        vec![0.25, 0.75]
    );
    let error = space.complete_p_list(&[1.5]).unwrap_err();
    assert!(format!("{error}").contains("outside [0, 1]"), "{error}");
}

#[test]
fn horizontal_bands_stack_from_the_top_edge_downward() {
    let space = SampleSpace::new();
    let bands = &space
        .horizontal_division(
            &[0.25, 0.75],
            &fmn_library::probability::horizontal_default_colors(),
        )
        .unwrap_or_else(|error| fail(format!("{error}")));
    assert_eq!(bands.children().len(), 2);
    let tops: Vec<f64> = bands
        .children()
        .iter()
        .map(|band| band.bbox_point(UP).expect("bbox")[1])
        .collect();
    let bottoms: Vec<f64> = bands
        .children()
        .iter()
        .map(|band| band.bbox_point(DOWN).expect("bbox")[1])
        .collect();
    // First band owns the top quarter; second runs to the bottom.
    assert!((tops[0] - 1.5).abs() < 1e-9, "first band at the top edge");
    assert!(
        (bottoms[0] - tops[1]).abs() < 1e-9,
        "bands share boundaries"
    );
    assert!(
        (bottoms[1] + 1.5).abs() < 1e-9,
        "last band reaches the floor"
    );
    // Heights follow the probabilities.
    assert!((tops[0] - bottoms[0] - 0.25 * 3.0).abs() < 1e-9);
    assert!((tops[1] - bottoms[1] - 0.75 * 3.0).abs() < 1e-9);
}

#[test]
fn vertical_bands_stack_from_the_left_edge_rightward() {
    let space = SampleSpace::new();
    let bands = &space
        .vertical_division(&[0.6], &vertical_default_colors())
        .unwrap_or_else(|error| fail(format!("{error}")));
    assert_eq!(
        bands.children().len(),
        2,
        "remainder band completes the set"
    );
    let lefts: Vec<f64> = bands
        .children()
        .iter()
        .map(|band| band.bbox_point(LEFT).expect("bbox")[0])
        .collect();
    let rights: Vec<f64> = bands
        .children()
        .iter()
        .map(|band| band.bbox_point(RIGHT).expect("bbox")[0])
        .collect();
    assert!((lefts[0] + 1.5).abs() < 1e-9, "first band at the left edge");
    assert!((rights[0] - lefts[1]).abs() < 1e-9);
    assert!((rights[1] - 1.5).abs() < 1e-9, "last band reaches the wall");
}

#[test]
fn braces_and_labels_seat_one_per_band() {
    let space = SampleSpace::new();
    let parts = space
        .vertical_division(&[0.4], &vertical_default_colors())
        .unwrap_or_else(|error| fail(format!("{error}")));
    let decorated = space
        .subdivision_braces_and_labels(
            &parts,
            &["P(A)", "P(B)"],
            DOWN,
            fmn_library::probability::TITLE_BUFF,
            &book(),
        )
        .unwrap_or_else(|error| fail(format!("{error}")));
    // [braces…, labels…] with one per band.
    assert_eq!(decorated.children().len(), 4);
}

#[test]
fn title_shrinks_to_the_space_width_when_wider() {
    let narrow = SampleSpace::new().width(0.5);
    let title = narrow
        .title(
            &book(),
            "Sample space",
            fmn_library::probability::TITLE_BUFF,
        )
        .unwrap_or_else(|error| fail(format!("{error}")));
    let right = title.bbox_point(RIGHT).expect("bbox");
    let left = title.bbox_point(LEFT).expect("bbox");
    assert!(
        right[0] - left[0] <= 0.5 + 1e-9,
        "title must not exceed the space width"
    );
    // And it sits above the space.
    let down = title.bbox_point(DOWN).expect("bbox");
    assert!(down[1] > 1.5, "title above the top edge, got {}", down[1]);
}

#[test]
fn self_golden_locks_the_canonical_divided_space() {
    let space = SampleSpace::new();
    let base = space
        .build()
        .unwrap_or_else(|error| fail(format!("{error}")));
    let parts = space
        .horizontal_division(
            &[0.3],
            &fmn_library::probability::horizontal_default_colors(),
        )
        .unwrap_or_else(|error| fail(format!("{error}")));
    let labels = space
        .subdivision_braces_and_labels(
            &parts,
            &["p", "1-p"],
            LEFT,
            fmn_library::probability::TITLE_BUFF,
            &book(),
        )
        .unwrap_or_else(|error| fail(format!("{error}")));

    let digest_of = |root: &VMobject| {
        let mut bytes = Vec::new();
        let mut stack = vec![root];
        while let Some(current) = stack.pop() {
            for point in current.points() {
                bytes.extend_from_slice(&point[0].to_bits().to_le_bytes());
                bytes.extend_from_slice(&point[1].to_bits().to_le_bytes());
                bytes.extend_from_slice(&point[2].to_bits().to_le_bytes());
            }
            for child in current.children() {
                stack.push(child);
            }
        }
        sha256(&bytes).to_hex()
    };

    const BASE_GOLDEN: &str = "b3df7f2dd1a21dc91464a55d92d3a4e4d9f087a0d49705412728aab4836b997a";
    const PARTS_GOLDEN: &str = "d2cfdcb45dba0739b88ed304294509058405c5622b8e35fca55b447dd9ebe3fc";
    // Re-locked when Brace depth moved to the Reference's 0.2668 (fm-5wq.53).
    const LABELS_GOLDEN: &str = "11c0ac834c33efdb477117a0aeca4834082678eb9a987c9c21bdef1ff96fa2e5";
    if BASE_GOLDEN.starts_with("PLACEHOLDER") {
        fail(format!(
            "SELF GOLDEN SEEDS sample_space: base={} parts={} labels={}",
            digest_of(&base),
            digest_of(&parts),
            digest_of(&labels)
        ));
    }
    assert_eq!(digest_of(&base), BASE_GOLDEN);
    assert_eq!(digest_of(&parts), PARTS_GOLDEN);
    assert_eq!(digest_of(&labels), LABELS_GOLDEN);
}
