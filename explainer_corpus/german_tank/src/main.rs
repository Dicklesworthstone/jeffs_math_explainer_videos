//! How Much Can Five Serial Numbers Reveal?: five board-game boxes, the one
//! thing their numbers prove, fair handfuls drawn by computer from a
//! thousand and from seventy, the gap recipe found by counting the gaps, the
//! question and its one assumption made exact, every pair from a company of
//! five boxes averaging exactly five, the gaps trading places, the proof
//! that it is right on average for a fair handful of any count, how far one
//! estimate can be from the count, the range ninety-five percent confidence
//! gives, a rival recipe that doubles the average, why the largest number
//! says everything a fair handful can, how averaging inside the groups of
//! the largest number turns the doubling recipe into the gap recipe, and why
//! no unbiased recipe misses by less; how a biased shop breaks the estimate;
//! then (in docs/storyboard-draft.md) other assumptions and what it does not
//! promise, rendered
//! end to end by franken_manim's native Rust front door and the corpus's
//! shared kit.
//!
//! ```text
//! tank facts                         # every number the video shows or says (key<TAB>value)
//! tank script                        # the narration (generated from docs/storyboard.md)
//! tank render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! tank help                          # the full option list
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
/// The chapters written in full but not yet implemented, in the same
/// format, and the authored pause intentions for every chapter.
#[cfg(test)]
pub(crate) const DRAFT: &str = include_str!("../docs/storyboard-draft.md");
#[cfg(test)]
pub(crate) const PAUSES: &str = include_str!("../docs/pauses.tsv");

const EXPLAINER: Explainer = Explainer {
    crate_name: "german_tank",
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
    /// for and the value those words denote. A changed value or word breaks
    /// this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        // 01: five boxes
        ("01_2", "Five friends", "G-F1.handful", "5"),
        ("01_3", "counting up from one", "G-F1.first", "1"),
        ("01_3", "The first box made is number one", "G-F1.first", "1"),
        ("01_3", "the next is number two", "G-F1.second", "2"),
        (
            "01_4",
            "Fourteen, twenty-seven, thirty-three, fifty-one and sixty",
            "G-F1.numbers",
            "14,27,33,51,60",
        ),
        ("01_7", "It made at least sixty", "G-F1.at_least", "60"),
        ("01_7", "Box sixty exists", "G-F1.largest", "60"),
        ("01_7", "boxes one to sixty", "G-F1.first", "1"),
        ("01_7", "boxes one to sixty", "G-F1.at_least", "60"),
        ("01_8", "Above sixty", "G-F1.at_least", "60"),
        ("01_8", "right after sixty", "G-F1.at_least", "60"),
        ("01_9", "these five numbers", "G-F1.handful", "5"),
        ("01_9", "a thousand boxes", "G-F1.thousand", "1000"),
        ("01_9", "would five boxes from a thousand", "G-F1.handful", "5"),
        ("01_9", "would five boxes from a thousand", "G-F1.thousand", "1000"),
        // 02: a thousand boxes
        ("02_1", "how the five were chosen", "G-F1.handful", "5"),
        ("02_2", "Five boxes come out", "G-F1.handful", "5"),
        (
            "02_2",
            "any five are exactly as likely as any other five",
            "G-F1.handful",
            "5",
        ),
        ("02_4", "a thousand boxes", "G-F2.big", "1000"),
        ("02_4", "five numbers", "G-F1.handful", "5"),
        ("02_5", "ten fair handfuls", "G-F2.draws", "10"),
        ("02_5", "from a thousand", "G-F2.big", "1000"),
        ("02_5", "past six hundred", "G-F2.over", "600"),
        ("02_5", "past six hundred", "G-F2.big_largest_over_600", "true"),
        ("02_6", "Five numbers", "G-F1.handful", "5"),
        ("02_6", "sixty or less", "G-F1.largest", "60"),
        ("02_6", "fewer than one handful in a million", "G-F2.million", "1000000"),
        (
            "02_6",
            "fewer than one handful in a million",
            "G-F2.big_all_at_most_60_under_one_in_a_million",
            "true",
        ),
        ("02_7", "a company of seventy", "G-F2.small", "70"),
        ("02_7", "Nearly half", "G-F2.small_all_at_most_60.percent", "45"),
        ("02_7", "sixty or below", "G-F1.largest", "60"),
        ("02_7", "Three of these ten", "G-F2.small_fit_shown", "3"),
        ("02_7", "Three of these ten", "G-F2.draws", "10"),
        ("02_8", "Numbers from a thousand", "G-F2.big", "1000"),
        ("02_8", "over a hundred and fifty apart", "G-F2.spacing_over", "150"),
        (
            "02_8",
            "over a hundred and fifty apart",
            "G-F2.big_spacing_over_150",
            "true",
        ),
        ("02_8", "From seventy", "G-F2.small", "70"),
        ("02_8", "about twelve apart", "G-F2.small_spacing_about", "12"),
        // 03: counting the gaps
        ("03_1", "Below fourteen", "G-F3.first_seen", "14"),
        ("03_1", "there are thirteen", "G-F3.first_gap", "13"),
        ("03_2", "Between fourteen", "G-F3.first_seen", "14"),
        ("03_2", "and twenty-seven", "G-F3.second_seen", "27"),
        ("03_2", "twelve more", "G-F3.second_gap", "12"),
        (
            "03_3",
            "Then five, then seventeen, then eight",
            "G-F3.last_gaps",
            "5,17,8",
        ),
        ("03_4", "Five gaps hold", "G-F3.gap_count", "5"),
        ("03_4", "fifty-five unseen boxes", "G-F3.unseen", "55"),
        ("03_4", "eleven per gap", "G-F3.average_gap", "11"),
        ("03_5", "one more gap", "G-F3.top_gaps", "1"),
        ("03_5", "above sixty", "G-F1.largest", "60"),
        ("03_6", "one more space", "G-F3.top_gaps", "1"),
        ("03_8", "About seventy-one", "G-F3.estimate", "71"),
        ("03_8", "Sixty, plus", "G-F1.largest", "60"),
        ("03_8", "plus one average gap", "G-F3.top_gaps", "1"),
        ("03_8", "of eleven", "G-F3.average_gap", "11"),
        ("03_10", "not about seventy-one", "G-F3.estimate", "71"),
        // 04: what we assume
        ("04_1", "sixty, seventy-one or five hundred", "G-F1.largest", "60"),
        ("04_1", "sixty, seventy-one or five hundred", "G-F3.estimate", "71"),
        ("04_1", "sixty, seventy-one or five hundred", "G-F4.far_count", "500"),
        ("04_2", "friends' five boxes", "G-F1.handful", "5"),
        ("04_2", "every handful of five", "G-F1.handful", "5"),
        ("04_4", "Of the sixty boxes", "G-F1.largest", "60"),
        ("04_4", "from one to sixty", "G-F1.first", "1"),
        ("04_4", "from one to sixty", "G-F1.largest", "60"),
        ("04_4", "five are the friends'", "G-F1.handful", "5"),
        ("04_4", "sixty minus five", "G-F1.largest", "60"),
        ("04_4", "sixty minus five", "G-F1.handful", "5"),
        ("04_4", "fifty-five", "G-F3.unseen", "55"),
        ("04_5", "With any five numbers", "G-F1.handful", "5"),
        ("04_5", "hold the largest minus five", "G-F1.handful", "5"),
        ("04_5", "plus the largest minus five, over five", "G-F1.handful", "5"),
        (
            "04_5",
            "sixty plus fifty-five over five, seventy-one",
            "G-F1.largest",
            "60",
        ),
        (
            "04_5",
            "sixty plus fifty-five over five, seventy-one",
            "G-F3.unseen",
            "55",
        ),
        (
            "04_5",
            "sixty plus fifty-five over five, seventy-one",
            "G-F1.handful",
            "5",
        ),
        (
            "04_5",
            "sixty plus fifty-five over five, seventy-one",
            "G-F3.estimate",
            "71",
        ),
        ("04_6", "if the five numbers were", "G-F1.handful", "5"),
        (
            "04_6",
            "one, two, three, four and five",
            "G-F4.edge_handful",
            "1,2,3,4,5",
        ),
        ("04_7", "Five. There are no gaps", "G-F4.edge_estimate", "5"),
        ("04_7", "Five. There are no gaps", "G-F4.edge_gaps", "0,0,0,0,0"),
        // 05: every handful
        ("05_1", "A company of seventy-one boxes", "G-F5.company", "71"),
        ("05_1", "over thirteen million", "G-F5.millions", "13"),
        ("05_1", "over thirteen million", "G-F5.handfuls_over_millions", "true"),
        ("05_1", "handfuls of five", "G-F1.handful", "5"),
        ("05_1", "made just five boxes", "G-F5.boxes", "5"),
        ("05_1", "you see two", "G-F5.seen", "2"),
        ("05_2", "ten possible pairs", "G-F5.pairs", "10"),
        ("05_2", "from one and two", "G-F5.first_pair", "1,2"),
        ("05_2", "up to four and five", "G-F5.last_pair", "4,5"),
        ("05_3", "the pair two and five", "G-F5.example_high_pair", "2,5"),
        ("05_3", "Below five", "G-F5.example_high_largest", "5"),
        ("05_3", "one, three and four", "G-F5.example_high_unseen", "1,3,4"),
        ("05_3", "That's three boxes", "G-F5.example_high_unseen_count", "3"),
        ("05_3", "five minus two", "G-F5.example_high_largest", "5"),
        ("05_3", "five minus two", "G-F5.seen", "2"),
        ("05_3", "in two gaps", "G-F5.seen", "2"),
        ("05_3", "half of the largest minus two", "G-F5.seen", "2"),
        ("05_4", "For two and five", "G-F5.example_high_pair", "2,5"),
        ("05_4", "five plus three halves", "G-F5.example_high_largest", "5"),
        ("05_4", "five plus three halves", "G-F5.example_high_unseen_count", "3"),
        ("05_4", "six and a half", "G-F5.example_high_decimal", "6.5"),
        ("05_4", "For one and two", "G-F5.example_low_pair", "1,2"),
        ("05_4", "just two", "G-F5.example_low", "2"),
        ("05_5", "all ten estimates", "G-F5.pairs", "10"),
        ("05_5", "the ten average", "G-F5.pairs", "10"),
        ("05_6", "Exactly five", "G-F5.average", "5"),
        ("05_6", "averages only four", "G-F5.largest_average", "4"),
        // 06: gaps that trade places
        ("06_1", "the ten pairs", "G-F5.pairs", "10"),
        ("06_1", "from five boxes", "G-F5.boxes", "5"),
        ("06_1", "three boxes unseen", "G-F6.unseen", "3"),
        ("06_1", "Its two numbers", "G-F5.seen", "2"),
        ("06_1", "into three gaps", "G-F6.positions", "3"),
        ("06_1", "between the two", "G-F5.seen", "2"),
        ("06_2", "One and two leave", "G-F5.first_pair", "1,2"),
        ("06_2", "zero, zero and three", "G-F6.low_gaps", "0,0,3"),
        ("06_2", "Two and five leave", "G-F5.example_high_pair", "2,5"),
        ("06_2", "one, two and zero", "G-F6.high_gaps", "1,2,0"),
        ("06_2", "its own three gaps", "G-F6.positions", "3"),
        ("06_2", "add up to three", "G-F6.unseen", "3"),
        ("06_3", "One, two and zero", "G-F6.high_gaps", "1,2,0"),
        (
            "06_3",
            "skip one box, take the next, skip two, take the next, and stop",
            "G-F6.high_gaps",
            "1,2,0",
        ),
        ("06_3", "rebuilds two and five", "G-F5.example_high_pair", "2,5"),
        ("06_4", "The ten lists", "G-F5.pairs", "10"),
        ("06_4", "split three unseen boxes", "G-F6.unseen", "3"),
        ("06_4", "into three gaps, each exactly once", "G-F6.positions", "3"),
        (
            "06_4",
            "every way to split three unseen boxes into three gaps, each exactly once",
            "G-F6.one_pair_per_list",
            "true",
        ),
        ("06_4", "one in ten", "G-F6.each_chance", "1/10"),
        ("06_5", "For one and two", "G-F5.first_pair", "1,2"),
        ("06_5", "zero, zero and three becomes", "G-F6.low_gaps", "0,0,3"),
        ("06_5", "three, zero and zero", "G-F6.low_traded", "3,0,0"),
        ("06_5", "four and five", "G-F6.low_partner", "4,5"),
        ("06_6", "the same ten numbers", "G-F5.pairs", "10"),
        ("06_7", "all three columns", "G-F6.positions", "3"),
        ("06_7", "the same ten numbers", "G-F5.pairs", "10"),
        ("06_8", "Each holds ten boxes", "G-F6.column_sums", "10,10,10"),
        ("06_8", "over ten pairs", "G-F5.pairs", "10"),
        ("06_8", "each gap averages one box", "G-F6.position_averages", "1,1,1"),
        ("06_8", "Three unseen boxes", "G-F6.unseen", "3"),
        ("06_8", "by three gaps", "G-F6.positions", "3"),
        ("06_9", "friends' five boxes", "G-F1.handful", "5"),
        ("06_9", "Five numbers cut", "G-F1.handful", "5"),
        ("06_9", "into six gaps", "G-F6.friends_positions", "6"),
        ("06_9", "only five arcs", "G-F6.friends_seen_gaps", "5"),
        ("06_9", "the sixth gap", "G-F6.friends_positions", "6"),
        ("06_10", "Above sixty", "G-F1.largest", "60"),
        ("06_10", "the top gap averaged one box", "G-F6.top_average", "1"),
        // 07: right on average
        ("07_1", "friends' five numbers", "G-F1.handful", "5"),
        ("07_1", "N minus five unseen boxes", "G-F1.handful", "5"),
        ("07_1", "into six gaps", "G-F6.friends_positions", "6"),
        ("07_1", "five they can see", "G-F6.friends_seen_gaps", "5"),
        ("07_2", "Every handful of five", "G-F1.handful", "5"),
        ("07_2", "a list of six gaps", "G-F6.friends_positions", "6"),
        ("07_2", "adding up to N minus five", "G-F1.handful", "5"),
        ("07_3", "all six gaps", "G-F6.friends_positions", "6"),
        ("07_3", "hold N minus five boxes", "G-F1.handful", "5"),
        (
            "07_3",
            "each gap averages N minus five, over six",
            "G-F7.gap_average_formula",
            "(N-5)/6",
        ),
        ("07_4", "the five gaps it can see", "G-F6.friends_seen_gaps", "5"),
        ("07_4", "that's eleven", "G-F3.average_gap", "11"),
        ("07_5", "eleven is one handful's average", "G-F3.average_gap", "11"),
        ("07_5", "its two gaps below", "G-F5.seen", "2"),
        ("07_5", "only three times in ten", "G-F7.tiny_agree", "3"),
        ("07_5", "only three times in ten", "G-F5.pairs", "10"),
        ("07_6", "over all ten pairs", "G-F5.pairs", "10"),
        ("07_6", "the guesses average one box", "G-F7.tiny_guess_average", "1"),
        (
            "07_6",
            "the guesses average one box, and so do the top gaps",
            "G-F7.tiny_top_average",
            "1",
        ),
        (
            "07_6",
            "each seen gap averages N minus five, over six",
            "G-F7.gap_average_formula",
            "(N-5)/6",
        ),
        ("07_6", "the average of the five", "G-F6.friends_seen_gaps", "5"),
        ("07_8", "just one box", "G-F7.one_box_handful", "1"),
        (
            "07_9",
            "doubles the number and subtracts one",
            "G-F7.one_box_recipe",
            "2m-1",
        ),
        ("07_9", "One box leaves two gaps", "G-F7.one_box_handful", "1"),
        ("07_9", "One box leaves two gaps", "G-F7.one_box_gaps", "2"),
        ("07_9", "Box thirty-seven", "G-F7.one_box", "37"),
        ("07_9", "thirty-six boxes below it", "G-F7.one_box_below", "36"),
        ("07_9", "seventy-three", "G-F7.one_box_estimate", "73"),
        ("07_10", "needed five", "G-F1.handful", "5"),
        ("07_10", "one more gap than there are numbers", "G-F3.top_gaps", "1"),
        // 08: one estimate
        ("08_1", "not about our seventy-one", "G-F3.estimate", "71"),
        ("08_1", "made exactly seventy-one boxes", "G-F8.count", "71"),
        ("08_1", "buy five fresh boxes", "G-F1.handful", "5"),
        ("08_2", "Here are twenty such handfuls", "G-F8.again_draws", "20"),
        ("08_3", "about forty-five", "G-F8.again_lowest_about", "45"),
        ("08_3", "eighty-four point two", "G-F8.again_highest", "84.2"),
        (
            "08_3",
            "three of them land exactly on seventy-one",
            "G-F8.again_exact",
            "3",
        ),
        ("08_3", "three of them land exactly on seventy-one", "G-F8.count", "71"),
        ("08_3", "about seventy-two", "G-F8.again_average_about", "72"),
        ("08_4", "Twenty is only twenty", "G-F8.again_draws", "20"),
        ("08_4", "over thirteen million handfuls of five", "G-F5.millions", "13"),
        ("08_4", "over thirteen million handfuls of five", "G-F1.handful", "5"),
        (
            "08_4",
            "Just one handful tops out at five",
            "G-F8.topped_by_smallest",
            "1",
        ),
        ("08_4", "Just one handful tops out at five", "G-F1.handful", "5"),
        ("08_4", "Five handfuls top out at six", "G-F8.topped_by_next", "5"),
        ("08_4", "Five handfuls top out at six", "G-F8.next_largest", "6"),
        (
            "08_4",
            "Over nine hundred thousand top out at seventy-one",
            "G-F8.topped_by_count_over",
            "900000",
        ),
        (
            "08_4",
            "Over nine hundred thousand top out at seventy-one",
            "G-F8.count",
            "71",
        ),
        ("08_4", "sixty-seven stacks", "G-F8.stacks", "67"),
        ("08_5", "exactly seventy-one, the true count", "G-F8.count", "71"),
        (
            "08_6",
            "About one handful in twenty-nine",
            "G-F8.exactly_right_one_in",
            "29",
        ),
        ("08_6", "Three in twenty was lucky", "G-F8.again_exact", "3"),
        ("08_6", "Three in twenty was lucky", "G-F8.again_draws", "20"),
        ("08_7", "balance at seventy-one exactly", "G-F8.average", "71"),
        (
            "08_7",
            "our twenty draws averaged about seventy-two",
            "G-F8.again_draws",
            "20",
        ),
        (
            "08_7",
            "our twenty draws averaged about seventy-two",
            "G-F8.again_average_about",
            "72",
        ),
        ("08_7", "Fifty-eight percent", "G-F8.overestimate_percent", "58"),
        ("08_7", "by about eight boxes", "G-F8.overshoot_about", "8"),
        ("08_7", "miss by twelve", "G-F8.undershoot", "12"),
        ("08_8", "about nine boxes", "G-F8.average_miss_about", "9"),
        ("08_8", "about eleven point seven", "G-F8.sd_about", "11.7"),
        ("08_9", "about nine boxes", "G-F8.average_miss_about", "9"),
        ("08_9", "our seventy-one", "G-F3.estimate", "71"),
        // 09: how far off
        ("09_1", "at least sixty", "G-F1.at_least", "60"),
        (
            "09_1",
            "a fair handful of five all be sixty or less",
            "G-F1.handful",
            "5",
        ),
        (
            "09_1",
            "a fair handful of five all be sixty or less",
            "G-F1.largest",
            "60",
        ),
        ("09_2", "If the count were seventy-one", "G-F8.count", "71"),
        ("09_2", "about forty-two percent", "G-F9.at_most_60.71.percent", "42"),
        ("09_2", "If it were a hundred", "G-F9.second_count", "100"),
        ("09_2", "about seven percent", "G-F9.at_most_60.100.percent", "7"),
        ("09_3", "At a hundred and seven boxes", "G-F9.range_top", "107"),
        ("09_3", "just over five percent", "G-F9.level_percent", "5"),
        ("09_3", "At a hundred and eight", "G-F9.first_rare", "108"),
        ("09_4", "from sixty to a hundred and seven", "G-F9.range", "60,107"),
        ("09_4", "at least five times in a hundred", "G-F9.level", "1/20"),
        ("09_5", "Fix the count at seventy-one", "G-F8.count", "71"),
        (
            "09_5",
            "the twenty handfuls drawn by computer",
            "G-F8.again_draws",
            "20",
        ),
        (
            "09_6",
            "Nineteen of the twenty ranges cover seventy-one",
            "G-F9.again_cover",
            "19",
        ),
        (
            "09_6",
            "Nineteen of the twenty ranges cover seventy-one",
            "G-F8.again_draws",
            "20",
        ),
        (
            "09_6",
            "Nineteen of the twenty ranges cover seventy-one",
            "G-F8.count",
            "71",
        ),
        ("09_6", "topped by thirty-eight", "G-F9.again_miss_largest", "38"),
        (
            "09_6",
            "this way cover seventy-one about ninety-six times in a hundred",
            "G-F8.count",
            "71",
        ),
        (
            "09_6",
            "this way cover seventy-one about ninety-six times in a hundred",
            "G-F9.cover_71_percent",
            "96",
        ),
        ("09_6", "never fewer than ninety-five", "G-F9.confidence_percent", "95"),
        (
            "09_7",
            "That's what ninety-five percent confidence means",
            "G-F9.confidence_percent",
            "95",
        ),
        (
            "09_7",
            "not a ninety-five percent chance",
            "G-F9.confidence_percent",
            "95",
        ),
        (
            "09_8",
            "handfuls of five from the first sixty boxes",
            "G-F1.handful",
            "5",
        ),
        (
            "09_8",
            "handfuls of five from the first sixty boxes",
            "G-F1.largest",
            "60",
        ),
        (
            "09_8",
            "all the handfuls of five from the whole company",
            "G-F1.handful",
            "5",
        ),
        ("09_8", "At a hundred and seven boxes", "G-F9.range_top", "107"),
        ("09_8", "just over five percent", "G-F9.level_percent", "5"),
        ("09_9", "had ten boxes instead", "G-F9.ten_boxes", "10"),
        ("09_10", "Ten numbers all sixty or less", "G-F9.ten_boxes", "10"),
        ("09_10", "Ten numbers all sixty or less", "G-F1.largest", "60"),
        ("09_10", "sixty through seventy-nine", "G-F9.ten_boxes_range", "60,79"),
        // 10: a rival recipe
        ("10_1", "from one up to the count", "G-F1.first", "1"),
        ("10_1", "With seventy-one boxes", "G-F8.count", "71"),
        ("10_1", "the middle is thirty-six", "G-F10.middle_71", "36"),
        ("10_1", "twice the middle, minus one", "G-F10.from_middle", "2*middle-1"),
        ("10_2", "Twice the average, minus one", "G-F10.recipe", "2*average-1"),
        ("10_3", "Our five add up", "G-F1.handful", "5"),
        ("10_3", "a hundred and eighty-five", "G-F10.sum", "185"),
        ("10_3", "an average of thirty-seven", "G-F10.average", "37"),
        ("10_3", "doubling says seventy-three", "G-F10.doubling", "73"),
        ("10_3", "the gap recipe's seventy-one", "G-F3.estimate", "71"),
        ("10_4", "any number from one to the count", "G-F1.first", "1"),
        ("10_5", "Keep the box at sixty", "G-F1.largest", "60"),
        ("10_5", "the other four", "G-F10.others", "4"),
        ("10_5", "two, three, four and five", "G-F10.silly_others", "2,3,4,5"),
        ("10_6", "Twenty-eight point six", "G-F10.silly_doubling_decimal", "28.6"),
        ("10_6", "the sixty boxes we know were made", "G-F1.at_least", "60"),
        ("10_6", "says seventy-one for this handful", "G-F10.silly_gap", "71"),
        ("10_6", "topped by sixty", "G-F1.largest", "60"),
        ("10_7", "a company of seventy-one boxes", "G-F8.count", "71"),
        (
            "10_7",
            "about one handful in five",
            "G-F10.below_largest_71_one_in",
            "5",
        ),
        ("10_8", "exactly seventy-one boxes", "G-F8.count", "71"),
        ("10_8", "sixty-seven bars", "G-F8.stacks", "67"),
        ("10_8", "the total of the five numbers", "G-F1.handful", "5"),
        (
            "10_8",
            "from five all the way to a hundred and thirty-seven",
            "G-F10.doubling_range_71",
            "5,137",
        ),
        ("10_9", "Both balance at seventy-one", "G-F10.doubling_average_71", "71"),
        ("10_9", "fourteen point three", "G-F10.doubling_miss_71_decimal", "14.3"),
        ("10_9", "nine point two", "G-F8.average_miss_decimal", "9.2"),
        ("10_9", "seventeen point eight", "G-F10.doubling_sd_about", "17.8"),
        ("10_9", "eleven point seven", "G-F8.sd_about", "11.7"),
        ("10_10", "uses all five", "G-F1.handful", "5"),
        ("10_10", "the other four", "G-F10.others", "4"),
        // 11: the largest number says it all
        ("11_1", "own five boxes", "G-F1.handful", "5"),
        ("11_1", "the largest number is sixty", "G-F1.largest", "60"),
        ("11_1", "the other four", "G-F10.others", "4"),
        ("11_1", "any four different numbers below sixty", "G-F10.others", "4"),
        ("11_1", "any four different numbers below sixty", "G-F1.largest", "60"),
        ("11_2", "the tiny company of five boxes", "G-F5.boxes", "5"),
        ("11_2", "four of its ten pairs", "G-F11.pairs_topped", "4"),
        ("11_2", "four of its ten pairs", "G-F5.pairs", "10"),
        ("11_2", "have five as their larger number", "G-F11.pair_top", "5"),
        (
            "11_2",
            "the smaller one is one, two, three or four",
            "G-F11.pair_smaller",
            "1,2,3,4",
        ),
        ("11_2", "those four pairs", "G-F11.pairs_topped", "4"),
        ("11_2", "each is one time in four", "G-F11.pair_each", "1/4"),
        ("11_3", "six boxes, or eight", "G-F11.pair_more", "6,8"),
        (
            "11_3",
            "fifteen and then twenty-eight",
            "G-F11.pair_more_pairs",
            "15,28",
        ),
        ("11_3", "a larger number of five", "G-F11.pair_top", "5"),
        (
            "11_3",
            "the pairs topped by five are still the same four",
            "G-F11.pair_top",
            "5",
        ),
        (
            "11_3",
            "the pairs topped by five are still the same four",
            "G-F11.pairs_topped",
            "4",
        ),
        ("11_3", "each is still one time in four", "G-F11.pair_each", "1/4"),
        ("11_4", "the same with sixty", "G-F1.largest", "60"),
        ("11_4", "from sixty boxes up", "G-F1.at_least", "60"),
        (
            "11_4",
            "topped by sixty is sixty plus any four numbers",
            "G-F1.largest",
            "60",
        ),
        (
            "11_4",
            "topped by sixty is sixty plus any four numbers",
            "G-F10.others",
            "4",
        ),
        (
            "11_4",
            "over four hundred and fifty thousand",
            "G-F11.others_sets_over",
            "450000",
        ),
        ("11_4", "groups of four", "G-F10.others", "4"),
        ("11_4", "the chance that sixty is the largest", "G-F1.largest", "60"),
        ("11_4", "exactly sixty boxes", "G-F1.at_least", "60"),
        ("11_4", "it's one in twelve", "G-F11.top_chance.60", "1/12"),
        ("11_4", "with seventy-one", "G-F8.count", "71"),
        ("11_4", "about one in twenty-nine", "G-F11.top_chance.71.one_in", "29"),
        ("11_4", "with a thousand", "G-F1.thousand", "1000"),
        (
            "11_4",
            "about one in eighteen million",
            "G-F11.top_chance.1000.millions",
            "18",
        ),
        ("11_5", "the other four", "G-F10.others", "4"),
        ("11_7", "the other four", "G-F10.others", "4"),
        ("11_7", "the gaps below sixty", "G-F1.largest", "60"),
        ("11_7", "hold fifty-five numbers", "G-F3.unseen", "55"),
        ("11_7", "sixty minus five", "G-F1.largest", "60"),
        ("11_7", "sixty minus five", "G-F1.handful", "5"),
        ("11_8", "the other four", "G-F10.others", "4"),
        // 12: averaging away the wobble
        ("12_1", "the tiny company of five boxes", "G-F5.boxes", "5"),
        (
            "12_1",
            "any two are as likely to be seen as any other two",
            "G-F5.seen",
            "2",
        ),
        ("12_1", "its ten pairs again", "G-F5.pairs", "10"),
        (
            "12_1",
            "adds the two numbers and takes away one",
            "G-F12.pair_doubling",
            "1*total-1",
        ),
        ("12_1", "adds the two numbers and takes away one", "G-F5.seen", "2"),
        ("12_2", "They average five", "G-F12.doubling_average", "5"),
        (
            "12_2",
            "from two all the way up to eight",
            "G-F12.doubling_range",
            "2,8",
        ),
        ("12_3", "the four pairs topped by five", "G-F11.pairs_topped", "4"),
        ("12_3", "the four pairs topped by five", "G-F11.pair_top", "5"),
        ("12_3", "get four different estimates", "G-F11.pairs_topped", "4"),
        ("12_4", "That makes four groups", "G-F12.groups", "4"),
        (
            "12_4",
            "topped by two, three, four and five",
            "G-F12.group_tops",
            "2,3,4,5",
        ),
        (
            "12_4",
            "with one, two, three and four pairs",
            "G-F12.group_sizes",
            "1,2,3,4",
        ),
        ("12_5", "five, six, seven and eight", "G-F12.ends_in_5", "5,6,7,8"),
        ("12_6", "Six and a half", "G-F12.ends_in_5_average", "13/2"),
        ("12_6", "every pair topped by five", "G-F11.pair_top", "5"),
        ("12_6", "Three and four average", "G-F12.ends_in_3", "3,4"),
        ("12_6", "average three and a half", "G-F12.ends_in_3_average", "7/2"),
        ("12_6", "Four, five and six average five", "G-F12.ends_in_4", "4,5,6"),
        (
            "12_6",
            "Four, five and six average five",
            "G-F12.ends_in_4_average",
            "5",
        ),
        ("12_6", "the lone pair topped by two says two", "G-F12.ends_in_2", "2"),
        ("12_8", "a company of six boxes, or eight", "G-F11.pair_more", "6,8"),
        ("12_8", "the pairs topped by five", "G-F11.pair_top", "5"),
        (
            "12_8",
            "still average six and a half",
            "G-F12.ends_in_5_average",
            "13/2",
        ),
        ("12_9", "a chance in ten", "G-F5.pairs", "10"),
        ("12_9", "its number of pairs in ten", "G-F5.pairs", "10"),
        ("12_9", "four in ten", "G-F12.top_group_chance", "2/5"),
        ("12_10", "the ten pairs' average", "G-F5.pairs", "10"),
        ("12_10", "comes back, exactly five", "G-F12.weighted_average", "5"),
        ("12_10", "the four groups", "G-F12.groups", "4"),
        ("12_10", "four and a quarter", "G-F12.equal_weight_average", "17/4"),
        ("12_11", "an arrow from five, the true count", "G-F5.boxes", "5"),
        ("12_11", "the group topped by four", "G-F12.cancel_group", "4"),
        ("12_11", "minus one, zero and plus one", "G-F12.cancel_misses", "-1,0,1"),
        ("12_12", "the group topped by five", "G-F12.same_side_group", "5"),
        ("12_12", "zero, one, two and three", "G-F12.same_side_misses", "0,1,2,3"),
        (
            "12_12",
            "misses by one and a half",
            "G-F12.same_side_average_miss",
            "3/2",
        ),
        ("12_13", "all ten pairs", "G-F5.pairs", "10"),
        ("12_13", "one point four", "G-F12.doubling_miss", "1.4"),
        ("12_13", "one point two", "G-F12.grouped_miss", "1.2"),
        (
            "12_15",
            "squared misses average three",
            "G-F12.doubling_squared_miss",
            "3",
        ),
        ("12_15", "two and a quarter", "G-F12.grouped_squared_miss", "9/4"),
        ("12_15", "about one point seven", "G-F12.doubling_sd_about", "1.7"),
        ("12_15", "exactly one point five", "G-F12.grouped_sd_decimal", "1.5"),
        ("12_15", "the drop, three quarters", "G-F12.spread_inside", "3/4"),
        // 13: nothing unbiased does better
        ("13_2", "always says seventy-one", "G-F13.always", "71"),
        ("13_2", "if the count is seventy-one", "G-F13.always", "71"),
        ("13_3", "has two boxes", "G-F13.smallest_company", "2"),
        ("13_3", "its only pair is one and two", "G-F5.first_pair", "1,2"),
        ("13_3", "To average two", "G-F13.smallest_company", "2"),
        (
            "13_3",
            "must say two when the larger number is two",
            "G-F13.forced_at_2",
            "2",
        ),
        (
            "13_3",
            "must say two when the larger number is two",
            "G-F13.smallest_company",
            "2",
        ),
        ("13_4", "Three boxes make three pairs", "G-F13.next_company", "3"),
        ("13_4", "Three boxes make three pairs", "G-F13.pairs_from_3", "3"),
        (
            "13_4",
            "topped by two must still say two",
            "G-F13.smallest_company",
            "2",
        ),
        ("13_4", "topped by two must still say two", "G-F13.forced_at_2", "2"),
        ("13_4", "The other two are topped by three", "G-F13.new_pairs.3", "2"),
        ("13_4", "The other two are topped by three", "G-F13.next_company", "3"),
        (
            "13_4",
            "all three estimates must average three",
            "G-F13.pairs_from_3",
            "3",
        ),
        (
            "13_4",
            "all three estimates must average three",
            "G-F13.next_company",
            "3",
        ),
        ("13_4", "those two must say three and a half", "G-F13.new_pairs.3", "2"),
        (
            "13_4",
            "those two must say three and a half",
            "G-F13.forced_at_3_decimal",
            "3.5",
        ),
        ("13_5", "Four boxes make six pairs", "G-F13.predict_company", "4"),
        ("13_5", "Four boxes make six pairs", "G-F13.pairs_from_4", "6"),
        ("13_5", "three of them are topped by four", "G-F13.new_pairs.4", "3"),
        ("13_5", "three of them are topped by four", "G-F13.predict_company", "4"),
        ("13_5", "The other three already", "G-F13.pairs_from_3", "3"),
        ("13_5", "for the new three", "G-F13.new_pairs.4", "3"),
        ("13_6", "Five. That's the only value", "G-F13.forced_at_4", "5"),
        ("13_6", "all six estimates", "G-F13.pairs_from_4", "6"),
        ("13_6", "an average of four", "G-F13.predict_company", "4"),
        ("13_6", "at five boxes", "G-F13.last_company", "5"),
        ("13_6", "its four new pairs", "G-F13.new_pairs.5", "4"),
        ("13_6", "forced to six and a half", "G-F13.forced_at_5", "13/2"),
        ("13_7", "exactly one unknown value", "G-F13.new_values_per_count", "1"),
        ("13_7", "five seen from exactly sixty boxes", "G-F1.handful", "5"),
        ("13_7", "five seen from exactly sixty boxes", "G-F1.at_least", "60"),
        ("13_7", "one in twelve", "G-F13.share_at_60", "1/12"),
        (
            "13_9",
            "three times the smaller number, minus one",
            "G-F13.smaller_recipe",
            "3*smaller-1",
        ),
        ("13_9", "from two all the way to eleven", "G-F13.smaller_range", "2,11"),
        ("13_9", "the ten pairs", "G-F5.pairs", "10"),
        ("13_9", "still average five", "G-F13.smaller_average", "5"),
        (
            "13_10",
            "two, three and a half, five, and six and a half",
            "G-F13.smaller_grouped",
            "2,7/2,5,13/2",
        ),
        ("13_11", "two point four", "G-F13.smaller_miss_decimal", "2.4"),
        (
            "13_11",
            "against the gap recipe's one point two",
            "G-F12.grouped_miss",
            "1.2",
        ),
        (
            "13_13",
            "seven sixths of the largest number, minus one",
            "G-F13.shrunk_recipe",
            "7/6*largest-1",
        ),
        ("13_13", "the friends' five boxes", "G-F1.handful", "5"),
        ("13_13", "it says sixty-nine", "G-F13.shrunk_friends", "69"),
        ("13_13", "seventy-one boxes", "G-F8.count", "71"),
        ("13_13", "it averages sixty-nine", "G-F13.shrunk_average_71", "69"),
        ("13_13", "two short of the count", "G-F13.shrunk_bias_71", "-2"),
        ("13_13", "about eight point seven", "G-F13.shrunk_miss_71_shown", "8.7"),
        ("13_13", "nine point two", "G-F8.average_miss_decimal", "9.2"),
        ("13_14", "exactly five boxes", "G-F13.edge_count", "5"),
        ("13_14", "seven sixths says", "G-F13.shrunk_factor", "7/6"),
        ("13_14", "four and five sixths", "G-F13.shrunk_at_edge", "29/6"),
        (
            "13_14",
            "from five to ten",
            "G-F13.shrunk_loses_squared_at",
            "5,6,7,8,9,10",
        ),
        ("14_1", "the same five boxes", "G-F1.handful", "5"),
        ("14_1", "all five friends", "G-F1.handful", "5"),
        ("14_1", "the first seventy-one boxes", "G-F14.shop_boxes", "71"),
        ("14_2", "two hundred", "G-F14.made", "200"),
        ("14_2", "seventy-one", "G-F14.shop_estimate", "71"),
        ("14_3", "seventy-one", "G-F14.shop_average", "71"),
        ("14_3", "a hundred and twenty-nine", "G-F14.shop_short_by", "129"),
        ("14_4", "a hundred and fifty-one up", "G-F14.late_from", "151"),
        ("14_5", "two hundred and thirty-nine", "G-F14.late_estimate", "239"),
        ("14_5", "a hundred and fifty-seven", "G-F14.late_first_gap", "157"),
        ("14_6", "two hundred boxes", "G-F14.made", "200"),
        ("14_6", "all five numbers", "G-F1.handful", "5"),
        ("14_6", "above a hundred and fifty", "G-F14.late_from", "151"),
        (
            "14_6",
            "less than once in a thousand",
            "G-F14.fair_all_above_under_one_in_1000",
            "true",
        ),
        ("14_7", "the other four numbers", "G-F10.others", "4"),
        ("15_1", "starts at one", "G-F1.first", "1"),
        ("15_2", "Two gaps", "G-F15.hidden_count", "2"),
        ("15_2", "one at each end", "G-F1.first", "1"),
        ("15_2", "four inner gaps", "G-F15.inner_count", "4"),
        ("15_3", "Between fourteen and sixty", "G-F1.numbers", "14,27,33,51,60"),
        ("15_3", "forty-two boxes", "G-F15.inner_unseen", "42"),
        ("15_3", "ten and a half per gap", "G-F15.inner_average", "21/2"),
        (
            "15_3",
            "fourteen to sixty spans forty-seven numbers",
            "G-F15.span_numbers",
            "47",
        ),
        ("15_4", "one average inner gap", "G-F1.first", "1"),
        ("15_5", "Sixty-eight", "G-F15.spread_estimate", "68"),
        ("15_5", "two hidden gaps instead of one", "G-F15.hidden_count", "2"),
        ("15_6", "Every handful of five", "G-F1.handful", "5"),
        ("15_7", "Return to numbering from one", "G-F1.first", "1"),
        ("15_7", "ten boxes were made", "G-F15.deliveries_made", "10"),
        (
            "15_7",
            "boxes one to five, or boxes six to ten",
            "G-F15.deliveries",
            "1,2,3,4,5;6,7,8,9,10",
        ),
        ("15_8", "one in two chance", "G-F15.deliveries_each_box", "1/2"),
        ("15_9", "starts-at-one", "G-F1.first", "1"),
        ("15_9", "estimates five or eleven", "G-F15.delivery_estimates", "5,11"),
        ("15_9", "average eight", "G-F15.deliveries_average", "8"),
        ("15_9", "not ten", "G-F15.deliveries_made", "10"),
        ("16_2", "from one", "G-F1.first", "1"),
        ("16_3", "five friends", "G-F1.handful", "5"),
        ("16_3", "Box sixty", "G-F1.largest", "60"),
        ("16_3", "at least sixty", "G-F1.largest", "60"),
        ("16_3", "about seventy-one", "G-F3.estimate", "71"),
        ("16_5", "seventy-one", "G-F3.estimate", "71"),
        ("16_5", "one estimate from one handful", "G-F1.first", "1"),
        ("16_5", "ninety-five percent", "G-F9.confidence_percent", "95"),
        ("16_5", "sixty to a hundred and seven", "G-F9.range", "60,107"),
        ("16_6", "those five", "G-F1.handful", "5"),
        ("16_6", "one shop", "G-F1.first", "1"),
        ("16_6", "numbered from one", "G-F1.first", "1"),
        ("16_7", "box ninety", "G-F16.sixth", "90"),
        ("16_8", "Ninety", "G-F16.sixth", "90"),
        ("16_8", "one average gap", "G-F1.first", "1"),
        ("16_8", "Six gaps", "G-F16.size", "6"),
        ("16_8", "below ninety", "G-F16.sixth", "90"),
        ("16_8", "eighty-four boxes", "G-F16.unseen", "84"),
        ("16_8", "fourteen per gap", "G-F16.average_gap", "14"),
        ("16_8", "a hundred and four", "G-F16.estimate", "104"),
    ];

    /// Sizes said on purpose without a value: the stakes and the unknown
    /// tail are vague by design, and no fact stands behind them.
    const VAGUE: &[(&str, &str)] = &[
        ("01_2", "a few dozen"),
        ("01_2", "tens of thousands"),
        ("01_8", "hundreds more"),
        ("14_1", "one store"),
        ("15_6", "One last trap"),
        ("16_1", "Second World War"),
    ];

    /// The words that say a number or a size.
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
        "hundreds",
        "thousand",
        "thousands",
        "million",
        "dozen",
        "dozens",
        "first",
        "second",
        "half",
        "twice",
        "double",
    ];

    /// Each word of `text`, lowercased, with the byte it starts at.
    fn words_at(text: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        let mut start = None;
        for (i, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            if c.is_alphabetic() {
                start.get_or_insert(i);
            } else if let Some(s) = start.take() {
                out.push((s, text[s..i].to_lowercase()));
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
            assert_eq!(text.matches(words).count(), 1, "{id} must say {words:?} once");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        for (id, words) in VAGUE {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(words).count(), 1, "{id} must say {words:?} once");
        }
        // and every number word any line says sits inside a phrase above
        for (id, text) in script.lines() {
            let covered: Vec<(usize, usize)> = SPOKEN
                .iter()
                .map(|(i, w, ..)| (*i, *w))
                .chain(VAGUE.iter().copied())
                .filter(|(i, _)| *i == id.as_str())
                .filter_map(|(_, w)| text.find(w).map(|at| (at, at + w.len())))
                .collect();
            for (at, word) in words_at(text) {
                if NUMBER_WORDS.contains(&word.as_str()) {
                    assert!(
                        covered.iter().any(|&(a, b)| a <= at && at < b),
                        "{id} says {word:?} outside every phrase in SPOKEN or VAGUE: {text}"
                    );
                }
            }
        }
    }

    /// The coverage check itself, at its edges: a word inside a phrase
    /// counts, a word just outside does not, hyphenated numbers split.
    #[test]
    fn number_words_are_found_where_they_are() {
        let w = words_at("Box sixty, twenty-seven won't.");
        let found: Vec<&str> = w.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(found, ["box", "sixty", "twenty", "seven", "won", "t"]);
        assert_eq!(w[1].0, 4);
        assert_eq!(w[3].0, "Box sixty, twenty-".len());
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// The draft continues the production storyboard where it stops: its
    /// first chapter is the one after the registry's last, its chapters
    /// run on in order, and no beat of it is already in script.tsv.
    #[test]
    fn the_draft_continues_the_registry() {
        let draft = explainer_kit::storyboard::parse(DRAFT).unwrap();
        let last = keys().last().unwrap()[..2].parse::<u32>().unwrap();
        for (i, board) in draft.iter().enumerate() {
            assert_eq!(
                board.key[..2].parse::<u32>().unwrap(),
                last + 1 + i as u32,
                "{}",
                board.key
            );
        }
        assert_eq!(
            last + draft.len() as u32,
            16,
            "the full sixteen-chapter plan is accounted for"
        );
        let script = Script::parse(SCRIPT).unwrap();
        for beat in draft.iter().flat_map(|b| &b.beats) {
            assert!(
                script.text(&beat.id).is_none(),
                "{} is drafted and also in script.tsv",
                beat.id
            );
        }
    }

    /// Each chapter asks one prediction, and the next line's first words
    /// answer it: the one certain thing about the count, where a fair
    /// handful from a thousand lands, the first estimate, the edge
    /// handful's estimate, the tiny company's average, where the friends'
    /// hidden gap is, what the recipe does with one box, how often an
    /// estimate is exactly right, what ten boxes do to the range, what the
    /// doubling recipe says for the strange handful, what the other four
    /// numbers add once the largest is known, what the biggest group of
    /// pairs averages, and the value four boxes force.
    #[test]
    fn the_prediction_is_asked_once_and_answered_next() {
        let boards = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let expected = [
            (
                "01_five_boxes",
                "how many boxes the company made?",
                "It made at least sixty",
            ),
            (
                "02_a_thousand_boxes",
                "where would a fair handful's five numbers usually land?",
                "All over the line",
            ),
            (
                "03_counting_the_gaps",
                "how many boxes did the company make?",
                "About seventy-one",
            ),
            (
                "04_what_we_assume",
                "if the five numbers were one, two, three, four and five?",
                "Five.",
            ),
            ("05_every_handful", "What do the ten average?", "Exactly five"),
            ("06_gaps_trade_places", "Where is the sixth gap?", "Above sixty"),
            (
                "07_right_on_average",
                "with a handful of just one box?",
                "It doubles the number",
            ),
            (
                "08_one_estimate",
                "the estimate is exactly seventy-one, the true count?",
                "About one handful in twenty-nine",
            ),
            ("09_how_far_off", "would the range get wider or narrower?", "Narrower"),
            (
                "10_a_rival_recipe",
                "What does doubling say for this handful?",
                "Twenty-eight point six",
            ),
            (
                "11_the_largest_says_it_all",
                "how much can the other four tell you about the count?",
                "Nothing",
            ),
            ("12_averaging_the_wobble", "What's their average?", "Six and a half"),
            (
                "13_nothing_does_better",
                "What must the recipe say for the new three?",
                "Five",
            ),
            (
                "14_an_unfair_draw",
                "Will the recipe guess too low, or too high?",
                "Too high",
            ),
            ("15_other_assumptions", "What's the estimate now?", "Sixty-eight"),
            (
                "16_five_boxes_again",
                "What's the new estimate?",
                "Ninety, plus one average gap",
            ),
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

    /// Every authored pause intention names a real boundary: its cue exists
    /// (in script.tsv, or in the draft for a chapter not yet implemented),
    /// its phrase is in that cue once, a transition's phrase ends its cue,
    /// and its range is a range.
    #[test]
    fn every_pause_intention_names_a_real_boundary() {
        let script = Script::parse(SCRIPT).unwrap();
        let draft = explainer_kit::storyboard::parse(DRAFT).unwrap();
        let drafted = |id: &str| {
            draft
                .iter()
                .flat_map(|b| &b.beats)
                .find(|beat| beat.id == id)
                .and_then(|beat| beat.spoken())
        };
        let mut rows = 0;
        for row in PAUSES.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let cols: Vec<&str> = row.split('\t').collect();
            assert_eq!(cols.len(), 6, "{row}");
            let (cue, after, role) = (cols[0], cols[1], cols[2]);
            let (min, max): (f64, f64) = (cols[3].parse().unwrap(), cols[4].parse().unwrap());
            let text = script
                .text(cue)
                .map(str::to_owned)
                .or_else(|| drafted(cue))
                .unwrap_or_else(|| panic!("pause row for unknown cue {cue}"));
            assert_eq!(text.matches(after).count(), 1, "{cue}: {after:?} once in {text:?}");
            if role == "cue_transition" {
                assert!(text.ends_with(after), "{cue}: a transition follows the cue's end");
            }
            assert!(0.0 < min && min <= max, "{cue}: {min}..{max}");
            assert!(!cols[5].is_empty(), "{cue}: every pause says why");
            rows += 1;
        }
        assert!(rows > 0);
    }
}
