//! Two Losing Games Can Win, Here Is the Catch: Parrondo's games. Game A is
//! one slightly bent coin; game B reads your capital and tosses a bad coin
//! when it divides by three, a good one otherwise. Each loses on its own; a
//! fair coin choosing A or B before every round wins. The video follows the
//! long-run share of rounds spent at each remainder of the capital (a crowd
//! on three corners), rendered end to end by franken_manim's native Rust
//! front door and the corpus's shared kit.
//!
//! ```text
//! parrondo facts                     # every number the video shows or says (key<TAB>value)
//! parrondo script                    # the narration (generated from docs/storyboard.md)
//! parrondo render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! parrondo help                      # the full option list
//! ```
//!
//! The registry, `docs/storyboard.md` and `script.tsv` hold only chapters
//! with native scenes (01–18). The whole planned film is
//! `docs/storyboard-draft.md`; its facts are all computed here already.

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
    crate_name: "parrondo",
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
    /// for and the value those words denote. A changed coin, seed or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        // 01: one player, game A's coin, a cent a round, the long run
        ("01_2", "seven dollars", "run.start", "7"),
        ("01_4", "forty-nine and a half percent", "a.hundred.wins", "49.5"),
        ("01_6", "forty-nine and a half wins", "a.hundred.wins", "49.5"),
        ("01_6", "fifty and a half losses", "a.hundred.losses", "50.5"),
        ("01_8", "One dollar per hundred rounds", "a.hundred.net", "-1"),
        ("01_8", "a cent a round", "pub.a.mean.cents", "-1.00"),
        ("01_9", "a hundred thousand rounds", "run.rounds", "100000"),
        ("01_9", "about a thousand dollars below", "run.a.expected.net", "-1000"),
        ("01_10", "the cent a round decides", "pub.a.mean.cents", "-1.00"),
        // 02–03: remainder examples, published chances, hypothetical and actual run
        ("02_2", "rows of three", "rules.modulus", "3"),
        (
            "02_3",
            "Seven dollars make",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "02_3",
            "Eight leave two",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "02_3",
            "Nine fill three",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        ("02_3", "zero, one or two", "rules.modulus", "3"),
        (
            "02_4",
            "nine and a half percent",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "02_5",
            "seventy-four and a half percent",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "02_6",
            "ten dollars",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "02_7",
            "Ten dollars make three rows",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "02_8",
            "Seven becomes eight, and eight becomes nine",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        ("03_1", "Two counts in every three", "rules.modulus", "3"),
        ("03_2", "fifty-two point eight percent", "pub.b.thirds.win.pct", "52.8"),
        ("03_4", "a hundred thousand rounds", "run.rounds", "100000"),
        ("03_5", "a third", "thirds.pct", "33.3"),
        ("03_5", "more than thirty-eight percent", "run.b.at0.pct", "38.4"),
        // 04–05: ordinary-coin control, long-run means and the same player's mix
        ("04_2", "forty-nine percent", "ordinary.coins.pct", "49,48"),
        ("04_2", "one forty-eight", "ordinary.coins.pct", "49,48"),
        ("04_3", "forty-eight and a half percent", "ordinary.mixed.pct", "48.5"),
        ("04_4", "Game A loses a cent a round", "pub.a.mean.cents", "-1.00"),
        ("04_4", "a little under a cent", "pub.b.mean.cents", "-0.87"),
        ("04_6", "a hundred thousand rounds", "run.rounds", "100000"),
        ("04_6", "well over a thousand dollars", "run.mix.net", "1748"),
        ("05_1", "a multiple of three", "rules.modulus", "3"),
        ("05_1", "more than thirty-eight percent", "run.b.at0.pct", "38.4"),
        ("05_2", "a multiple of three", "rules.modulus", "3"),
        ("05_2", "thirty-four and a half percent", "run.mix.at0.pct", "34.5"),
        ("05_2", "closer to a third", "thirds.pct", "33.3"),
        ("05_3", "one and a half cents a round", "pub.mix.mean.cents", "+1.57"),
        // 06–07: remainder states, measured visits and the rounded teaching example
        ("06_1", "three corners", "rules.modulus", "3"),
        ("06_1", "zero, one and two", "rules.modulus", "3"),
        ("06_2", "seven dollars", "run.start", "7"),
        (
            "06_4",
            "Eight dollars plus one is nine",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "06_4",
            "nine leaves nothing over",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        (
            "06_5",
            "forward nine and a half percent",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "06_5",
            "forward seventy-four and a half percent",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "06_6",
            "forward forty-nine and a half percent",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "06_6",
            "back fifty and a half",
            "coins.published.tails.pct",
            "50.5,90.5,25.5",
        ),
        (
            "07_1",
            "six dollars or six hundred",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        ("07_2", "corner zero", "rules.modulus", "3"),
        ("07_3", "One third", "thirds.pct", "33.3"),
        ("07_3", "thirty-three and a half percent", "run.a.at0.pct", "33.5"),
        ("07_4", "more than thirty-eight percent", "run.b.at0.pct", "38.4"),
        ("07_5", "half a point", "rules.handicap", "1/200"),
        ("07_6", "a ten percent coin", "coins.round.heads.pct", "50,10,75"),
        ("07_6", "seventy-five percent coin", "coins.round.heads.pct", "50,10,75"),
        ("07_7", "equal thirds", "thirds.pct", "33.3"),
        (
            "07_7",
            "fifty-three point three percent",
            "round.b.thirds.win.pct",
            "53.3",
        ),
        // 08–09: expected crowds, actual tosses, conserved flows and weights
        ("08_1", "really a third each", "thirds.pct", "33.3"),
        ("08_1", "twenty-six copies", "crowd.size", "26"),
        ("08_1", "corner zero", "rules.modulus", "3"),
        ("08_3", "At corner two", "rules.modulus", "3"),
        ("08_3", "A tenth of the crowd", "coins.round.heads.pct", "50,10,75"),
        ("08_3", "nine tenths fall back", "coins.round.tails.pct", "50,90,25"),
        (
            "08_3",
            "Two point six and twenty-three point four",
            "crowd.round.1",
            "0.0,2.6,23.4",
        ),
        ("08_3", "twenty-six times each chance", "crowd.size", "26"),
        (
            "08_4",
            "three quarters of corner two",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        ("08_4", "the total stays twenty-six", "crowd.size", "26"),
        ("08_5", "corner zero", "rules.modulus", "3"),
        ("08_5", "steadies above a third", "thirds.pct", "33.3"),
        ("08_6", "ten players on corner zero", "crowd.counts", "10,4,12"),
        ("08_6", "four on corner one", "crowd.counts", "10,4,12"),
        ("08_6", "twelve on corner two", "crowd.counts", "10,4,12"),
        ("08_6", "Not a third each", "thirds.pct", "33.3"),
        ("08_7", "these twenty-six", "crowd.size", "26"),
        ("08_8", "a thousand rounds", "realcrowd.rounds", "1000"),
        ("08_8", "exactly ten, four and twelve", "crowd.counts", "10,4,12"),
        ("08_8", "only forty times", "realcrowd.exact", "40"),
        ("08_8", "on average", "realcrowd.mean", "10.0,4.0,12.0"),
        ("09_1", "ten players toss the bad coin", "crowd.counts", "10,4,12"),
        ("09_1", "one wins and moves forward", "crowd.forward", "1,3,9"),
        ("09_1", "nine drop back to corner two", "crowd.back", "9,1,3"),
        ("09_2", "the good coin splits four players", "crowd.counts", "10,4,12"),
        ("09_2", "Three move on to corner two", "crowd.forward", "1,3,9"),
        ("09_2", "one falls back to zero", "crowd.back", "9,1,3"),
        ("09_3", "it splits twelve", "crowd.counts", "10,4,12"),
        ("09_3", "Nine wrap around to zero", "crowd.forward", "1,3,9"),
        ("09_3", "three fall back to one", "crowd.back", "9,1,3"),
        ("09_5", "one plus nine, ten", "crowd.arrivals", "1+9,1+3,9+3"),
        ("09_5", "one plus three, four", "crowd.arrivals", "1+9,1+3,9+3"),
        ("09_5", "nine plus three, twelve", "crowd.arrivals", "1+9,1+3,9+3"),
        ("09_6", "One, three and nine make thirteen", "crowd.winners", "13"),
        ("09_6", "exactly half of twenty-six", "round.b.win", "1/2"),
        ("09_6", "Without the half point", "rules.handicap", "1/200"),
        ("09_7", "Ten of twenty-six", "crowd.counts", "10,4,12"),
        ("09_7", "thirty-eight and a half percent", "crowd.share0.pct", "38.5"),
        ("09_7", "more than a third", "thirds.pct", "33.3"),
        ("09_8", "corner zero", "rules.modulus", "3"),
        (
            "09_9",
            "five thirteenths, two thirteenths and six thirteenths",
            "round.b.weights",
            "5/13,2/13,6/13",
        ),
        // 10–11: the trap, an independent die, then the computed mixed crowd.
        ("10_1", "corner zero", "rules.modulus", "3"),
        ("10_2", "nine times in ten", "coins.round.tails.pct", "50,90,25"),
        ("10_2", "around to zero again", "rules.modulus", "3"),
        ("10_3", "ten, four and twelve", "crowd.counts", "10,4,12"),
        ("10_4", "three players at corner one", "crowd.forward", "1,3,9"),
        ("10_4", "nine at corner two", "crowd.forward", "1,3,9"),
        ("10_4", "of its twelve wins a round, nine", "crowd.good_wins_at2", "3/4"),
        ("10_4", "back to zero", "rules.modulus", "3"),
        ("10_5", "a multiple of three", "rules.modulus", "3"),
        ("10_7", "one round in three", "thirds.pct", "33.3"),
        (
            "10_7",
            "fifty-three point three percent",
            "round.b.thirds.win.pct",
            "53.3",
        ),
        (
            "10_8",
            "fifty-three point three percent",
            "round.b.thirds.win.pct",
            "53.3",
        ),
        ("10_8", "exactly half", "round.b.win", "1/2"),
        ("11_2", "At corner zero", "rules.modulus", "3"),
        ("11_2", "A's fifty percent", "coins.round.heads.pct", "50,10,75"),
        ("11_2", "B's ten percent", "coins.round.heads.pct", "50,10,75"),
        (
            "11_2",
            "forward thirty percent in all",
            "round.mix.coins.pct",
            "30,62.5",
        ),
        (
            "11_3",
            "between fifty and seventy-five",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        ("11_3", "sixty-two and a half percent", "round.mix.coins.pct", "30,62.5"),
        ("11_4", "ten, four and twelve", "crowd.counts", "10,4,12"),
        ("11_5", "Off corner zero", "rules.modulus", "3"),
        ("11_5", "thirty-eight point five percent", "crowd.share0.pct", "38.5"),
        (
            "11_5",
            "about thirty-four point six",
            "mixshift.round.1",
            "34.6,28.8,36.5",
        ),
        (
            "11_6",
            "about thirty-four point six percent",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        (
            "11_6",
            "twenty-five point four percent",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        (
            "11_6",
            "up from fifteen point four",
            "round.b.weights.pct",
            "38.5,15.4,46.2",
        ),
        ("11_7", "toward equal thirds", "thirds.pct", "33.3"),
        ("11_7", "off corner zero", "rules.modulus", "3"),
        ("12_2", "wins exactly half the time", "round.mix.a_rounds.win", "1/2"),
        (
            "12_3",
            "a seventy-five percent coin for a ten percent one",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        ("12_4", "thirty-eight point five percent", "crowd.share0.pct", "38.5"),
        (
            "12_5",
            "thirty-four point six percent",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        ("12_6", "With a third of B's rounds", "thirds.pct", "33.3"),
        (
            "12_6",
            "fifty-three point three percent",
            "round.b.thirds.win.pct",
            "53.3",
        ),
        ("12_6", "own thirty-eight point five", "crowd.share0.pct", "38.5"),
        ("12_6", "exactly half", "round.b.win", "1/2"),
        (
            "12_7",
            "seventy-five percent, minus sixty-five percent",
            "round.b.line.pct",
            "75,65",
        ),
        ("12_7", "exactly five thirteenths", "round.b.fair_share", "5/13"),
        (
            "12_8",
            "thirty-four point six percent",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        (
            "12_9",
            "Fifty-two point five percent",
            "round.mix.b_rounds.win.pct",
            "52.5",
        ),
        ("12_9", "the A rounds' fifty", "round.mix.a_rounds.win", "1/2"),
        ("12_9", "fifty-one point three percent", "round.mix.win.pct", "51.3"),
        (
            "12_10",
            "just over two and a half cents a round",
            "round.mix.mean.cents",
            "+2.54",
        ),
        ("12_12", "keeps a third of the rounds", "thirds.pct", "33.3"),
        ("12_12", "most of the way", "round.mix.toward_thirds.pct", "76"),
        ("13_1", "the half point off every coin", "rules.handicap", "1/200"),
        ("13_2", "losing a cent a round", "pub.a.mean.cents", "-1.00"),
        ("13_3", "from thirty-eight point five", "crowd.share0.pct", "38.5"),
        (
            "13_3",
            "to thirty-eight point four percent",
            "pub.b.weights.pct",
            "38.4,15.4,46.2",
        ),
        (
            "13_3",
            "from thirty-four point six",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        (
            "13_3",
            "to thirty-four and a half",
            "pub.mix.weights.pct",
            "34.5,25.4,40.1",
        ),
        ("13_4", "drops by half a point", "rules.handicap", "1/200"),
        (
            "13_4",
            "thirty-seven point seven percent",
            "pub.b.fair_share.pct",
            "37.7",
        ),
        ("13_4", "a little under a cent a round", "pub.b.mean.cents", "-0.87"),
        (
            "13_5",
            "just over two and a half cents a round",
            "round.mix.mean.cents",
            "+2.54",
        ),
        (
            "13_6",
            "At thirty-four and a half percent",
            "pub.mix.weights.pct",
            "34.5,25.4,40.1",
        ),
        (
            "13_6",
            "fifty-two point one percent",
            "pub.mix.b_rounds.win.pct",
            "52.1",
        ),
        ("13_6", "fifty point eight percent", "pub.mix.win.pct", "50.8"),
        ("13_7", "about one cent a round", "handicap.mix_cost.cents", "0.97"),
        (
            "13_7",
            "just over one and a half cents a round",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        (
            "13_8",
            "thirty-eight point four percent in game B",
            "run.b.at0.pct",
            "38.4",
        ),
        ("13_8", "thirty-four and a half in the mix", "run.mix.at0.pct", "34.5"),
        ("13_9", "A loses a cent a round", "pub.a.mean.cents", "-1.00"),
        ("13_9", "B a little under a cent", "pub.b.mean.cents", "-0.87"),
        ("13_10", "nineteen ninety-nine", "source.year", "1999"),
        // 14–15: the 1,418's net flow, real seeded rounds, B's balances and the two products
        (
            "14_2",
            "one thousand four hundred eighteen players",
            "mixcrowd.size",
            "1418",
        ),
        (
            "14_2",
            "Four hundred ninety stand at corner zero",
            "mixcrowd.counts",
            "490,360,568",
        ),
        (
            "14_2",
            "three hundred sixty at corner one",
            "mixcrowd.counts",
            "490,360,568",
        ),
        (
            "14_2",
            "five hundred sixty-eight at corner two",
            "mixcrowd.counts",
            "490,360,568",
        ),
        (
            "14_2",
            "thirty-four point six percent",
            "round.mix.weights.pct",
            "34.6,25.4,40.1",
        ),
        (
            "14_3",
            "one hundred forty-seven players step forward",
            "mixcrowd.forward",
            "147,225,355",
        ),
        (
            "14_3",
            "thirty percent of the four hundred ninety",
            "round.mix.coins.pct",
            "30,62.5",
        ),
        ("14_3", "of the four hundred ninety", "mixcrowd.counts", "490,360,568"),
        (
            "14_3",
            "one hundred thirty-five step back",
            "mixcrowd.back",
            "343,135,213",
        ),
        (
            "14_3",
            "thirty-seven and a half percent",
            "round.mix.tails.pct",
            "70,37.5",
        ),
        ("14_3", "of the three hundred sixty", "mixcrowd.counts", "490,360,568"),
        (
            "14_4",
            "Two hundred twenty-five forward",
            "mixcrowd.forward",
            "147,225,355",
        ),
        ("14_4", "two hundred thirteen back", "mixcrowd.back", "343,135,213"),
        (
            "14_4",
            "Three hundred fifty-five forward",
            "mixcrowd.forward",
            "147,225,355",
        ),
        ("14_4", "three hundred forty-three back", "mixcrowd.back", "343,135,213"),
        ("14_6", "Twelve, every time", "mixcrowd.net", "12,12,12"),
        ("14_6", "Twelve more players go forward", "mixcrowd.net", "12,12,12"),
        ("14_6", "on all three edges", "rules.modulus", "3"),
        ("14_8", "Thirty-six dollars", "mixcrowd.gain", "36"),
        ("14_8", "twelve for each of the three edges", "mixcrowd.net", "12,12,12"),
        (
            "14_8",
            "Seven hundred twenty-seven players win",
            "mixcrowd.winners",
            "727",
        ),
        ("14_8", "six hundred ninety-one lose", "mixcrowd.losers", "691"),
        (
            "14_8",
            "the one thousand four hundred eighteen players",
            "mixcrowd.size",
            "1418",
        ),
        (
            "14_8",
            "just over two and a half cents a round",
            "round.mix.mean.cents",
            "+2.54",
        ),
        ("14_9", "Why twelve on every edge", "mixcrowd.net", "12,12,12"),
        (
            "14_9",
            "twelve more players arrive than leave",
            "mixcrowd.net",
            "12,12,12",
        ),
        (
            "14_9",
            "keeps its three hundred sixty",
            "mixcrowd.counts",
            "490,360,568",
        ),
        (
            "14_10",
            "all one thousand four hundred eighteen",
            "mixcrowd.size",
            "1418",
        ),
        (
            "14_10",
            "seven, four and minus seventeen",
            "realmix.round.1.net",
            "7,4,-17",
        ),
        ("14_10", "lost six dollars", "realmix.round.1.gain", "-6"),
        ("14_11", "Over a thousand real rounds", "realmix.rounds", "1000"),
        (
            "14_11",
            "each edge averages eleven point nine",
            "realmix.mean.net",
            "11.9,11.9,11.9",
        ),
        (
            "14_11",
            "thirty-five point eight dollars a round",
            "realmix.mean.gain",
            "35.8",
        ),
        ("14_12", "gains three times it each round", "rules.modulus", "3"),
        ("14_12", "that net flow is zero", "crowd.net", "0,0,0"),
        ("14_12", "settled twenty-six", "crowd.size", "26"),
        (
            "14_12",
            "one and one, three and three, nine and nine",
            "crowd.forward",
            "1,3,9",
        ),
        (
            "14_12",
            "one and one, three and three, nine and nine",
            "crowd.back",
            "9,1,3",
        ),
        ("14_13", "When the net flow is zero", "crowd.net", "0,0,0"),
        ("14_13", "ten players times a tenth", "crowd.counts", "10,4,12"),
        (
            "14_13",
            "ten players times a tenth",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        (
            "14_13",
            "four players times a quarter",
            "coins.round.tails.pct",
            "50,90,25",
        ),
        ("14_13", "one each way", "crowd.forward", "1,3,9"),
        (
            "14_13",
            "Four times three quarters",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        ("14_13", "equals twelve times a quarter", "crowd.counts", "10,4,12"),
        ("14_13", "three each way", "crowd.forward", "1,3,9"),
        (
            "14_13",
            "equals ten times nine tenths",
            "coins.round.tails.pct",
            "50,90,25",
        ),
        ("14_13", "nine each way", "crowd.forward", "1,3,9"),
        ("14_14", "Multiply the three balances", "rules.modulus", "3"),
        ("14_14", "ten, four and twelve", "crowd.counts", "10,4,12"),
        (
            "14_14",
            "a tenth times three quarters times three quarters",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        (
            "14_14",
            "nine tenths times a quarter times a quarter",
            "coins.round.tails.pct",
            "50,90,25",
        ),
        (
            "14_14",
            "nine over one hundred sixty",
            "round.products.b",
            "9/160,9/160",
        ),
        ("14_15", "winning three times in a row", "rules.modulus", "3"),
        ("14_15", "losing three times in a row", "rules.modulus", "3"),
        ("15_1", "when the net flow is zero", "crowd.net", "0,0,0"),
        ("15_2", "Put ten players at corner zero", "crowd.counts", "10,4,12"),
        (
            "15_2",
            "A tenth of them step forward",
            "coins.round.heads.pct",
            "50,10,75",
        ),
        ("15_2", "one player stepping back", "crowd.back", "9,1,3"),
        ("15_2", "where a quarter step back", "coins.round.tails.pct", "50,90,25"),
        ("15_2", "That takes four players", "crowd.counts", "10,4,12"),
        ("15_2", "three of those four step forward", "crowd.forward", "1,3,9"),
        ("15_2", "needs three stepping back", "crowd.back", "9,1,3"),
        ("15_2", "that takes twelve", "crowd.counts", "10,4,12"),
        ("15_3", "Twelve times three quarters is nine", "crowd.forward", "1,3,9"),
        ("15_3", "ten times nine tenths is nine", "crowd.back", "9,1,3"),
        ("15_3", "ten, four and twelve", "crowd.counts", "10,4,12"),
        ("15_4", "from corner zero's", "rules.modulus", "3"),
        ("15_5", "Corner zero's weight", "rules.modulus", "3"),
        ("15_6", "the net flow is zero", "crowd.net", "0,0,0"),
        ("15_7", "Put that into the third", "rules.modulus", "3"),
        ("15_8", "Corner zero's weight is positive", "rules.modulus", "3"),
        ("15_9", "B's net flow is zero", "crowd.net", "0,0,0"),
        (
            "15_9",
            "fifty-three point three percent",
            "round.b.thirds.win.pct",
            "53.3",
        ),
        (
            "15_10",
            "thirty percent times sixty-two and a half percent",
            "round.mix.coins.pct",
            "30,62.5",
        ),
        (
            "15_10",
            "seventy percent times thirty-seven and a half percent",
            "round.mix.tails.pct",
            "70,37.5",
        ),
        (
            "15_11",
            "Seventy-five over six hundred forty",
            "round.products.mix.common",
            "75/640,63/640",
        ),
        (
            "15_11",
            "sixty-three over six hundred forty",
            "round.products.mix.common",
            "75/640,63/640",
        ),
        (
            "15_12",
            "crowd of one thousand four hundred eighteen",
            "mixcrowd.size",
            "1418",
        ),
        ("15_12", "forty-nine sixty-fourths", "round.mix.sign.factor", "49/64"),
        (
            "15_12",
            "differ by three over one hundred sixty",
            "round.products.mix.diff",
            "3/160",
        ),
        ("15_12", "Twelve players of net flow", "mixcrowd.net", "12,12,12"),
        (
            "15_12",
            "four hundred ninety players at corner zero",
            "mixcrowd.counts",
            "490,360,568",
        ),
        (
            "15_12",
            "one hundred forty-seven over sixteen",
            "round.mix.sign.sides",
            "147/16",
        ),
        (
            "15_13",
            "just over two and a half cents a round",
            "round.mix.mean.cents",
            "+2.54",
        ),
        ("15_14", "shave the half point off again", "rules.handicap", "1/200"),
        ("15_14", "so A loses, a cent a round", "pub.a.mean.cents", "-1.00"),
        ("15_14", "so B loses, just under a cent", "pub.b.mean.cents", "-0.87"),
        (
            "15_14",
            "just over one and a half cents a round",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        ("15_15", "the three chances forward", "rules.modulus", "3"),
        ("15_15", "the three chances back", "rules.modulus", "3"),
        // 16–17: schedules played round by round, A's share, and what a winning average promises
        ("16_2", "Half the rounds are A", "sched.a_share", "1/2,1/2,1/2"),
        ("16_2", "half are B", "sched.a_share", "1/2,1/2,1/2"),
        ("16_3", "the same half and half", "sched.a_share", "1/2,1/2,1/2"),
        (
            "16_4",
            "about two thirds of a cent a round",
            "pub.sched.ab.cents",
            "-0.67",
        ),
        ("16_5", "cost one cent each", "pub.a.mean.cents", "-1.00"),
        (
            "16_5",
            "thirty-seven point seven percent",
            "pub.b.fair_share.pct",
            "37.7",
        ),
        (
            "16_6",
            "thirty-eight point four percent on corner zero",
            "pub.b.weights.pct",
            "38.4,15.4,46.2",
        ),
        (
            "16_6",
            "thirty point seven percent",
            "pub.sched.ab.first.b.at0.pct",
            "30.7",
        ),
        (
            "16_6",
            "settle at thirty-eight percent",
            "pub.sched.ab.cycle.b.pct",
            "38.0,46.4,15.6",
        ),
        (
            "16_7",
            "about sixty-nine percent",
            "pub.sched.ab.cycle.a.pct2",
            "23.47,7.59,68.95",
        ),
        (
            "16_7",
            "just under half of them",
            "coins.published.heads.pct",
            "49.5,9.5,74.5",
        ),
        (
            "16_7",
            "Almost thirty-eight percent",
            "pub.sched.ab.refill.pct2",
            "34.13,3.83,37.96",
        ),
        ("16_8", "twenty-nine and a half percent", "pub.mix.coins.pct", "29.5,62"),
        (
            "16_8",
            "sixty-two percent at the others",
            "pub.mix.coins.pct",
            "29.5,62",
        ),
        ("16_8", "just over one and a half cents", "pub.mix.mean.cents", "+1.57"),
        (
            "16_9",
            "about thirty-six percent",
            "pub.sched.aabb.b.at0.pct",
            "36.0,33.3",
        ),
        (
            "16_9",
            "only about thirty-three",
            "pub.sched.aabb.b.at0.pct",
            "36.0,33.3",
        ),
        (
            "16_9",
            "about one and a half cents a round",
            "pub.sched.aabb.cents",
            "+1.47",
        ),
        (
            "16_10",
            "wins one point five seven cents",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        ("16_10", "loses zero point six seven", "pub.sched.ab.cents", "-0.67"),
        ("16_10", "wins one point four seven", "pub.sched.aabb.cents", "+1.47"),
        (
            "16_11",
            "forty-nine and forty-eight percent",
            "ordinary.coins.pct",
            "49,48",
        ),
        (
            "16_11",
            "exactly three cents a round",
            "ordinary.sched.cents",
            "-3.00,-3.00,-3.00",
        ),
        ("16_12", "one round in twenty", "pub.share.points", "1/20,9/10"),
        (
            "16_12",
            "about a quarter of a cent a round",
            "pub.share.1_20.cents",
            "-0.23",
        ),
        ("16_12", "nine rounds in ten", "pub.share.points", "1/20,9/10"),
        ("16_12", "about a third of a cent", "pub.share.9_10.cents", "-0.34"),
        ("16_13", "from about seven percent", "pub.share.crossings.whole", "7,85"),
        (
            "16_13",
            "to about eighty-five percent",
            "pub.share.crossings.whole",
            "7,85",
        ),
        (
            "16_14",
            "seven point zero three",
            "pub.share.crossings.fine",
            "7.03,84.71",
        ),
        (
            "16_14",
            "eighty-four point seven one percent",
            "pub.share.crossings.fine",
            "7.03,84.71",
        ),
        (
            "16_14",
            "At exactly seven percent it still loses",
            "pub.share.whole.cents",
            "-0.0034,-0.0183",
        ),
        (
            "16_14",
            "at eighty-five percent it has already stopped winning",
            "pub.share.whole.cents",
            "-0.0034,-0.0183",
        ),
        ("17_1", "the first thousand rounds", "run.short", "1000"),
        ("17_1", "sixteen dollars ahead", "run.b.net.short", "16"),
        ("17_1", "the hundred thousandth round", "run.rounds", "100000"),
        ("17_1", "five hundred forty-eight dollars down", "run.b.net", "-548"),
        ("17_2", "start a whole crowd at zero dollars", "horizon.start", "0"),
        ("17_2", "for a hundred rounds", "horizon.rounds", "100,1000"),
        (
            "17_2",
            "a player with minus one dollar simply stands on corner two",
            "remainder.examples",
            "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0",
        ),
        ("17_3", "After a hundred rounds", "horizon.rounds", "100,1000"),
        (
            "17_3",
            "with the most on corner two",
            "pub.mix.weights.pct",
            "34.5,25.4,40.1",
        ),
        (
            "17_4",
            "fifty-one point seven percent",
            "horizon.short.mix",
            "51.7,39.6,8.7",
        ),
        (
            "17_4",
            "Behind: thirty-nine point six",
            "horizon.short.mix",
            "51.7,39.6,8.7",
        ),
        ("17_4", "eight point seven", "horizon.short.mix", "51.7,39.6,8.7"),
        (
            "17_5",
            "one dollar and twenty-nine cents up",
            "horizon.mix.mean",
            "+1.29,+15.42",
        ),
        (
            "17_5",
            "the mix's one point five seven cents",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        (
            "17_5",
            "would make one fifty-seven",
            "horizon.mix.at_rate",
            "1.57,15.70",
        ),
        (
            "17_5",
            "costs about twenty-eight cents, once",
            "horizon.mix.start_cost",
            "0.28",
        ),
        (
            "17_5",
            "almost two in five players are behind",
            "horizon.short.mix",
            "51.7,39.6,8.7",
        ),
        (
            "17_6",
            "a thousand rounds instead of a hundred",
            "horizon.rounds",
            "100,1000",
        ),
        (
            "17_7",
            "Sixty-eight point nine percent",
            "horizon.long.mix",
            "68.9,28.7,2.4",
        ),
        ("17_7", "more than two in three", "horizon.long.mix", "68.9,28.7,2.4"),
        ("17_7", "from a hundred to a thousand", "horizon.rounds", "100,1000"),
        (
            "17_8",
            "Behind: twenty-eight point seven percent",
            "horizon.long.mix",
            "68.9,28.7,2.4",
        ),
        ("17_8", "Level: two point four", "horizon.long.mix", "68.9,28.7,2.4"),
        (
            "17_8",
            "about one and a half cents a round",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        (
            "17_8",
            "fifteen dollars and forty-two cents",
            "horizon.mix.mean",
            "+1.29,+15.42",
        ),
        (
            "17_8",
            "from about nine dollars to about thirty",
            "horizon.mix.spread",
            "9.3,29.6",
        ),
        ("17_9", "Ten times the rounds", "horizon.rounds.ratio", "10"),
        (
            "17_9",
            "three point one six times the spread",
            "horizon.mix.spread.ratio",
            "3.16",
        ),
        ("17_9", "the square root of ten", "horizon.rounds.ratio", "10"),
        (
            "17_9",
            "at a hundred or a thousand rounds",
            "horizon.rounds",
            "100,1000",
        ),
        ("17_10", "the player's sixteen dollars", "run.b.net.short", "16"),
        ("17_10", "Start a crowd at seven dollars", "run.start", "7"),
        ("17_10", "game B alone for a thousand rounds", "run.short", "1000"),
        (
            "17_10",
            "Thirty-four point five percent finish ahead",
            "horizon.long.b.from_start",
            "34.5,14.6",
        ),
        (
            "17_10",
            "fourteen point six percent",
            "horizon.long.b.from_start",
            "34.5,14.6",
        ),
        ("17_10", "at least sixteen dollars up", "run.b.net.short", "16"),
        (
            "17_11",
            "Over a thousand rounds from zero dollars",
            "horizon.rounds",
            "100,1000",
        ),
        (
            "17_11",
            "sixty-one point two percent",
            "horizon.long.a.behind.pct",
            "61.2",
        ),
        (
            "17_11",
            "sixty-three point nine percent",
            "horizon.long.b.behind.pct",
            "63.9",
        ),
        (
            "17_11",
            "only twenty-eight point seven percent behind",
            "horizon.long.mix",
            "68.9,28.7,2.4",
        ),
        (
            "17_12",
            "the mix's one point five seven cents",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        (
            "17_13",
            "about one and a half cents a round",
            "pub.mix.mean.cents",
            "+1.57",
        ),
        (
            "17_13",
            "just over half after a hundred rounds",
            "horizon.short.mix",
            "51.7,39.6,8.7",
        ),
        (
            "17_13",
            "more than two in three after a thousand",
            "horizon.long.mix",
            "68.9,28.7,2.4",
        ),
        ("17_13", "that cent and a half", "pub.mix.mean.cents", "+1.57"),
        ("18_1", "loses a cent a round", "pub.a.mean.cents", "-1.00"),
        ("18_2", "corner zero", "pub.b.weights.pct", "38.4,15.4,46.2"),
        ("18_3", "off corner zero", "pub.mix.weights.pct", "34.5,25.4,40.1"),
        ("18_3", "more than half the time", "pub.mix.b_rounds.win.pct", "52.1"),
        ("18_4", "divides by three", "rules.modulus", "3"),
        (
            "18_6",
            "about twenty-eight cents a round",
            "pub.reverse.mean.cents",
            "-27.6",
        ),
        (
            "18_7",
            "about thirty-one cents a round",
            "pub.swapped.mean.cents",
            "+31.4",
        ),
        ("18_8", "gain is zero", "round.b.mean", "0"),
    ];

    /// Words that state a quantity. "One" and "two" are left out: here they
    /// count things (one player, two games, each one), not amounts.
    const NUMBER_WORDS: &[&str] = &[
        "zero",
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
        "million",
        "half",
        "halves",
        "third",
        "thirds",
        "quarter",
        "percent",
        "cent",
        "cents",
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
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
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

    /// The voice keeps its vocabulary and its scope (storyboard, "Vocabulary"
    /// and "Sources and scope"): no drift or expected value, no strategy or
    /// advice, no promise that switching always wins, and the handicap never
    /// called half a cent.
    #[test]
    fn the_voice_keeps_its_words_and_its_scope() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            for phrase in [
                "drift",
                "expected value",
                "stationary",
                "strategy",
                "half a cent",
                "invest",
                "always win",
                "guarantee",
            ] {
                assert!(!lower.contains(phrase), "{id} says {phrase:?}: {text}");
            }
        }
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// Each produced chapter is the full draft's chapter, word for word and
    /// picture for picture, so the draft never drifts from what is built.
    #[test]
    fn the_produced_chapters_are_the_drafts() {
        let draft = include_str!("../docs/storyboard-draft.md");
        let section = |doc: &'static str, key: &str| -> &'static str {
            let from = doc
                .find(&format!("\n## {key}:"))
                .unwrap_or_else(|| panic!("no section {key}"));
            let rest = &doc[from + 1..];
            let to = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 4);
            &rest[..to]
        };
        for key in keys() {
            assert_eq!(
                section(STORYBOARD, key).trim_end(),
                section(draft, key).trim_end(),
                "{key} differs from the draft"
            );
        }
    }
}
