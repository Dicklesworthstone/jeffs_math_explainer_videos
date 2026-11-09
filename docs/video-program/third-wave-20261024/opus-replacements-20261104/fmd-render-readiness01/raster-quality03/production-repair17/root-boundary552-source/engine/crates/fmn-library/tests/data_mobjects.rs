//! fm-n64 tranche 2: the Data mobjects — CSV→TableMobject end-to-end
//! through the suite's frame parser, BarChart structural fixtures against
//! the Reference's probability.py conventions, numpy-compatible rounding,
//! and bit-locked self-goldens.

use fmn_core::constants::{BLUE, YELLOW};
use fmn_hash::sha256;
use fmn_text::FontBook;

use fmn_library::data_mobjects::{BarChart, TableMobject, format_scalar, numpy_round2};

fn fail(message: String) -> ! {
    std::panic::panic_any(message)
}

/// Original test CSV (never Reference bytes): column order deliberately
/// unsorted to prove insertion order survives, one null cell, mixed types.
const SAMPLE_CSV: &str = "region,score,active\nwest,3.5,true\neast,,false\nnorth,7,true\n";
fn book() -> FontBook {
    // The bundled default face set; Scribe resolves text against it.
    FontBook::bundled().unwrap_or_else(|error| fail(format!("bundled fonts: {error}")))
}

#[test]
fn csv_column_order_and_cells_survive_the_frame_round_trip() {
    let table =
        TableMobject::from_csv(SAMPLE_CSV, ',').unwrap_or_else(|error| fail(format!("{error}")));
    assert_eq!(table.headers(), ["region", "score", "active"]);
    assert_eq!(table.rows().len(), 3);
    assert_eq!(table.rows()[0], ["west", "3.5", "true"]);
    // The null cell renders empty per the documented rule.
    assert_eq!(table.rows()[1][1], "");
    assert_eq!(table.rows()[1][2], "false");
    assert_eq!(table.rows()[2], ["north", "7", "true"]);
}

#[test]
fn scalar_formatting_follows_the_documented_rules() {
    use fp_types::Scalar;
    assert_eq!(format_scalar(None), "");
    assert_eq!(
        format_scalar(Some(&Scalar::Null(fp_types::NullKind::NaT))),
        ""
    );
    assert_eq!(format_scalar(Some(&Scalar::Bool(true))), "true");
    assert_eq!(format_scalar(Some(&Scalar::Int64(-42))), "-42");
    assert_eq!(format_scalar(Some(&Scalar::Float64(3.5))), "3.5");
    assert_eq!(format_scalar(Some(&Scalar::Float64(-0.0))), "0");
    assert_eq!(format_scalar(Some(&Scalar::Utf8("hello".into()))), "hello");
    assert_eq!(format_scalar(Some(&Scalar::Timedelta64(90_000))), "90000ns");
}

#[test]
fn table_builds_a_ruled_grid_through_scribe() {
    let table =
        TableMobject::from_csv(SAMPLE_CSV, ',').unwrap_or_else(|error| fail(format!("{error}")));
    let family = table
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));
    // Rules: outline + header rule + 2 row separators + 2 column rules = 6;
    // cells: 3 headers + 9 body = 12. Children: [rules…, cells…].
    assert_eq!(family.children().len(), 6 + 12);
}

#[test]
fn bar_chart_matches_the_reference_geometry() {
    let chart = BarChart::new(vec![0.25, 0.5, 1.0])
        .bar_names(vec!["a".into(), "b".into(), "c".into()])
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));
    // x-axis + y-axis + 5 y-ticks + 4 labels + bars-group + 3 names.
    assert_eq!(chart.children().len(), 2 + 5 + 4 + 1 + 3);

    // Bars: width buff = width/(2n) = 1, heights v/max*height, bottoms on
    // a common baseline (the whole chart is centred afterwards, like the
    // Reference's trailing self.center()), left corners pitched by buff.
    let bars = &chart.children()[11];
    assert_eq!(bars.children().len(), 3);
    let bottoms: Vec<f64> = bars
        .children()
        .iter()
        .map(|bar| bar.bbox_point(fmn_core::constants::DOWN).expect("bbox")[1])
        .collect();
    assert!(
        bottoms.iter().all(|y| (*y - bottoms[0]).abs() < 1e-9),
        "bars share one baseline: {bottoms:?}"
    );
    let lefts: Vec<f64> = bars
        .children()
        .iter()
        .map(|bar| bar.bbox_point(fmn_core::constants::LEFT).expect("bbox")[0])
        .collect();
    for i in 0..3 {
        let right = bars.children()[i]
            .bbox_point(fmn_core::constants::RIGHT)
            .expect("bbox");
        let left = bars.children()[i]
            .bbox_point(fmn_core::constants::LEFT)
            .expect("bbox");
        assert!((right[0] - left[0] - 1.0).abs() < 1e-9, "bar {i} width");
        if i > 0 {
            assert!((lefts[i] - lefts[i - 1] - 1.0).abs() < 1e-9, "pitch {i}");
        }
        let expected_height = [0.25, 0.5, 1.0][i] * 4.0;
        let up = bars.children()[i]
            .bbox_point(fmn_core::constants::UP)
            .expect("bbox");
        assert!(
            (up[1] - bottoms[i] - expected_height).abs() < 1e-9,
            "bar {i} height"
        );
    }
    // Gradient endpoints across the three bars: BLUE … YELLOW.
    let first_fill = bars.children()[0].style().fill_color;
    let last_fill = bars.children()[2].style().fill_color;
    assert_eq!(first_fill, BLUE);
    assert_eq!(last_fill, YELLOW);
}

#[test]
fn derived_max_scales_bars_to_the_chart_height() {
    let chart = BarChart::new(vec![10.0, 20.0])
        .max_value(None)
        .n_ticks(2)
        .label_y_axis(false)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));
    // No y-labels with label_y_axis(false); ticks = 3 marks.
    let bars = &chart.children()[2 + 3];
    let tallest = &bars.children()[1];
    let up = tallest.bbox_point(fmn_core::constants::UP).expect("bbox");
    let down = tallest.bbox_point(fmn_core::constants::DOWN).expect("bbox");
    assert!((up[1] - down[1] - 4.0).abs() < 1e-9, "tallest fills height");
}

#[test]
fn numpy_round2_is_half_to_even() {
    assert!((numpy_round2(0.125) - 0.12).abs() < 1e-9, "ties go even");
    assert!((numpy_round2(0.135) - 0.14).abs() < 1e-9, "ties go even");
    assert!((numpy_round2(0.124) - 0.12).abs() < 1e-9);
    assert!((numpy_round2(1.005) - 1.0).abs() < 1e-9 || (numpy_round2(1.005) - 1.01).abs() < 1e-9);
}

#[test]
fn self_goldens_lock_the_canonical_table_and_chart() {
    let table =
        TableMobject::from_csv(SAMPLE_CSV, ',').unwrap_or_else(|error| fail(format!("{error}")));
    let table_family = table
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let chart = BarChart::new(vec![0.5, 1.0])
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let digest_of = |root: &fmn_library::vmobject::VMobject| {
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

    const TABLE_GOLDEN: &str = "d2a42ffd84ae8b6f45e5aec370d66d5c8cd87fe760b41830c32c66ed8a82ec6a";
    const CHART_GOLDEN: &str = "1546065087e3328dac330f5a40dc66208d159489c9ed503b0d4784add0b3266e";
    assert_eq!(digest_of(&table_family), TABLE_GOLDEN, "table drifted");
    assert_eq!(digest_of(&chart), CHART_GOLDEN, "chart drifted");
}
