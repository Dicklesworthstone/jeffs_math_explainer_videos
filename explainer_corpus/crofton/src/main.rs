//! Measure a Curve Without Following It: the Cauchy–Crofton formula, which
//! recovers a curve's length from how often straight lines cross it,
//! averaged over every way of turning the sheet of lines, rendered end to
//! end by franken_manim's native Rust front door and the corpus's shared
//! kit. Chapters 01 to 28 have native scenes; the full draft of the
//! film is `docs/storyboard-draft.md`.
//!
//! ```text
//! crofton facts                     # every number the video shows or says (key<TAB>value)
//! crofton script                    # the narration (generated from docs/storyboard.md)
//! crofton render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! crofton help                      # the full option list
//! ```

mod chapters;
mod coast_chart;
mod drop_samples;
mod facts;
mod line_chart;
mod model;
mod palette;
mod pin_lines;
mod score;
mod stick_band;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat (the
/// tests check that script.tsv and the chapter registry follow it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "crofton",
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

// No narration_units.json yet: nothing has been synthesized for this topic,
// so there is no synthesis input to hold to the script (root builds it when
// it qualifies the voice).
#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::cli::check_script_owners;

    use super::*;

    fn keys() -> Vec<&'static str> {
        chapters::registry().iter().map(|c| c.key).collect()
    }

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed coast, step or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("28_1", "seven", "open.crossings", "7"),
        ("28_1", "twenty two", "open.estimate", "22.0"),
        ("28_1", "twenty five point eight", "coast.length", "25.8"),
        (
            "28_3",
            "twenty five point five three",
            "ladder.4.coast_estimate",
            "25.53",
        ),
        ("28_6", "two point eight three", "square.max_width", "2.83"),
        ("28_7", "three distinct", "square.corner_contacts", "3"),
        ("28_9", "eight", "square.rim", "8"),
        ("28_9", "two point five five", "square.mean", "2.55"),
        ("28_11", "twenty seven percent", "square.four_share_percent", "27"),
        ("28_12", "seventy three percent", "square.two_share_percent", "73"),
        ("27_2", "two percent", "dividers.halving_percent", "2"),
        ("27_5", "one point two five", "richardson.west_coast_d", "1.25"),
        (
            "27_6",
            "one point one eight nine",
            "richardson.halving_factor",
            "1.189207",
        ),
        ("27_6", "nineteen percent", "richardson.halving_percent", "19"),
        ("26_1", "three point five", "scale.beach", "3.5"),
        ("26_1", "seven semicircles", "scale.ripples", "7"),
        ("26_2", "a quarter centimetre", "scale.radius", "0.25"),
        ("26_2", "seven pi over four", "scale.beach_exact", "7/4pi"),
        ("26_2", "five point five", "scale.rippled_beach", "5.5"),
        ("26_3", "fourteen", "scale.ripples_fine", "14"),
        ("26_4", "twenty-two point six", "dividers.2_0.rippled_length", "22.6"),
        ("26_6", "eight point eight five", "scale.rippled_mean_crossings", "8.85"),
        ("26_6", "twenty-seven point eight", "scale.rippled_length", "27.8"),
        ("26_7", "fourteen crossings", "scale.selected_beach_crossings", "14"),
        ("26_8", "a quarter", "scale.one_ripple_mean", "0.25"),
        ("26_8", "Seven together", "scale.ripples", "7"),
        ("26_8", "one point seven five", "scale.beach_mean", "1.75"),
        ("26_9", "twenty-seven point eight", "scale.rippled_length", "27.8"),
        ("26_9", "twenty-two point six", "dividers.2_0.rippled_length", "22.6"),
        (
            "26_10",
            "ten, a hundred, even a thousand",
            "history.vistula_multipliers",
            "10,100,1000",
        ),
        ("25_4", "eight point two two", "gap.2.mean", "8.22"),
        ("25_4", "four point one one", "gap.4.mean", "4.11"),
        ("25_4", "two point zero five", "gap.8.mean", "2.05"),
        ("25_5", "thousand retained drops", "drops.many", "1000"),
        ("25_6", "four point four", "gap.2.one_drop_spread", "4.4"),
        ("25_6", "eight point one", "gap.4.one_drop_spread", "8.1"),
        ("25_6", "fourteen point nine", "gap.8.one_drop_spread", "14.9"),
        ("25_8", "twenty-two point six", "dividers.2_0.length", "22.6"),
        ("25_10", "twenty-five point one", "dividers.1_0.length", "25.1"),
        ("25_10", "twenty-five point six", "dividers.0_5.length", "25.6"),
        ("25_11", "eleven whole steps", "dividers.1_9.steps", "11"),
        ("25_11", "twenty-one point six", "dividers.1_9.length", "21.6"),
        ("24_1", "Six equally spaced directions", "steinhaus.directions", "6"),
        ("24_1", "two point three percent low", "ladder.6.low_percent", "-2.3"),
        ("24_1", "one point two percent high", "ladder.6.high_percent", "1.2"),
        (
            "24_1",
            "twenty-five point seven three",
            "ladder.6.coast_estimate",
            "25.73",
        ),
        ("24_2", "thirty degrees", "steinhaus.angle", "30"),
        ("24_2", "six directions", "steinhaus.directions", "6"),
        ("24_5", "six line families", "steinhaus.directions", "6"),
        ("24_7", "three point eight two", "steinhaus.gap_mm", "3.82"),
        (
            "24_10",
            "two hundred fifty-seven point two seven",
            "steinhaus.coast_mean_mm",
            "257.27",
        ),
        (
            "24_10",
            "two hundred fifty-eight point one three",
            "steinhaus.coast_length_mm",
            "258.13",
        ),
        (
            "23_3",
            "twenty-five point five three",
            "ladder.4.coast_estimate",
            "25.53",
        ),
        (
            "23_3",
            "one point one percent low",
            "ladder.4.coast_error_percent",
            "-1.1",
        ),
        ("23_4", "six centimetres", "stick.length", "6"),
        ("23_6", "five point two percent low", "ladder.4.low_percent", "-5.2"),
        ("23_6", "two point six percent high", "ladder.4.high_percent", "2.6"),
        (
            "23_9",
            "twenty-one point five percent low",
            "ladder.2.low_percent",
            "-21.5",
        ),
        ("23_9", "eleven point one percent high", "ladder.2.high_percent", "11.1"),
        ("23_10", "a hundred percent low", "ladder.1.low_percent", "-100"),
        ("23_10", "fifty-seven percent high", "ladder.1.high_percent", "57"),
        ("22_1", "a thousand drops", "drops.many", "1000"),
        ("22_2", "first ten", "drops.few", "10"),
        ("22_2", "all thousand", "drops.many", "1000"),
        (
            "22_3",
            "twenty-five point nine centimetres",
            "drops.many.estimate",
            "25.9",
        ),
        ("22_4", "a thousand independent drops", "drops.many", "1000"),
        ("22_4", "point one four centimetres", "drops.many.spread", "0.14"),
        ("22_9", "About two thousand", "drops.for_tenth_about", "2000"),
        ("22_9", "nineteen sixty-five", "drops.for_tenth", "1965"),
        ("21_1", "ten drops", "drops.few", "10"),
        ("21_4", "eight point two crossings", "drops.few.mean", "8.2"),
        ("21_4", "multiply by pi centimetres per crossing", "coin.worth", "3.14"),
        (
            "21_4",
            "twenty-five point eight centimetres",
            "drops.few.estimate",
            "25.8",
        ),
        ("21_5", "ten drops", "drops.few", "10"),
        ("21_6", "a thousand retained drops", "drops.many", "1000"),
        ("21_6", "five to twelve", "drops.sample_range", "5..12"),
        ("21_6", "thirteen crossings", "drops.count_range", "5..13"),
        (
            "21_6",
            "zero point zero three six percent",
            "drops.top_count_percent",
            "0.036",
        ),
        ("21_7", "four point four centimetres", "drops.one.spread", "4.4"),
        ("21_8", "ten independent drops", "drops.few", "10"),
        ("21_8", "one point four centimetres", "drops.few.spread", "1.4"),
        ("21_9", "ten-drop experiment", "drops.few", "10"),
        ("21_9", "a thousand times", "drops.ten.repeats", "1000"),
        ("21_9", "group of ten", "drops.few", "10"),
        ("21_9", "one centimetre", "drops.error_cm", "1"),
        ("21_9", "forty-three percent", "drops.ten.far_percent", "43"),
        ("21_10", "forty-three percent", "drops.ten.far_exact_percent", "43"),
        ("20_6", "eighteen thirty-two", "cauchy.reported_year", "1832"),
        (
            "20_6",
            "sixteen point four three three",
            "crofton.mean_reach_3dp",
            "16.433",
        ),
        ("20_7", "two centimetres across", "coin.width", "2"),
        ("20_7", "six point two eight centimetres", "coin.rim", "6.28"),
        ("20_8", "Twelve point five seven", "chart.coin_total", "12.57"),
        ("20_9", "one centimetre", "coin.radius", "1"),
        ("20_9", "twice", "formula.coin_counts", "2,1,0"),
        ("20_10", "one contact", "formula.coin_counts", "2,1,0"),
        ("20_11", "two centimetres", "coin.width", "2"),
        ("20_11", "weight two", "coin.crossings", "2"),
        ("20_11", "four pi", "chart.coin_total", "12.57"),
        ("20_11", "two pi centimetres", "coin.rim", "6.28"),
        ("20_11", "six point two eight", "coin.rim", "6.28"),
        ("20_12", "two crossings", "coin.crossings", "2"),
        ("20_12", "sixty-four percent", "stick.mean_reach_percent", "64"),
        ("20_12", "eight point two two", "crofton.mean_crossings", "8.22"),
        ("20_12", "two-centimetre spacing", "sheet.gap", "2"),
        ("19_2", "zero", "chart.selected_counts", "0,1,6"),
        ("19_2", "one", "chart.selected_counts", "0,1,6"),
        ("19_2", "crosses once", "chart.selected_counts", "0,1,6"),
        ("19_3", "six crossings", "chart.grid_max", "6"),
        (
            "19_4",
            "three hundred and sixty angle columns",
            "chart.grid_columns",
            "360",
        ),
        (
            "19_4",
            "a hundred and eighty-four distance rows",
            "chart.grid_rows",
            "184",
        ),
        ("19_5", "thirty-five percent", "chart.grid_miss_percent", "35"),
        ("19_5", "seventy-one percent", "chart.grid_once_percent", "71"),
        ("19_5", "six is the largest", "chart.grid_max", "6"),
        ("19_6", "marked six contributes six", "chart.grid_max", "6"),
        ("19_8", "fifty-one point six three", "chart.grid_total_2dp", "51.63"),
        ("19_9", "fifty-one point six three", "chart.coast_total", "51.63"),
        ("19_11", "eighteen sixty-eight", "crofton.year", "1868"),
        ("01_1", "sixteen centimetres", "card.width", "16"),
        ("01_3", "a single point", "ruler.headland_touches", "1"),
        ("01_4", "two centimetres", "dividers.opening_step", "2"),
        ("01_5", "Eleven steps", "dividers.2_0.steps", "11"),
        ("02_3", "two centimetres apart", "sheet.gap", "2"),
        ("02_6", "Seven", "open.crossings", "7"),
        ("02_7", "Only two of the lines", "open.lines_crossed", "2"),
        ("02_7", "four times", "open.lower_crossings", "4"),
        ("02_7", "twice round the bay", "open.bay_crossings", "2"),
        ("02_7", "three times", "open.upper_crossings", "3"),
        ("02_7", "once near the end", "open.end_crossings", "1"),
        ("03_1", "Seven crossings", "open.crossings", "7"),
        ("03_1", "seven is a count", "open.crossings", "7"),
        ("03_2", "about two centimetres longer", "swap.extra", "2"),
        ("03_2", "twice more", "swap.big_more", "2"),
        ("03_3", "seven small half circles", "scale.ripples", "7"),
        ("03_4", "two centimetres apart", "sheet.gap", "2"),
        ("03_4", "worth two centimetres", "sheet.gap", "2"),
        ("03_5", "Seven times", "open.crossings", "7"),
        ("03_5", "times two", "sheet.gap", "2"),
        ("03_5", "fourteen centimetres", "open.gap_guess", "14"),
        ("03_5", "fourteen point seven apart", "coast.ends_apart", "14.7"),
        ("03_7", "four straight runs", "coast.straight_runs", "4"),
        ("03_7", "seven pieces of circles", "coast.arcs", "7"),
        ("04_1", "stay at seven", "open.crossings", "7"),
        (
            "04_2",
            "from seven to nine, dips to eight for a moment, and settles back at seven",
            "slide.sequence",
            "7,9,8,7",
        ),
        ("04_4", "Now it's eight", "turn.45.crossings", "8"),
        ("04_6", "two centimetres", "sheet.gap", "2"),
        ("04_7", "Eight again", "turn.45.crossings", "8"),
        ("04_9", "the count is eight again", "turn.45.crossings", "8"),
        ("04_10", "Our first count, seven", "open.crossings", "7"),
        ("05_1", "two centimetres across", "coin.width", "2"),
        ("05_2", "crosses this line twice", "coin.crossings", "2"),
        ("05_3", "Always two", "coin.crossings", "2"),
        (
            "05_3",
            "one line runs through the coin, and only one",
            "coin.lines_through",
            "1",
        ),
        ("05_3", "crosses that line twice", "coin.crossings", "2"),
        ("05_4", "between two lines", "coin.touching_lines", "2"),
        ("05_4", "a single point", "coin.touch_points", "1"),
        ("05_4", "still makes two", "coin.crossings", "2"),
        ("05_5", "six point two eight centimetres", "coin.rim", "6.28"),
        ("05_6", "Two crossings", "coin.crossings", "2"),
        ("05_6", "six point two eight centimetres", "coin.rim", "6.28"),
        ("05_6", "worth pi centimetres", "coin.worth", "3.14"),
        ("05_6", "three point one four", "coin.worth", "3.14"),
        ("06_1", "worth pi centimetres", "coin.worth", "3.14"),
        ("06_2", "seven crossings", "open.crossings", "7"),
        ("06_3", "Seven times pi", "open.crossings", "7"),
        ("06_3", "twenty-two point zero centimetres", "open.estimate", "22.0"),
        ("06_4", "twenty-five point eight centimetres", "coast.length", "25.8"),
        ("06_5", "three point eight centimetres short", "open.short_by", "3.8"),
        ("06_5", "about fifteen percent", "open.error_percent", "-15"),
        ("06_6", "But seven", "open.crossings", "7"),
        ("06_6", "the count is nine", "bet.nine_crossings", "9"),
        ("06_6", "Nine times pi", "bet.nine_crossings", "9"),
        ("06_6", "twenty-eight point three", "bet.nine_estimate", "28.3"),
        ("06_6", "two point five centimetres too long", "bet.nine_long_by", "2.5"),
        ("07_1", "six centimetres long", "stick.length", "6"),
        ("07_2", "crosses three lines", "stick.home_across", "3"),
        ("07_2", "pi centimetres a crossing", "coin.worth", "3.14"),
        ("07_2", "nine point four", "stick.across_estimate", "9.4"),
        ("07_4", "Zero", "stick.home_along", "0"),
        ("07_4", "nine point four", "stick.across_estimate", "9.4"),
        ("07_4", "or zero", "stick.along_estimate", "0"),
        ("07_7", "the full six centimetres", "stick.shadow_across", "6"),
        ("07_7", "the count is three", "stick.home_across", "3"),
        ("08_1", "thirty degrees", "stick.tilt", "30"),
        ("08_1", "shrinks to three centimetres", "stick.reach30", "3"),
        ("08_2", "crosses two lines", "stick.tilted_first", "2"),
        ("08_2", "stay at two", "stick.tilted_first", "2"),
        (
            "08_3",
            "from two to one, and back to two",
            "stick.tilted_counts",
            "2,1,2",
        ),
        ("08_3", "half of them at one crossing", "stick.tilted_one_share", "0.50"),
        ("08_3", "and half at two", "stick.tilted_two_share", "0.50"),
        ("08_3", "one and a half lines", "stick.at30_mean", "1.5"),
        ("08_4", "Why one and a half", "stick.at30_mean", "1.5"),
        ("08_4", "one whole gap, two centimetres", "sheet.gap", "2"),
        ("08_4", "the one centimetre left over", "stick.tilted_leftover", "1"),
        ("08_5", "always held one line", "stick.gap_piece_counts", "1"),
        ("08_5", "half the time", "stick.leftover_share", "0.50"),
        ("08_5", "One, plus a half", "stick.at30_mean", "1.5"),
        ("08_6", "And one and a half", "stick.at30_mean", "1.5"),
        ("08_6", "the shadow, three centimetres", "stick.reach30", "3"),
        ("08_6", "divided by the gap, two", "sheet.gap", "2"),
        ("08_7", "four and a half centimetres", "stick.test_shadow", "4.5"),
        ("08_8", "Two and a quarter", "stick.test_mean", "2.25"),
        (
            "08_8",
            "three lines for a quarter of the slide",
            "stick.test_three_share",
            "0.25",
        ),
        ("08_8", "and two for the rest", "stick.test_first", "2"),
        ("08_8", "four and a half divided by two", "stick.test_shadow", "4.5"),
        ("08_9", "the shadow is six centimetres", "stick.shadow_across", "6"),
        ("08_9", "three whole gaps", "stick.across_rule", "3"),
        ("08_9", "six over two is three", "stick.across_rule", "3"),
        ("08_9", "zero over two is zero", "stick.along_rule", "0"),
        ("09_3", "the full six centimetres", "stick.shadow_across", "6"),
        ("09_5", "three point eight two centimetres", "stick.mean_reach", "3.82"),
        ("09_5", "about sixty-four percent", "stick.mean_reach_percent", "64"),
        ("09_6", "one point nine one crossings", "stick.turned_mean", "1.91"),
        ("09_7", "pi centimetres a crossing", "coin.worth", "3.14"),
        ("09_7", "six point zero centimetres", "stick.turned_estimate", "6.0"),
        ("09_7", "nine point four", "stick.across_estimate", "9.4"),
        ("09_7", "or zero", "stick.along_estimate", "0"),
        ("09_8", "sixty-four percent", "stick.mean_reach_percent", "64"),
        ("10_2", "here that's seven", "open.crossings", "7"),
        ("10_4", "casts no shadow", "beach.shadow_0", "0"),
        ("10_5", "crosses it twice", "open.bay_crossings", "2"),
        ("10_6", "Our first count was seven", "open.crossings", "7"),
        ("10_6", "nine and eight as well", "slide.sequence", "7,9,8,7"),
        ("10_6", "above seven, or below", "open.crossings", "7"),
        ("10_8", "fifteen point two six centimetres", "turn.0.reach", "15.26"),
        ("10_8", "seven point six three crossings", "turn.0.slide_mean", "7.63"),
        ("10_9", "So seven was a low reading", "open.crossings", "7"),
        ("11_1", "eight point seven four crossings", "turn.45.slide_mean", "8.74"),
        ("11_2", "it cast no shadow", "beach.shadow_0", "0"),
        ("11_2", "three and a half centimetres", "beach.shadow_90", "3.5"),
        ("11_2", "eight point two three", "turn.90.slide_mean", "8.23"),
        ("11_3", "seven point nine one", "turn.135.slide_mean", "7.91"),
        ("11_4", "twenty-two point zero centimetres", "open.estimate", "22.0"),
        (
            "11_5",
            "Twenty-five point five three centimetres",
            "turn.slide_estimate",
            "25.53",
        ),
        ("11_5", "about one percent short", "turn.slide_short_percent", "1"),
        ("11_7", "twenty-five point seven four", "grid.8.estimate", "25.74"),
        ("11_7", "twenty-five point eight one", "grid.32.estimate", "25.81"),
        (
            "11_8",
            "twenty-five point eight one centimetres",
            "grid.3600.estimate",
            "25.81",
        ),
        ("16_3", "two centimetres", "lines.stick", "2"),
        ("16_3", "ten thousand", "lines.count", "10000"),
        ("17_8", "ten thousand accepted lines", "lines.count", "10000"),
        ("17_10", "Seven hundred and ninety-six", "lines.even.middle", "796"),
        ("17_10", "eight hundred and eleven", "lines.even.corner", "811"),
        ("18_1", "six centimetre stick", "stick.length", "6"),
        ("18_4", "six centimetre stick", "stick.length", "6"),
        ("18_4", "area twelve", "chart.stick_band", "12.00"),
        ("18_6", "still twelve", "chart.stick_band", "12.00"),
        ("18_8", "stays twelve", "chart.stick_band", "12.00"),
        (
            "16_6",
            "two thousand one hundred and seventy-eight",
            "lines.pins.middle",
            "2178",
        ),
        (
            "16_6",
            "One thousand two hundred and seventy-three",
            "lines.pins.corner",
            "1273",
        ),
        ("11_9", "three thousand six hundred directions", "grid.most", "3600"),
        ("12_2", "in eighteen sixty", "barbier.year", "1860"),
        ("12_3", "the bay is crossed twice", "open.bay_crossings", "2"),
        ("12_3", "you get the coast's seven", "open.crossings", "7"),
        ("12_5", "our six centimetre stick", "stick.length", "6"),
        ("12_5", "it crosses two lines", "move.count", "2"),
        ("12_5", "instead of three", "stick.home_across", "3"),
        ("12_5", "Two again", "move.sheet_count", "2"),
        ("12_8", "so it counts twice", "cross.through", "2"),
        ("12_8", "cross each half once", "cross.count", "2"),
        ("12_9", "four and a half centimetres", "pieces.three_quarters", "4.5"),
        (
            "12_10",
            "fourteen point seven three centimetres",
            "ends.apart_2dp",
            "14.73",
        ),
        (
            "12_10",
            "a fourteen and a fifteen centimetre stick",
            "ends.bracket.0",
            "14,15",
        ),
        ("12_11", "between fourteen and fifteen times", "ends.bracket.0", "14,15"),
        (
            "12_11",
            "Between fourteen point seven and fourteen point eight",
            "ends.bracket.1",
            "14.7,14.8",
        ),
        (
            "12_11",
            "between fourteen point seven three and fourteen point seven four",
            "ends.bracket.2",
            "14.73,14.74",
        ),
        (
            "12_12",
            "twenty-one point three nine centimetres",
            "chain.joints.length",
            "21.39",
        ),
        ("12_12", "crossed only five times", "chain.joints.crossings", "5"),
        (
            "12_13",
            "Twenty-three point nine one centimetres",
            "chain.cuts.length",
            "23.91",
        ),
        (
            "12_13",
            "then twenty-five point seven one",
            "chain.sticks.length",
            "25.71",
        ),
        ("12_13", "both catch all seven", "chain.cuts.crossings", "7"),
        ("12_13", "catch all seven", "chain.sticks.crossings", "7"),
        ("12_16", "at two points", "coin.crossings", "2"),
        ("12_16", "two pi centimetres", "coin.rim", "6.28"),
        ("12_17", "worth pi centimetres", "coin.worth", "3.14"),
        (
            "12_17",
            "twenty-five point eight one over pi",
            "coast.length_2dp",
            "25.81",
        ),
        ("12_17", "about eight point two two", "crofton.mean_crossings", "8.22"),
        ("13_1", "our six centimetre stick", "stick.length", "6"),
        (
            "13_1",
            "about one point nine one crossings",
            "stick.turned_mean",
            "1.91",
        ),
        (
            "13_1",
            "about three point eight two centimetres",
            "stick.mean_reach",
            "3.82",
        ),
        ("13_1", "sixty-four percent", "stick.mean_reach_percent", "64"),
        ("13_2", "and it crosses none", "stick.home_along", "0"),
        ("13_2", "Here it crosses one line", "hook.home_count", "1"),
        ("13_2", "six centimetres", "stick.length", "6"),
        (
            "13_3",
            "It stays the same, about one point nine one",
            "hook.mean",
            "1.91",
        ),
        (
            "13_4",
            "about ten point seven percent",
            "stick.shares_percent",
            "10.7,22.9,31.2,35.3",
        ),
        ("13_4", "Bent, it never does", "hook.counts", "1,2,3,4"),
        ("13_5", "counts both of them", "four.through", "2"),
        ("13_5", "still one point nine one", "four.mean", "1.91"),
        ("13_5", "add up to six centimetres", "stick.length", "6"),
        ("13_7", "crossed the lines seven times", "open.crossings", "7"),
        (
            "13_7",
            "about eight point two two times",
            "crofton.mean_crossings",
            "8.22",
        ),
        (
            "13_7",
            "twenty-five point eight one centimetres",
            "coast.length_2dp",
            "25.81",
        ),
        ("14_1", "it met the lines at two points", "coin.crossings", "2"),
        ("14_2", "two centimetres", "coin.width", "2"),
        ("14_4", "Mark three points", "reuleaux.arcs", "3"),
        ("14_4", "each two centimetres", "reuleaux.width", "2"),
        ("14_5", "two centimetres", "reuleaux.width", "2"),
        ("14_7", "two centimetres", "reuleaux.width", "2"),
        (
            "14_8",
            "one point seven three centimetres",
            "triangle.width_flat",
            "1.73",
        ),
        ("14_8", "comes back to two", "triangle.width_upright", "2.00"),
        ("15_1", "two centimetres apart", "sheet.gap", "2"),
        ("15_1", "One line runs between them", "reuleaux.lines_between", "1"),
        ("15_1", "crosses the rim twice", "reuleaux.home_count", "2"),
        ("15_4", "it averages two crossings", "reuleaux.mean_crossings", "2"),
        ("15_5", "six point two eight", "reuleaux.rim", "6.28"),
        ("15_6", "sixty degrees", "reuleaux.arc_degrees", "60"),
        ("15_6", "radius two", "reuleaux.width", "2"),
        ("15_6", "Lift the three arcs off", "reuleaux.arcs", "3"),
        ("15_7", "six centimetres long", "triangle.perimeter", "6"),
        ("15_7", "With six corners", "reuleaux.chain_corners", "3,6,12"),
        (
            "15_7",
            "six point two one centimetres",
            "reuleaux.chain_lengths",
            "6.00,6.21,6.27",
        ),
        ("15_7", "and with twelve", "reuleaux.chain_corners", "3,6,12"),
        (
            "15_7",
            "six point two seven",
            "reuleaux.chain_lengths",
            "6.00,6.21,6.27",
        ),
        ("15_8", "in eighteen sixty", "barbier.year", "1860"),
        ("15_8", "two centimetres wide", "reuleaux.width", "2"),
        (
            "15_9",
            "twenty-five point eight one centimetres",
            "coast.length_2dp",
            "25.81",
        ),
    ];

    /// Number words that define a thing rather than count something the
    /// facts hold: the dividers have two points.
    const DEFINITIONAL: &[(&str, &str)] = &[
        ("28_2", "pi centimetres"),
        ("28_9", "pi centimetres per crossing"),
        ("28_9", "eight over pi"),
        ("28_2", "two centimetres"),
        ("28_2", "half turn"),
        ("28_3", "four fixed directions"),
        ("28_4", "two centimetres"),
        ("28_5", "two crossings"),
        ("28_5", "then four"),
        ("28_5", "then two"),
        ("28_5", "one of those"),
        ("28_6", "two centimetres"),
        ("28_6", "two times the square root of two"),
        ("28_6", "one gap"),
        ("28_6", "two gaps"),
        ("28_6", "one line or two lines"),
        ("28_6", "two crossings"),
        ("28_7", "zero"),
        ("28_7", "Two or four"),
        ("28_7", "one"),
        ("28_8", "four crossings"),
        ("28_9", "four sides of two centimetres"),
        ("28_10", "four crossings"),
        ("28_10", "two is one minus p"),
        ("28_10", "two times one minus p"),
        ("28_10", "four times p"),
        ("28_10", "two plus two p"),
        ("28_10", "four crossing"),
        ("28_10", "zero"),
        ("28_11", "eight over pi"),
        ("28_11", "Subtract two"),
        ("28_11", "divide by two"),
        ("28_11", "four crossings"),
        ("28_11", "four over pi minus one"),
        ("28_12", "two crossings"),
        ("28_12", "Four crossings means four intersections"),
        ("28_12", "two sheet lines"),
        ("28_13", "once more"),
        ("27_1", "one centimetre"),
        ("27_2", "half centimetre"),
        ("27_6", "two to the power D minus one"),
        ("27_7", "two trends"),
        ("27_7", "starting length to one"),
        ("27_7", "one centimetre"),
        ("27_8", "a quarter centimetre"),
        ("27_8", "an eighth"),
        ("27_10", "two drawings"),
        ("26_8", "single ripple"),
        ("26_1", "half a circle"),
        ("26_2", "pi times its radius"),
        ("26_2", "pi over two"),
        ("26_3", "half the radius"),
        ("26_4", "two centimetre dividers"),
        ("26_5", "two centimetre gap"),
        ("26_6", "half turn"),
        ("26_6", "by pi"),
        ("26_8", "at least one crossing"),
        ("26_9", "two centimetre divider"),
        ("25_1", "two centimetres"),
        ("25_1", "four centimetres"),
        ("25_1", "then eight"),
        ("25_2", "a half turn"),
        ("25_3", "pi centimetres"),
        ("25_3", "two pi and four pi"),
        ("25_4", "two times the length divided by pi times the gap"),
        ("25_4", "pi times the gap over two"),
        ("25_6", "Square each estimate's deviation"),
        ("25_6", "average those squares"),
        ("25_6", "the square root"),
        ("25_8", "two centimetre walk"),
        ("25_9", "one centimetre"),
        ("25_10", "one centimetre walk"),
        ("25_10", "Half a centimetre"),
        ("25_11", "one point nine centimetres"),
        ("25_11", "two centimetre walk"),
        ("24_4", "two times the length over pi"),
        ("24_4", "two L over pi d"),
        ("24_5", "twelve L over pi d"),
        ("24_7", "Twelve over pi millimetres"),
        ("24_8", "twelve over pi"),
        ("23_1", "four directions"),
        ("23_1", "a half turn"),
        ("23_6", "four equally spaced directions"),
        ("23_8", "two directions"),
        ("23_10", "one direction"),
        ("22_1", "a half turn"),
        ("22_1", "one gap"),
        ("22_3", "one observed average"),
        ("22_5", "zero"),
        ("22_5", "one drop"),
        ("22_6", "n squared"),
        ("22_6", "one over the square root of n"),
        ("22_7", "four times as many drops"),
        ("22_7", "half as large"),
        ("22_8", "a tenth of a centimetre"),
        ("22_9", "single-drop standard deviation"),
        ("21_2", "a half turn"),
        ("21_2", "one gap"),
        ("20_2", "one full line"),
        ("20_3", "one column"),
        ("20_4", "a half turn"),
        ("20_4", "one half"),
        ("20_4", "twice the length"),
        ("20_6", "two over pi"),
        ("20_6", "Divide the total by pi"),
        ("20_8", "Twice the rim"),
        ("20_10", "zero area"),
        ("20_11", "one half"),
        ("20_11", "pi radians wide"),
        ("20_13", "a half turn"),
        ("19_5", "cross once"),
        ("19_3", "most of its contribution away"),
        ("19_6", "six times its area"),
        ("19_7", "twice its length"),
        ("19_9", "Twice the coast's exact length"),
        ("19_10", "twice the length"),
        ("19_12", "a full half turn"),
        ("19_12", "zero chart area"),
        ("18_4", "over a half turn"),
        ("18_4", "twice the stick's length"),
        ("18_8", "over a complete half turn"),
        ("18_9", "After a half turn"),
        ("17_3", "distance zero"),
        ("17_4", "from zero to half a turn"),
        ("17_7", "both coordinates again"),
        ("17_9", "both sticks"),
        ("17_10", "the corner one"),
        ("17_11", "the two recipes"),
        ("01_4", "two points a fixed step apart"),
        ("01_6", "its two points"),
        ("02_4", "Here's one"),
        ("02_7", "The lower one"),
        ("02_7", "The upper one"),
        ("03_2", "one big half circle"),
        ("03_3", "at one placement"),
        ("03_4", "is one crossing worth"),
        ("03_5", "the two ends"),
        ("04_5", "two choices"),
        ("04_6", "exactly one gap"),
        ("04_7", "within one gap"),
        ("04_8", "half a turn"),
        ("04_9", "within half a turn"),
        ("04_10", "one reading from one placement"),
        ("05_5", "pi times its width"),
        ("05_5", "one full turn"),
        ("06_6", "one count, at one placement"),
        ("06_7", "A single placement"),
        ("06_8", "never one count"),
        ("06_8", "pi times that average"),
        ("07_3", "between two of them"),
        ("07_4", "every direction at once"),
        ("08_2", "one whole gap"),
        ("08_3", "every slide once"),
        ("08_5", "each one left"),
        ("08_5", "while one passed through"),
        ("08_6", "holds one line"),
        ("08_6", "holds one for its share"),
        ("09_1", "one direction at a time"),
        ("09_2", "half a turn tries every direction once"),
        ("09_3", "the two we measured"),
        ("09_4", "more than half the stick"),
        ("09_5", "the two ends of the arch"),
        ("09_5", "half its width times the full height"),
        ("09_5", "half the stick"),
        ("09_7", "A single placement"),
        ("09_8", "two over pi"),
        ("10_2", "one of these sticks"),
        ("10_3", "runs one way across the lines"),
        ("10_5", "once on the way down and once on the way back up"),
        ("10_9", "this one way"),
        ("11_1", "all run one way"),
        ("11_3", "And one more eighth"),
        ("11_3", "three eighths of a turn"),
        ("11_4", "Average those four"),
        ("11_4", "multiply by pi"),
        ("11_6", "But four directions are only four"),
        ("11_6", "half a turn"),
        ("11_7", "Our four directions are these four points"),
        ("11_7", "Eight give"),
        ("11_7", "thirty-two give"),
        ("11_8", "two decimal places"),
        ("12_1", "on one coast"),
        ("12_1", "one fixed number"),
        ("12_2", "without a single integral"),
        ("12_2", "two ideas"),
        ("12_3", "once on its way down and once on its way back up"),
        ("12_7", "Cut the stick in half"),
        ("12_7", "Each half counts its own crossings"),
        ("12_7", "One half loses a crossing"),
        ("12_8", "Lay one half across the other"),
        ("12_8", "meets both halves"),
        ("12_9", "since one can be moved onto the other"),
        ("12_9", "each half has half the whole stick's average"),
        ("12_9", "each third a third"),
        ("12_9", "each quarter a quarter"),
        ("12_9", "Three quarters together"),
        ("12_9", "have three quarters of it"),
        ("12_9", "one number, the average for one centimetre"),
        ("12_10", "the coast's two ends"),
        ("12_10", "the shorter one inside it"),
        ("12_15", "One number"),
        ("12_15", "just one curve"),
        ("12_16", "So two is two pi times the number"),
        ("12_16", "the number is one over pi"),
        ("12_17", "its length over pi crossings"),
        ("13_1", "Its length over pi"),
        ("13_1", "twelve over pi"),
        ("13_1", "why it's two over pi"),
        ("13_2", "two arms of three centimetres"),
        ("13_3", "Each arm is a three centimetre stick"),
        ("13_3", "its two arms"),
        ("13_4", "one of its arms"),
        ("13_5", "Bend it once more"),
        ("13_6", "its length over pi crossings"),
        ("13_7", "its length over pi"),
        ("13_7", "pi times the average count"),
        ("14_2", "Press two straight rulers"),
        ("14_4", "on one corner"),
        ("14_5", "One touches a corner"),
        ("15_3", "for that one slide"),
        ("15_3", "once in every gap"),
        ("15_3", "Each of those is one slide"),
        ("15_3", "probability zero"),
        ("15_5", "Two pi centimetres"),
        ("15_6", "make half of that circle"),
        ("15_6", "Its full circumference is four pi"),
        ("15_6", "So half of it is two pi again"),
        ("15_7", "And a third way"),
        ("15_8", "pi times the width"),
        ("16_2", "two pins"),
        ("16_3", "two equal vertical sticks"),
    ];

    /// The words the guard below listens for.
    const NUMBER_WORDS: &[&str] = &[
        "zero",
        "one",
        "once",
        "single",
        "two",
        "twice",
        "double",
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
        "hundred",
        "thousand",
        "million",
        "half",
        "halves",
        "third",
        "quarter",
        "dozen",
        "pi",
    ];

    /// The number words of `text` that lie outside every one of `phrases`
    /// (matched without regard to case).
    fn number_words_outside(text: &str, phrases: &[&str]) -> Vec<String> {
        let lower = text.to_ascii_lowercase();
        let mut covered = vec![false; lower.len()];
        for phrase in phrases {
            let phrase = phrase.to_ascii_lowercase();
            for (at, _) in lower.match_indices(&phrase) {
                covered[at..at + phrase.len()].fill(true);
            }
        }
        let mut out = Vec::new();
        let mut start = None;
        for (i, c) in lower.char_indices().chain(std::iter::once((lower.len(), ' '))) {
            match (c.is_alphabetic(), start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    let word = &lower[s..i];
                    if NUMBER_WORDS.contains(&word) && !covered[s..i].iter().all(|&c| c) {
                        out.push(word.to_owned());
                    }
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
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        for (id, phrase) in DEFINITIONAL {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} no longer says {phrase:?}");
        }
        // and every number word any line says lies inside a phrase above
        for (id, text) in script.lines() {
            let phrases: Vec<&str> = SPOKEN
                .iter()
                .filter(|row| row.0 == id)
                .map(|row| row.1)
                .chain(DEFINITIONAL.iter().filter(|row| row.0 == id).map(|row| row.1))
                .collect();
            let loose = number_words_outside(text, &phrases);
            assert!(
                loose.is_empty(),
                "{id} says {loose:?} outside every SPOKEN phrase: {text}"
            );
        }
    }

    /// The guard is phrase-scoped: a known phrase covers only its own words,
    /// so a second number in the same line is caught, and so is a number
    /// that merely shares a line with a known phrase.
    #[test]
    fn the_number_guard_catches_a_number_outside_its_phrase() {
        let phrases = ["two centimetres"];
        assert!(number_words_outside("Open them to two centimetres.", &phrases).is_empty());
        assert_eq!(
            number_words_outside("Open them to two centimetres, then three.", &phrases),
            ["three"]
        );
        assert_eq!(number_words_outside("Two steps of two centimetres.", &phrases), ["two"]);
        assert_eq!(number_words_outside("One crossing is worth pi.", &[]), ["one", "pi"]);
        // a number word inside a longer word is not a number
        assert!(number_words_outside("A pine stone, often tenfold.", &[]).is_empty());
    }

    /// 01_5's prediction has the answer 01_6 gives, from the facts: every
    /// divider walk the film makes comes out short of the coast. The smaller
    /// step of 01_7 needs more steps and cuts less off each bend, and 01_7
    /// says no more than that, because a smaller step need not walk further:
    /// 1.9 cm dividers walk less than 2 cm ones.
    #[test]
    fn the_dividers_answer_is_the_facts_answer() {
        let script = Script::parse(SCRIPT).unwrap();
        assert!(script.text("01_6").unwrap().starts_with("Too short"));
        let smaller = script.text("01_7").unwrap();
        assert!(smaller.contains("cuts less off its bend"), "{smaller}");
        assert!(smaller.contains("needs more steps"), "{smaller}");
        for promise in ["longer", "further", "closer"] {
            assert!(!smaller.contains(promise), "01_7 promises {promise:?}: {smaller}");
        }
        let f = facts::facts().unwrap();
        let number = |key: &str| -> f64 { f.get(key).unwrap().parse().unwrap() };
        let coast = number("coast.length");
        for (_, key) in facts::DIVIDER_STEPS {
            let walked = number(&format!("dividers.{key}.length"));
            assert!(walked < coast, "{key}: {walked} is not short of {coast}");
        }
        assert!(number("dividers.1_0.steps") > number("dividers.2_0.steps"));
        assert!(number("dividers.1_0.length") > number("dividers.2_0.length"));
        assert_eq!(number("dividers.1_9.steps"), number("dividers.2_0.steps"));
        assert!(number("dividers.1_9.length") < number("dividers.2_0.length"));
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
