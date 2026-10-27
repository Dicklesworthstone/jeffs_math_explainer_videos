//! Why Your Bus Wait Is Longer Than You Expect: two bus routes, every bus on
//! time, gaps of two and ten minutes, and why a passenger who turns up at
//! random waits four minutes twenty, not three. The inspection paradox (length-
//! biased sampling), rendered end to end by franken_manim's native Rust
//! front door and the corpus's shared kit.
//!
//! ```text
//! inspection_paradox facts            # every number the video shows or says (key<TAB>value)
//! inspection_paradox script           # the narration (generated from docs/storyboard.md)
//! inspection_paradox render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! inspection_paradox help             # the full option list
//! ```
//!
//! All twelve chapters have scene source; the whole film is
//! `docs/storyboard-draft.md` (see `chapters/mod.rs`).

mod chapters;
mod facts;
mod model;
mod palette;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat, for
/// the chapters with scene source (the tests check that script.tsv and the
/// chapter registry follow it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");
/// The full draft, all twelve chapters (the tests check that it holds the
/// storyboard verbatim).
#[cfg(test)]
const DRAFT: &str = include_str!("../docs/storyboard-draft.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "inspection_paradox",
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
    /// for and the value those words denote. A changed timetable or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        // 01: the stop, the two routes, the sign, the guess, the answer
        ("01_2", "Two bus routes", "F-T1.routes", "2"),
        ("01_2", "every twelve minutes", "F-T1.period", "12"),
        ("01_2", "two minutes after", "F-T1.offset", "2"),
        ("01_3", "Ten buses an hour", "F-T1.buses_per_hour", "10"),
        ("01_3", "every six minutes on average", "F-T1.mean_gap", "6"),
        ("01_5", "half of six", "F-T1.mean_gap", "6"),
        ("01_5", "half of six, three minutes", "F-T1.half_mean_gap", "3"),
        ("01_5", "be three minutes", "F-T1.half_mean_gap", "3"),
        (
            "01_6",
            "Four minutes and twenty seconds",
            "F-W1.mean_wait_clock",
            "4:20",
        ),
        ("01_6", "an extra minute and twenty seconds", "F-W1.extra_clock", "1:20"),
        // 02: two ways to count the same timetable
        ("02_1", "Two minutes", "F-T1.short", "2"),
        ("02_1", "ten minutes", "F-T1.long", "10"),
        ("02_1", "Five short gaps", "F-T1.short_count", "5"),
        ("02_1", "five long ones", "F-T1.long_count", "5"),
        ("02_2", "Half the tags say two minutes", "F-S1.bus_short_share", "1/2"),
        ("02_2", "say two minutes", "F-T1.short", "2"),
        ("02_2", "half say ten", "F-T1.long", "10"),
        ("02_2", "they average six", "F-S1.bus_mean_gap", "6"),
        ("02_5", "Five in six", "F-S1.person_long_share", "5/6"),
        ("02_5", "one in the middle of every minute", "F-W2.per_minute", "1"),
        ("02_5", "Sixty in the hour", "F-S1.passengers", "60"),
        ("02_5", "fifty of them", "F-S1.passengers_in_long", "50"),
        ("02_6", "fifty of the sixty minutes", "F-T1.long_minutes", "50"),
        ("02_6", "of the sixty minutes", "F-T1.hour", "60"),
        ("02_6", "Half the gaps", "F-T1.long_share_of_gaps", "1/2"),
        ("02_6", "five sixths of the time", "F-T1.long_time_share", "5/6"),
        ("02_7", "Each gap gets one tag", "F-S1.tags_per_gap", "1"),
        // 03: change only the red route's offset
        ("03_1", "six minutes after the blue", "F-O1.even_offset", "6"),
        ("03_1", "every gap is six minutes", "F-O1.even_gap", "6"),
        ("03_2", "Half the buses end one", "F-G1.bus_share_every_offset", "1/2"),
        ("03_2", "half the passengers stand in one", "F-G1.share.d6", "1/2"),
        ("03_3", "The same ten buses", "F-G1.buses_every_offset", "10"),
        (
            "03_3",
            "every six minutes on average",
            "F-G1.mean_gap_every_offset",
            "6",
        ),
        ("03_4", "Half of them still end", "F-G1.bus_share_every_offset", "1/2"),
        (
            "03_4",
            "Seven of every twelve, then eight, nine, ten",
            "F-G1.spoken_of_12",
            "7,8,9,10",
        ),
        ("03_4", "of every twelve", "F-T1.period", "12"),
        ("03_6", "a twelve-minute gap", "F-O1.together_period", "12"),
        ("03_6", "a bus every six minutes", "F-G1.mean_gap_every_offset", "6"),
        ("03_7", "counts each gap once", "F-S1.tags_per_gap", "1"),
        // 04: the same twelve minutes and passengers, counted by their waits
        ("04_1", "the same twelve minutes", "F-T1.period", "12"),
        ("04_1", "five times over", "F-T1.cycles_per_hour", "5"),
        ("04_1", "two minutes later", "F-T1.offset", "2"),
        ("04_1", "ten minutes after that", "F-T1.long", "10"),
        ("04_2", "twelve of our passengers", "F-W2.people", "12"),
        ("04_2", "one in the middle of each minute", "F-W2.per_minute", "1"),
        ("04_3", "The two passengers in the short gap", "F-W2.short_people", "2"),
        (
            "04_3",
            "a minute and a half, and half a minute",
            "F-W2.short_waits",
            "3/2,1/2",
        ),
        ("04_3", "One minute each", "F-W1.short_mean_wait", "1"),
        ("04_4", "The ten passengers in the long gap", "F-W2.long_people", "10"),
        (
            "04_4",
            "nine and a half minutes down to half a minute",
            "F-W2.long_waits_range",
            "19/2,1/2",
        ),
        ("04_4", "Five minutes each", "F-W1.long_mean_wait", "5"),
        ("04_5", "all twelve passengers", "F-W2.people", "12"),
        ("04_6", "Not three", "F-T1.half_mean_gap", "3"),
        ("04_6", "The two passengers in the short gap", "F-W2.short_people", "2"),
        ("04_6", "waited two minutes between them", "F-W2.short_total", "2"),
        ("04_6", "The ten in the long gap", "F-W2.long_people", "10"),
        ("04_6", "waited fifty", "F-W2.long_total", "50"),
        ("04_7", "fifty-two minutes of waiting", "F-W2.total", "52"),
        ("04_7", "shared by twelve passengers", "F-W2.people", "12"),
        ("04_7", "Four and a third minutes each", "F-W2.mean_wait", "13/3"),
        (
            "04_7",
            "four minutes and twenty seconds",
            "F-W1.mean_wait_clock",
            "4:20",
        ),
        ("04_8", "Half of six", "F-T1.mean_gap", "6"),
        ("04_8", "the average of one and five", "F-W1.short_mean_wait", "1"),
        ("04_8", "the average of one and five", "F-W1.long_mean_wait", "5"),
        ("04_8", "five times as many", "F-W2.people_ratio", "5"),
        // 05: geometric area and its conversion at one passenger per minute
        ("05_1", "those twelve minutes", "F-T1.period", "12"),
        ("05_4", "our twelve passengers", "F-W2.people", "12"),
        ("05_4", "a bar one minute wide", "F-A1.bar_width", "1"),
        (
            "05_4",
            "exactly as much as the triangles",
            "F-A1.bars_equal_triangles",
            "true",
        ),
        ("05_4", "one passenger per minute", "F-W2.per_minute", "1"),
        ("05_5", "half of two times two", "F-T1.short", "2"),
        ("05_5", "which is two", "F-A1.short_area", "2"),
        ("05_5", "half of ten times ten", "F-T1.long", "10"),
        ("05_5", "which is fifty", "F-A1.long_area", "50"),
        ("05_5", "one passenger per minute", "F-W2.per_minute", "1"),
        ("05_5", "the fifty-two minutes", "F-A1.total_area", "52"),
        ("05_6", "five times as wide", "F-A1.width_ratio", "5"),
        ("05_7", "Twenty-five times", "F-A1.area_ratio", "25"),
        ("05_7", "five times as wide", "F-A1.width_ratio", "5"),
        ("05_7", "five times as tall", "F-A1.height_ratio", "5"),
        ("05_7", "Five times as many passengers", "F-W2.people_ratio", "5"),
        ("05_7", "five times as long on average", "F-A1.mean_wait_ratio", "5"),
        ("05_8", "Double a gap", "F-A1.double", "2"),
        ("05_8", "four times the waiting", "F-A1.double_area_ratio", "4"),
        // 06: two averages, over buses and over passengers
        ("06_1", "the two ways of counting", "F-S1.averages", "2"),
        ("06_1", "the tags say six minutes", "F-S1.bus_mean_gap", "6"),
        ("06_2", "One in six", "F-S1.person_short_share", "1/6"),
        ("06_2", "says two minutes", "F-T1.short", "2"),
        ("06_2", "Five in six", "F-S1.person_long_share", "5/6"),
        ("06_2", "say ten", "F-T1.long", "10"),
        (
            "06_4",
            "Eight minutes and forty seconds",
            "F-S1.person_gap_clock",
            "8:40",
        ),
        ("06_4", "A sixth of the passengers", "F-S1.person_short_share", "1/6"),
        ("06_4", "passengers say two", "F-T1.short", "2"),
        ("06_4", "five sixths", "F-S1.person_long_share", "5/6"),
        ("06_4", "say ten", "F-T1.long", "10"),
        ("06_4", "two honest averages", "F-S1.averages", "2"),
        ("06_5", "waits half that gap", "F-S1.wait_share_of_gap", "1/2"),
        ("06_5", "Half of eight minutes forty", "F-S1.wait_share_of_gap", "1/2"),
        ("06_5", "Half of eight minutes forty", "F-S1.person_gap_clock", "8:40"),
        ("06_5", "is four minutes twenty", "F-S1.half_person_gap_clock", "4:20"),
        ("06_7", "left four minutes twenty", "F-S1.age_clock", "4:20"),
        ("06_7", "Behind you four twenty", "F-S1.age_clock", "4:20"),
        ("06_7", "ahead of you four twenty", "F-W1.mean_wait_clock", "4:20"),
        (
            "06_7",
            "the eight forty of your gap",
            "F-S1.age_plus_wait_clock",
            "8:40",
        ),
        // 07: no bus was late; only the spacing changes
        ("07_3", "six minutes after the blue", "F-O1.even_offset", "6"),
        ("07_3", "Every gap is six minutes", "F-O1.even_gap", "6"),
        ("07_4", "Three minutes", "F-O1.even_wait", "3"),
        ("07_4", "Half of six", "F-O1.even_gap", "6"),
        ("07_4", "Half of six", "F-S1.wait_share_of_gap", "1/2"),
        ("07_5", "The same ten buses an hour", "F-G1.buses_every_offset", "10"),
        ("07_5", "average gap of six minutes", "F-G1.mean_gap_every_offset", "6"),
        ("07_6", "Two minutes apart", "F-T1.offset", "2"),
        ("07_6", "it's four minutes twenty", "F-O1.wait.d2", "4:20"),
        ("07_6", "Six apart", "F-O1.best_offset", "6"),
        ("07_6", "it's three", "F-O1.min_wait", "3"),
        ("07_6", "the two buses come together", "F-T1.routes", "2"),
        ("07_6", "every twelve minutes", "F-O1.together_period", "12"),
        ("07_6", "the average is six", "F-O1.max_wait", "6"),
        ("07_7", "from three minutes to six", "F-O1.min_wait", "3"),
        ("07_7", "from three minutes to six", "F-O1.max_wait", "6"),
        // 08: the formula from the triangles; the spread; never below half
        ("08_1", "The gap of two", "F-T1.short", "2"),
        ("08_1", "holds two square minutes", "F-A1.short_area", "2"),
        ("08_1", "the gap of ten", "F-T1.long", "10"),
        ("08_1", "holds fifty", "F-A1.long_area", "50"),
        ("08_1", "half its length squared", "F-F1.area_factor", "1/2"),
        (
            "08_3",
            "half the average of the squared gaps",
            "F-F1.area_factor",
            "1/2",
        ),
        (
            "08_4",
            "two squared is four and ten squared is a hundred",
            "F-F1.squares",
            "4,100",
        ),
        ("08_4", "two squared", "F-T1.short", "2"),
        ("08_4", "ten squared", "F-T1.long", "10"),
        ("08_4", "the average square is fifty-two", "F-F1.mean_square", "52"),
        ("08_4", "Six squared", "F-T1.mean_gap", "6"),
        ("08_4", "Six squared is only thirty-six", "F-F1.mean_gap_squared", "36"),
        ("08_4", "The extra sixteen", "F-F1.spread", "16"),
        ("08_5", "half the average gap, three minutes", "F-F1.area_factor", "1/2"),
        ("08_5", "half the average gap, three minutes", "F-T1.half_mean_gap", "3"),
        ("08_5", "plus sixteen", "F-F1.spread", "16"),
        ("08_5", "over twice six", "F-T1.mean_gap", "6"),
        ("08_5", "a minute and twenty seconds", "F-F1.penalty_clock", "1:20"),
        ("08_6", "that minute and twenty seconds", "F-W1.extra_clock", "1:20"),
        ("08_7", "less than half the average gap", "F-X1.bound_factor", "1/2"),
        ("08_8", "at least half the average gap", "F-X1.bound_factor", "1/2"),
        ("08_8", "exactly half", "F-X1.bound_factor", "1/2"),
        // 09: buses at random; no memory; the gap you land in; the scale
        ("09_2", "a bus every six minutes", "F-R1.mean_gap", "6"),
        ("09_4", "Six minutes.", "F-R1.mean_wait", "6"),
        ("09_4", "not half of it", "F-X1.bound_factor", "1/2"),
        ("09_5", "a gap averages six minutes", "F-R1.mean_gap", "6"),
        ("09_6", "the average gap squared, thirty-six", "F-R1.spread", "36"),
        ("09_6", "Three minutes", "F-R1.half_mean", "3"),
        ("09_6", "plus thirty-six", "F-R1.spread", "36"),
        ("09_6", "over twelve", "F-R1.twice_mean", "12"),
        ("09_6", "Six.", "F-R1.mean_wait", "6"),
        ("09_7", "six minutes before you", "F-R1.mean_age", "6"),
        ("09_7", "averages twelve minutes", "F-R1.person_gap", "12"),
        ("09_7", "twice the average gap", "F-R1.person_gap_ratio", "2"),
        ("09_8", "Even gaps wait three minutes", "F-O1.even_wait", "3"),
        ("09_8", "random buses wait six", "F-R1.mean_wait", "6"),
        ("09_8", "sits at four twenty", "F-W1.mean_wait_clock", "4:20"),
        ("09_8", "even past six", "F-R1.mean_wait", "6"),
        // 10: the same sum elsewhere; the class size paradox; the profiler;
        // Terada. The "one" of "one ten millisecond call" is the loop's single
        // long call, the 1 in F-P1.long_call_share = 1/(SHORT_CALLS + 1)
        ("10_2", "those twelve passengers", "F-W2.people", "12"),
        ("10_2", "two classes", "F-C1.classes", "2"),
        ("10_2", "one of two students and one of ten", "F-C1.sizes", "2,10"),
        ("10_2", "six students", "F-C1.school_mean", "6"),
        ("10_5", "Eight and two thirds", "F-C1.student_mean", "26/3"),
        ("10_5", "Ten of the twelve students", "F-C1.big_class_students", "10"),
        ("10_5", "of the twelve students", "F-C1.students", "12"),
        ("10_6", "in nineteen seventy-seven", "F-C1.year", "1977"),
        ("10_7", "one ten millisecond call", "F-P1.long_ms", "10"),
        ("10_7", "five two millisecond calls", "F-P1.short_calls", "5"),
        ("10_7", "two millisecond calls", "F-P1.short_ms", "2"),
        ("10_7", "half the samples", "F-P1.long_sample_share", "1/2"),
        ("10_7", "a sixth of the calls", "F-P1.long_call_share", "1/6"),
        ("10_9", "over a century ago", "F-H1.over_a_century", "true"),
        ("10_9", "In nineteen twenty-two", "F-H1.year", "1922"),
        // 11: back to the stop; the triangles' average; the planners
        ("11_1", "Ten buses an hour", "F-T1.buses_per_hour", "10"),
        ("11_1", "every six minutes on average", "F-T1.mean_gap", "6"),
        ("11_2", "Five times in six", "F-T1.long_time_share", "5/6"),
        ("11_3", "Fifty-two square minutes", "F-A1.total_area", "52"),
        ("11_3", "over twelve minutes", "F-T1.period", "12"),
        (
            "11_3",
            "Four minutes and twenty seconds",
            "F-W1.mean_wait_clock",
            "4:20",
        ),
        ("11_4", "one minute before your bus", "F-K1.early", "1"),
        ("11_5", "One minute", "F-K1.wait", "1"),
        ("11_6", "one minute before it", "F-K1.early", "1"),
        ("11_6", "Ten would wait for each blue bus", "F-K2.per_blue", "10"),
        ("11_6", "two for each red", "F-K2.per_red", "2"),
        ("11_7", "Five in six would still stand", "F-K2.in_long_share", "5/6"),
        ("11_7", "every wait would be one minute", "F-K2.waits", "1"),
        // 12: the last timetable, its area over its time, the cost and the
        // margin. "Forty" and "twelve" are spoken twice in 12_4, once in the
        // sum and once in the division, so each has a row per phrase
        ("12_1", "one a minute", "F-W2.per_minute", "1"),
        ("12_2", "four minutes after the blue", "F-Q1.offset", "4"),
        ("12_2", "the gaps are four and eight", "F-Q1.gaps", "4,8"),
        ("12_3", "The gap of four", "F-Q1.short", "4"),
        ("12_3", "half of four squared", "F-F1.area_factor", "1/2"),
        ("12_3", "eight square minutes", "F-Q1.short_area", "8"),
        ("12_3", "The gap of eight", "F-Q1.long", "8"),
        ("12_3", "half of eight squared", "F-F1.area_factor", "1/2"),
        ("12_3", "thirty-two square minutes", "F-Q1.long_area", "32"),
        ("12_4", "forty square minutes", "F-Q1.total_area", "40"),
        ("12_4", "the twelve minutes they cover", "F-T1.period", "12"),
        ("12_4", "Forty over twelve", "F-Q1.total_area", "40"),
        ("12_4", "over twelve is", "F-T1.period", "12"),
        ("12_4", "three and a third minutes", "F-Q1.mean_wait", "10/3"),
        (
            "12_4",
            "Three minutes and twenty seconds",
            "F-Q1.mean_wait_clock",
            "3:20",
        ),
        ("12_5", "closer the wait comes to three minutes", "F-O1.min_wait", "3"),
        ("12_6", "four minutes twenty", "F-W1.mean_wait_clock", "4:20"),
        ("12_7", "allow the full ten minutes", "F-W1.allowance", "10"),
        ("12_8", "Ten minutes is the worst case", "F-W1.allowance", "10"),
    ];

    /// The words the guard below listens for: every number word the full
    /// draft says. A line holding any of them must have a row in [`SPOKEN`].
    const NUMBER_WORDS: &[&str] = &[
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "twelve", "sixteen",
        "nineteen", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "hundred", "half", "halves", "third",
        "thirds", "sixth", "sixths", "twice", "double", "once",
    ];

    /// Phrases where "one" stands for a bus, not a count: the guard skips
    /// exactly these and still hears every other "one".
    const PRONOUN_ONES: &[&str] = &["the blue one", "missed one"];

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
        for (id, text) in script.lines() {
            let mut lower = text.to_lowercase();
            for phrase in PRONOUN_ONES {
                lower = lower.replace(phrase, " ");
            }
            let says_a_number = lower
                .split(|c: char| !c.is_alphabetic())
                .any(|w| NUMBER_WORDS.contains(&w));
            if says_a_number {
                assert!(
                    SPOKEN.iter().any(|(i, ..)| i == id),
                    "{id} says a number with no row in SPOKEN: {text}"
                );
            }
        }
    }

    /// The guard's negative control: a "one" outside the pronoun phrases
    /// still counts as a number, and the pronoun phrases are really in the
    /// script (a reworded line drops its exemption).
    #[test]
    fn the_pronoun_exemption_is_narrow() {
        let script = Script::parse(SCRIPT).unwrap();
        for phrase in PRONOUN_ONES {
            assert!(
                script.lines().iter().any(|(_, text)| text.contains(phrase)),
                "no line says {phrase:?}: drop it from PRONOUN_ONES"
            );
        }
        let mut lower = "one bus at a time, like the blue one".to_owned();
        for phrase in PRONOUN_ONES {
            lower = lower.replace(phrase, " ");
        }
        assert!(lower.split(|c: char| !c.is_alphabetic()).any(|w| w == "one"));
    }

    /// The film's honest direction (root Mail 6766): no line says the bus is
    /// always late, and the answer is given with every bus on time. The
    /// title's claim, longer than you expect, is against the guess of half
    /// the average gap, and comes from where random moments fall. (Chapter
    /// 07 of the
    /// draft names bunching only to set it apart, so the word itself is not
    /// banned.)
    #[test]
    fn the_narration_never_says_the_bus_is_always_late() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            assert!(!text.to_lowercase().contains("always late"), "{id}: {text}");
        }
        let answer = script.text("01_6").unwrap();
        assert!(answer.contains("every bus is exactly on time"), "{answer}");
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// The full draft holds the storyboard: every section after the
    /// storyboard's preamble is in storyboard-draft.md verbatim, so a
    /// chapter edited in one file cannot drift from its copy in the other.
    #[test]
    fn the_full_draft_holds_the_storyboard_verbatim() {
        let sections: Vec<&str> = STORYBOARD.split("\n## ").skip(1).collect();
        assert!(!sections.is_empty(), "storyboard.md has no sections");
        for s in sections {
            let section = format!("\n## {}", s.trim_end());
            let heading = s.lines().next().unwrap_or_default();
            assert!(
                DRAFT.contains(&section),
                "storyboard-draft.md lacks or changes \"## {heading}\""
            );
        }
        for key in keys() {
            assert!(
                DRAFT.contains(&format!("\n## {key}:")),
                "the draft has no chapter {key}"
            );
        }
    }
}
