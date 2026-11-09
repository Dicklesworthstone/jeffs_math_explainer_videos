#![forbid(unsafe_code)]

use fmn_geom::svg::{SvgDocument, SvgError};

const FUZZ_REPRODUCER: &[u8] = b"<!/[\x05\x00<!\xC2\xA4</</=[\x05";

#[test]
fn malformed_markup_probe_never_slices_inside_utf8() {
    let error = SvgDocument::parse(FUZZ_REPRODUCER)
        .expect_err("the malformed declaration must be refused without panicking");
    assert!(matches!(error, SvgError::Malformed { line: 1, .. }));
}

#[test]
fn doctype_probe_remains_ascii_case_insensitive() {
    for source in [
        b"<!DOCTYPE svg><svg/>".as_slice(),
        b"<!doctype svg><svg/>".as_slice(),
        b"<!DoCtYpE svg><svg/>".as_slice(),
    ] {
        assert_eq!(
            SvgDocument::parse(source),
            Err(SvgError::DoctypeRefused { line: 1 })
        );
    }
}
