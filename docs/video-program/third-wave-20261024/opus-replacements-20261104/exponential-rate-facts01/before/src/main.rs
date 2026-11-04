//! Native exponential video draft; not an accepted audiovisual production.
mod chapters;
mod facts;
mod score;

use explainer_kit::Explainer;

const EXPLAINER: Explainer = Explainer {
    crate_name: "exponential_magic",
    script: include_str!("../script.tsv"),
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
