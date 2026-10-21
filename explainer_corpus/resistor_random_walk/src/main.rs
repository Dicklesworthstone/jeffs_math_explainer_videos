//! A Resistor Network Is Secretly Answering a Probability Question: one
//! small network of identical wires, its voltages and a random walker's
//! chances computed independently and shown to obey the same averaging
//! rule, rendered end to end by franken_manim's native Rust front door and
//! the corpus's shared kit.
//!
//! ```text
//! resistor facts                         # every number the video shows or says (key<TAB>value)
//! resistor script                        # the narration (generated from docs/storyboard.md)
//! resistor render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! resistor help                          # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod palette;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat (the
/// tests check that script.tsv and the chapter registry follow it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "resistor_random_walk",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    // the owner's narration direction: authored, thought-following gaps
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

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed value or word breaks
    /// this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        // 01: the line walk
        ("01_1", "Five identical wires", "R-F10.wires", "5"),
        ("01_1", "six junctions", "R-F10.junctions", "6"),
        ("01_1", "one wire above the bottom", "R-F10.start", "1"),
        ("01_2", "one of its two wires", "R-F10.wires_at_middle", "2"),
        ("01_3", "Up, down, up, up, up and up", "R-F10.moves", "up,down,up,up,up,up"),
        ("01_5", "Half the time it lands one wire up", "R-F10.step", "1/2"),
        ("01_5", "half the time one wire down", "R-F10.step", "1/2"),
        ("01_6", "those two far ends", "R-F10.wires_at_middle", "2"),
        ("01_7", "From the top the chance is one", "R-F10.top_chance", "1"),
        ("01_7", "from the bottom it's zero", "R-F10.bottom_chance", "0"),
        ("01_7", "one wire above the bottom", "R-F10.start", "1"),
        ("01_8", "One fifth", "R-F10.answer", "1/5"),
        ("01_8", "five equal steps", "R-F10.wires", "5"),
        ("01_8", "climb from zero to one", "R-F10.chances", "0,1/5,2/5,3/5,4/5,1"),
        ("01_9", "its chance of walking for ever is zero", "R-F10.forever", "0,0,0,0,0,0"),
        ("01_9", "with zero at both ends", "R-F10.forever", "0,0,0,0,0,0"),
        // 02: the line as a circuit
        ("02_1", "at one volt", "R-F11.top_volts", "1"),
        ("02_1", "the bottom at zero", "R-F11.bottom_volts", "0"),
        ("02_3", "every wire carries the same current", "R-F11.same_current", "true"),
        ("02_4", "Five equal drops", "R-F11.drop_count", "5"),
        ("02_4", "share the one volt", "R-F11.top_volts", "1"),
        ("02_4", "one wire above the bottom", "R-F10.start", "1"),
        ("02_5", "One fifth of a volt", "R-F11.answer", "1/5"),
        ("02_5", "the voltages and the chances agree", "R-F11.same", "true"),
        ("02_7", "its two wires", "R-F10.wires_at_middle", "2"),
        ("02_8", "the two answers had to agree", "R-F11.same", "true"),
        // 03: the bridge, and the school shortcuts
        ("03_1", "four junctions", "R-F1.junctions", "4"),
        ("03_1", "doubled in two places", "R-F1.doubled", "2"),
        ("03_1", "doubled", "R-F1.side_by_side", "2"),
        ("03_1", "the top at one volt", "R-F1.top_volts", "1"),
        ("03_1", "the bottom at zero", "R-F1.bottom_volts", "0"),
        ("03_2", "its four wires", "R-F1.wires_at_middle", "4"),
        ("03_3", "turn out to be three fifths", "R-F2.left", "3/5"),
        ("03_3", "and two fifths", "R-F2.right", "2/5"),
        ("03_3", "one, one, two fifths and zero", "R-F2.left.far_ends", "1,1,2/5,0"),
        ("03_3", "average to three fifths", "R-F2.left.average", "3/5"),
        ("03_5", "Two wires side by side", "R-F1.side_by_side", "2"),
        ("03_5", "carries current twice as easily", "R-F12.pair_conductance", "2"),
        ("03_5", "act as one twice as hard", "R-F12.row_conductance", "1/2"),
        ("03_6", "five wires remain", "R-F12.links", "5"),
        ("03_7", "No two join the same two junctions", "R-F12.side_by_side", "0"),
        ("03_7", "three wires meet at each middle junction", "R-F12.meeting", "3,3"),
        ("03_7", "no two are in a row", "R-F12.in_a_row", "0"),
        ("03_8", "a bridge across the middle", "R-F12.bridge", "left-right"),
        // 04: the walker on the bridge
        ("04_1", "one of its four wires", "R-F1.wires_at_middle", "4"),
        ("04_2", "its four wires", "R-F1.wires_at_middle", "4"),
        ("04_3", "From the top the chance is one", "R-F3.top_chance", "1"),
        ("04_3", "from the bottom it's zero", "R-F3.bottom_chance", "0"),
        ("04_4", "Three fifths again", "R-F3.left", "3/5"),
        ("04_4", "Three fifths again", "R-F2.left", "3/5"),
        ("04_4", "only one solution", "R-X1.solutions", "1"),
        ("04_5", "The two numbers agree", "R-F3.same", "true"),
        // 05: the balances
        ("05_2", "Each wire of the pair", "R-F1.side_by_side", "2"),
        ("05_2", "drops two fifths of a volt", "R-F4.drop.top", "2/5"),
        ("05_2", "one fifth to the right", "R-F4.drop.right", "1/5"),
        ("05_2", "three fifths to the bottom", "R-F4.drop.bottom", "3/5"),
        ("05_3", "Two fifths and two fifths in", "R-F4.in", "2/5,2/5"),
        ("05_3", "one fifth and three fifths out", "R-F4.out", "1/5,3/5"),
        ("05_4", "four times the left", "R-F4.coefficient", "4"),
        ("05_4", "one plus one plus the right plus zero", "R-F9.left.far_ends", "top,top,right,bottom"),
        ("05_5", "The pair is two wires", "R-F1.side_by_side", "2"),
        ("05_5", "the top's one counts twice", "R-F1.side_by_side", "2"),
        ("05_6", "One wire goes up, one to the left, and two go down", "R-F13.right.far_ends", "top,left,bottom,bottom"),
        ("05_6", "four times the right", "R-F4.coefficient", "4"),
        ("05_7", "One plus the left plus zero plus zero", "R-F13.right.far_ends", "top,left,bottom,bottom"),
        // 06: the walker's rule
        ("06_1", "one of the four wires", "R-F1.wires_at_middle", "4"),
        ("06_2", "One half", "R-F5.to_top", "1/2"),
        ("06_2", "not one third", "R-F5.trap", "1/3"),
        ("06_2", "two of the four wires", "R-F5.top_wires", "2"),
        ("06_2", "two of the four wires", "R-F1.wires_at_middle", "4"),
        ("06_2", "counts twice", "R-F1.side_by_side", "2"),
        ("06_2", "its two currents", "R-F5.top_wires", "2"),
        ("06_3", "Down, it's zero", "R-F3.bottom_chance", "0"),
        ("06_3", "A quarter of each", "R-F3.wire_chance", "1/4"),
        ("06_4", "steps up with chance a quarter, left with a quarter and down with a half", "R-F13.right.steps", "1/4,1/4,1/2"),
        ("06_4", "a quarter of one", "R-F3.top_chance", "1"),
        ("06_4", "a half of zero", "R-F3.bottom_chance", "0"),
        ("06_5", "Times four", "R-F4.coefficient", "4"),
        ("06_5", "four times the left is two plus the right", "R-F13.left_constant", "2"),
        ("06_5", "four times the right is one plus the left", "R-F13.right_constant", "1"),
        ("06_6", "the circuit's two balances, letter for letter", "R-F13.walk_same", "true"),
        // 07: the solve and the routes
        ("07_1", "solve them once", "R-F13.walk_same", "true"),
        ("07_1", "the left is two plus the right", "R-F13.left_constant", "2"),
        ("07_1", "over four", "R-F4.coefficient", "4"),
        ("07_1", "multiply by four", "R-F4.coefficient", "4"),
        ("07_2", "Sixteen times the right", "R-F13.sixteen", "16"),
        ("07_2", "equals four plus", "R-F13.four", "4"),
        ("07_2", "four plus two plus the right", "R-F13.left_constant", "2"),
        ("07_2", "fifteen times the right", "R-F13.fifteen", "15"),
        ("07_2", "is six", "R-F13.six", "6"),
        ("07_2", "the right is two fifths", "R-F13.right", "2/5"),
        ("07_3", "two and two fifths", "R-F13.left_sum", "12/5"),
        ("07_3", "over four", "R-F4.coefficient", "4"),
        ("07_3", "which is three fifths", "R-F13.left", "3/5"),
        ("07_3", "because fifteen isn't zero", "R-X1.determinant", "15"),
        ("07_3", "no other pair of values keeps both balances", "R-X1.solutions", "1"),
        ("07_4", "with chance one half", "R-F14.up_now", "1/2"),
        ("07_4", "plus one sixteenth", "R-F14.across_up", "1/16"),
        ("07_5", "comes back to the left with chance one sixteenth", "R-F14.back", "1/16"),
        ("07_6", "nine sixteenths plus", "R-F14.exit_top", "9/16"),
        ("07_6", "a sixteenth of itself", "R-F14.back", "1/16"),
        ("07_6", "Fifteen sixteenths of it", "R-F14.kept", "15/16"),
        ("07_6", "which makes it three fifths", "R-F14.sum", "3/5"),
        ("07_8", "reaching the bottom first", "R-F14.bottom_first.left", "2/5"),
        ("07_9", "Two fifths", "R-F14.bottom_first.left", "2/5"),
        ("07_9", "with chance one", "R-F3.stops", "1,1"),
        ("07_9", "the two chances add to one", "R-F3.stops", "1,1"),
        // 08: average, and average again
        ("08_1", "Guess zero for both", "R-F7.start", "0,0"),
        ("08_2", "its four wires", "R-F1.wires_at_middle", "4"),
        ("08_3", "one, one, zero and zero", "R-F7.round1.left_ends", "1,1,0,0"),
        ("08_3", "one and three zeros", "R-F7.round1.right_ends", "1,0,0,0"),
        ("08_3", "to one half. The right averages one and three zeros to one quarter", "R-F7.round1", "1/2,1/4"),
        ("08_4", "Nine sixteenths and three eighths", "R-F7.round2", "9/16,3/8"),
        ("08_4", "nineteen thirty-seconds and twenty-five sixty-fourths", "R-F7.round3", "19/32,25/64"),
        ("08_5", "shrinks to a quarter every round", "R-F7.shrink", "1/4"),
        ("08_5", "only one wire at each junction", "R-F7.unknown_wires", "1"),
        ("08_6", "exactly three fifths", "R-F2.left", "3/5"),
        ("08_7", "halves, quarters, eighths and so on", "R-F7.dyadic", "true"),
        ("08_7", "three fifths is none of them", "R-F7.never", "true"),
        ("08_7", "after five rounds", "R-F7.within", "5"),
        ("08_7", "within a thousandth", "R-F7.tolerance", "1/1000"),
        ("08_8", "three fifths and two fifths", "R-F7.limit", "3/5,2/5"),
        // 09: only one solution
        ("09_1", "only one solution", "R-X1.solutions", "1"),
        ("09_2", "zero at the top and bottom", "R-F9.ends", "0,0"),
        ("09_3", "If above zero", "R-F9.ends", "0,0"),
        ("09_4", "spreads along wires to the top", "R-F9.path.left", "left,top"),
        ("09_4", "the top, which is zero", "R-F9.ends", "0,0"),
        ("09_5", "the peak is zero", "R-F9.peak", "0"),
        ("09_5", "likewise the lowest value", "R-F9.low", "0"),
        ("09_6", "two junctions wired only to each other", "R-X8.island.junctions", "2"),
        ("09_6", "still only one solution", "R-X1.solutions", "1"),
        ("09_7", "crosses its one wire", "R-X8.island.wires", "1"),
        ("09_7", "never reaching the top or bottom", "R-X8.island.path", "false"),
        ("09_7", "Any equal pair keeps the rule", "R-X8.island.keeps", "true,true,true"),
        // for all time by no path of wires; the 12 steps followed agree
        ("09_9", "Its chance is zero", "R-X8.island.path", "false"),
        ("09_9", "Its chance is zero", "R-X8.island.at_ends", "0,0"),
        // 10: unequal wires
        ("10_2", "one up, two across and one down", "R-F15.left.conductances", "1,2,1"),
        ("10_4", "Across", "R-F15.most_likely", "across"),
        ("10_4", "with chance two in four, one half", "R-F15.left.steps", "1/4,1/2,1/4"),
        ("10_4", "two in four", "R-F15.left.total", "4"),
        ("10_4", "the left is a quarter, plus half the right", "R-F15.left.steps", "1/4,1/2,1/4"),
        ("10_4", "a fifth, plus two fifths of the left", "R-F15.right.steps", "1/5,2/5,2/5"),
        ("10_5", "seven sixteenths and three eighths", "R-F15.voltages", "7/16,3/8"),
        ("10_5", "as voltages and as chances", "R-F15.same", "true"),
        // 11: when to stop
        ("11_1", "each round from the one before", "R-F16.simultaneous", "true"),
        ("11_2", "run it twice, from all zeros and from all ones", "R-F16.starts", "0,1"),
        ("11_2", "never swaps the order of two guesses", "R-F16.monotone", "true"),
        ("11_2", "the true values stay between the two runs", "R-F16.between", "true"),
        ("11_3", "within a thousandth", "R-F7.tolerance", "1/1000"),
        ("11_3", "after nine rounds", "R-F16.first_within", "9"),
        ("11_4", "What happens to the two runs there", "R-F16.starts", "0,1"),
        ("11_5", "They never move", "R-X9.island.moves", "false"),
        ("11_5", "its zeros stay zeros", "R-X9.island.low", "0"),
        ("11_5", "its ones stay ones", "R-X9.island.high", "1"),
        ("11_5", "its voltage isn't unique", "R-X9.determinant", "0"),
        ("11_6", "one solution", "R-X1.solutions", "1"),
    ];

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        for (id, words, key, value) in SPOKEN {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        // and no line carries a number word the table above does not know
        let numbers = [
            "zero",
            "zeros",
            "one",
            "ones",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "nine",
            "fifteen",
            "sixteen",
            "nineteen",
            "twenty",
            "thirty",
            "sixty",
            "fifth",
            "fifths",
            "third",
            "half",
            "halves",
            "quarter",
            "quarters",
            "eighths",
            "sixteenth",
            "sixteenths",
            "fourths",
            "thousandth",
            "twice",
            "doubled",
            "once",
        ];
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            let says_a_number = lower.split(|c: char| !c.is_alphabetic()).any(|w| numbers.contains(&w));
            if says_a_number {
                assert!(
                    SPOKEN.iter().any(|(i, ..)| i == id),
                    "{id} says a number with no row in SPOKEN: {text}"
                );
            }
        }
    }

    /// The synthesis input, narration_units.json, was built from this
    /// script: it names the script's hash, and holds every line once, in
    /// order (`python3 -I narration_units.py --check` checks the rest; with
    /// no argument it rebuilds the file after a script edit).
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json is stale: run python3 -I narration_units.py"
        );
        let script = Script::parse(SCRIPT).unwrap();
        let mut from = 0;
        for (id, _) in script.lines() {
            let quoted = format!("\"{id}\"");
            assert_eq!(units.matches(&quoted).count(), 1, "{id} is not in exactly one unit");
            let at = units[from..]
                .find(&quoted)
                .unwrap_or_else(|| panic!("{id} is out of script order"));
            from += at + quoted.len();
        }
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// Each chapter asks one prediction, and the next line's first words
    /// answer it: the line walker's chance, the line's voltage, the merge
    /// no shortcut allows, the bridge walker's chance, the right junction's
    /// balance, the chance of a first step up, the chance of the bottom
    /// first, the round that reaches the answer, the count of solutions once
    /// an island is added, the unequal wires' likeliest step, and the two
    /// runs beside an island.
    #[test]
    fn the_prediction_is_asked_once_and_answered_next() {
        let boards = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let expected = [
            ("01_which_end", "one wire above the bottom?", "One fifth"),
            ("02_the_line_as_a_circuit", "one wire above the bottom?", "One fifth of a volt"),
            ("03_a_bridge", "merge next?", "None"),
            ("04_the_same_number", "left junction?", "Three fifths"),
            ("05_why_an_average", "four times the right equal?", "One plus the left"),
            ("06_the_walkers_rule", "reaches the top?", "One half"),
            ("07_one_answer_two_ways", "bottom first?", "Two fifths"),
            ("08_round_after_round", "exactly three fifths?", "Never"),
            ("09_only_one", "only one solution?", "No"),
            ("10_any_wires", "step first?", "Across"),
            ("11_when_to_stop", "two runs there?", "They never move"),
        ];
        assert_eq!(boards.len(), expected.len());
        for (board, (key, asks, answers)) in boards.iter().zip(expected) {
            assert_eq!(board.key, key);
            let beats = &board.beats;
            let asked: Vec<usize> = (0..beats.len()).filter(|&i| beats[i].predict()).collect();
            assert_eq!(asked.len(), 1, "one prediction in {key}");
            let (question, answer) = (&beats[asked[0]], &beats[asked[0] + 1]);
            assert!(question.spoken().unwrap().ends_with(asks), "{key}");
            assert!(answer.spoken().unwrap().starts_with(answers), "{key}");
        }
    }
}
