//! The one convention for tests whose inputs may be absent (fm-5wq.48).
//!
//! Some inputs are private or gitignored: the pinned Reference checkout
//! (`scripts/manim_ref`), the 3b1b video corpus (`scripts/videos_ref`), the TeX
//! corpus, the Look Gallery's Reference captures, a built wasm artifact. A test
//! that needs one calls [`skip_or_fail`] and returns when the input is missing,
//! so a green run states what it did not check instead of passing silently.
//! It lives here because every crate with such a test depends on fmn-core.

/// Record that `test` could not run because `input` is absent.
///
/// Under `FMN_REQUIRE_FULL_INPUTS=1` (an owned-host or land gate) this panics,
/// naming the input. Otherwise it writes `SKIPPED <test> <input>` to stderr and
/// appends the same line to the file named by `FMN_SKIP_LEDGER` when that is
/// set; `scripts/check.sh` prints that ledger at the end of the gate.
///
/// # Panics
/// When `FMN_REQUIRE_FULL_INPUTS=1`.
pub fn skip_or_fail(test: &str, input: &str) {
    if std::env::var("FMN_REQUIRE_FULL_INPUTS").as_deref() == Ok("1") {
        // ubs:ignore — deliberate: under the strict flag a missing input fails the calling test.
        panic!("{test}: required input missing: {input} (FMN_REQUIRE_FULL_INPUTS=1)");
    }
    let line = format!("SKIPPED {test} {input}\n");
    eprint!("{line}");
    if let Ok(path) = std::env::var("FMN_SKIP_LEDGER") {
        use std::io::Write as _;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}
