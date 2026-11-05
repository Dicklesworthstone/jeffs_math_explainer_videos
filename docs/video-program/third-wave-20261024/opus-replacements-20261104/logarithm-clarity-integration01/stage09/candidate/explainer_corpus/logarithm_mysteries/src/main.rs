//! Native logarithm film, rebuild candidate (mx-video-program-program-87do.63);
//! not an accepted audiovisual production: uncompiled here, unrendered,
//! narration unsynthesised and unheard. Mirrors the canonical contract;
//! adds the `model` module the chapters and facts compute from.
mod chapters;
mod facts;
mod model;
mod score;
use explainer_kit::Explainer;

const EXPLAINER: Explainer = Explainer {
    crate_name: "logarithm_mysteries",
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
