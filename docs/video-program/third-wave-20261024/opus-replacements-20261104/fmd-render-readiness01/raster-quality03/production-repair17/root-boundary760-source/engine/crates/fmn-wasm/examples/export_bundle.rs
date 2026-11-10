//! Exports the tier-2 demo timeline bundle to `demo/wasm/bundle.fmtl`
//! (fm-oee): the bytes `demo/wasm/player.html` scrubs in the browser.
//!
//! ```text
//! cargo run -p fmn-wasm --example export_bundle
//! ```
//!
//! The bundle is deterministic: rerunning this after an engine change that
//! leaves the scene untouched rewrites identical bytes.

use std::path::Path;

fn main() {
    let bytes = fmn_wasm::demo_bundle::demo_bundle().expect("demo bundle exports");
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../demo/wasm/bundle.fmtl");
    std::fs::write(&out, &bytes).expect("bundle.fmtl writes");
    let digest = fmn_hash::sha256(&bytes);
    println!(
        "wrote {} ({} bytes, sha256 {digest})",
        out.display(),
        bytes.len()
    );
}
