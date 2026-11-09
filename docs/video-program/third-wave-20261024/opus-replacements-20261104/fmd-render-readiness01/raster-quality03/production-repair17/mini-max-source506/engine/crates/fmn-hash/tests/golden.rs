//! Golden-bytes lock for the canonical serialization format (fm-6h6, §6.7).
//!
//! A fixed document exercising every primitive at a frozen schema version is
//! serialized and compared, byte for byte, against a committed fixture. This is
//! the regression gate for the durable format: any change to the header layout,
//! field encoding, float canonicalization, or checksum will drift these bytes
//! and fail here — which is exactly the alarm the migration policy exists to
//! force (a real layout change must bump `major`, per AGENTS.md / D-17).
//!
//! To regenerate after a *deliberate, version-bumped* format change:
//! `REGEN=1 cargo test -p fmn-hash --test golden`, then commit the fixture.

use fmn_hash::{Reader, Schema, UnknownPolicy, Writer, sha256};
use std::path::PathBuf;

/// The frozen schema for the golden document: family `FMNG`, id 1, v1.0.
const GOLDEN_SCHEMA: Schema = Schema::new(*b"FMNG", 1, 1, 0);

/// Construct the canonical golden document. The field sequence *is* the schema;
/// it deliberately spans every put-method so the whole encoder is locked.
fn golden_doc() -> Vec<u8> {
    let mut w = Writer::new(GOLDEN_SCHEMA);
    w.put_u8(0xAB)
        .put_bool(true)
        .put_u16(0x1234)
        .put_u32(0x89AB_CDEF)
        .put_u64(0x0123_4567_89AB_CDEF)
        .put_i32(-1)
        .put_i64(-2)
        .put_f32(core::f32::consts::PI)
        .put_f64(core::f64::consts::E)
        .put_f64(-0.0) // canonicalized to +0.0 at the boundary
        .put_str("franken_manim")
        .put_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
        .put_digest(&sha256(b"anchor"));
    w.finish().expect("golden encodes")
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/golden_v1_0.hex")
}

#[test]
fn golden_bytes_are_frozen() {
    let doc = golden_doc();
    let hex = to_hex(&doc);
    let path = fixture_path();

    if std::env::var_os("REGEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, format!("{hex}\n")).unwrap();
        eprintln!("regenerated {}", path.display());
        return;
    }

    let expected = std::fs::read_to_string(&path).expect(
        "missing golden fixture crates/fmn-hash/fixtures/golden_v1_0.hex; bootstrap with REGEN=1",
    );
    assert_eq!(
        hex,
        expected.trim(),
        "canonical format drifted for schema FMNG v1.0 — a deliberate layout change must bump `major`"
    );
}

#[test]
fn golden_round_trips_field_for_field() {
    // The committed golden must also decode cleanly under strict policy, so the
    // fixture proves both directions, not just the write path.
    let doc = golden_doc();
    let mut r = Reader::open(
        &doc,
        GOLDEN_SCHEMA,
        fmn_hash::Limits::DEFAULT,
        UnknownPolicy::Strict,
    )
    .expect("golden opens");
    assert_eq!(r.get_u8().unwrap(), 0xAB);
    assert!(r.get_bool().unwrap());
    assert_eq!(r.get_u16().unwrap(), 0x1234);
    assert_eq!(r.get_u32().unwrap(), 0x89AB_CDEF);
    assert_eq!(r.get_u64().unwrap(), 0x0123_4567_89AB_CDEF);
    assert_eq!(r.get_i32().unwrap(), -1);
    assert_eq!(r.get_i64().unwrap(), -2);
    assert_eq!(r.get_f32().unwrap(), core::f32::consts::PI);
    assert_eq!(r.get_f64().unwrap(), core::f64::consts::E);
    assert_eq!(r.get_f64().unwrap().to_bits(), 0.0_f64.to_bits());
    assert_eq!(r.get_str().unwrap(), "franken_manim");
    assert_eq!(r.get_bytes().unwrap(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(r.get_digest().unwrap(), sha256(b"anchor"));
    r.finish().unwrap();
}

#[test]
fn valid_checksum_does_not_authorize_a_different_schema() {
    // The checksum proves that these bytes were not corrupted; it does not
    // grant permission to interpret a document under another schema.
    let doc = golden_doc();
    let wrong_schema = Schema::new(*b"FMNG", 2, 1, 0);
    assert!(matches!(
        Reader::open(
            &doc,
            wrong_schema,
            fmn_hash::Limits::DEFAULT,
            UnknownPolicy::Strict,
        ),
        Err(fmn_hash::SerialError::SchemaMismatch {
            expected: 2,
            found: 1
        })
    ));
}

#[test]
fn byte_fields_preserve_exact_input_identity() {
    fn encode(bytes: &[u8]) -> Vec<u8> {
        let mut writer = Writer::new(GOLDEN_SCHEMA);
        writer.put_bytes(bytes);
        writer.finish().expect("byte identity encoding")
    }

    let first = encode(&[0x00, 0xff, 0x00]);
    let same = encode(&[0x00, 0xff, 0x00]);
    let changed = encode(&[0x00, 0xff, 0x01]);

    assert_eq!(first, same);
    assert_ne!(first, changed);
    assert_ne!(sha256(&first), sha256(&changed));
}
