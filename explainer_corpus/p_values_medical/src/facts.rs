//! Every number "Why P-Values Confuse Even Doctors" shows or says, computed
//! here and checked two ways: `pin` against a literal from the fact sheet
//! (a source's printed number, or the research brief's checked arithmetic),
//! and `./x p_values_medical facts-check` against `oracle.py` (same keys).
//! The clinics are counted person by person from `model.rs`; the study's
//! p-value is a count over all 2^20 assignments of its null world.

use explainer_kit::Facts;

use crate::model::{
    Clinic, IconArray, Q, Tail, bayes, clinic_a, clinic_b, disease, negative, no_disease, positive, positives_block_of,
    share, study, tail_count, tail_share,
};

/// Dates and printed numbers from the sources (fact sheet rows M-C; the
/// oracle prints the same literals).
pub mod cited {
    /// Windish, Huot and Green, "Medicine Residents' Understanding of the
    /// Biostatistics and Results in the Medical Literature", JAMA
    /// 2007;298(9):1010–1022, published 5 September 2007. Abstract: "The
    /// survey was completed by 277 of 367 residents (75.5%) in 11 residency
    /// programs." Methods (JAMA's indexed publisher text, read by the
    /// orchestrator on 2026-10-17; a direct fetch timed out): data collected
    /// February to July 2006, in 11 Connecticut programs. So the video says
    /// the survey was *published* in 2007, never that it ran then.
    pub const PUBLISHED_YEAR: u32 = 2007;
    pub const PUBLISHED: &str = "2007-09-05";
    pub const COLLECTED_YEAR: u32 = 2006;
    pub const COLLECTED_MONTHS: &str = "February to July";
    pub const RESPONDENTS: u64 = 277;
    pub const RESIDENTS: u64 = 367;
    pub const RESPONSE_PERCENT: &str = "75.5";
    pub const PROGRAMS: u32 = 11;
    /// Gao, "P-values – a chronic conundrum", BMC Medical Research
    /// Methodology 2020;20:167, reporting the JAMA survey: "88% of the
    /// residents expressed fair to complete confidence in understanding
    /// p-values" and "58.8% of the residents selected choice c which was
    /// designated by the authors as the correct answer".
    pub const CONFIDENT_PERCENT: u32 = 88;
    pub const KEYED_CHOICE: &str = "c";
    /// Choice c verbatim as Gao reprints it, one of "four potential answers
    /// for a correct interpretation of p > 0.05" (the JAMA question's own
    /// clinical stem was not retrieved, so the video never shows a stem).
    pub const KEYED_TEXT: &str =
        "The probability is greater than 1 in 20 that a difference this large could occur by chance alone.";
    pub const KEYED_FOR: &str = "p > 0.05";
    /// 58.8, in tenths of a percent.
    pub const KEYED_PERMILLE: i128 = 588;
    pub const GAO_YEAR: u32 = 2020;
}

/// The icon array's row length (a hundred people a row).
pub const GRID_COLS: usize = 100;

/// The pinned literals' source: the research brief's checked arithmetic.
const BRIEF: &str = "docs/video-program/research-20261016/evidence/pvalues-medical-opening.md";

/// The width both chapters draw their blocks of positives at (chapter 01's
/// gcd block of clinic A is 45 wide too; `model` tests that they agree).
pub const BLOCK_COLS: u64 = 45;

/// Chapter 03's re-tosses: the seed (fixed before any toss was looked at:
/// the year the survey's data were collected) and how many tosses it shows.
pub const RETOSS_SEED: u64 = 2006;
pub const RETOSSES: usize = 4;

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    survey(&mut f)?;
    let pins_a = [
        ("positives", "585"),
        ("ppv_percent_2dp", "15.38"),
        ("reverse_percent_2dp", "84.62"),
    ];
    clinic(&mut f, "M-F1", clinic_a(), &pins_a)?;
    clinic(
        &mut f,
        "M-F2",
        clinic_b(),
        &[("positives", "1350"), ("ppv_percent_2dp", "66.67")],
    )?;
    compare(&mut f)?;
    trial(&mut f)?;
    Ok(f)
}

fn survey(f: &mut Facts) -> Result<(), String> {
    use cited::*;
    f.add("M-C1.survey.published_year", PUBLISHED_YEAR)?;
    f.add("M-C1.survey.published", PUBLISHED)?;
    f.add("M-C1.survey.collected_year", COLLECTED_YEAR)?;
    f.add("M-C1.survey.collected_months", COLLECTED_MONTHS)?;
    f.add("M-C1.survey.respondents", RESPONDENTS)?;
    f.add("M-C1.survey.residents", RESIDENTS)?;
    f.pin(
        "M-C1.survey.response_percent",
        Q::ratio(RESPONDENTS, RESIDENTS).percent(1),
        RESPONSE_PERCENT,
        "Windish et al. 2007, abstract",
    )?;
    f.add("M-C1.survey.programs", PROGRAMS)?;
    f.add("M-C1.survey.confident_percent", CONFIDENT_PERCENT)?;
    f.add("M-C1.survey.keyed_choice", KEYED_CHOICE)?;
    let keyed = Q::new(KEYED_PERMILLE, 10);
    f.add("M-C1.survey.keyed_percent", keyed.decimal(1))?;
    f.add("M-C1.survey.keyed_percent_spoken", keyed.decimal(0))?;
    f.add("M-C2.gao.year", GAO_YEAR)?;
    Ok(())
}

/// A clinic's counts, the shares counted over its icon array, its grid and
/// its block of positives in rows of [`BLOCK_COLS`]; `pins` are the keys
/// (after the prefix) checked against the research brief's literals.
fn clinic(f: &mut Facts, prefix: &str, clinic: Clinic, pins: &[(&str, &str)]) -> Result<(), String> {
    let c = clinic.counts()?;
    let array = IconArray::build(&c, GRID_COLS)?;
    let people = array.people();
    let ppv = share(people, disease, positive)?;
    // the formula the counting earns must say what the counting said
    let formula = bayes(clinic.prevalence, clinic.sensitivity, clinic.false_alarm);
    if formula != ppv {
        return Err(format!("{prefix}: counting gives {ppv}, Bayes' rule {formula}"));
    }
    let reverse = share(people, no_disease, positive)?;
    let tempting = share(people, negative, no_disease)?;
    let block = positives_block_of(&c, BLOCK_COLS)?;
    let values: Vec<(&str, String)> = vec![
        ("prevalence.exact", clinic.prevalence.to_string()),
        ("with", c.with.to_string()),
        ("without", c.without.to_string()),
        ("true_pos", c.true_pos.to_string()),
        ("false_pos", c.false_pos.to_string()),
        ("positives", c.positives().to_string()),
        ("ppv.exact", ppv.to_string()),
        ("ppv_percent_2dp", ppv.percent(2)),
        ("people", c.people.to_string()),
        ("prevalence.one_in", (Q::ONE / clinic.prevalence).to_string()),
        ("sensitivity.exact", clinic.sensitivity.to_string()),
        ("sensitivity_percent", clinic.sensitivity.percent(0)),
        ("false_alarm.exact", clinic.false_alarm.to_string()),
        ("false_alarm_percent", clinic.false_alarm.percent(0)),
        ("false_neg", c.false_neg.to_string()),
        ("true_neg", c.true_neg.to_string()),
        ("ppv_percent", ppv.percent(0)),
        ("reverse.exact", reverse.to_string()),
        ("reverse_percent_2dp", reverse.percent(2)),
        ("tempting_percent", tempting.percent(0)),
        ("grid.cols", array.cols.to_string()),
        ("grid.rows", array.rows.to_string()),
        ("grid.disease_rows", array.disease_rows.to_string()),
        ("grid.false_pos_per_row", array.false_pos_per_row.to_string()),
        ("block.cols", block.cols.to_string()),
        ("block.rows", block.rows.to_string()),
        ("block.disease_rows", block.disease_rows.to_string()),
    ];
    for (k, value) in values {
        let key = format!("{prefix}.{k}");
        match pins.iter().find(|(p, _)| *p == k) {
            Some((_, want)) => f.pin(&key, value, want, BRIEF)?,
            None => f.add(&key, value)?,
        };
    }
    if let Some((k, _)) = pins.iter().find(|(k, _)| f.get(&format!("{prefix}.{k}")).is_none()) {
        return Err(format!("{prefix}: a pin names {k}, which the clinic does not register"));
    }
    Ok(())
}

/// What changes from clinic A to clinic B (chapter 02): ten times the
/// prevalence, ten times the people caught, and the false-alarm rate not at
/// all (each clinic registers its own).
fn compare(f: &mut Facts) -> Result<(), String> {
    let (a, b) = (clinic_a(), clinic_b());
    let (ca, cb) = (a.counts()?, b.counts()?);
    f.add("M-F2.prevalence_vs_a", b.prevalence / a.prevalence)?;
    f.add("M-F2.caught_vs_a", Q::ratio(cb.true_pos, ca.true_pos))?;
    if a.sensitivity != b.sensitivity || a.false_alarm != b.false_alarm {
        return Err("chapter 02 compares the same test: its two rates must be the same in both clinics".into());
    }
    Ok(())
}

fn trial(f: &mut Facts) -> Result<(), String> {
    let s = study();
    let world = s.null_world();
    let k = s.observed();
    let total: u64 = world.iter().sum();
    f.add("M-T1.pairs", s.pairs)?;
    f.add("M-T1.patients", 2 * s.pairs)?;
    f.add("M-T1.observed", k)?;
    f.add("M-T1.assignments", total)?;
    let counts: Vec<String> = world.iter().map(u64::to_string).collect();
    f.add("M-T1.null.counts", counts.join(","))?;
    // the histogram's tallest bar and where it stands (chapter 03)
    let (mode, max) = world
        .iter()
        .enumerate()
        .max_by_key(|&(_, &c)| c)
        .map(|(k, &c)| (k, c))
        .ok_or("an empty null world")?;
    f.add("M-T1.null.max", max)?;
    f.add("M-T1.null.mode", mode)?;
    // chapter 03's re-tosses of the coins: whole assignments from the seeded
    // stream, and the count each gives (M-T3)
    let tosses = s.retosses(RETOSS_SEED, RETOSSES);
    let toss_counts: Vec<u32> = tosses.iter().map(|&a| s.favouring(a)).collect();
    let list = |xs: &[u32]| xs.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
    f.add("M-T3.retoss.seed", RETOSS_SEED)?;
    f.add("M-T3.retoss.assignments", list(&tosses))?;
    f.add("M-T3.retoss.counts", list(&toss_counts))?;
    // the planted wrong statistic: how many A patients the coins treated
    // (not the count of pairs favouring the treatment)
    f.add("M-X3.a_treated", s.treated_a.count_ones())?;
    f.pin(
        "M-T1.tail.at_least",
        tail_count(&world, k, Tail::AtLeast),
        "21700",
        BRIEF,
    )?;
    let p = tail_share(&world, k, Tail::AtLeast);
    f.add("M-T1.p.exact", p)?;
    f.pin("M-T1.p_10dp", p.decimal(10), "0.0206947327", BRIEF)?;
    f.add("M-T1.p_percent_2dp", p.percent(2))?;
    f.add("M-T1.p_percent_1dp", p.percent(1))?;
    // the wrong tails (fact sheet M-X1): each named on screen as not the p-value
    for (name, tail) in [
        ("exactly", Tail::Exactly),
        ("more_than", Tail::MoreThan),
        ("at_most", Tail::AtMost),
        ("two_sided", Tail::TwoSided),
    ] {
        f.add(&format!("M-X1.{name}"), tail_count(&world, k, tail))?;
        if name != "at_most" {
            f.add(
                &format!("M-X1.{name}_percent_2dp"),
                tail_share(&world, k, tail).percent(2),
            )?;
        }
    }
    let near = k - 1;
    f.add("M-T2.observed", near)?;
    f.add("M-T2.tail.at_least", tail_count(&world, near, Tail::AtLeast))?;
    f.pin(
        "M-T2.p_10dp",
        tail_share(&world, near, Tail::AtLeast).decimal(10),
        "0.0576591492",
        BRIEF,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pin_holds_and_no_key_repeats() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 50, "{}", f.len());
    }

    /// The numbers chapter 01 builds its picture from, each the fact it
    /// shows: one row in a hundred has the disease, five false alarms a row,
    /// and the positives regroup into 13 rows of 45 with 2 rows of disease.
    #[test]
    fn the_chapter_picture_is_the_counts() {
        let f = facts().unwrap();
        let want = [
            ("M-F1.people", "10000"),
            ("M-F1.with", "100"),
            ("M-F1.without", "9900"),
            ("M-F1.true_pos", "90"),
            ("M-F1.false_pos", "495"),
            ("M-F1.positives", "585"),
            ("M-F1.grid.disease_rows", "1"),
            ("M-F1.grid.false_pos_per_row", "5"),
            ("M-F1.block.cols", "45"),
            ("M-F1.block.rows", "13"),
            ("M-F1.block.disease_rows", "2"),
            ("M-F1.ppv.exact", "2/13"),
            ("M-F1.ppv_percent", "15"),
            ("M-F1.tempting_percent", "95"),
            ("M-F1.false_alarm_percent", "5"),
            ("M-C1.survey.keyed_percent_spoken", "59"),
        ];
        for (key, value) in want {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// Chapter 02's picture: clinic B counted, its block 45 wide like clinic
    /// A's (30 rows, 20 of them with the disease), the same test's rates in
    /// both, and ten times the people caught.
    #[test]
    fn the_second_clinic_is_the_counts() {
        let f = facts().unwrap();
        let want = [
            ("M-F2.people", "10000"),
            ("M-F2.prevalence.one_in", "10"),
            ("M-F2.with", "1000"),
            ("M-F2.true_pos", "900"),
            ("M-F2.without", "9000"),
            ("M-F2.false_pos", "450"),
            ("M-F2.positives", "1350"),
            ("M-F2.ppv.exact", "2/3"),
            ("M-F2.ppv_percent", "67"),
            ("M-F2.reverse.exact", "1/3"),
            ("M-F2.grid.disease_rows", "10"),
            ("M-F2.grid.false_pos_per_row", "5"),
            ("M-F2.block.cols", "45"),
            ("M-F2.block.rows", "30"),
            ("M-F2.block.disease_rows", "20"),
            ("M-F2.caught_vs_a", "10"),
            ("M-F2.prevalence_vs_a", "10"),
        ];
        for (key, value) in want {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
        for rate in ["sensitivity.exact", "false_alarm.exact", "false_alarm_percent"] {
            assert_eq!(f.get(&format!("M-F1.{rate}")), f.get(&format!("M-F2.{rate}")), "{rate}");
        }
        assert_ne!(f.get("M-F1.ppv.exact"), f.get("M-F2.ppv.exact"));
    }

    /// The oracle prints every key the video prints, with the same value,
    /// and no other (the oracle is run the way facts-check runs it).
    #[test]
    fn the_oracle_agrees() {
        // run exactly as `./x p_values_medical facts-check` does: through uv,
        // with NumPy, which the oracle needs to replay chapter 03's seeded
        // re-tosses (a bare python3 may have no NumPy); a failure to run it
        // is a failure, never a skip
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let out = std::process::Command::new("uv")
            .args([
                "run",
                "--quiet",
                "--no-project",
                "--with",
                "numpy",
                "python3",
                "-I",
                "-B",
                "oracle.py",
            ])
            .current_dir(dir)
            .output()
            .expect("uv runs the oracle with NumPy (as tools/facts_check.py does); install uv");
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let ours = facts().unwrap().print();
        let oracle = String::from_utf8(out.stdout).unwrap();
        let diff = explainer_kit::oracle_diff(&ours, &oracle);
        assert!(diff.is_empty(), "{diff:#?}");
    }
}
