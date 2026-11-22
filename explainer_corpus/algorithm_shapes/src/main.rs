//! Native delivery uses the shared narration, score, caption and master contracts.
mod score;

use algorithm_shapes_explainer::{SCRIPT, chapters, facts};
use explainer_kit::Explainer;

const EXPLAINER: Explainer = Explainer {
    crate_name: "algorithm_shapes",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    // Chapter glyphs are captured by the shared live scene probe.
    glyph_text: &[],
    glyph_tex: &[],
    glyph_mono: &[],
    identity: score::identity,
    // Fixed placement headroom prevents 24-to-48 kHz resampler overshoot.
    // Original Frederick recordings remain unchanged.
    narration_gain_db: -3.0,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: None,
};

fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER,
        std::env::args().skip(1),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
}
