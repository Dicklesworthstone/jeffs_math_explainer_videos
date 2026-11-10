//! Checks over the fifteen registered chapters, the chapter 08 and 09 models, and the source bindings for the review
//! 711 repairs and the stage10 repairs of review 721. They read source text and compute model values; none of them
//! renders a frame. Stage10 adds the ending chapters to the hook scans, binds their six authored think holds, and
//! pins the numbers the reworded ending cues say.
use crate::applications_model::{self as model, DEMO};
use crate::chapters::{circuit_scene, decay_scene, registry};
use crate::facts::facts;
use crate::slide_model as ruler;
use explainer_kit::Facts;

const SCRIPT: &str = include_str!("../script.tsv");
const RATE_LAW: &str = include_str!("chapters/rate_law.rs");
const FACTORIALS: &str = include_str!("chapters/factorials.rs");
const PRODUCT_GRID: &str = include_str!("chapters/product_grid.rs");
const TINY: &str = include_str!("chapters/tiny_multipliers.rs");
const ONLY_SOLUTION: &str = include_str!("chapters/only_solution.rs");
const RULER: &str = include_str!("chapters/calculating_ruler.rs");
const DECAY: &str = include_str!("chapters/decay_scene.rs");
const CIRCUIT: &str = include_str!("chapters/circuit_scene.rs");
const CHANGING: &str = include_str!("chapters/changing_rates.rs");
const MONEY: &str = include_str!("chapters/money_limits.rs");
const TURNING: &str = include_str!("chapters/turning_rate.rs");
const EULER: &str = include_str!("chapters/euler_identity.rs");
const BRIDGE: &str = include_str!("chapters/log_bridge.rs");

/// Chapter keys in film order, each with its number of cues.
const CHAPTERS: [(&str, usize); 15] = [
    ("01_rate_law", 5),
    ("02_factorials", 6),
    ("03_product_grid", 7),
    ("04_tiny_multipliers", 6),
    ("05_whole_sum", 8),
    ("06_only_solution", 9),
    ("07_calculating_ruler", 10),
    ("08_decay_law", 8),
    ("09_charging_gap", 8),
    ("10_changing_rates", 5),
    ("11_money_limits", 3),
    ("12_chance_and_limits", 5),
    ("13_turning_rate", 5),
    ("14_euler_identity", 6),
    ("15_logarithm_bridge", 3),
];
const SCENES: [(&str, &str); 13] = [
    ("rate_law", RATE_LAW),
    ("factorials", FACTORIALS),
    ("product_grid", PRODUCT_GRID),
    ("tiny_multipliers", TINY),
    ("only_solution", ONLY_SOLUTION),
    ("calculating_ruler", RULER),
    ("decay_scene", DECAY),
    ("circuit_scene", CIRCUIT),
    ("changing_rates", CHANGING),
    ("money_limits", MONEY),
    ("turning_rate", TURNING),
    ("euler_identity", EULER),
    ("log_bridge", BRIDGE),
];
/// How many sentences each new or reworded cue has, which is what its hooks count on: chapters 08 and 09, the two
/// earlier cues reworded in stage10, and every ending cue.
const NEW_CUE_SENTENCES: [(&str, usize); 45] = [
    ("03_3", 5),
    ("06_2", 6),
    ("08_1", 5),
    ("08_2", 6),
    ("08_3", 6),
    ("08_4", 6),
    ("08_5", 6),
    ("08_6", 5),
    ("08_7", 4),
    ("08_8", 4),
    ("09_1", 6),
    ("09_2", 6),
    ("09_3", 6),
    ("09_4", 4),
    ("09_5", 5),
    ("09_6", 4),
    ("09_7", 5),
    ("09_8", 5),
    ("10_1", 10),
    ("10_2", 12),
    ("10_3", 7),
    ("10_4", 8),
    ("10_5", 11),
    ("11_1", 7),
    ("11_2", 15),
    ("11_3", 8),
    ("12_1", 9),
    ("12_2", 13),
    ("12_3", 7),
    ("12_4", 9),
    ("12_5", 10),
    ("13_1", 18),
    ("13_2", 9),
    ("13_3", 9),
    ("13_4", 9),
    ("13_5", 8),
    ("14_1", 8),
    ("14_2", 6),
    ("14_3", 6),
    ("14_4", 10),
    ("14_5", 5),
    ("14_6", 5),
    ("15_1", 7),
    ("15_2", 7),
    ("15_3", 5),
];
/// The ending's prediction holds (review 721 M10): chapter, source, the think-time constant, its authored value, the
/// range the review proposed, the cue before and the cue after. The values still have to be set against Frederick
/// Surrey takes in audio-review; these checks bind only the source.
type EndingHold = (&'static str, &'static str, &'static str, &'static str, f64, f64, &'static str, &'static str);
const ENDING_HOLDS: [EndingHold; 6] = [
    (
        "changing_rates",
        CHANGING,
        "THINK_10_2",
        "4.0",
        3.0,
        4.0,
        "10_2",
        "10_3",
    ),
    ("money_limits", MONEY, "THINK_11_2", "4.0", 3.0, 4.0, "11_2", "11_3"),
    ("money_limits", MONEY, "THINK_12_1", "3.5", 3.0, 4.0, "12_1", "12_2"),
    ("money_limits", MONEY, "THINK_12_3", "2.5", 2.0, 3.0, "12_3", "12_4"),
    ("turning_rate", TURNING, "THINK_13_2", "3.0", 2.0, 3.0, "13_2", "13_3"),
    ("euler_identity", EULER, "THINK_14_1", "2.5", 2.0, 3.0, "14_1", "14_2"),
];
/// Numbers the reworded ending cues say, with the fact each one reads: cue, spoken words, fact key, oracle string.
const ENDING_SPOKEN: [(&str, &str, &str, &str); 10] = [
    (
        "10_2",
        "Four cells finish near one point four",
        "change.steps.4",
        "1.40625",
    ),
    (
        "10_5",
        "Four cells finish near one point four",
        "change.steps.4",
        "1.40625",
    ),
    (
        "10_5",
        "Sixteen finish near one point zero nine",
        "change.steps.16",
        "1.08705",
    ),
    (
        "11_2",
        "It is about zero point one eight two",
        "money.rate.good",
        "0.18232",
    ),
    (
        "11_2",
        "about minus zero point one zero five",
        "money.rate.bad",
        "-0.10536",
    ),
    (
        "12_2",
        "zero point nine to the fiftieth power",
        "coin.typical",
        "0.00515",
    ),
    ("12_2", "factor of about a hundred", "coin.five_heads", "97.65625"),
    ("12_2", "one hundred thirty-one and a half", "coin.average", "131.50"),
    (
        "13_1",
        "one and a half across and one up",
        "turn.example",
        "1.50000 1.00000",
    ),
    (
        "13_1",
        "minus one plus one and a half i",
        "turn.example_turned",
        "-1.00000 1.50000",
    ),
];
/// Spoken mathematics made unambiguous before narration (pronunciation inventory 728, root's clarifications and
/// stage11's wording for review 726 N2 and N3):
/// cue, the words it now says, the words it no longer says.
const SPOKEN_MATH: [(&str, &str, &str); 13] = [
    (
        "03_3",
        "the square of the sum x plus y, divided by two",
        "contributes x plus y, squared",
    ),
    (
        "06_2",
        "the difference y prime minus y, multiplied by E of minus x",
        "make y prime minus y, times",
    ),
    ("09_4", "one over e times its starting size", "one over e of its"),
    (
        "10_3",
        "E applied to the product of the rate and the width",
        "E of the rate times the width",
    ),
    ("12_5", "E evaluated at k times t", "E of k t"),
    ("13_1", "x plus i y becomes minus y plus i x", "a plus i b"),
    ("13_2", "E evaluated at i times t", "E of i t"),
    ("13_3", "the length of i times z", "length of i z"),
    ("13_4", "E evaluated at i times t", "E of i t"),
    (
        "14_5",
        "E evaluated at i times s, times E evaluated at i times t, equals E evaluated at i times the sum",
        "E of i s",
    ),
    (
        "14_6",
        "a real part called a, plus i times a real coefficient called w",
        "a plus i w",
    ),
    (
        "14_6",
        "E evaluated at a times t, times E evaluated at i times w times t",
        "E of a t",
    ),
    ("15_3", "E evaluated at two pi times i", "E of two pi"),
];
/// Each prediction hold: chapter, source, the kit's name there, the stage's name there, the cue before, the cue after.
const HOLDS: [(&str, &str, &str, &str, &str, &str); 8] = [
    ("rate_law", RATE_LAW, "kit", "stage", "01_1", "01_2"),
    ("factorials", FACTORIALS, "k", "stage", "02_1", "02_2"),
    ("product_grid", PRODUCT_GRID, "k", "s", "03_2", "03_3"),
    ("tiny_multipliers", TINY, "k", "stage", "05_1", "05_2"),
    ("only_solution", ONLY_SOLUTION, "k", "stage", "06_3", "06_4"),
    ("calculating_ruler", RULER, "k", "s", "07_5", "07_6"),
    ("decay_scene", DECAY, "k", "s", "08_4", "08_5"),
    ("circuit_scene", CIRCUIT, "k", "s", "09_6", "09_7"),
];
/// Every display string chapters 08 and 09 register, as the Decimal oracle prints it.
const DISPLAY: [(&str, &str); 31] = [
    ("decay.interval", "1.0000000000"),
    ("decay.fixed.empty_time", "1.4426950409"),
    ("decay.step.0", "100 → 50"),
    ("decay.step.1", "50 → 25"),
    ("decay.step.2", "25 → 12.5"),
    ("decay.second.step", "50 → 25"),
    ("decay.warmed.left_after_old_half_life", "0.25"),
    ("decay.second_order.left_from_1", "0.50"),
    ("decay.second_order.left_from_half", "0.67"),
    ("decay.molecules", "20"),
    ("decay.molecules.left_at_end", "1"),
    ("circuit.display.battery", "12 V"),
    ("circuit.display.ohms", "10,000 ohms"),
    ("circuit.display.ohms_doubled", "20,000 ohms"),
    ("circuit.display.farads", "100 microfarads"),
    ("circuit.display.farads_doubled", "200 microfarads"),
    ("circuit.display.gap_left", "36.8%"),
    ("circuit.display.filled", "63.2%"),
    ("circuit.display.one_tau_volts", "7.59 V"),
    ("circuit.display.tau", "1 s"),
    ("circuit.display.tau_r_doubled", "2 s"),
    ("circuit.display.tau_c_doubled", "2 s"),
    ("circuit.display.start_current", "1.20 mA"),
    ("circuit.display.start_current_r_doubled", "0.60 mA"),
    ("circuit.display.start_current_c_doubled", "1.20 mA"),
    ("circuit.display.resolution", "0.05 V"),
    ("circuit.display.settle_time", "5.48 s"),
    ("circuit.display.final_gap", "0.030 V"),
    ("circuit.display.window_low", "11.7 V"),
    ("circuit.display.window_start", "4 s"),
    ("circuit.display.window_end", "6 s"),
];
/// Sample numeric rows, as the Decimal oracle prints them.
const NUMERIC: [(&str, &str); 14] = [
    ("chem_half_1", "50.0000000000"),
    ("chem_half_8", "0.3906250000"),
    ("chem_second_order_3", "0.2500000000"),
    ("rc_tau_s", "1.0000000000"),
    ("rc_000_voltage_v", "0.0000000000"),
    ("rc_000_gap_v", "12.0000000000"),
    ("rc_000_current_a", "0.0012000000"),
    ("rc_000_charge_c", "0.0000000000"),
    ("rc_010_voltage_v", "7.5854467059"),
    ("rc_010_gap_v", "4.4145532941"),
    ("rc_010_current_a", "0.0004414553"),
    ("rc_010_charge_c", "0.0007585447"),
    ("rc_100_voltage_v", "11.9994552008"),
    ("rc_100_gap_v", "0.0005447992"),
];
/// What the voice says and what the screen shows for the same quantity: cue, spoken words, fact key, display string.
const SPOKEN: [(&str, &str, &str, &str); 10] = [
    ("08_4", "from one hundred to fifty", "decay.step.0", "100 → 50"),
    ("08_5", "It finishes at twenty-five", "decay.second.step", "50 → 25"),
    ("08_5", "twenty-five to twelve and a half", "decay.step.2", "25 → 12.5"),
    ("09_4", "twelve-volt battery", "circuit.display.battery", "12 V"),
    ("09_4", "ten thousand ohms", "circuit.display.ohms", "10,000 ohms"),
    (
        "09_4",
        "one hundred microfarads",
        "circuit.display.farads",
        "100 microfarads",
    ),
    (
        "09_4",
        "Resistance times capacitance is one second",
        "circuit.display.tau",
        "1 s",
    ),
    (
        "09_5",
        "thirty-six point eight percent",
        "circuit.display.gap_left",
        "36.8%",
    ),
    (
        "09_5",
        "sixty-three point two percent",
        "circuit.display.filled",
        "63.2%",
    ),
    (
        "09_5",
        "seven point five nine volts",
        "circuit.display.one_tau_volts",
        "7.59 V",
    ),
];

/// The chapter's code, without its test module.
fn production(source: &'static str) -> &'static str {
    source
        .split("\n#[cfg(test)]\nmod tests {")
        .next()
        .expect("split yields a first part")
}
fn assert_in_order(source: &str, needles: &[&str]) {
    let mut from = 0;
    for &needle in needles {
        let at = source[from..]
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} missing or out of order"));
        from += at + needle.len();
    }
}
fn cues() -> Vec<(&'static str, &'static str)> {
    SCRIPT
        .lines()
        .map(|line| line.split_once('\t').expect("a tab separates id and text"))
        .collect()
}
fn cue(id: &str) -> &'static str {
    cues()
        .into_iter()
        .find(|&(name, _)| name == id)
        .unwrap_or_else(|| panic!("{id} is not in the script"))
        .1
}
/// The kit's rule: a sentence ends at `.`, `?` or `!` followed by a space or the end of the line.
fn sentences(text: &str) -> usize {
    let bytes = text.as_bytes();
    (0..bytes.len())
        .filter(|&i| matches!(bytes[i], b'.' | b'?' | b'!') && (i + 1 == bytes.len() || bytes[i + 1] == b' '))
        .count()
}
/// Every `until_sentence` hook whose index is a literal, in source order.
fn sentence_hooks(source: &'static str) -> Vec<(&'static str, usize)> {
    let mut hooks = Vec::new();
    for marker in [r#"until_sentence(s, ""#, r#"until_sentence(stage, ""#] {
        for part in production(source).split(marker).skip(1) {
            let (id, rest) = part.split_once('"').expect("the cue id is closed");
            let index = rest
                .trim_start_matches(", ")
                .split(')')
                .next()
                .expect("split yields a first part");
            if let Ok(index) = index.parse::<usize>() {
                hooks.push((id, index));
            }
        }
    }
    hooks
}
/// Every `until_phrase` hook whose cue and phrase are both string literals on one line.
fn phrase_hooks(source: &'static str) -> Vec<(&'static str, &'static str)> {
    let mut hooks = Vec::new();
    for part in production(source).split("until_phrase(").skip(1) {
        let Some((_, after_open)) = part.split_once('"') else {
            continue;
        };
        let Some((id, after_id)) = after_open.split_once('"') else {
            continue;
        };
        let Some(after_comma) = after_id.strip_prefix(", \"") else {
            continue;
        };
        let Some((phrase, _)) = after_comma.split_once('"') else {
            continue;
        };
        hooks.push((id, phrase));
    }
    hooks
}
fn central(f: impl Fn(f64) -> f64, t: f64) -> f64 {
    const H: f64 = 1e-5;
    (f(t + H) - f(t - H)) / (2.0 * H)
}

#[test]
fn the_registry_lists_the_fifteen_chapters_in_film_order() {
    let keys: Vec<&str> = registry().iter().map(|chapter| chapter.key).collect();
    assert_eq!(keys, CHAPTERS.map(|(key, _)| key));
}
#[test]
fn the_script_has_every_cue_of_the_fifteen_chapters_and_the_counted_words() {
    let all = cues();
    assert_eq!(all.len(), 94);
    for (key, expected) in CHAPTERS {
        let prefix = &key[..3];
        assert_eq!(
            all.iter().filter(|(id, _)| id.starts_with(prefix)).count(),
            expected,
            "{key}"
        );
    }
    let words: usize = all.iter().map(|(_, text)| text.split_whitespace().count()).sum();
    assert_eq!(words, 7013);
}
#[test]
fn every_cue_belongs_to_one_chapter_and_the_ids_run_in_film_order() {
    let chapters = registry();
    let mut order = Vec::new();
    for (id, _) in cues() {
        let (chapter, line) = id.split_once('_').expect("ids are NN_N");
        let prefix = format!("{chapter}_");
        let owners = chapters.iter().filter(|c| c.key.starts_with(prefix.as_str())).count();
        assert_eq!(owners, 1, "{id}");
        order.push((
            chapter.parse::<u32>().expect("chapter number"),
            line.parse::<u32>().expect("line number"),
        ));
    }
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
}
#[test]
fn the_09_7_sentence_says_rescale() {
    assert!(cue("09_7").contains("Rescale the time axis of the slower trace by the right factor"));
    assert!(!cue("09_7").contains("Stretch"));
}
#[test]
fn the_08_5_sentence_keeps_the_half_life_to_constant_k() {
    assert!(
        cue("08_5").contains("That repeating interval is the half-life of this model, as long as k stays constant.")
    );
    assert!(!cue("08_5").contains("constant-rate"));
}
#[test]
fn the_new_cues_keep_the_sentences_their_hooks_count_on() {
    for (id, expected) in NEW_CUE_SENTENCES {
        assert_eq!(sentences(cue(id)), expected, "{id}");
    }
}
#[test]
fn every_literal_sentence_hook_names_a_sentence_its_cue_has() {
    for (name, source) in SCENES {
        let hooks = sentence_hooks(source);
        assert!(!hooks.is_empty(), "{name}");
        for (id, index) in hooks {
            let count = sentences(cue(id));
            assert!(index < count, "{name}: {id} has {count} sentences, so no index {index}");
        }
    }
}
#[test]
fn sentence_hooks_of_one_cue_only_move_forward() {
    for (name, source) in SCENES {
        for pair in sentence_hooks(source).windows(2) {
            if pair[0].0 == pair[1].0 {
                assert!(pair[0].1 < pair[1].1, "{name}: {pair:?}");
            }
        }
    }
}
#[test]
fn every_literal_phrase_hook_is_said_exactly_once_in_its_cue() {
    for (name, source) in SCENES {
        for (id, phrase) in phrase_hooks(source) {
            assert_eq!(cue(id).matches(phrase).count(), 1, "{name}: {id}: {phrase}");
        }
    }
    assert_eq!(phrase_hooks(DECAY).len(), 4);
    assert_eq!(phrase_hooks(RULER).len(), 2);
    // Review 721 L4: the larger cell counts of 14_2 wait for their words.
    assert_eq!(
        phrase_hooks(EULER),
        [("14_2", "sixty-four"), ("14_2", "one thousand twenty-four")]
    );
}
#[test]
fn every_ending_think_hold_stays_bound_to_its_source() {
    for (name, source, think, value, low, high, before, after) in ENDING_HOLDS {
        let text = production(source);
        let declared = format!("const {think}: f64 = {value};");
        assert_eq!(text.matches(declared.as_str()).count(), 1, "{name}: {declared}");
        let seconds: f64 = value.parse().expect("a literal number of seconds");
        assert!((low..=high).contains(&seconds), "{name}: {think}");
        let said = format!(r#"k.say(stage, "{before}")?;"#);
        let breath = format!("k.hold(stage)?;\n    k.mark(stage, \"predict\");\n    k.music_breath(stage, {think})?;");
        let next = format!(r#"k.say(stage, "{after}")?;"#);
        assert_in_order(text, &[said.as_str(), breath.as_str(), next.as_str()]);
        let at = text.find(breath.as_str()).expect("the hold exists");
        let resumes = text[at..].find(next.as_str()).expect("the next cue is said") + at;
        assert!(
            !text[at + breath.len()..resumes].contains(".say("),
            "{name}: no line between the hold and {after}"
        );
    }
    for (name, source) in [
        ("changing_rates", CHANGING),
        ("money_limits", MONEY),
        ("turning_rate", TURNING),
        ("euler_identity", EULER),
        ("log_bridge", BRIDGE),
    ] {
        let rows = ENDING_HOLDS.iter().filter(|row| row.0 == name).count();
        assert_eq!(production(source).matches("music_breath(").count(), rows, "{name}");
    }
}
#[test]
fn the_spoken_ending_numbers_are_the_oracle_strings() {
    let f = facts().expect("facts register and pass their self-check");
    for (id, words, key, value) in ENDING_SPOKEN {
        assert_eq!(cue(id).matches(words).count(), 1, "{id}: {words}");
        assert_eq!(f.get(key), Some(value), "{key}");
    }
}
#[test]
fn the_spoken_mathematics_says_what_each_operation_applies_to() {
    for (id, now, before) in SPOKEN_MATH {
        assert_eq!(cue(id).matches(now).count(), 1, "{id}: {now}");
        assert!(!cue(id).contains(before), "{id}: {before}");
    }
}
#[test]
fn the_rescale_sentence_drives_the_curves_together() {
    assert_in_order(
        production(CIRCUIT),
        &[
            r#"k.until_sentence(s, "09_7", 2)?;"#,
            "morph(s, &[(slower_still, squeezed)], 1.6)?;",
            r#"k.until_sentence(s, "09_7", 3)?;"#,
        ],
    );
}
#[test]
fn every_prediction_hold_stays_bound_to_its_source() {
    for (name, source, k, s, before, after) in HOLDS {
        let text = production(source);
        assert_eq!(text.matches("music_breath(").count(), 1, "{name}");
        let next = format!(r#"{k}.say({s}, "{after}")?;"#);
        let steps = [
            format!(r#"{k}.say({s}, "{before}")?;"#),
            format!("{k}.hold({s})?;"),
            format!(r#"{k}.mark({s}, "predict");"#),
            format!("{k}.music_breath({s}, 4.0)?;"),
            next.clone(),
        ];
        assert_in_order(text, &steps.iter().map(String::as_str).collect::<Vec<_>>());
        let breath = text.find("music_breath(").expect("the hold exists");
        let resumes = text.find(next.as_str()).expect("the next cue is said");
        assert!(
            !text[breath..resumes].contains(".say("),
            "{name}: no line between the hold and {after}"
        );
    }
}
#[test]
fn the_new_scenes_are_named_for_their_chapters() {
    assert!(production(DECAY).contains(r#""08_decay_law""#));
    assert!(production(CIRCUIT).contains(r#""09_charging_gap""#));
}
#[test]
fn the_two_new_chapters_register_454_keys_between_them() {
    let mut f = Facts::new();
    decay_scene::append_facts(&mut f).expect("decay rows");
    assert_eq!(f.len(), 29);
    circuit_scene::append_facts(&mut f).expect("circuit rows");
    assert_eq!(f.len(), 454);
}
#[test]
fn registering_a_new_chapter_twice_is_refused() {
    let mut f = Facts::new();
    decay_scene::append_facts(&mut f).expect("first registration");
    assert!(decay_scene::append_facts(&mut f).is_err());
    let mut f = Facts::new();
    circuit_scene::append_facts(&mut f).expect("first registration");
    assert!(circuit_scene::append_facts(&mut f).is_err());
}
#[test]
fn the_film_facts_hold_the_oracle_families_and_exact_strings() {
    let f = facts().expect("facts register and pass their self-check");
    // Stage10 adds seven ending keys: change.midpoint.4, coin.swap, coin.five_heads, coin.alternating_end,
    // turn.example, turn.example_turned and bridge.unit_ratio.
    assert_eq!(f.len(), 1104 + 454 + 206 + 7);
    for n in 0..9 {
        for family in ["chem_half", "chem_second_order"] {
            assert!(f.get(&format!("{family}_{n}")).is_some(), "{family}_{n}");
        }
    }
    for n in 0..=100 {
        for label in ["voltage_v", "gap_v", "current_a", "charge_c"] {
            assert!(f.get(&format!("rc_{n:03}_{label}")).is_some(), "rc_{n:03}_{label}");
        }
    }
    for (key, expected) in DISPLAY.into_iter().chain(NUMERIC) {
        assert_eq!(f.get(key), Some(expected), "{key}");
    }
}
#[test]
fn the_spoken_numbers_are_the_displayed_numbers() {
    let f = facts().expect("facts register and pass their self-check");
    for (id, words, key, shown) in SPOKEN {
        assert!(cue(id).contains(words), "{id}: {words}");
        assert_eq!(f.get(key), Some(shown), "{key}");
    }
}
#[test]
fn both_distance_rows_name_the_cell_count_of_the_bound() {
    let text = production(TINY);
    assert_eq!(text.matches(r"\text{{distance at {last} cells}}").count(), 2);
    assert!(!text.contains(r"\text{{distance}}"));
}
#[test]
fn the_distance_for_three_starts_at_one_and_then_joins_the_distance_for_two() {
    assert_in_order(
        production(RULER),
        &[
            r#"k.until_sentence(s, "07_3", 4)?;"#,
            "v(m::position(1.0), SECOND_BAR_Y)",
            "v(m::position(b), SECOND_BAR_Y)",
            "show(s, &[d1])?;",
            r#"until_phrase(&k, s, "07_3", "the distance for three")?;"#,
            "show(s, &[d2])?;",
            r#"until_phrase(&k, s, "07_3", "is the distance for six")?;"#,
            "slide(s, d2, v(m::distance(a), BAR_Y - SECOND_BAR_Y), 1.0)?",
            r#"k.say(s, "07_4")?;"#,
        ],
    );
    let (a, b) = ruler::EXAMPLES[0];
    assert!((ruler::position(1.0) + ruler::distance(b) - ruler::position(b)).abs() < 1e-12);
    assert!((ruler::position(a) + ruler::distance(b) - ruler::position(a * b)).abs() < 1e-12);
}
#[test]
fn the_voice_names_each_mark_while_it_is_marked() {
    let text = production(RULER);
    assert_in_order(
        text,
        &[
            r#"k.until_sentence(s, "07_2", 2)?;"#,
            "label_place(m::LABELS[0], LOWER_Y)",
            "show(s, &[first_mark])?;",
            r#"k.until_sentence(s, "07_2", 3)?;"#,
            "hide(s, &[first_mark])?;",
            r#"k.until_sentence(s, "07_2", 4)?;"#,
        ],
    );
    assert_in_order(
        text,
        &[
            r#"k.say(s, "07_4")?;"#,
            r#"k.until_sentence(s, "07_4", 1)?;"#,
            "show(s, &index_names)?;",
            r#"k.until_sentence(s, "07_4", 2)?;"#,
            "hide(s, &index_names)?;",
            r#"Some("left index")"#,
        ],
    );
    assert_in_order(
        text,
        &[
            r#"k.until_sentence(s, "07_7", 3)?;"#,
            "let marked = pointer(&k, s, value)?;",
            "show(s, &[hair, read, marked[0], marked[1]])?;",
            "joined.extend(marked);",
            r#"k.say(s, "07_8")?;"#,
            "hide(s, &joined)?;",
        ],
    );
    assert_eq!(ruler::LABELS[0], 1);
    assert_eq!(ruler::LABELS[ruler::LABELS.len() - 1], 10);
}
#[test]
fn the_gap_voltage_and_current_satisfy_the_circuit_equations() {
    let tau = DEMO.state(0.0).time_constant_s;
    for step in 1..=100 {
        let t = f64::from(step) / 10.0;
        let now = DEMO.state(t);
        let gap_rate = central(|x| DEMO.state(x).gap_v, t);
        let voltage_rate = central(|x| DEMO.state(x).voltage_v, t);
        assert!((gap_rate + now.gap_v / tau).abs() < 1e-8, "gap at {t}");
        assert!(
            (voltage_rate - (DEMO.battery_v - now.voltage_v) / tau).abs() < 1e-8,
            "voltage at {t}"
        );
        assert!(
            (now.current_a - DEMO.capacitance_f * voltage_rate).abs() < 1e-12,
            "current at {t}"
        );
    }
}
#[test]
fn the_second_order_curve_solves_its_law_and_its_reciprocal_grows_linearly() {
    for initial in [0.5, 1.0, 2.0] {
        for k in [0.5, 1.0, 3.0] {
            let amount = |x: f64| model::second_order(initial, k, x);
            for step in 1..=40 {
                let t = f64::from(step) / 10.0;
                assert!(
                    (central(amount, t) + k * amount(t).powi(2)).abs() < 1e-7,
                    "{initial} {k} {t}"
                );
                assert!((central(|x| 1.0 / amount(x), t) - k).abs() < 1e-8, "{initial} {k} {t}");
            }
        }
    }
}
#[test]
fn first_order_fractions_multiply_over_successive_intervals() {
    for k in [0.1, 1.0, 3.0] {
        for first in [0.0, 0.7, 2.5] {
            for second in [0.0, 1.1, 4.0] {
                let joined = model::remaining(k, first + second);
                assert!((joined - model::remaining(k, first) * model::remaining(k, second)).abs() < 1e-14);
            }
        }
    }
}
#[test]
fn one_half_life_halves_the_amount_from_any_starting_time() {
    for k in [0.1, 1.0, 3.0] {
        let h = model::half_life(k);
        for t in [0.0, 0.7, 2.5, 9.0] {
            assert!((model::remaining(k, t + h) / model::remaining(k, t) - 0.5).abs() < 1e-13);
        }
    }
}
#[test]
fn a_changed_rate_constant_changes_the_half_life() {
    assert!((model::half_life(2.0) - model::half_life(1.0) / 2.0).abs() < 1e-14);
    assert!((model::remaining(2.0, model::half_life(1.0)) - 0.25).abs() < 1e-14);
}
