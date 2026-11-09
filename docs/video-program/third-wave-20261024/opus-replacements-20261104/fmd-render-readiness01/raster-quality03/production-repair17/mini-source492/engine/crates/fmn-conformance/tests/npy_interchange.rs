//! The `.npy` fixture-interchange loop, end to end (§16.3 plane 1, fm-xb3):
//! `scripts/gen_npy_fixtures.py` drives the pinned Reference's own
//! `utils/bezier.py` and saves its outputs with `np.save`; this test verifies
//! manifest integrity (sha256 via fmn-hash), decodes with the owned reader,
//! recomputes each case with fmn-geom, and compares at the doctrine's loose
//! cross-engine tolerance (§16.4).
//!
//! The re-encode check also locks byte-compatibility with numpy's writer:
//! `write_npy(read_npy(bytes)) == bytes`, so fixtures round-trip through
//! Python tooling without churn.

use fmn_conformance::npy::read_npy;
use fmn_conformance::npy::write_npy;
use fmn_conformance::oracles::FixtureCorpus;
use fmn_conformance::tolerance::{NanPolicy, check_points_abs};
use fmn_core::constants::TAU;
use fmn_core::types::Vec3;
use fmn_geom::bezier::{partial_quadratic, quadratic_points_for_arc};
use std::path::PathBuf;

/// Loose cross-engine tolerance: both sides compute these formulas in f64,
/// but op order differs; 1e-6 is far looser than the observed drift and far
/// tighter than any geometric significance.
const TOL: f64 = 1e-6;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/npy")
}

fn load_manifest() -> FixtureCorpus {
    FixtureCorpus::load(&fixture_dir())
        .expect("fixtures/npy/MANIFEST.tsv must satisfy the canonical bounded authority")
}

/// Read one fixture, verifying its manifest hash first.
fn load_points(name: &str, manifest: &FixtureCorpus) -> Vec<Vec3> {
    let bytes = manifest
        .bytes(name)
        .expect("named fixture must exist and match its manifest hash");
    let array = manifest
        .array(name)
        .expect("named fixture must match its manifest dtype and shape");
    // Byte-compatibility with np.save: our writer must reproduce the file.
    assert_eq!(
        write_npy(&array),
        bytes,
        "{name}: owned writer is not byte-compatible with np.save"
    );
    array
        .to_points()
        .expect("the point fixture census contains only Nx3 f64 arrays")
}

#[test]
fn arc_fixtures_match_fmn_geom() {
    let manifest = load_manifest();
    for (name, angle, n) in [
        ("arc_quarter_n4.npy", TAU / 4.0, 4usize),
        ("arc_full_n8.npy", TAU, 8),
        ("arc_neg_third_n2.npy", -TAU / 3.0, 2),
    ] {
        let reference = load_points(name, &manifest);
        let ours = quadratic_points_for_arc(angle, n).expect("the fixture arc is valid");
        let comparison = check_points_abs(&reference, &ours, TOL, NanPolicy::Reject);
        assert!(comparison.is_ok(), "{name}: {comparison:?}");
    }
}

#[test]
fn partial_quad_fixture_matches_fmn_geom() {
    let manifest = load_manifest();
    let reference = load_points("partial_quad.npy", &manifest);
    // The same asymmetric off-axis quadratic the generator hardcodes.
    let quad: [Vec3; 3] = [[-1.0, 0.5, 0.25], [0.75, 2.0, -0.5], [2.0, -1.0, 1.0]];
    let ours = partial_quadratic(&quad, 0.25, 0.75);
    let comparison = check_points_abs(&reference, &ours, TOL, NanPolicy::Reject);
    assert!(comparison.is_ok(), "partial_quad: {comparison:?}");
}

#[test]
fn every_manifest_row_has_its_file_and_hash() {
    let manifest = load_manifest();
    assert_eq!(
        manifest.len(),
        9,
        "expected the generator-defined fixture set"
    );
    for name in manifest.names() {
        let bytes = manifest
            .bytes(name)
            .expect("listed fixture must exist and match its manifest hash");
        let array = manifest
            .array(name)
            .expect("listed fixture must match its manifest dtype and shape");
        assert_eq!(
            write_npy(&array),
            bytes,
            "{name}: owned writer is not byte-compatible with np.save"
        );
    }
}

// ------------------------------------------- per-field snapshot export

/// §8.7's fixture-interchange hook (fm-879): any snapshot record column
/// exports as a `.npy` NumPy can read — here locked by round-tripping
/// through the owned writer/reader (whose byte-compatibility with numpy's
/// writer the re-encode test above already pins).
#[test]
fn snapshot_field_exports_as_npy() {
    use fmn_conformance::npy::{NpyArray, NpyData};
    use fmn_mobject::record::{RecordBuffer, RecordSchema};
    use fmn_mobject::{Mobject, Stage};

    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    let flat: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0, 2.0, 0.0, 2.0, 0.0, 0.0];
    entry.buffer.write_range("point", 0, &flat);

    let column = stage.get(mob).unwrap().buffer.read_column("point").unwrap();
    let rows = column.len() / 3;
    let array = NpyArray::new(vec![rows, 3], NpyData::F32(column.clone())).unwrap();
    let bytes = write_npy(&array);
    let back = read_npy(&bytes).unwrap();
    assert_eq!(back.as_f32().unwrap(), column.as_slice());
    assert_eq!(back.shape, vec![3, 3]);
}
