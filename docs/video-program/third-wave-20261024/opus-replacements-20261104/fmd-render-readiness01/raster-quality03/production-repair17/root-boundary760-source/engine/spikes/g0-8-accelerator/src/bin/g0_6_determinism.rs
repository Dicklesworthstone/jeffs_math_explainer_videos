//! The G0-6 runner: render the determinism frame, hash it, print the record.
//!
//! ```text
//! cargo run --release --bin g0_6_determinism -- [output-dir]
//! ```
//!
//! stdout is the record in `key<TAB>value` form and nothing else — no
//! timestamps, no timings, no host names — so two platforms' outputs diff
//! cleanly and a re-run on one platform is byte-identical to the last. Anything
//! a human wants to read goes to stderr.
//!
//! With an output directory it also writes the frame as a PNG, for the same
//! reason G0-8 does: a hash nobody has looked at is a hash nobody trusts.

use fmn_spike_accelerator::cpu::{self, Precision};
use fmn_spike_accelerator::determinism::{self, Record};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("G0-6 determinism spike (fm-zn9)");
    eprintln!(
        "frame {}x{} @ tile {}, alpha {}",
        determinism::WIDTH,
        determinism::HEIGHT,
        determinism::TILE,
        determinism::ALPHA
    );
    eprintln!("rustc target: {}", determinism::platform_tag());

    let record = Record::measure();
    print!("{}", record.to_tsv());

    if let Some(dir) = std::env::args().nth(1) {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir)?;
        let ir = determinism::frame_ir();
        for (name, precision) in [
            ("g0-6-frame.png", Precision::Reference),
            ("g0-6-frame-f32.png", Precision::AnnexF32),
        ] {
            let surface = cpu::render_at(&ir, precision);
            let bytes = fmn_codec::png::encode_rgba8(
                surface.width,
                surface.height,
                &surface.to_srgb8(),
                fmn_codec::deflate::CompressionLevel::Default,
            );
            std::fs::write(dir.join(name), bytes)?;
        }
        eprintln!("wrote PNGs to {}", dir.display());
    }
    Ok(())
}
