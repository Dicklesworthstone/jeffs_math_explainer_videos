//! Mental Math Tricks That Actually Work (and Why) — Video F of the corpus:
//! squares of numbers ending in 5, two-digit products by parts as a running
//! total, two numbers equally far from a middle, squares near 50 (the trick
//! Hans Bethe showed Richard Feynman), percents by blocks, halving and
//! doubling, and round then fix, each with the picture that makes it true
//! and "your turn" quizzes with a countdown, rendered end to end by
//! franken_manim's native Rust front door and the corpus's shared kit.
//!
//! ```text
//! mental facts                         # every number the video shows or says (key<TAB>value)
//! mental script                        # the narration (generated from docs/storyboard.md)
//! mental render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! mental help                          # the full option list
//! ```
//!
//! Every chapter is a real scene (`chapters/`), built on the shared drawing
//! kit of `chapters/draw.rs`.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat (the
/// tests hold the script, the registry and the quizzes to it).
#[cfg(test)]
const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "mental_math",
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

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// The line-gap classes of this video's real script. The ten plain
    /// between-line pairs of chapters 02 and 07 are the ones the narration
    /// pilot built and the owner will hear; the pair across every chapter join
    /// is not one class; and the planned gaps of the whole script spread where
    /// the constant breath is the planted negative. The owner chose the
    /// thought rule for the finals (2026-10-14), so the video runs on it.
    #[test]
    fn the_thought_classes_of_the_real_script_follow_the_thought() {
        use explainer_kit::GapClass::{Beat, Continuation, Topic};
        use explainer_kit::LineGaps;
        use explainer_kit::narration::{BREATH, check_gap_spread};

        assert_eq!(
            EXPLAINER.line_gaps,
            LineGaps::Thought,
            "the owner's choice for the finals"
        );
        let script = Script::parse(SCRIPT).unwrap();
        let class = |a: &str, b: &str| explainer_kit::classify_gap(script.text(a).unwrap(), script.text(b).unwrap());
        for (a, b, want) in [
            ("02_1", "02_2", Beat),
            ("02_2", "02_3", Continuation),
            ("02_3", "02_4", Continuation),
            ("02_5", "02_6", Continuation),
            ("02_7", "02_8", Beat),
            ("02_8", "02_9", Continuation),
            ("07_1", "07_2", Beat),
            ("07_2", "07_3", Topic),
            ("07_4", "07_5", Topic),
            ("07_5", "07_6", Topic),
        ] {
            assert_eq!(class(a, b), want, "{a} -> {b}");
        }
        let lines = script.lines();
        let chapter_of = |id: &str| id.split('_').next().unwrap_or(id).to_owned();
        let (within, across): (Vec<_>, Vec<_>) = lines
            .windows(2)
            .partition(|w| chapter_of(&w[0].0) == chapter_of(&w[1].0));
        let planned: Vec<f64> = within
            .iter()
            .map(|w| explainer_kit::classify_gap(&w[0].1, &w[1].1).audible_s())
            .collect();
        check_gap_spread(&planned).expect("the planned gaps of the real script spread");
        assert!(
            check_gap_spread(&vec![BREATH; planned.len()]).is_err(),
            "constant gaps fail"
        );
        // the chapter joins are not ten copies of one value: all three classes open chapters
        let joins: Vec<_> = across
            .iter()
            .map(|w| explainer_kit::classify_gap(&w[0].1, &w[1].1))
            .collect();
        assert_eq!(joins.len(), keys().len() - 1);
        for want in [Beat, Continuation, Topic] {
            assert!(joins.contains(&want), "no chapter opens as {want:?}: {joins:?}");
        }
    }

    /// Every quiz in `model::QUIZZES` is a storyboard beat that asks a
    /// question and then counts down for exactly the quiz's seconds, and
    /// every countdown in the storyboard is one of those quizzes.
    #[test]
    fn every_countdown_is_a_quiz_and_every_quiz_has_its_countdown() {
        let board = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let beats: Vec<_> = board.iter().flat_map(|c| &c.beats).collect();
        for q in model::QUIZZES {
            let asked: Vec<_> = beats.iter().filter(|b| b.id == q.line).collect();
            assert_eq!(asked.len(), 1, "{}: one beat asks the quiz", q.line);
            assert!(asked[0].predict(), "{}: a quiz asks a question", q.line);
            assert_eq!(
                asked[0].listen_s(),
                Some(f64::from(q.seconds)),
                "{}: the countdown",
                q.line
            );
        }
        let counted: Vec<&str> = beats
            .iter()
            .filter(|b| b.hold.contains("countdown"))
            .map(|b| b.id.as_str())
            .collect();
        let quizzes: Vec<&str> = model::QUIZZES.iter().map(|q| q.line).collect();
        assert_eq!(counted, quizzes);
    }

    /// A whole number below a million as the script says it: "one hundred
    /// five", "eleven thousand twenty-five".
    fn words(n: u64) -> String {
        const SMALL: [&str; 20] = [
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
        ];
        const TENS: [&str; 10] = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        match n {
            0..20 => SMALL[n as usize].to_owned(),
            20..100 if n.is_multiple_of(10) => TENS[(n / 10) as usize].to_owned(),
            20..100 => format!("{}-{}", TENS[(n / 10) as usize], SMALL[(n % 10) as usize]),
            100..1000 if n.is_multiple_of(100) => format!("{} hundred", words(n / 100)),
            100..1000 => format!("{} hundred {}", words(n / 100), words(n % 100)),
            _ if n.is_multiple_of(1000) => format!("{} thousand", words(n / 1000)),
            _ => format!("{} thousand {}", words(n / 1000), words(n % 1000)),
        }
    }

    /// How the script reads a fact aloud.
    #[derive(Clone, Copy)]
    enum Read {
        /// In full: 1058 is "one thousand fifty-eight".
        Full,
        /// Whole hundreds counted: 2500 is "twenty-five hundred".
        Hundreds,
        /// A square near fifty, as two pairs: 2304 is "twenty-three oh four".
        Pairs,
        /// Money: "7.20" is "seven dollars twenty", "9.00" is "nine dollars".
        Dollars,
        /// Money under a dollar: "0.04" is "four cents".
        Cents,
        /// A count of halves: 103 is "fifty-one and a half".
        Halves,
        /// The round estimate: 1000 is "a thousand".
        Round,
    }

    /// The fact's value as the voice reads it; an error if the value cannot
    /// be read that way (money that is not money, hundreds that are not whole).
    fn read(how: Read, value: &str) -> Result<String, String> {
        let whole = || {
            value
                .parse::<u64>()
                .map_err(|_| format!("{value:?} is not a whole number"))
        };
        let cents = || -> Result<u64, String> {
            let not_money = || format!("{value:?} is not money");
            let (dollars, cents) = value.split_once('.').ok_or_else(not_money)?;
            let dollars: u64 = dollars.parse().map_err(|_| not_money())?;
            let cents: u64 = cents.parse().map_err(|_| not_money())?;
            Ok(dollars * 100 + cents)
        };
        Ok(match how {
            Read::Full => words(whole()?),
            Read::Hundreds if whole()?.is_multiple_of(100) => format!("{} hundred", words(whole()? / 100)),
            Read::Hundreds => return Err(format!("{value} is not whole hundreds")),
            Read::Pairs => {
                let (front, back) = (whole()? / 100, whole()? % 100);
                let back = if back < 10 {
                    format!("oh {}", words(back))
                } else {
                    words(back)
                };
                format!("{} {back}", words(front))
            }
            Read::Dollars => match (cents()? / 100, cents()? % 100) {
                (dollars, 0) => format!("{} dollars", words(dollars)),
                (dollars, cents) => format!("{} dollars {}", words(dollars), words(cents)),
            },
            Read::Cents if cents()? < 100 => format!("{} cents", words(cents()?)),
            Read::Cents => return Err(format!("{value} is a dollar or more")),
            Read::Halves => format!("{} and a half", words(whole()? / 2)),
            Read::Round if whole()? == 1000 => "a thousand".to_owned(),
            Read::Round => return Err(format!("{value} has no round reading")),
        })
    }

    /// The part of a fact a line speaks: `key` is the whole value, `key#2`
    /// the third item of a comma list, `key@0` and `key@1` the two numbers
    /// of a quiz's ask ("85^2", "6x43").
    fn pick(facts: &explainer_kit::Facts, cite: &str) -> Result<String, String> {
        let fact = |key: &str| facts.get(key).ok_or_else(|| format!("no fact {key}"));
        let item = |key: &str, i: &str, split: &[char]| -> Result<String, String> {
            let i: usize = i.parse().map_err(|_| format!("{cite}: not an index"))?;
            let value = fact(key)?;
            let part = value
                .split(split)
                .nth(i)
                .ok_or_else(|| format!("{cite}: no such item"))?;
            Ok(part.to_owned())
        };
        if let Some((key, i)) = cite.split_once('#') {
            item(key, i, &[','])
        } else if let Some((key, i)) = cite.split_once('@') {
            item(key, i, &['^', 'x'])
        } else {
            Ok(fact(cite)?.to_owned())
        }
    }

    /// Every number the narration speaks, tied to the fact it speaks for and
    /// to the way it is read. The words are spelled from the fact's value
    /// and must be in the line, so a changed number, or a changed reading,
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, Read, &str)] = &[
        ("01_1", Read::Full, "F-Q.01_1.ask@0"),
        ("01_2", Read::Full, "F-H1.trick.square"),
        ("01_2", Read::Full, "F-H1.trick.front"),
        ("01_2", Read::Full, "F-H1.trick.next"),
        ("01_2", Read::Full, "F-H1.trick.hundreds"),
        ("01_4", Read::Full, "F-H1.paper.row_count"),
        ("01_4", Read::Full, "F-H1.paper.carries"),
        ("01_6", Read::Full, "F-H1.trick.hundreds"),
        ("02_1", Read::Full, "F-M.n35.strip_long"),
        ("02_1", Read::Full, "F-T1.n35.front"),
        ("02_2", Read::Full, "F-T1.n35.next"),
        ("02_2", Read::Full, "F-T1.n35.hundreds"),
        ("02_3", Read::Full, "F-T1.n35.square"),
        ("02_5", Read::Full, "F-M.n35.strip_wide"),
        ("02_7", Read::Full, "F-M.n35.tall"),
        ("02_7", Read::Full, "F-M.n35.wide"),
        ("02_8", Read::Hundreds, "F-M.n35.rectangle_area"),
        ("02_9", Read::Full, "F-M.n35.corner"),
        ("03_1", Read::Full, "F-Q.03_1.ask@0"),
        ("03_2", Read::Full, "F-Q.03_1.steps#0"),
        ("03_2", Read::Full, "F-Q.03_1.steps#1"),
        ("03_2", Read::Full, "F-Q.03_1.steps#2"),
        ("03_2", Read::Full, "F-Q.03_1.answer"),
        ("03_5", Read::Full, "F-M.n35.corner"),
        ("03_7", Read::Full, "F-T1.n135.front"),
        ("03_7", Read::Full, "F-T1.n135.next"),
        ("04_1", Read::Full, "F-T2.estimate#0"),
        ("04_1", Read::Full, "F-T2.estimate#1"),
        ("04_1", Read::Round, "F-T2.estimate#2"),
        ("04_3", Read::Full, "F-T2.parts.first#0"),
        ("04_3", Read::Full, "F-T2.parts.first#1"),
        ("04_3", Read::Full, "F-T2.parts.second#0"),
        ("04_3", Read::Full, "F-T2.parts.second#1"),
        ("04_3", Read::Full, "F-T2.held.pieces"),
        ("04_4", Read::Full, "F-T2.size.lead"),
        ("04_5", Read::Hundreds, "F-T2.pieces#0"),
        ("04_5", Read::Full, "F-T2.size.too_small"),
        ("04_7", Read::Full, "F-T2.pieces#1"),
        ("04_7", Read::Full, "F-T2.pieces#3"),
        ("04_8", Read::Full, "F-T2.held.pieces"),
        ("05_2", Read::Hundreds, "F-T2.running#0"),
        ("05_2", Read::Full, "F-T2.running#1"),
        ("05_2", Read::Full, "F-T2.running#2"),
        ("05_2", Read::Full, "F-T2.running#3"),
        ("05_4", Read::Full, "F-Q.05_4.ask@0"),
        ("05_4", Read::Full, "F-Q.05_4.ask@1"),
        ("05_5", Read::Full, "F-Q.05_4.steps#0"),
        ("05_5", Read::Full, "F-Q.05_4.answer"),
        ("05_6", Read::Full, "F-Q.05_6.ask@0"),
        ("05_6", Read::Full, "F-Q.05_6.ask@1"),
        ("05_7", Read::Full, "F-Q.05_6.steps#0"),
        ("05_7", Read::Full, "F-Q.05_6.steps#1"),
        ("05_7", Read::Full, "F-Q.05_6.steps#2"),
        ("05_7", Read::Full, "F-Q.05_6.steps#3"),
        ("05_7", Read::Full, "F-T1.n135.square"),
        ("06_1", Read::Dollars, "F-T5.bill"),
        ("06_1", Read::Dollars, "F-T5.bill.ten"),
        ("06_2", Read::Dollars, "F-T5.bill.five"),
        ("06_2", Read::Dollars, "F-T5.bill.fifteen"),
        ("06_5", Read::Dollars, "F-Q.06_4.steps#0"),
        ("06_5", Read::Dollars, "F-Q.06_4.steps#1"),
        ("06_5", Read::Dollars, "F-Q.06_4.answer"),
        ("06_6", Read::Full, "F-T5.swap.deep"),
        ("06_6", Read::Full, "F-T5.swap.across"),
        ("06_7", Read::Dollars, "F-T5.swap.other_way"),
        ("06_8", Read::Full, "F-T5.swap.cents"),
        ("07_1", Read::Full, "F-T6.five.ten"),
        ("07_1", Read::Full, "F-T6.five.product"),
        ("07_2", Read::Full, "F-T6.halve.halved"),
        ("07_2", Read::Full, "F-T6.halve.doubled"),
        ("07_2", Read::Full, "F-T6.halve.product"),
        ("07_3", Read::Full, "F-Q.07_3.ask@0"),
        ("07_3", Read::Full, "F-Q.07_3.ask@1"),
        ("07_4", Read::Full, "F-Q.07_3.steps#0"),
        ("07_4", Read::Full, "F-Q.07_3.steps#1"),
        ("07_4", Read::Full, "F-Q.07_3.answer"),
        ("07_5", Read::Dollars, "F-T7.price.each"),
        ("07_5", Read::Dollars, "F-T7.price.rounded"),
        ("07_5", Read::Cents, "F-T7.price.fix"),
        ("07_5", Read::Dollars, "F-T7.price.total"),
        ("07_7", Read::Dollars, "F-Q.07_6.steps#1"),
        ("07_7", Read::Cents, "F-Q.07_6.steps#2"),
        ("07_7", Read::Dollars, "F-Q.07_6.answer"),
        ("08_1", Read::Full, "F-M.n50.tall"),
        ("08_1", Read::Full, "F-M.n50.wide"),
        ("08_2", Read::Full, "F-T3.middle"),
        ("08_2", Read::Full, "F-T3.distance"),
        ("08_5", Read::Hundreds, "F-T3.middle_squared"),
        ("08_5", Read::Full, "F-T3.distance_squared"),
        ("08_5", Read::Full, "F-T3.product"),
        ("08_6", Read::Full, "F-Q.08_6.ask@0"),
        ("08_6", Read::Full, "F-Q.08_6.ask@1"),
        ("08_7", Read::Full, "F-Q.08_6.steps#0"),
        ("08_7", Read::Full, "F-Q.08_6.steps#1"),
        ("08_7", Read::Hundreds, "F-Q.08_6.steps#2"),
        ("08_7", Read::Full, "F-Q.08_6.answer"),
        ("08_8", Read::Halves, "F-T3.no_middle.sum"),
        ("09_1", Read::Full, "F-C1.bethe.number"),
        ("09_2", Read::Hundreds, "F-C1.bethe.rough"),
        ("09_3", Read::Pairs, "F-C1.bethe.exact"),
        ("09_5", Read::Hundreds, "F-T4.fifty_squared"),
        ("10_1", Read::Full, "F-M.n48.strip_wide"),
        ("10_2", Read::Full, "F-T4.below.fifties"),
        ("10_2", Read::Hundreds, "F-T4.below.fifties_area"),
        ("10_2", Read::Hundreds, "F-T4.below.rough"),
        ("10_3", Read::Full, "F-M.n48.corner"),
        ("10_5", Read::Full, "F-M.n53.strip_wide"),
        ("10_6", Read::Full, "F-T4.above.fifties"),
        ("10_6", Read::Hundreds, "F-T4.above.fifties_area"),
        ("10_6", Read::Full, "F-M.n53.corner"),
        ("10_6", Read::Full, "F-T4.above.square"),
        ("11_1", Read::Full, "F-Q.11_1.ask@0"),
        ("11_2", Read::Full, "F-Q.11_1.steps#1"),
        ("11_2", Read::Full, "F-Q.11_1.steps#2"),
        ("11_2", Read::Pairs, "F-Q.11_1.answer"),
        ("11_3", Read::Hundreds, "F-T4.carries.rough"),
        ("11_3", Read::Full, "F-T4.carries.corner"),
        ("11_3", Read::Pairs, "F-T4.carries.square"),
        ("12_2", Read::Dollars, "F-Q.12_1.steps#0"),
        ("12_2", Read::Dollars, "F-Q.12_1.answer"),
        ("12_3", Read::Full, "F-Q.12_3.ask@0"),
        ("12_3", Read::Full, "F-Q.12_3.ask@1"),
        ("12_4", Read::Hundreds, "F-Q.12_3.steps#2"),
        ("12_4", Read::Full, "F-Q.12_3.answer"),
        ("13_1", Read::Full, "F-Q.13_1.ask@0"),
        ("13_1", Read::Full, "F-Q.13_1.ask@1"),
        ("13_2", Read::Halves, "F-N.middle_halves"),
        ("13_3", Read::Full, "F-N.product"),
        ("13_5", Read::Full, "F-N.estimate#0"),
        ("13_5", Read::Full, "F-N.estimate#1"),
        ("13_5", Read::Hundreds, "F-N.estimate#2"),
        ("14_1", Read::Full, "F-Q.14_1.ask@0"),
        ("14_2", Read::Full, "F-T1.n55.front"),
        ("14_2", Read::Full, "F-T1.n55.next"),
        ("14_2", Read::Full, "F-T1.n55.hundreds"),
        ("14_2", Read::Pairs, "F-T1.n55.square"),
        ("14_3", Read::Full, "F-M.n35.tall"),
        ("14_3", Read::Full, "F-M.n35.wide"),
        ("14_3", Read::Full, "F-M.n35.corner"),
        ("14_3", Read::Full, "F-M.n48.tall"),
        ("14_3", Read::Full, "F-M.n48.wide"),
        ("14_3", Read::Full, "F-M.n48.corner"),
        ("14_3", Read::Full, "F-M.n50.tall"),
        ("14_3", Read::Full, "F-M.n50.wide"),
        ("14_3", Read::Full, "F-M.n50.corner"),
    ];

    #[test]
    fn the_readings_spell_numbers_as_the_script_does() {
        assert_eq!(words(105), "one hundred five");
        assert_eq!(words(11_025), "eleven thousand twenty-five");
        assert_eq!(words(18_225), "eighteen thousand two hundred twenty-five");
        assert_eq!(words(1040), "one thousand forty");
        let read = |how, value| read(how, value).unwrap();
        assert_eq!(read(Read::Hundreds, "2500"), "twenty-five hundred");
        assert_eq!(read(Read::Pairs, "2304"), "twenty-three oh four");
        assert_eq!(read(Read::Pairs, "3025"), "thirty twenty-five");
        assert_eq!(read(Read::Dollars, "7.20"), "seven dollars twenty");
        assert_eq!(read(Read::Dollars, "9.00"), "nine dollars");
        assert_eq!(read(Read::Cents, "0.04"), "four cents");
        assert_eq!(read(Read::Halves, "103"), "fifty-one and a half");
        assert_eq!(read(Read::Round, "1000"), "a thousand");
        // a reading that does not fit its value is an error, not a guess
        assert!(super::tests::read(Read::Hundreds, "1058").is_err());
        assert!(super::tests::read(Read::Cents, "1.04").is_err());
        assert!(super::tests::read(Read::Dollars, "nine").is_err());
    }

    /// A word that is, or is part of, a spoken number.
    fn number_word(word: &str) -> bool {
        matches!(word, "hundred" | "thousand" | "oh") || word.split('-').all(|part| (0..100).any(|n| words(n) == part))
    }

    /// The line says the number: its words stand together inside one clause
    /// (between commas and full stops), and are not a piece of a longer
    /// number, so "four" is not found in "forty-four" or "four hundred".
    fn says(line: &str, phrase: &str) -> bool {
        let want: Vec<&str> = phrase.split(' ').collect();
        line.split(['.', ',', '?', '!']).any(|clause| {
            let have: Vec<&str> = clause.split_whitespace().collect();
            (0..have.len()).any(|at| {
                have[at..].starts_with(&want)
                    && !(at > 0 && number_word(have[at - 1]) && number_word(want[0]))
                    && !have.get(at + want.len()).is_some_and(|next| number_word(next))
            })
        })
    }

    #[test]
    fn a_number_is_only_said_when_it_stands_alone() {
        assert!(says(
            "eight times nine is seventy-two. then write twenty-five.",
            "eight"
        ));
        assert!(says(
            "eight times nine is seventy-two. then write twenty-five.",
            "seventy-two"
        ));
        assert!(!says("then forty-four.", "four"));
        assert!(!says("four hundred, minus one.", "four"));
        assert!(says("four hundred, minus one.", "four hundred"));
        assert!(!says("one hundred forty.", "one hundred"));
        assert!(says(
            "ten times ten, one hundred. add ten times four, one hundred forty.",
            "one hundred"
        ));
        assert!(!says("twenty-three hundred.", "twenty-three"));
        assert!(says("call it four times twenty, eighty dollars.", "eighty dollars"));
        assert!(says("a quarter of eight. two dollars.", "two dollars"));
        assert!(says("thirty twenty-five, both ways.", "thirty twenty-five"));
    }

    #[test]
    fn every_spoken_number_is_a_fact() {
        let facts = facts::facts().unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        let mut missing = Vec::new();
        for &(line, how, cite) in SPOKEN {
            let said = script.text(line).map(str::to_lowercase);
            let phrase = pick(&facts, cite).and_then(|value| read(how, &value));
            match (said, phrase) {
                (Some(said), Ok(phrase)) if says(&said, &phrase) => {}
                (Some(_), Ok(phrase)) => {
                    missing.push(format!("{line}: \"{phrase}\" ({cite}) is not in the line"));
                }
                (None, _) => missing.push(format!("{line}: no such line")),
                (_, Err(why)) => missing.push(format!("{line}: {why}")),
            }
        }
        assert!(missing.is_empty(), "\n{}", missing.join("\n"));
        // every chapter that speaks numbers is covered
        let covered: std::collections::BTreeSet<&str> = SPOKEN.iter().map(|(line, ..)| &line[..2]).collect();
        assert_eq!(covered.len(), keys().len());
    }
}
