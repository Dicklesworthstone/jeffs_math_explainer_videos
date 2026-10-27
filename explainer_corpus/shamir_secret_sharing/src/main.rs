//! Three Keys Open the Secret. Two Reveal Nothing.: Shamir's threshold
//! secret sharing on an eleven-mark dial, why two shares leave every secret
//! possible and three rebuild it, rendered end to end by franken_manim's
//! native Rust front door and the corpus's shared kit.
//!
//! ```text
//! vault facts                         # every number the video shows or says (key<TAB>value)
//! vault script                        # the narration (generated from docs/storyboard.md)
//! vault render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! vault help                          # the full option list
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
    crate_name: "shamir_secret_sharing",
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
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER,
        std::env::args().skip(1),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
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
    /// for and the value those words denote. A changed count or word breaks
    /// this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("23_1", "one, two and three", "S-F1.opening_trio", "1,2,3"),
        ("23_1", "eight, three and one", "S-F1.opening_shares", "8,3,1"),
        ("23_1", "five", "S-F1.secret", "5"),
        ("23_1", "position zero", "S-F4.secret_position", "0"),
        ("23_2", "four and five", "S-F1.pair", "4,5"),
        ("23_2", "two and six", "S-F1.pair_shares", "2,6"),
        ("23_2", "eleven different recipes", "S-F13.curves_left", "11"),
        ("23_2", "one for each", "S-F7.ways_each", "1"),
        ("23_3", "one of the hundred and twenty-one", "S-F6.spin_pairs", "121"),
        ("23_4", "four of seven", "S-F13.four_of_seven.needed", "4"),
        ("23_5", "Three spins", "S-F13.four_of_seven.spins", "3"),
        ("23_5", "at most three", "S-F13.four_of_seven.spins", "3"),
        ("23_6", "first three", "S-F13.four_of_seven.spins", "3"),
        (
            "23_6",
            "four, zero and nine",
            "S-F13.four_of_seven.cards",
            "4,0,9,3,9,10,0",
        ),
        ("23_6", "eleven columns", "S-F13.four_of_seven.curves_left", "11"),
        ("23_6", "one for each secret", "S-F13.four_of_seven.ways_each", "1"),
        ("23_6", "exactly one", "S-F13.four_of_seven.ways_each", "1"),
        (
            "23_7",
            "trustee four brings three",
            "S-F13.four_of_seven.cards",
            "4,0,9,3,9,10,0",
        ),
        ("23_7", "secret five", "S-F1.secret", "5"),
        ("23_8", "Any four", "S-F13.four_of_seven.needed", "4"),
        ("23_8", "four distinct roots", "S-F13.four_of_seven.needed", "4"),
        ("23_8", "at most three", "S-F13.four_of_seven.spins", "3"),
        ("23_8", "Any three shares", "S-F13.four_of_seven.spins", "3"),
        ("23_10", "Any three", "S-F1.threshold", "3"),
        ("23_10", "Any two", "S-F1.too_few", "2"),
        ("01_1", "Five trustees", "S-F1.trustees", "5"),
        ("01_1", "turned to one mark", "S-F1.opening_marks", "1"),
        ("01_2", "the eleven marks", "S-F1.marks", "11"),
        ("01_3", "tell one trustee", "S-F0.keepers", "1"),
        (
            "01_3",
            "one accident or one lost note",
            "S-F0.keeper.losses_to_shut",
            "1",
        ),
        ("01_3", "one bribe", "S-F0.keeper.bribes_to_open", "1"),
        ("01_4", "give all five a copy", "S-F1.trustees", "5"),
        // the one absence that shut the single keeper's vault
        ("01_4", "One absence no longer locks", "S-F0.keeper.losses_to_shut", "1"),
        ("01_4", "any one can let it slip", "S-F0.copies.bribes_to_open", "1"),
        ("01_5", "Any three of them", "S-F1.threshold", "3"),
        ("01_5", "any two should learn nothing", "S-F1.too_few", "2"),
        ("01_5", "That number, three", "S-F1.threshold", "3"),
        ("01_6", "a threshold of three", "S-F1.threshold", "3"),
        ("01_6", "two trustees can be away", "S-F1.can_be_away", "2"),
        ("02_2", "both of two trustees", "S-F0.pieces.both_needed", "2"),
        ("02_2", "into two ordinary numbers", "S-F0.pieces.both_needed", "2"),
        ("02_2", "two for A", "S-F0.pieces.a", "2"),
        ("02_2", "three for B", "S-F0.pieces.b", "3"),
        ("02_3", "add two and three, get five", "S-F0.pieces.sum", "5"),
        ("02_3", "B's three", "S-F0.pieces.b", "3"),
        ("02_4", "at least three", "S-F0.pieces.b", "3"),
        ("02_4", "Zero, one and two", "S-F0.pieces.b_alone_ruled_out", "0,1,2"),
        ("02_4", "only eight possible", "S-F0.pieces.b_alone_left", "8"),
        ("02_6", "B's three", "S-F0.pieces.b", "3"),
        ("02_6", "eight possible", "S-F0.pieces.b_alone_left", "8"),
        ("03_1", "From nine", "S-F2.a_share", "9"),
        ("03_1", "turn seven marks", "S-F2.b_share", "7"),
        ("03_1", "past ten", "S-F2.top", "10"),
        ("03_1", "round through zero", "S-F2.ten_plus_one", "0"),
        ("03_1", "stop at five", "S-F2.sum", "5"),
        ("03_2", "nine plus seven is five", "S-F2.sum", "5"),
        ("03_2", "Ten plus one is zero", "S-F2.ten_plus_one", "0"),
        ("03_2", "five take away seven is nine", "S-F2.turn_back", "9"),
        ("03_3", "stops, nine", "S-F2.a_share", "9"),
        ("03_3", "from nine to the secret, seven", "S-F2.b_share", "7"),
        ("03_4", "turn nine on by seven, reach five", "S-F2.sum", "5"),
        ("03_4", "B's seven", "S-F2.b_share", "7"),
        (
            "03_5",
            "If A had zero, the secret would be seven",
            "S-F2.if_a_held.0",
            "7",
        ),
        ("03_5", "If A had one, eight", "S-F2.if_a_held.1", "8"),
        ("03_5", "all eleven marks", "S-F1.marks", "11"),
        ("03_5", "all eleven secrets", "S-F2.distinct_secrets", "11"),
        ("03_5", "once each", "S-F2.ways_each", "1"),
        ("03_6", "B's seven", "S-F2.b_share", "7"),
        ("04_1", "Back to five trustees", "S-F1.trustees", "5"),
        ("04_1", "a threshold of three", "S-F1.threshold", "3"),
        ("04_1", "two fair spins", "S-F6.spin_count", "2"),
        // a share for each trustee
        ("04_1", "hands out five shares", "S-F1.trustees", "5"),
        ("04_1", "eight, three, one, two and six", "S-F6.shares", "8,3,1,2,6"),
        ("04_2", "Trustees one, two and three", "S-F1.opening_trio", "1,2,3"),
        ("04_2", "their eight, three and one", "S-F1.opening_shares", "8,3,1"),
        ("04_2", "the secret, five", "S-F1.secret", "5"),
        ("04_3", "Trustees four and five", "S-F1.pair", "4,5"),
        ("04_3", "hold two and six", "S-F1.pair_shares", "2,6"),
        ("04_3", "one time in eleven", "S-F12.guess_chance", "1/11"),
        ("04_3", "the other ten times", "S-F12.guess_misses", "10"),
        ("04_4", "two trustees can never", "S-F1.too_few", "2"),
        ("04_4", "their two shares", "S-F1.too_few", "2"),
        // "not one" mark: none
        ("04_4", "not one mark ruled out", "S-F7.ruled_out", "0"),
        ("04_5", "Could two shares", "S-F1.too_few", "2"),
        ("04_5", "when three open", "S-F1.threshold", "3"),
        ("05_1", "from zero to ten", "S-F7.secrets", "0,1,2,3,4,5,6,7,8,9,10"),
        ("05_1", "hundred and twenty-one ways", "S-F6.spin_pairs", "121"),
        ("05_1", "its two spins", "S-F6.spin_count", "2"),
        ("05_2", "trustees four and five", "S-F1.pair", "4,5"),
        ("05_2", "a two and a six", "S-F1.pair_shares", "2,6"),
        ("05_3", "exactly one way", "S-F7.ways_each", "1"),
        ("05_3", "two and six fit", "S-F1.pair_shares", "2,6"),
        ("05_5", "Two shares can't", "S-F1.too_few", "2"),
        ("05_5", "and three can", "S-F1.threshold", "3"),
        ("06_1", "any three", "S-F1.threshold", "3"),
        ("06_1", "of five", "S-F1.trustees", "5"),
        ("06_1", "Put ten padlocks", "S-F3.locks", "10"),
        ("06_1", "one for each pair", "S-F3.locks_per_pair", "1"),
        ("06_2", "four and five", "S-F1.pair", "4,5"),
        ("06_2", "with one, two and three", "S-F3.pair_lock.holders", "1,2,3"),
        ("06_3", "six keys", "S-F3.keys_each", "6"),
        ("06_3", "Any three trustees", "S-F1.threshold", "3"),
        ("06_3", "all three of them", "S-F1.threshold", "3"),
        ("06_4", "trustees four and five", "S-F1.pair", "4,5"),
        // how many locks stop the pair: just their own
        ("06_5", "Just one", "S-F3.locks_a_pair_cannot_open", "1"),
        ("06_5", "two different pairs", "S-F3.merged.pairs", "2"),
        ("06_5", "the three or four trustees", "S-F3.merged.unions", "3,4"),
        ("06_5", "So ten locks", "S-F3.fewest_locks", "10"),
        ("06_6", "nineteen seventy-nine", "S-X1.shamir.year", "1979"),
        ("06_6", "Any six", "S-F3.shamir.needed", "6"),
        ("06_6", "of eleven scientists", "S-F3.shamir.scientists", "11"),
        ("06_6", "no five can", "S-F3.shamir.too_few", "5"),
        ("06_7", "four hundred and sixty-two locks", "S-F3.shamir.locks", "462"),
        ("06_7", "two hundred and fifty-two keys", "S-F3.shamir.keys_each", "252"),
        ("06_8", "one number", "S-F3.dealer.numbers_each", "1"),
        ("07_1", "any two", "S-F4.needed", "2"),
        ("07_1", "of five", "S-F1.trustees", "5"),
        ("07_1", "position zero", "S-F4.secret_position", "0"),
        ("07_1", "height five", "S-F1.secret", "5"),
        ("07_2", "spins once", "S-F4.spin_count", "1"),
        ("07_2", "gives four", "S-F4.slope", "4"),
        ("07_2", "climbs four", "S-F4.slope", "4"),
        ("07_3", "Trustee one sits at position one", "S-F4.position_of.1", "1"),
        ("07_3", "trustee two at position two", "S-F4.position_of.2", "2"),
        (
            "07_3",
            "Nine, thirteen, seventeen, twenty-one and twenty-five",
            "S-F4.heights",
            "9,13,17,21,25",
        ),
        ("07_4", "Any two trustees", "S-F4.needed", "2"),
        ("07_4", "their two points", "S-F4.needed", "2"),
        ("07_4", "the height at zero", "S-F4.secret_position", "0"),
        ("07_5", "One point is not enough", "S-F4.too_few", "1"),
        ("07_5", "Trustee five", "S-F4.paper.holder", "5"),
        ("07_5", "holds twenty-five", "S-F4.paper.share", "25"),
        ("07_6", "Just three", "S-F4.paper.count", "3"),
        ("07_6", "zero, five or ten", "S-F4.paper.candidates", "0,5,10"),
        ("07_6", "is twenty-five", "S-F4.paper.share", "25"),
        // the line through (5, 25) climbing a whole spin a step
        ("07_6", "five times a spin", "S-F4.paper.holder", "5"),
        // each height wound round the dial: height, whole turns, mark
        ("08_1", "Nine stays nine", "S-F4.wrap.1", "9,0,9"),
        (
            "08_1",
            "Thirteen goes once round the dial and lands on two",
            "S-F4.wrap.2",
            "13,1,2",
        ),
        ("08_1", "Seventeen lands on six", "S-F4.wrap.3", "17,1,6"),
        ("08_1", "twenty-one on ten", "S-F4.wrap.4", "21,1,10"),
        ("08_1", "twenty-five on three", "S-F4.wrap.5", "25,2,3"),
        ("08_2", "turn on four", "S-F4.slope", "4"),
        ("08_3", "trustees two and three", "S-F4.adjacent", "2,3"),
        ("08_3", "holding two and six", "S-F4.adjacent.shares", "2,6"),
        ("08_3", "one step apart", "S-F4.adjacent.apart", "1"),
        ("08_3", "differ by four", "S-F4.adjacent.step", "4"),
        ("08_3", "each step adds four", "S-F4.adjacent.step", "4"),
        ("08_4", "Step back twice", "S-F4.back.steps", "2"),
        ("08_4", "from trustee two", "S-F4.back.from", "2"),
        ("08_4", "Two take away four is nine", "S-F4.back.1", "2,4,9"),
        ("08_4", "nine take away four is five", "S-F4.back.2", "9,4,5"),
        ("08_5", "trustee two is alone", "S-F5.holder", "2"),
        ("08_5", "holding two", "S-F5.share", "2"),
        // twice: trustee 2's position times the spin
        ("08_5", "the secret plus twice the spin", "S-F5.holder", "2"),
        ("08_5", "the eleven marks", "S-F1.marks", "11"),
        ("08_6", "All eleven", "S-F5.distinct", "11"),
        ("08_6", "one for each spin", "S-F5.per_spin", "1"),
        ("08_6", "Spin zero means the secret is two", "S-F5.spin.0", "2"),
        // spin 1's secret, then its working: the share take away twice 1
        ("08_6", "Spin one means", "S-F5.spin.1", "0"),
        ("08_6", "two take away two, zero", "S-F5.spin.1.working", "2,2,0"),
        (
            "08_7",
            "the doubles of zero to ten",
            "S-F4.times2.of",
            "0,1,2,3,4,5,6,7,8,9,10",
        ),
        (
            "08_7",
            "zero, two, four, six, eight, ten, then one, three, five, seven and nine",
            "S-F4.times2",
            "0,2,4,6,8,10,1,3,5,7,9",
        ),
        ("08_7", "Every mark, once", "S-F4.times2.hits_each", "1"),
        ("08_8", "trustee two's share", "S-F5.holder", "2"),
        ("08_8", "B's seven", "S-F2.b_share", "7"),
        ("09_1", "any three", "S-F1.threshold", "3"),
        ("09_1", "of five", "S-F1.trustees", "5"),
        ("09_1", "spins twice", "S-F6.spin_count", "2"),
        ("09_1", "gives seven, then seven again", "S-F6.spins", "7,7"),
        (
            "09_2",
            "plus seven times their position, plus seven times their position squared",
            "S-F6.spins",
            "7,7",
        ),
        // position, the secret, the two spins' terms, the height
        (
            "09_3",
            "Trustee one gets five plus seven plus seven, nineteen",
            "S-F6.terms.1",
            "1,5,7,7,19",
        ),
        (
            "09_3",
            "Trustee two gets five plus fourteen plus twenty-eight, forty-seven",
            "S-F6.terms.2",
            "2,5,14,28,47",
        ),
        ("09_4", "Three, four and five get", "S-F6.rest", "3,4,5"),
        (
            "09_4",
            "eighty-nine, a hundred and forty-five, and two hundred and fifteen",
            "S-F6.rest.heights",
            "89,145,215",
        ),
        ("09_4", "one smooth curve", "S-F6.paper.curves", "1"),
        ("09_4", "any three of them pin it down", "S-F1.threshold", "3"),
        ("09_5", "trustee two's forty-seven", "S-F6.asked", "2,47"),
        (
            "09_6",
            "Forty-seven is four elevens and three more, so it lands on three",
            "S-F6.wrap.2",
            "47,4,3",
        ),
        ("09_6", "All five shares", "S-F1.trustees", "5"),
        ("09_6", "eight, three, one, two and six", "S-F6.shares", "8,3,1,2,6"),
        ("09_7", "One spin gives a line", "S-F4.spin_count", "1"),
        ("09_7", "for any two", "S-F4.needed", "2"),
        ("09_7", "Two spins give a curve", "S-F6.spin_count", "2"),
        ("09_7", "for any three", "S-F1.threshold", "3"),
        ("10_1", "Back to trustees four and five", "S-F1.pair", "4,5"),
        ("10_1", "with their two and six", "S-F1.pair_shares", "2,6"),
        ("10_1", "Through their two points", "S-F7.pair.count", "2"),
        ("10_1", "the secret five", "S-F1.secret", "5"),
        ("10_1", "with spins seven and seven", "S-F7.true_spins", "7:7"),
        ("10_2", "their two points", "S-F7.pair.count", "2"),
        ("10_2", "the secret zero instead", "S-F7.asked.first", "0"),
        ("10_3", "Exactly one", "S-F7.ways_each", "1"),
        ("10_3", "with spins one and four", "S-F7.spins.0", "1,4"),
        // the secret zero's curve at each of the pair: position, height, mark
        (
            "10_3",
            "trustee four's height would be sixty-eight, which lands on two",
            "S-F7.check.4",
            "4,68,2",
        ),
        (
            "10_3",
            "trustee five's a hundred and five, which lands on six",
            "S-F7.check.5",
            "5,105,6",
        ),
        ("10_3", "For the secret one", "S-F7.asked.second", "1"),
        ("10_3", "the spins would be zero and nine", "S-F7.spins.1", "0,9"),
        ("10_4", "all eleven", "S-F7.fitting", "11"),
        (
            "10_5",
            "the hundred and twenty-one spin pairs",
            "S-F6.spin_pairs",
            "121",
        ),
        ("10_5", "exactly one of them", "S-F7.ways_each", "1"),
        ("10_5", "trustees four and five", "S-F1.pair", "4,5"),
        ("10_5", "a two and a six", "S-F1.pair_shares", "2,6"),
        ("11_1", "the same two points", "S-F7.pair.count", "2"),
        (
            "11_2",
            "a hundred and forty-five and two hundred and fifteen",
            "S-F7.paper.heights",
            "145,215",
        ),
        ("11_2", "For the secret zero", "S-F7.asked.first", "0"),
        (
            "11_2",
            "nine and a quarter and six and three quarters",
            "S-F7.paper.secret0_spins",
            "37/4,27/4",
        ),
        ("11_2", "Only the secret five", "S-F7.paper.whole_spin_secrets", "5"),
        ("11_3", "the eleven curves", "S-F7.fitting", "11"),
        ("11_3", "one pair each", "S-F7.ways_each", "1"),
        ("11_4", "let trustee one join them", "S-F7.join.trustee", "1"),
        ("11_4", "holding eight", "S-F7.join.share", "8"),
        ("11_4", "the eleven curves", "S-F7.fitting", "11"),
        ("11_4", "through eight at position one", "S-F7.join.point", "1,8"),
        ("11_5", "Just one", "S-F7.join.count", "1"),
        ("11_5", "passes position one", "S-F7.join.trustee", "1"),
        ("11_5", "the curve for five", "S-F7.join.secrets_left", "5"),
        ("11_5", "passes at eight", "S-F7.join.share", "8"),
        ("11_5", "from eleven secrets to one", "S-F7.join.drop", "11,1"),
        ("11_6", "every two shares", "S-F7.pair.count", "2"),
        // how many shares, then how many secrets they leave, for every pair
        (
            "11_6",
            "Two shares leave all eleven secrets",
            "S-F7.all_pairs.two_leave",
            "2,11",
        ),
        (
            "11_6",
            "any third share leaves exactly one",
            "S-F7.all_pairs.third_leaves",
            "3,1",
        ),
        ("12_1", "have eleven marks", "S-F1.marks", "11"),
        ("12_1", "ten would seem", "S-F8.ten.marks", "10"),
        ("12_1", "the same two spins", "S-F6.spin_count", "2"),
        ("12_1", "on a ten-mark dial", "S-F8.ten.marks", "10"),
        ("12_1", "trustees four and five", "S-F1.pair", "4,5"),
        ("12_2", "On ten marks", "S-F8.ten.marks", "10"),
        (
            "12_2",
            "nine, seven, nine, five and five",
            "S-F8.ten.shares",
            "9,7,9,5,5",
        ),
        ("12_2", "trustees four and five", "S-F1.pair", "4,5"),
        ("12_2", "hold five and five", "S-F8.ten.pair_shares", "5,5"),
        ("12_2", "not two and six", "S-F1.pair_shares", "2,6"),
        ("12_3", "those two cards", "S-F7.pair.count", "2"),
        ("12_3", "On eleven marks", "S-F1.marks", "11"),
        ("12_3", "two and six fitted", "S-F1.pair_shares", "2,6"),
        ("12_3", "exactly one way", "S-F7.ways_each", "1"),
        ("12_3", "On ten marks", "S-F8.ten.marks", "10"),
        ("12_3", "five and five come", "S-F8.ten.pair_shares", "5,5"),
        ("12_3", "from ten spin pairs", "S-F8.ten.pair_spin_pairs", "10"),
        ("12_3", "has the secret five", "S-F8.ten.pair_secrets", "5"),
        ("12_3", "These two", "S-F7.pair.count", "2"),
        // the secret's, the first spin's and the second spin's amounts
        (
            "12_4",
            "Trustee four's is the secret, plus four of the first spin, plus sixteen of the second",
            "S-F8.pair_sum.height.4",
            "1,4,16",
        ),
        (
            "12_4",
            "Trustee five's is the secret, plus five of the first, plus twenty-five of the second",
            "S-F8.pair_sum.height.5",
            "1,5,25",
        ),
        (
            "12_5",
            "Five copies of trustee four's height",
            "S-F8.pair_sum.copies.4",
            "5,4",
        ),
        ("12_5", "hold twenty", "S-F8.pair_sum.first_spin", "20"),
        ("12_5", "four copies of trustee five's", "S-F8.pair_sum.copies.5", "4,5"),
        ("12_5", "the first spin cancels", "S-F8.pair_sum.first_spin_left", "0"),
        // the second spin's, the first spin's and the secret's amounts left
        (
            "12_5",
            "the secret, minus twenty of the second spin",
            "S-F8.pair_sum.spin_terms",
            "-20,0,1",
        ),
        ("12_6", "On ten marks", "S-F8.ten.marks", "10"),
        (
            "12_6",
            "twenty of anything lands on zero",
            "S-F8.ten.twenty_lands",
            "20,0",
        ),
        ("12_6", "the second spin drops out too", "S-F8.ten.leftover", "0"),
        (
            "12_6",
            "five times five, minus four times five",
            "S-F8.ten.pair_formula_terms",
            "5,5,-4,5",
        ),
        (
            "12_6",
            "is twenty-five minus twenty",
            "S-F8.ten.pair_formula_products",
            "25,-20",
        ),
        ("12_6", "which is five", "S-F8.ten.pair_formula_value", "5"),
        ("12_7", "On eleven marks", "S-F1.marks", "11"),
        (
            "12_7",
            "minus twenty lands on two",
            "S-F8.eleven.minus_twenty_lands",
            "-20,2",
        ),
        (
            "12_7",
            "the secret plus twice the second spin",
            "S-F8.eleven.leftover",
            "2",
        ),
        ("12_7", "their two and six", "S-F1.pair_shares", "2,6"),
        (
            "12_7",
            "five times two, minus four times six",
            "S-F8.eleven.pair_formula_terms",
            "5,2,-4,6",
        ),
        ("12_7", "is minus fourteen", "S-F8.eleven.pair_formula_unwrapped", "-14"),
        ("12_7", "which lands on eight", "S-F8.eleven.pair_formula_value", "8"),
        ("12_7", "makes eight", "S-F8.eleven.pair_formula_value", "8"),
        ("12_7", "exactly one second spin", "S-F8.eleven.second_spin_each", "1"),
        ("12_7", "so eight fits them all", "S-F8.eleven.pair_formula_value", "8"),
        ("12_8", "On ten marks", "S-F8.ten.marks", "10"),
        ("12_8", "nine of the", "S-F8.ten.pairs_that_leak", "9"),
        ("12_8", "ten pairs of trustees", "S-F7.all_pairs.pairs", "10"),
        ("12_8", "two pairs always find it", "S-F8.ten.pairs_that_find", "2"),
        (
            "12_8",
            "trustees one and three",
            "S-F8.ten.pair_that_learns_nothing",
            "1,3",
        ),
        ("13_1", "On ten marks", "S-F8.ten.marks", "10"),
        ("13_1", "Two times five is ten", "S-F8.ten.zero_product_sum", "2,5,10"),
        ("13_1", "which lands on zero", "S-F8.ten.zero_product_mark", "0"),
        ("13_2", "multiplying by five", "S-F8.multiplier", "5"),
        ("13_2", "On ten marks", "S-F8.ten.marks", "10"),
        ("13_2", "five times any mark", "S-F8.multiplier", "5"),
        ("13_2", "lands on zero or five", "S-F8.ten.times5_values", "0,5"),
        ("13_2", "the ten marks crowd onto two", "S-F8.ten.times5_reached", "2"),
        ("13_3", "On ten marks", "S-F8.ten.marks", "10"),
        (
            "13_3",
            "nine, three, seven, one and five",
            "S-F8.line.shares",
            "9,3,7,1,5",
        ),
        ("13_3", "Trustee five's share", "S-F8.line.holder", "5"),
        ("13_3", "the secret plus five times the spin", "S-F8.line.holder", "5"),
        (
            "13_3",
            "five times the spin is zero or five",
            "S-F8.ten.times5_values",
            "0,5",
        ),
        ("13_3", "leaves two possible secrets", "S-F8.line.candidate_count", "2"),
        ("13_3", "here zero and five", "S-F8.line.candidates", "0,5"),
        ("13_4", "on eleven marks", "S-F1.marks", "11"),
        ("13_4", "every mark by five", "S-F8.multiplier", "5"),
        ("13_5", "All eleven", "S-F8.eleven.times5_reached", "11"),
        ("13_5", "One goes to five", "S-F8.eleven.times5_step.1", "1,5"),
        ("13_5", "two to ten", "S-F8.eleven.times5_step.2", "2,10"),
        (
            "13_5",
            "three to fifteen, which wraps round to four",
            "S-F8.eleven.times5_step.3",
            "3,15,4",
        ),
        ("13_5", "lands five further", "S-F8.multiplier", "5"),
        ("13_5", "reached exactly once", "S-F8.eleven.times5_each", "1"),
        ("13_5", "on eleven marks", "S-F1.marks", "11"),
        ("13_5", "trustee five's share", "S-F8.line.holder", "5"),
        ("13_6", "Ten is two times five", "S-F8.ten.split", "10,2,5"),
        (
            "13_6",
            "two and five can each bring part of the ten",
            "S-F8.ten.split",
            "10,2,5",
        ),
        ("13_6", "their product lands on zero", "S-F8.ten.zero_product_mark", "0"),
        ("13_6", "Eleven doesn't split", "S-F1.marks", "11"),
        (
            "13_6",
            "its only factors are one and itself",
            "S-F8.eleven.factors",
            "1,11",
        ),
        ("13_7", "a multiple of eleven", "S-F1.marks", "11"),
        ("13_7", "has eleven among its primes", "S-F1.marks", "11"),
        ("13_7", "from one to ten", "S-F8.eleven.nonzero_range", "1,10"),
        ("13_7", "on eleven marks", "S-F1.marks", "11"),
        (
            "13_8",
            "multiplying by five, five times their difference",
            "S-F8.multiplier",
            "5",
        ),
        (
            "13_8",
            "every multiplier from one to ten",
            "S-F8.eleven.nonzero_range",
            "1,10",
        ),
        ("13_8", "the eleven marks", "S-F1.marks", "11"),
        ("13_8", "On ten marks", "S-F8.ten.marks", "10"),
        (
            "13_8",
            "only one, three, seven and nine",
            "S-F8.ten.rows_that_shuffle",
            "1,3,7,9",
        ),
        ("13_9", "a position besides zero", "S-F4.secret_position", "0"),
        ("13_9", "For five trustees", "S-F1.trustees", "5"),
        (
            "13_9",
            "seven marks would already do",
            "S-F8.smallest_prime_for_five",
            "7",
        ),
        ("14_1", "trustees one, two and three", "S-F1.opening_trio", "1,2,3"),
        (
            "14_1",
            "brought their eight, three and one",
            "S-F1.opening_shares",
            "8,3,1",
        ),
        ("14_1", "They know three points", "S-F1.threshold", "3"),
        ("14_1", "how can three numbers", "S-F1.threshold", "3"),
        ("14_1", "give five", "S-F1.secret", "5"),
        (
            "14_2",
            "nineteen, forty-seven and eighty-nine",
            "S-F9.paper.heights",
            "19,47,89",
        ),
        ("14_2", "Three points pin a parabola down", "S-F1.threshold", "3"),
        ("14_2", "its height at zero", "S-F4.secret_position", "0"),
        ("14_3", "Trustee one's helper", "S-F9.helper.1.position", "1"),
        ("14_3", "is one at", "S-F9.helper.on", "1"),
        ("14_3", "at position one", "S-F9.helper.1.position", "1"),
        ("14_3", "and zero at positions", "S-F9.helper.off", "0"),
        ("14_3", "positions two and three", "S-F9.helper.1.zeros", "2,3"),
        ("14_3", "Trustee two's", "S-F9.helper.2.position", "2"),
        ("14_3", "at position two", "S-F9.helper.2.position", "2"),
        ("14_3", "trustee three's", "S-F9.helper.3.position", "3"),
        ("14_3", "at position three", "S-F9.helper.3.position", "3"),
        ("14_3", "each zero", "S-F9.helper.off", "0"),
        ("14_3", "the other two positions", "S-F9.helper.others", "2"),
        ("14_4", "nineteen times the first", "S-F9.helper.1.scaled", "19,1"),
        ("14_4", "forty-seven times the second", "S-F9.helper.2.scaled", "47,2"),
        ("14_4", "eighty-nine times the third", "S-F9.helper.3.scaled", "89,3"),
        ("14_4", "At position one", "S-F9.helper.1.position", "1"),
        (
            "14_4",
            "the second and third helpers",
            "S-F9.sum_at.1.zero_helpers",
            "2,3",
        ),
        ("14_4", "helpers are zero", "S-F9.helper.off", "0"),
        ("14_4", "the sum there is nineteen", "S-F9.sum_at.1", "19"),
        ("14_4", "At position two", "S-F9.helper.2.position", "2"),
        ("14_4", "it's forty-seven", "S-F9.sum_at.2", "47"),
        ("14_4", "at three", "S-F9.helper.3.position", "3"),
        ("14_4", "it's eighty-nine", "S-F9.sum_at.3", "89"),
        ("14_4", "all three points", "S-F1.threshold", "3"),
        ("14_5", "is zero at both", "S-F9.helper.off", "0"),
        ("14_5", "positions two and three", "S-F9.helper.1.zeros", "2,3"),
        (
            "14_6",
            "Position minus two, times position minus three",
            "S-F9.helper.1.zeros",
            "2,3",
        ),
        ("14_6", "the product is zero at both", "S-F9.helper.off", "0"),
        ("14_6", "At position one", "S-F9.helper.1.position", "1"),
        (
            "14_6",
            "minus one times minus two",
            "S-F9.helper.1.factors_at_own",
            "-1,-2",
        ),
        ("14_6", "which is two", "S-F9.helper.1.own", "2"),
        ("14_6", "the helper is one there", "S-F9.helper.on", "1"),
        ("14_7", "the other two", "S-F9.helper.others", "2"),
        ("14_7", "Trustee two's", "S-F9.helper.2.position", "2"),
        (
            "14_7",
            "position minus one, times position minus three",
            "S-F9.helper.2.zeros",
            "1,3",
        ),
        ("14_7", "which is minus one", "S-F9.helper.2.own", "-1"),
        ("14_7", "at position two", "S-F9.helper.2.position", "2"),
        ("14_7", "so divide by minus one", "S-F9.helper.2.own", "-1"),
        ("14_7", "Trustee three's", "S-F9.helper.3.position", "3"),
        (
            "14_7",
            "position minus one, times position minus two",
            "S-F9.helper.3.zeros",
            "1,2",
        ),
        ("14_7", "which is two", "S-F9.helper.3.own", "2"),
        ("14_7", "at position three", "S-F9.helper.3.position", "3"),
        ("14_8", "the dealer's three points", "S-F1.threshold", "3"),
        (
            "14_8",
            "Only one curve of that form can do that",
            "S-F6.paper.curves",
            "1",
        ),
        ("14_8", "Its height at zero", "S-F4.secret_position", "0"),
        ("15_1", "at position zero", "S-F4.secret_position", "0"),
        ("15_1", "Trustee one's", "S-F9.helper.1.position", "1"),
        (
            "15_1",
            "minus two times minus three",
            "S-F9.helper.1.factors_at_zero",
            "-2,-3",
        ),
        ("15_1", "which is six", "S-F9.helper.1.at_zero", "6"),
        ("15_1", "so three", "S-F9.helper.1.weight", "3"),
        ("15_1", "trustee one's weight", "S-F9.helper.1.position", "1"),
        ("15_1", "counts for at zero", "S-F4.secret_position", "0"),
        ("15_2", "Trustee two's", "S-F9.helper.2.position", "2"),
        (
            "15_2",
            "minus one times minus three",
            "S-F9.helper.2.factors_at_zero",
            "-1,-3",
        ),
        ("15_2", "which is three", "S-F9.helper.2.at_zero", "3"),
        ("15_2", "divided by minus one", "S-F9.helper.2.own", "-1"),
        ("15_2", "its weight is minus three", "S-F9.helper.2.weight", "-3"),
        ("15_2", "Trustee three's", "S-F9.helper.3.position", "3"),
        (
            "15_2",
            "minus one times minus two",
            "S-F9.helper.3.factors_at_zero",
            "-1,-2",
        ),
        ("15_2", "which is two", "S-F9.helper.3.at_zero", "2"),
        ("15_2", "so one", "S-F9.helper.3.weight", "1"),
        (
            "15_2",
            "the positions, one, two and three",
            "S-F1.opening_trio",
            "1,2,3",
        ),
        (
            "15_3",
            "three times nineteen, minus three times forty-seven, plus one times eighty-nine",
            "S-F9.paper.weighted",
            "3,19,-3,47,1,89",
        ),
        (
            "15_3",
            "fifty-seven, minus a hundred and forty-one, plus eighty-nine",
            "S-F9.paper.terms",
            "57,-141,89",
        ),
        ("15_3", "which is five", "S-F9.paper.sum", "5"),
        ("15_4", "eight, three and one", "S-F1.opening_shares", "8,3,1"),
        (
            "15_5",
            "Three times eight, minus three times three, plus one times one",
            "S-F9.cards.weighted",
            "3,8,-3,3,1,1",
        ),
        ("15_5", "is sixteen", "S-F9.sum_unwrapped", "16"),
        ("15_5", "sixteen lands on five", "S-F9.sum_lands", "16,5"),
        ("15_5", "The dial turns to five", "S-F9.secret", "5"),
        ("15_6", "Nineteen is eight and one eleven", "S-F9.split.1", "19,8,1"),
        (
            "15_6",
            "forty-seven is three and four elevens",
            "S-F9.split.2",
            "47,3,4",
        ),
        ("15_6", "eighty-nine is one and eight elevens", "S-F9.split.3", "89,1,8"),
        ("15_7", "sixteen is five and one eleven", "S-F9.sum_split", "16,5,1"),
        (
            "15_7",
            "On the dial, they're three, eight and one",
            "S-F9.weights_on_dial",
            "3,8,1",
        ),
        ("16_1", "trustees two, four and five", "S-F10.trio", "2,4,5"),
        (
            "16_2",
            "Trustee two's helper starts as position minus four, times position minus five",
            "S-F10.trio",
            "2,4,5",
        ),
        (
            "16_3",
            "Trustee four uses position minus two, times position minus five",
            "S-F10.trio",
            "2,4,5",
        ),
        (
            "16_4",
            "Trustee five uses position minus two, times position minus four",
            "S-F10.trio",
            "2,4,5",
        ),
        ("16_1", "three, two and six", "S-F10.trio_shares", "3,2,6"),
        ("16_6", "three, two and six", "S-F10.trio_shares", "3,2,6"),
        (
            "16_7",
            "ten thirds of three is ten, minus five times two is minus ten, and eight thirds of six is sixteen",
            "S-F10.trio_shares",
            "3,2,6",
        ),
        (
            "16_1",
            "forty-seven, a hundred and forty-five, and two hundred and fifteen",
            "S-F10.paper.heights",
            "47,145,215",
        ),
        (
            "16_2",
            "product is six, so divide the whole helper by six",
            "S-F10.helper_at_own",
            "6,-2,3",
        ),
        ("16_3", "product is minus two", "S-F10.helper_at_own", "6,-2,3"),
        ("16_3", "Dividing ten by minus two", "S-F10.helper_at_own", "6,-2,3"),
        ("16_4", "product is three", "S-F10.helper_at_own", "6,-2,3"),
        ("16_2", "product is twenty", "S-F10.helper_at_zero", "20,10,8"),
        (
            "16_2",
            "twenty divided by six: ten thirds",
            "S-F10.helper_at_zero",
            "20,10,8",
        ),
        ("16_3", "it is ten", "S-F10.helper_at_zero", "20,10,8"),
        ("16_3", "Dividing ten by minus two", "S-F10.helper_at_zero", "20,10,8"),
        ("16_4", "it is eight", "S-F10.helper_at_zero", "20,10,8"),
        ("16_2", "make it one there", "S-F9.helper.on", "1"),
        ("16_2", "At zero", "S-F4.secret_position", "0"),
        ("16_3", "at zero", "S-F4.secret_position", "0"),
        ("16_4", "at zero", "S-F4.secret_position", "0"),
        ("16_5", "sum at zero", "S-F4.secret_position", "0"),
        ("16_2", "ten thirds", "S-F10.weights", "10/3,-5,8/3"),
        ("16_3", "weight, minus five", "S-F10.weights", "10/3,-5,8/3"),
        ("16_4", "eight thirds", "S-F10.weights", "10/3,-5,8/3"),
        (
            "16_5",
            "ten thirds of forty-seven, minus five times a hundred and forty-five, plus eight thirds of two hundred and fifteen",
            "S-F10.weights",
            "10/3,-5,8/3",
        ),
        (
            "16_8",
            "three and six are multiples of three",
            "S-F10.weights",
            "10/3,-5,8/3",
        ),
        (
            "16_5",
            "ten thirds of forty-seven, minus five times a hundred and forty-five, plus eight thirds of two hundred and fifteen",
            "S-F10.paper.heights",
            "47,145,215",
        ),
        ("16_5", "exactly five", "S-F10.paper.sum", "5"),
        (
            "16_7",
            "ten thirds of three is ten, minus five times two is minus ten, and eight thirds of six is sixteen",
            "S-F10.fraction_terms",
            "10,-10,16",
        ),
        ("16_7", "sum is sixteen", "S-F10.fraction_sum", "16"),
        ("16_7", "lands on five", "S-F10.secret", "5"),
        ("17_1", "spins zero and one", "S-F10.other.spins", "0,1"),
        ("17_1", "nine, twenty-one and thirty", "S-F10.other.heights", "9,21,30"),
        ("17_1", "nine, ten and eight", "S-F10.other.shares", "9,10,8"),
        (
            "17_2",
            "ten thirds, minus five and eight thirds",
            "S-F10.weights",
            "10/3,-5,8/3",
        ),
        ("17_2", "four thirds", "S-F10.other.fraction_sum", "4/3"),
        ("17_3", "eighty", "S-F10.fractional_spin_pairs", "80"),
        ("17_3", "hundred and twenty-one", "S-F6.spin_pairs", "121"),
        ("17_4", "Three times", "S-F10.clear", "3"),
        (
            "17_4",
            "ten, minus fifteen and eight",
            "S-F10.cleared_weights",
            "10,-15,8",
        ),
        ("17_4", "by three", "S-F10.clear", "3"),
        ("17_4", "sum three times", "S-F10.clear", "3"),
        ("17_5", "nine, ten and eight", "S-F10.other.shares", "9,10,8"),
        (
            "17_5",
            "ninety, minus a hundred and fifty, plus sixty-four",
            "S-F10.other.cleared_terms",
            "90,-150,64",
        ),
        ("17_5", "which is four", "S-F10.other.cleared_sum", "4"),
        ("17_5", "three times", "S-F10.clear", "3"),
        ("17_6", "three", "S-F10.clear", "3"),
        ("17_6", "four", "S-F10.other.cleared_lands", "4"),
        ("17_7", "Five", "S-F10.thrice", "5"),
        ("17_7", "Three", "S-F10.clear", "3"),
        ("17_7", "fifteen", "S-F10.thrice_unwrapped", "15"),
        ("17_7", "four", "S-F10.other.cleared_lands", "4"),
        ("17_7", "eleven marks", "S-F1.marks", "11"),
        ("17_2", "Four thirds", "S-F10.other.fraction_sum", "4/3"),
        ("17_6", "three times", "S-F10.clear", "3"),
        ("17_6", "times three", "S-F10.clear", "3"),
        ("17_7", "Three times five", "S-F10.thrice_unwrapped", "15"),
        ("17_7", "times three", "S-F10.clear", "3"),
        ("17_7", "exactly once", "S-F10.times3", "0,3,6,9,1,4,7,10,2,5,8"),
        ("18_1", "three, two and six", "S-F10.trio_shares", "3,2,6"),
        (
            "18_1",
            "Ten times three, minus fifteen times two, plus eight times six",
            "S-F10.cleared_terms",
            "30,-30,48",
        ),
        ("18_1", "forty-eight", "S-F10.cleared_sum", "48"),
        ("18_1", "four", "S-F10.cleared_lands", "4"),
        ("18_1", "times three", "S-F10.clear", "3"),
        ("18_1", "five", "S-F10.secret", "5"),
        ("18_2", "times three", "S-F10.clear", "3"),
        ("18_2", "one", "S-F10.times3", "0,3,6,9,1,4,7,10,2,5,8"),
        ("18_3", "Four", "S-F10.third", "4"),
        ("18_3", "Three times four is twelve", "S-F10.partner_product", "12"),
        ("18_3", "one", "S-F10.times3", "0,3,6,9,1,4,7,10,2,5,8"),
        ("18_3", "by three", "S-F10.clear", "3"),
        ("18_3", "by four", "S-F10.third", "4"),
        ("18_4", "four", "S-F10.third", "4"),
        ("18_4", "sixteen", "S-F10.undo_product", "16"),
        ("18_4", "five", "S-F10.secret", "5"),
        ("18_4", "dividing by three", "S-F10.clear", "3"),
        ("18_5", "Ten thirds", "S-F10.weights", "10/3,-5,8/3"),
        ("18_5", "ten times four", "S-F10.weight_products", "40,-5,32"),
        ("18_5", "forty", "S-F10.weight_products", "40,-5,32"),
        ("18_5", "seven", "S-F10.weights_on_dial", "7,6,10"),
        ("18_5", "Minus five", "S-F10.weights", "10/3,-5,8/3"),
        ("18_5", "six", "S-F10.weights_on_dial", "7,6,10"),
        ("18_5", "Eight thirds", "S-F10.weights", "10/3,-5,8/3"),
        ("18_5", "eight times four", "S-F10.weight_products", "40,-5,32"),
        ("18_5", "thirty-two", "S-F10.weight_products", "40,-5,32"),
        ("18_5", "ten", "S-F10.weights_on_dial", "7,6,10"),
        (
            "18_6",
            "Seven times three, plus six times two, plus ten times six",
            "S-F10.dial_terms",
            "21,12,60",
        ),
        ("18_6", "ninety-three", "S-F10.sum_unwrapped", "93"),
        ("18_6", "eight elevens", "S-F10.whole_turns", "8"),
        ("18_6", "five", "S-F10.secret", "5"),
        (
            "18_7",
            "zero",
            "S-F10.inverses",
            "1:1,2:6,3:4,4:3,5:9,6:2,7:8,8:7,9:5,10:10",
        ),
        ("18_7", "four", "S-F10.third", "4"),
        ("18_7", "eleven marks", "S-F1.marks", "11"),
        (
            "18_7",
            "one",
            "S-F10.inverses",
            "1:1,2:6,3:4,4:3,5:9,6:2,7:8,8:7,9:5,10:10",
        ),
        (
            "18_7",
            "exactly once",
            "S-F10.inverses",
            "1:1,2:6,3:4,4:3,5:9,6:2,7:8,8:7,9:5,10:10",
        ),
        ("18_8", "ten marks", "S-F8.ten.marks", "10"),
        (
            "18_8",
            "two, four, five, six and eight",
            "S-F10.ten.noninvertible",
            "2,4,5,6,8",
        ),
        ("19_1", "three", "S-F1.threshold", "3"),
        ("19_1", "of the five", "S-F1.trustees", "5"),
        ("19_1", "ten ways", "S-F11.trios", "10"),
        ("19_1", "all ten", "S-F11.trios_giving_secret", "10"),
        ("19_1", "rebuild five", "S-F10.secret", "5"),
        ("19_2", "two and four", "S-F11.roots", "2,4"),
        ("19_2", "trustee five", "S-F10.trio", "2,4,5"),
        ("19_4", "trustee five", "S-F10.trio", "2,4,5"),
        ("19_5", "Three", "S-F1.threshold", "3"),
        ("19_5", "trustee five", "S-F10.trio", "2,4,5"),
        ("19_5", "nine", "S-F11.trial_shares", "3,2,9"),
        ("19_5", "instead of six", "S-F10.trio_shares", "3,2,6"),
        ("19_5", "difference is three", "S-F11.third_difference", "3"),
        ("19_5", "two and four", "S-F11.roots", "2,4"),
        (
            "19_6",
            "position minus two, times position minus four",
            "S-F11.roots",
            "2,4",
        ),
        ("19_6", "minus four", "S-F11.roots", "2,4"),
        ("19_8", "at most one zero", "S-F11.line_most_zeros", "1"),
        ("19_9", "at most two zeros", "S-F11.most_zeros", "2"),
        ("19_9", "three matching positions", "S-F1.threshold", "3"),
        ("19_10", "three distinct trustees", "S-F1.threshold", "3"),
        ("19_10", "all ten groups", "S-F11.trios_giving_secret", "10"),
        ("19_10", "return five", "S-F10.secret", "5"),
        ("20_1", "ten-mark", "S-F8.ten.marks", "10"),
        ("20_1", "ten marks", "S-F8.ten.marks", "10"),
        ("20_1", "one, three and five", "S-F11.ten.trio", "1,3,5"),
        ("20_1", "nine, nine and five", "S-F11.ten.trio_shares", "9,9,5"),
        ("20_2", "five plus five times position", "S-F11.ten.ghost", "5,5,0"),
        ("20_2", "ten, twenty and thirty", "S-F11.ten.ghost.heights", "10,20,30"),
        ("20_2", "adds five", "S-F11.ten.ghost", "5,5,0"),
        ("20_4", "nine, nine and five", "S-F11.ten.trio_shares", "9,9,5"),
        ("20_4", "secret becomes zero", "S-F11.ten.moved_recipe", "0,2,7"),
        (
            "20_4",
            "zero plus two times position plus seven times position squared",
            "S-F11.ten.moved_recipe",
            "0,2,7",
        ),
        ("20_5", "five or zero", "S-F11.ten.trio_secrets", "0,5"),
        ("20_6", "ten marks", "S-F8.ten.marks", "10"),
        ("20_6", "five trustees", "S-F1.trustees", "5"),
        ("20_6", "Nine", "S-F11.ten.trios_that_pin", "9"),
        ("20_6", "ten groups", "S-F11.trios", "10"),
        ("20_7", "Eleven marks", "S-F1.marks", "11"),
        ("20_7", "at most one zero", "S-F11.line_most_zeros", "1"),
        ("20_7", "ten-mark", "S-F8.ten.marks", "10"),
        ("21_1", "eleven-mark", "S-F1.marks", "11"),
        ("21_1", "three genuine shares", "S-F1.threshold", "3"),
        ("21_1", "Any two", "S-F7.pair.count", "2"),
        ("21_1", "four and five", "S-F1.pair", "4,5"),
        ("21_1", "two and six", "S-F1.pair_shares", "2,6"),
        ("21_3", "exactly one", "S-F7.ways_each", "1"),
        ("21_3", "one hundred twenty-one", "S-F6.spin_pairs", "121"),
        ("21_5", "Eight", "S-F12.nonzero_last.excluded", "8"),
        (
            "21_5",
            "eight plus four times position",
            "S-F12.straight_recipe",
            "8,4,0",
        ),
        ("21_5", "trustee four", "S-F1.pair", "4,5"),
        ("21_5", "twenty-four", "S-F12.straight_heights", "24,28"),
        ("21_5", "wraps to two", "S-F1.pair_shares", "2,6"),
        ("21_5", "trustee five", "S-F1.pair", "4,5"),
        ("21_5", "twenty-eight", "S-F12.straight_heights", "24,28"),
        ("21_5", "wraps to six", "S-F1.pair_shares", "2,6"),
        ("21_5", "second spin is zero", "S-F12.straight_recipe", "8,4,0"),
        ("21_6", "secret eight", "S-F12.nonzero_last.excluded", "8"),
        ("21_6", "eight is impossible", "S-F12.nonzero_last.excluded", "8"),
        ("21_6", "other ten", "S-F12.nonzero_last.left", "10"),
        ("21_7", "eleven marks", "S-F1.marks", "11"),
        ("21_7", "one time in eleven", "S-F12.guess_chance", "1/11"),
        ("21_7", "Two shares", "S-F7.pair.count", "2"),
        ("22_1", "Trustee three", "S-F12.liar", "3"),
        ("22_1", "two instead of one", "S-F12.liar_hands_in", "2"),
        ("22_2", "three, minus three, and one", "S-F9.weights", "3,-3,1"),
        (
            "22_2",
            "Twenty-four minus nine plus two gives seventeen",
            "S-F12.liar_result_unwrapped",
            "17",
        ),
        ("22_2", "six", "S-F12.liar_result", "6"),
        ("22_2", "three submitted values", "S-F1.threshold", "3"),
        ("22_3", "six", "S-F12.liar_result", "6"),
        ("22_4", "fourth card", "S-F12.fourth", "4"),
        ("22_4", "trustee four", "S-F12.fourth", "4"),
        ("22_4", "first three", "S-F1.threshold", "3"),
        ("22_4", "holds five", "S-F12.fourth.predicted", "5"),
        ("22_4", "holds two", "S-F12.fourth.actual", "2"),
        ("22_5", "other three", "S-F1.threshold", "3"),
        ("22_5", "trustees one, two, three, and four", "S-F12.fourth", "4"),
        (
            "22_5",
            "eight, three, five, and six",
            "S-F12.four.secret_if_liar",
            "8,3,5,6",
        ),
        ("22_5", "four cards", "S-F12.fourth", "4"),
        ("22_6", "fifth trustee", "S-F1.trustees", "5"),
        ("22_6", "six", "S-F6.shares", "8,3,1,2,6"),
        ("22_7", "fifth position", "S-F1.trustees", "5"),
        ("22_7", "five cards", "S-F1.trustees", "5"),
        ("22_7", "four of", "S-F12.fourth", "4"),
        ("22_7", "secret is five", "S-F12.five.curve_secret", "5"),
        ("22_7", "one at trustee three", "S-F12.liar_true", "1"),
        ("22_7", "says two", "S-F12.liar_hands_in", "2"),
        ("22_8", "only one", "S-F12.five.curves_through_four", "1"),
        ("22_8", "four of these five", "S-F1.trustees", "5"),
        ("22_8", "at least three positions", "S-F12.five.min_common", "3"),
        ("22_9", "three points", "S-F1.threshold", "3"),
        ("22_9", "three distinct positions", "S-F1.threshold", "3"),
        ("22_9", "at least four", "S-F12.fourth", "4"),
    ];

    /// Number words said as an article, not a count, each with its line.
    const NOT_A_COUNT: &[(&str, &str)] = &[
        ("23_4", "away from zero"), // nonzero position condition
        ("23_5", "one fewer than"), // threshold relation
        (
            "23_5",
            "then add a times position, b times position squared, and c times position cubed",
        ), // symbolic polynomial definition
        ("23_5", "land on zero"),   // allowed coefficient support
        ("23_8", "Subtract two candidate cubics"), // hypothetical pair for uniqueness proof
        ("23_8", "zero everywhere"), // zero-polynomial exception
        ("23_9", "k minus one pieces"), // verbatim symbolic source excerpt
        ("22_1", "at most one false share"), // explicit error model assumption
        ("22_6", "at most one submitted card"), // same error model assumption
        ("22_8", "two different curves"), // hypothetical pair for uniqueness proof
        ("22_8", "those two positions"), // two hypothetical missed positions
        ("22_9", "two quadratics"), // hypothetical pair in recalled uniqueness theorem
        ("22_9", "at most one false card"), // same error model assumption
        ("22_10", "at most one false share"), // restated conditional guarantee
        ("21_2", "zero included"),  // definition of the allowed spin support
        ("21_4", "second spin of zero"), // changed sampling rule posed before outcome
        ("21_4", "Which one"),      // prediction question, not a measured count
        ("21_7", "zero allowed"),   // restores the original spin support
        ("20_2", "lands on zero"),  // full-turn wrap identity at the chosen trustees
        ("20_7", "not zero everywhere"), // excludes the zero polynomial
        ("19_3", "the difference lands on zero"), // definition of matching evaluations
        ("19_4", "make the difference zero"), // question before the count is revealed
        ("19_5", "that extra zero"), // the hypothetical third match
        ("19_6", "those two factors"), // explicitly displayed factorization
        ("19_6", "lands on zero exactly"), // product-zero identity
        ("19_7", "a product is zero"), // generic cancellation hypothesis
        ("19_7", "other factor must be zero"), // generic cancellation consequence
        ("19_7", "Two nonzero marks"), // generic pair in the product-zero argument
        ("19_8", "zero at a mark"), // root hypothesis
        ("19_8", "value at r is zero"), // polynomial remainder identity
        ("19_8", "position minus r"), // symbolic factor, not a numeric measurement
        ("19_8", "first factor is zero"), // x-r vanishes at r
        ("19_8", "not zero everywhere"), // exclusion of the zero polynomial
        ("19_9", "zero everywhere"), // zero-polynomial exceptional case
        ("19_10", "position zero"), // definition of the secret as constant term
        ("17_3", "A third of an eleven is not a whole eleven"), // fractional-multiple limitation
        ("17_4", "Clear the thirds first"), // order of operations
        ("01_1", "guard one vault"),
        ("02_1", "The first idea"),
        ("02_4", "less than zero"), // the definition of a nonnegative piece
        ("02_6", "a second way"),
        ("02_7", "fixed first"),
        ("05_1", "every one of the"),
        ("09_7", "the secret plus spins times powers"), // the rule's shape, no count
        ("10_4", "Each one is a dealing"),
        ("11_3", "every one of the"),
        ("12_3", "every one of them"),
        ("12_5", "rid of the first spin"), // the spin's name, as in "the first spin cancels"
        ("12_5", "Take one from the other"),
        ("13_1", "that aren't zero can multiply to zero"), // the question, for any marks
        ("13_7", "Landing on zero means"),                 // the definition
        ("13_7", "in only one way"),                       // the theorem, for every number
        ("13_7", "only if one of the marks does"),
        ("13_7", "lands on zero only when one of its factors is zero"),
        ("13_8", "if two marks gave the same answer"), // any two marks
        ("13_8", "would land on zero"),
        ("13_8", "each one only shuffles"),
        ("13_9", "each one has"),
        ("13_9", "the secret is one of its marks"),
        ("14_2", "so the one through these"),    // the curve, not a count
        ("14_4", "zero times anything is zero"), // the rule, for any height
        ("14_6", "Each factor is zero at its own position"), // either factor
        ("15_7", "wrapping first changes nothing"), // the order, not a count
    ];

    const NUMBER_WORDS: &[&str] = &[
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
        "thirty",
        "forty",
        "fifty",
        "sixty",
        "seventy",
        "eighty",
        "ninety",
        "hundred",
        "thousand",
        "first",
        "second",
        "third",
        "half",
        "twice",
        "double",
        "once",
        "times",
        "minus",
        "plus",
    ];

    /// The number words of `text`, each with its char offset.
    fn number_words(text: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        let mut word = String::new();
        let mut start = 0;
        for (i, c) in text.chars().chain(std::iter::once(' ')).enumerate() {
            if c.is_alphabetic() {
                if word.is_empty() {
                    start = i;
                }
                word.extend(c.to_lowercase());
            } else if !word.is_empty() {
                if NUMBER_WORDS.contains(&word.as_str()) {
                    out.push((start, std::mem::take(&mut word)));
                }
                word.clear();
            }
        }
        out
    }

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        for (id, words, key, value) in SPOKEN {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        for (id, words) in NOT_A_COUNT {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(words), "{id} no longer says {words:?}");
        }
        // and every number word of every line sits inside a phrase above
        for (id, text) in script.lines() {
            let covered: Vec<(usize, usize)> = SPOKEN
                .iter()
                .map(|(i, w, ..)| (*i, *w))
                .chain(NOT_A_COUNT.iter().copied())
                .filter(|(i, _)| i == id)
                .flat_map(|(_, w)| {
                    text.match_indices(w).map(move |(byte, _)| {
                        let from = text[..byte].chars().count();
                        (from, from + w.chars().count())
                    })
                })
                .collect();
            for (at, word) in number_words(text) {
                assert!(
                    covered.iter().any(|&(a, b)| a <= at && at < b),
                    "{id} says {word:?} with no row in SPOKEN or NOT_A_COUNT: {text}"
                );
            }
        }
    }

    /// The scan finds number words where they are, whatever their case, and
    /// not inside other words ("someone", "tone").
    #[test]
    fn the_number_scan_finds_words_not_pieces() {
        let found = number_words("One bribe, someone's tone, and THREE.");
        assert_eq!(found, vec![(0, "one".to_owned()), (31, "three".to_owned())]);
        assert!(number_words("Nothing here.").is_empty());
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// All planned chapters are implemented, row for row.
    #[test]
    fn the_production_chapters_are_the_drafts() {
        let draft = explainer_kit::storyboard::parse(include_str!("../docs/storyboard-draft.md")).unwrap();
        let production = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        assert_eq!(draft.len(), production.len(), "every draft chapter is implemented");
        for (p, d) in production.iter().zip(&draft) {
            assert_eq!(p, d, "chapter {} differs from the draft", p.key);
        }
    }
}
