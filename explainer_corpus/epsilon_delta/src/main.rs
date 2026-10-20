//! Epsilon and Delta: Why Getting Closer Wasn't Enough. Unbroken curves x^n
//! on [0, 1] whose limit jumps; a fixed point whose gap shrinks, and a
//! moving point whose gap stays one half; then the guarantee epsilon and
//! delta make precise, two constructive notes on choosing delta, and the
//! neighbourhood picture behind topology. Rendered end to end by
//! franken_manim's native Rust front door and the corpus's shared kit.
//!
//! ```text
//! epsdelta facts                      # every number the video shows or says (key<TAB>value)
//! epsdelta script                     # the narration (generated from docs/storyboard.md)
//! epsdelta render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! epsdelta help                       # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat. Only
/// the tests read it; the binary reads `script.tsv`.
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "epsilon_delta",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Thought,
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

    fn all_text() -> String {
        let script = Script::parse(SCRIPT).unwrap();
        script
            .lines()
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed number, word or fact
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "from zero", "C1.domain.left", "0"),
        ("01_1", "to one.", "C1.domain.right", "1"),
        ("01_1", "Square every height", "C1.squared", "2"),
        ("01_2", "from zero", "C1.domain.left", "0"),
        ("01_2", "up to one.", "C1.domain.right", "1"),
        ("01_3", "x equals nine tenths", "C1.fixed.x", "0.9"),
        ("01_3", "point nine,", "C1.fixed.n1", "0.9"),
        ("01_3", "point eight one,", "C1.fixed.n2", "0.81"),
        ("01_3", "point seven two nine,", "C1.fixed.n3", "0.729"),
        ("01_3", "toward zero.", "C1.limit.below_one", "0"),
        ("01_4", "At x equals one,", "C1.domain.right", "1"),
        ("01_4", "One to any power", "C1.domain.right", "1"),
        ("01_4", "power is one.", "C1.endpoint.height", "1"),
        ("01_6", "Flat at zero", "C1.limit.below_one", "0"),
        ("01_6", "all the way to one,", "C1.domain.right", "1"),
        ("01_6", "at height one.", "C1.limit.at_one", "1"),
        ("01_7", "Below one,", "C1.domain.right", "1"),
        ("01_7", "less than one.", "C1.domain.right", "1"),
        ("01_9", "half height", "C1.half", "1/2"),
        ("01_10", "at one half and", "C1.half", "1/2"),
        ("01_10", "the limit is zero.", "C1.limit.below_one", "0"),
        ("01_10", "toward one,", "C1.domain.right", "1"),
        ("01_10", "gap stays one half.", "C1.witness.gap", "1/2"),
        ("01_11", "shrinks to zero.", "C1.limit.below_one", "0"),
        ("02_1", "above one half.", "C1.half", "1/2"),
        ("02_1", "up to one,", "C1.domain.right", "1"),
        ("02_1", "x equals one belongs", "C1.domain.right", "1"),
        ("02_2", "at one,", "C1.domain.right", "1"),
        ("02_2", "at height one.", "C1.limit.at_one", "1"),
        ("02_2", "The gap there is zero.", "C2.gap_at_one", "0"),
        ("02_3", "at least one half", "C1.half", "1/2"),
        ("02_3", "short of one.", "C1.domain.right", "1"),
        ("02_6", "ten times,", "C2.zoom.0", "10"),
        ("02_6", "a hundred times,", "C2.zoom.1", "100"),
        ("02_6", "a thousand times.", "C2.zoom.2", "1000"),
        ("02_7", "x to the tenth.", "C2.restart_power", "10"),
        ("02_7", "power tenfold", "C2.zoom.step", "10"),
        ("02_7", "stretch tenfold", "C2.zoom.step", "10"),
        ("02_7", "point six nine", "C2.ln2.2dp", "0.69"),
        ("02_8", "gap of one half.", "C1.witness.gap", "1/2"),
        ("03_1", "seventeen thirty four", "C3.year", "1734"),
        ("03_2", "for a square", "C3.power", "2"),
        ("03_2", "from one half", "C3.base", "1/2"),
        ("03_2", "by h, not zero.", "C3.h.to", "0"),
        ("03_2", "h plus h squared", "C3.change", "h + h^2"),
        ("03_2", "slope is one plus h.", "C3.quotient", "1 + h"),
        ("03_3", "so one plus h", "C3.quotient", "1 + h"),
        ("03_3", "becomes one.", "C3.limit", "1"),
        ("03_4", "h equal to zero", "C3.h.to", "0"),
        ("03_5", "zero over zero", "C3.at_zero", "0/0"),
        ("03_6", "One is still", "C3.limit", "1"),
        ("03_6", "ever being zero.", "C3.h.to", "0"),
        ("03_6", "close in on one.", "C3.limit", "1"),
        ("03_7", "h within one tenth", "C3.delta.0", "1/10"),
        ("03_7", "of zero,", "C3.h.to", "0"),
        ("03_7", "other than zero,", "C3.h.to", "0"),
        ("03_7", "slope is within one tenth", "C3.eps.0", "1/10"),
        ("03_7", "tenth of one.", "C3.limit", "1"),
        ("03_7", "limit is one.", "C3.limit", "1"),
        ("04_1", "but zero.", "C3.h.to", "0"),
        ("04_2", "Against one tenth", "C4.eps", "1/10"),
        ("04_2", "answers one tenth", "C4.delta", "1/10"),
        ("04_2", "far from one", "C3.limit", "1"),
        ("04_2", "is from zero.", "C3.h.to", "0"),
        ("04_3", "one fifth", "C4.wide", "1/5"),
        ("04_3", "point one five", "C4.catch", "0.15"),
        ("04_4", "leaves zero out", "C3.h.to", "0"),
        ("04_4", "puts zero in", "C3.h.to", "0"),
        ("04_4", "hole with one,", "C4.fill.good", "1"),
        ("04_4", "one and a half", "C4.fill.bad", "1.5"),
        ("04_7", "x equals one,", "C4.end.x", "1"),
        ("04_7", "the square's domain", "C4.end.power", "2"),
        ("04_7", "Against one half", "C4.end.eps", "1/2"),
        ("04_7", "x squared is one half", "C4.end.level", "1/2"),
        ("04_7", "to one passes", "C4.end.x", "1"),
        ("05_1", "two thirds of epsilon", "C5.ratio", "2/3"),
        ("05_2", "x squared", "C3.power", "2"),
        ("05_2", "of a quarter", "C5.value", "1/4"),
        ("05_2", "near one half", "C5.a", "1/2"),
        ("05_3", "from one half", "C5.a", "1/2"),
        ("05_3", "x plus one half", "C5.a", "1/2"),
        ("05_4", "zero to one", "C1.domain.left", "0"),
        ("05_4", "to one,", "C1.domain.right", "1"),
        ("05_4", "x plus one half", "C5.a", "1/2"),
        ("05_4", "three halves", "C5.bound", "3/2"),
        ("05_5", "two thirds of epsilon", "C5.ratio", "2/3"),
        ("05_6", "Against one tenth", "C5.eps", "1/10"),
        ("05_7", "point zero nine two", "C5.widest.3dp", "0.092"),
        ("05_7", "One fifteenth", "C5.delta", "1/15"),
        ("05_8", "two thirds of epsilon", "C5.ratio", "2/3"),
        ("05_8", "three halves of delta", "C5.bound", "3/2"),
        ("06_1", "x squared", "C3.power", "2"),
        ("06_1", "at two,", "C6.a", "2"),
        ("06_1", "it is four", "C6.value", "4"),
        ("06_2", "x minus two", "C6.a", "2"),
        ("06_2", "x squared", "C3.power", "2"),
        ("06_2", "minus four", "C6.value", "4"),
        ("06_2", "h squared", "C3.power", "2"),
        ("06_2", "plus four h", "C6.slope", "4"),
        ("06_3", "h squared", "C3.power", "2"),
        ("06_3", "four times", "C6.slope", "4"),
        ("06_4", "Near two", "C6.a", "2"),
        ("06_4", "slope four", "C6.slope", "4"),
        ("06_4", "over four", "C6.slope", "4"),
        ("06_4", "one fortieth", "C6.naive.delta", "1/40"),
        ("06_4", "one tenth", "S.eps", "1/10"),
        ("06_5", "point zero two four nine", "C6.naive.h", "0.0249"),
        ("06_5", "x squared", "C3.power", "2"),
        ("06_5", "point one zero zero two two", "C6.naive.change.5dp", "0.10022"),
        ("06_6", "Adding four", "C6.added", "4"),
        ("06_6", "plus two,", "C6.half_slope", "2"),
        ("06_6", "squared, below", "C3.power", "2"),
        ("06_6", "below four", "C6.added", "4"),
        ("06_7", "root of four", "C6.added", "4"),
        ("06_7", "minus two", "C6.half_slope", "2"),
        ("06_7", "delta of zero", "C6.h.at_a", "0"),
        ("06_8", "Against one tenth", "S.eps", "1/10"),
        ("06_8", "point zero two four eight", "C6.delta.4dp", "0.0248"),
        ("06_8", "within one", "C6.cap", "1"),
        ("06_8", "x plus two", "C6.a", "2"),
        ("06_8", "below five", "C6.cap.factor", "5"),
        ("06_8", "smaller of one", "C6.cap", "1"),
        ("06_8", "over five", "C6.cap.factor", "5"),
        ("06_8", "point zero two.", "S.delta_simple.decimal", "0.02"),
        ("07_1", "x cubed", "C7.degree", "3"),
        ("07_1", "five x", "C7.quadratic", "5"),
        ("07_1", "x squared", "C7.power2", "2"),
        ("07_1", "near one", "C7.a", "1"),
        ("07_1", "it is six", "C7.value", "6"),
        ("07_1", "minus one", "C7.a", "1"),
        ("07_2", "h cubed", "C7.degree", "3"),
        ("07_2", "eight h", "C7.square", "8"),
        ("07_2", "h squared", "C7.power2", "2"),
        ("07_2", "thirteen h", "C7.linear", "13"),
        ("07_2", "three sizes", "C7.degree", "3"),
        ("07_3", "below one", "C6.cap", "1"),
        ("07_4", "three terms", "C7.degree", "3"),
        ("07_5", "Twenty two.", "D1.bound", "22"),
        ("07_5", "One,", "C7.cube", "1"),
        ("07_5", "eight and", "C7.square", "8"),
        ("07_5", "thirteen add", "C7.linear", "13"),
        ("07_5", "add to twenty two", "D1.bound", "22"),
        ("07_5", "smaller of one", "C6.cap", "1"),
        ("07_5", "over twenty two", "D1.bound", "22"),
        ("07_8", "smaller of one", "C6.cap", "1"),
        ("07_8", "If zero", "C1.limit.below_one", "0"),
        ("08_1", "on zero", "C1.domain.left", "0"),
        ("08_1", "to one.", "C1.domain.right", "1"),
        ("08_3", "one half", "C1.wait.tolerance", "1/2"),
        ("08_3", "nine tenths", "C1.fixed.x", "0.9"),
        ("08_3", "stage seven", "C1.wait.x0.9", "7"),
        ("08_3", "Point nine nine", "C8.second_x", "0.99"),
        ("08_3", "sixty nine", "C1.wait.x0.99", "69"),
        ("08_6", "half height", "C1.half", "1/2"),
        ("08_6", "Below one", "C1.domain.right", "1"),
        ("08_6", "is zero", "C1.limit.below_one", "0"),
        ("08_6", "one half.", "C1.witness.gap", "1/2"),
        ("09_2", "three gaps", "C9.pieces", "3"),
        ("09_3", "a third", "C9.share", "1/3"),
        ("09_4", "a third", "C9.share", "1/3"),
        ("09_6", "a third", "C9.share", "1/3"),
        ("10_2", "one minus one over n", "C1.domain.right", "1"),
        ("10_2", "approach one", "C1.domain.right", "1"),
        ("10_3", "is zero", "C1.limit.below_one", "0"),
        ("10_3", "is one", "C1.domain.right", "1"),
        ("10_6", "one over n", "C1.domain.right", "1"),
        ("10_7", "at zero", "C1.domain.left", "0"),
        ("10_7", "epsilon squared", "C3.power", "2"),
        ("10_8", "one over n squared", "C3.power", "2"),
        ("10_8", "output is one", "C1.domain.right", "1"),
        ("10_9", "near zero", "C1.domain.left", "0"),
        ("11_2", "x squared", "C3.power", "2"),
        ("11_2", "between one", "C11.lower", "1"),
        ("11_2", "and nine", "C11.upper", "9"),
        ("11_4", "Two open", "C11.parts", "2"),
        ("11_7", "includes zero", "C1.domain.left", "0"),
        ("12_1", "q squared", "C3.power", "2"),
        ("12_1", "minus two", "C12.radicand", "2"),
        ("12_1", "between one", "C12.left", "1"),
        ("12_1", "and two", "C12.right", "2"),
        ("12_2", "minus one", "C12.sign.left", "-1"),
        ("12_2", "and two", "C12.sign.right", "2"),
        ("12_3", "four times", "C12.bound", "4"),
        ("12_3", "by four", "C12.bound", "4"),
        ("12_4", "a zero", "C1.domain.left", "0"),
        ("12_5", "positive zero", "C1.domain.left", "0"),
        ("12_5", "root of two", "C12.radicand", "2"),
        ("12_6", "halves", "C12.bisection", "2"),
        ("12_7", "value zero", "C1.domain.left", "0"),
    ];

    /// The words that carry a number in this script.
    const NUMBER_WORDS: &[&str] = &[
        "zero",
        "one",
        "half",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "tenth",
        "tenths",
        "tenfold",
        "fifth",
        "quarter",
        "thirds",
        "third",
        "halves",
        "fifteenth",
        "fortieth",
        "square",
        "squared",
        "cubed",
        "thirteen",
        "twenty",
        "seventeen",
        "thirty",
        "fifty",
        "sixty",
        "hundred",
        "thousand",
    ];

    /// Number words used as plain language, not as a quantity a fact holds.
    const PLAIN: &[(&str, &str)] = &[
        ("01_3", "Pin one point"),
        ("06_6", "completes a square"),
        ("06_7", "the square root"),
        ("10_7", "The square root"),
        ("11_4", "one negative"),
        ("11_4", "one positive"),
        ("11_8", "square root"),
        ("12_5", "square root"),
    ];

    /// Each word of `text` and the byte it starts at.
    fn words(text: &str) -> Vec<(usize, &str)> {
        let mut out = Vec::new();
        let mut start = None;
        for (i, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            match (c.is_alphabetic(), start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    out.push((s, &text[s..i]));
                    start = None;
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        for (id, words, key, value) in SPOKEN {
            let line = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(line.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        // every number word of every line sits inside a phrase the table checks
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            let phrases = SPOKEN.iter().map(|&(i, w, ..)| (i, w)).chain(PLAIN.iter().copied());
            let covered: Vec<(usize, usize)> = phrases
                .filter(|(i, _)| i == id)
                .filter_map(|(_, w)| {
                    let w = w.to_lowercase();
                    lower.find(&w).map(|at| (at, at + w.len()))
                })
                .collect();
            for (start, word) in words(&lower) {
                if NUMBER_WORDS.contains(&word) {
                    assert!(
                        covered.iter().any(|&(a, b)| a <= start && start + word.len() <= b),
                        "{id}: {word:?} at {start} has no row in SPOKEN: {text}"
                    );
                }
            }
        }
    }

    /// The historical names the narration says, the only lines that say
    /// each, and the reference those lines' storyboard rows carry: the
    /// passage read at page level (the README's sources).
    const NAMED: &[(&str, &[&str], &str)] = &[
        ("berkeley", &["03_1", "03_3", "03_8"], "The Analyst §§XIII–XIV, pp. 5–6"),
        ("sprague", &["06_9"], "A Note on δ and ε"),
        ("davis", &["07_9"], "A Further Note on δ and ε"),
    ];

    /// What no line says: names whose attribution was not read at page level
    /// (brief), and priority claims.
    const UNNAMED: &[&str] = &[
        "cauchy",
        "weierstrass",
        "bolzano",
        "newton",
        "leibniz",
        "dedekind",
        "heine",
        "invented",
        "first to",
    ];

    /// The opening never claims the picture proves the limit, never calls
    /// the failure one of uniform continuity, and names and dates no one.
    #[test]
    fn the_opening_claims_only_what_it_shows() {
        let all = all_text();
        let lower = all.to_lowercase();
        assert!(all.contains("A picture of a few curves can't prove that. Arithmetic can."));
        for wrong in [
            "uniform continuity",
            "uniformly continuous",
            "the picture proves",
            "as you can see, the limit",
        ] {
            assert!(!lower.contains(wrong), "the script says {wrong:?}");
        }
        assert!(
            !all.chars().any(|c| c.is_ascii_digit()),
            "numbers are spelled as spoken"
        );
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            if id.starts_with("01_") || id.starts_with("02_") {
                let lower = text.to_lowercase();
                for (name, ..) in NAMED {
                    assert!(!lower.contains(name), "{id} names {name}");
                }
                assert!(!lower.contains("seventeen"), "{id} gives a year");
            }
        }
    }

    /// A historical name is said only in the lines whose storyboard rows
    /// cite the passage read for it, and no unread attribution is said at
    /// all; the one year said is the one that edition gives.
    #[test]
    fn every_name_said_has_its_passage() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            for wrong in UNNAMED {
                assert!(!lower.contains(wrong), "{id} says {wrong:?}");
            }
            for (name, lines, _) in NAMED {
                assert_eq!(lower.contains(name), lines.contains(&id.as_str()), "{id} and {name}");
            }
        }
        for (_, lines, reference) in NAMED {
            for id in *lines {
                let row = STORYBOARD
                    .lines()
                    .find(|l| l.starts_with(&format!("| {id} |")))
                    .unwrap_or_else(|| panic!("no storyboard row {id}"));
                assert!(row.contains(reference), "{id}'s row cites {reference:?}");
            }
        }
        assert!(script.text("03_1").unwrap().contains("seventeen thirty four"));
    }

    /// The fixed point and the moving point are kept apart: the fixed point
    /// is pinned before its heights are read, the moving point's gap is one
    /// half, and the closing line contrasts who picks the point.
    #[test]
    fn fixed_and_moving_are_never_confused() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        assert!(text("01_3").starts_with("Pin one point, x equals nine tenths."));
        assert!(text("01_9").starts_with("Now let the point move."));
        let close = text("01_11");
        let fix = close.find("Fix it first").unwrap();
        let shrinks = close.find("shrinks to zero").unwrap();
        let moves = close.find("Let it move with the curve").unwrap();
        assert!(fix < shrinks && shrinks < moves && close.ends_with("the gap never does."));
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
