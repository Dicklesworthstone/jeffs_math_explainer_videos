//! __TITLE__ — rendered end to end by franken_manim's native Rust facade and
//! the corpus's shared kit (`explainer_kit` + `explainer_score`).
//!
//! ```text
//! __SLUG__ facts                         # every number the video shows or says (key<TAB>value)
//! __SLUG__ script                        # the narration (generated from docs/storyboard.md)
//! __SLUG__ render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! __SLUG__ help                          # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod palette;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "__SLUG__",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: None,
};

fn main() {
    std::process::exit(explainer_kit::cli::run(&EXPLAINER, std::env::args().skip(1)));
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::cli::check_script_owners;

    use super::*;

    fn keys() -> Vec<&'static str> {
        chapters::registry().iter().map(|c| c.key).collect()
    }

    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "four points", "pick", "4"),
        ("01_1", "five thousand", "points", "5000"),
        ("01_2", "twenty-six trillion", "quadruples_trillions_floor", "26"),
    ];

    #[test]
    fn script_lines_match_chapter_registry_and_storyboard() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).expect("storyboard and script agree");
        let script = Script::parse(SCRIPT).expect("script.tsv parses");
        check_script_owners(&script, &keys(), EXPLAINER.line_owner).expect("every line has a chapter");
    }

    #[test]
    fn spoken_numbers_match_pinned_facts() {
        let script = Script::parse(SCRIPT).expect("script.tsv parses");
        let f = facts::facts().expect("facts self_check");
        let line = |id: &str| {
            script
                .lines()
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, text)| text.as_str())
                .unwrap_or_else(|| panic!("missing line {id}"))
        };
        for &(line_id, phrase, fact_key, expected) in SPOKEN {
            let text = line(line_id);
            assert!(
                text.contains(phrase),
                "line {line_id} no longer says {phrase:?}: {text:?}"
            );
            assert_eq!(
                f.get(fact_key),
                Some(expected),
                "fact {fact_key} changed while line {line_id} still says {phrase:?}"
            );
        }
    }
}
