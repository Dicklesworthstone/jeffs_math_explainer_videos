//! Native FrankenManim production in progress. Scenes and narration remain
//! unreviewed, and opening/closing integration and final acceptance are pending.
mod chapters;
mod facts;
mod score;

use explainer_kit::Explainer;

const SCRIPT: &str = include_str!("../script.tsv");

const EXPLAINER: Explainer = Explainer {
    crate_name: "five_math_paradoxes",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::TEXT,
    glyph_tex: &[],
    glyph_mono: &[],
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: Some(chapters::thumbnail::make),
};

fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER,
        std::env::args().skip(1),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
}
