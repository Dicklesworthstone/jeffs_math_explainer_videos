//! Adding a Seat: why one more seat can cost a town a seat (Hamilton's rule
//! and the Alabama paradox), the divisor rules that never take a seat back
//! (Jefferson, Adams, Webster, Hill) and what they cost, the quota rule that
//! keeps both promises, the population paradox, and the theorem that no rule
//! keeps the quota promise and the population promise together. One example
//! throughout: Ash, Birch and Cedar, 4,000, 9,000 and 26,000 people.
//!
//! Candidate rebuild (bead .30): uncompiled, unrendered, no narration yet.
//!
//! ```text
//! apportionment facts                 # every number the video shows or says (key<TAB>value)
//! apportionment script                # the narration (generated from docs/storyboard.md)
//! apportionment render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! apportionment help                  # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: the tests check that script.tsv and the registry follow it.
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "apportionment",
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
