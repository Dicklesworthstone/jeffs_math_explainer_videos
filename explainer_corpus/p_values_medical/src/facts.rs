//! Every number "Why P-Values Confuse Even Doctors" shows or says, computed
//! here and checked two ways: `pin` against a literal from the fact sheet
//! (a source's printed number, or the research brief's checked arithmetic),
//! and `./x p_values_medical facts-check` against `oracle.py` (same keys).
//! The clinics are counted person by person from `model.rs`; the study's
//! p-value is a count over all 2^20 assignments of its null world.

use explainer_kit::Facts;

use crate::model::{
    Clinic, IconArray, Q, Tail, bayes, clinic_a, clinic_b, disease, negative, no_disease, positive, positives_block,
    share, study, tail_count, tail_share,
};

/// Dates and printed numbers from the sources (fact sheet rows M-C; the
/// oracle prints the same literals).
pub mod cited {
    /// Windish, Huot and Green, "Medicine Residents' Understanding of the
    /// Biostatistics and Results in the Medical Literature", JAMA
    /// 2007;298(9):1010–1022: "The survey was completed by 277 of 367
    /// residents (75.5%) in 11 residency programs."
    pub const SURVEY_YEAR: u32 = 2007;
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

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    survey(&mut f)?;
    clinic(&mut f, "M-F1", clinic_a(), true)?;
    clinic(&mut f, "M-F2", clinic_b(), false)?;
    trial(&mut f)?;
    Ok(f)
}

fn survey(f: &mut Facts) -> Result<(), String> {
    use cited::*;
    f.add("M-C1.survey.year", SURVEY_YEAR)?;
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

/// A clinic's counts, the shares counted over its icon array, and (for the
/// chapter's own clinic) the grid and the regrouped block.
fn clinic(f: &mut Facts, prefix: &str, clinic: Clinic, full: bool) -> Result<(), String> {
    let c = clinic.counts()?;
    let array = IconArray::build(&c, GRID_COLS)?;
    let people = array.people();
    let ppv = share(people, disease, positive)?;
    // the formula the counting earns must say what the counting said
    let formula = bayes(clinic.prevalence, clinic.sensitivity, clinic.false_alarm);
    if formula != ppv {
        return Err(format!("{prefix}: counting gives {ppv}, Bayes' rule {formula}"));
    }
    let key = |k: &str| format!("{prefix}.{k}");
    f.add(&key("prevalence.exact"), clinic.prevalence)?;
    f.add(&key("with"), c.with)?;
    f.add(&key("without"), c.without)?;
    f.add(&key("true_pos"), c.true_pos)?;
    f.add(&key("false_pos"), c.false_pos)?;
    let (want_positives, want_ppv) = if full { ("585", "15.38") } else { ("1350", "66.67") };
    f.pin(&key("positives"), c.positives(), want_positives, BRIEF)?;
    f.add(&key("ppv.exact"), ppv)?;
    f.pin(&key("ppv_percent_2dp"), ppv.percent(2), want_ppv, BRIEF)?;
    if !full {
        return Ok(());
    }
    f.add(&key("people"), c.people)?;
    f.add(&key("prevalence.one_in"), Q::ONE / clinic.prevalence)?;
    f.add(&key("sensitivity.exact"), clinic.sensitivity)?;
    f.add(&key("sensitivity_percent"), clinic.sensitivity.percent(0))?;
    f.add(&key("false_alarm.exact"), clinic.false_alarm)?;
    f.add(&key("false_alarm_percent"), clinic.false_alarm.percent(0))?;
    f.add(&key("false_neg"), c.false_neg)?;
    f.add(&key("true_neg"), c.true_neg)?;
    f.add(&key("ppv_percent"), ppv.percent(0))?;
    let reverse = share(people, no_disease, positive)?;
    f.add(&key("reverse.exact"), reverse)?;
    f.pin(&key("reverse_percent_2dp"), reverse.percent(2), "84.62", BRIEF)?;
    let tempting = share(people, negative, no_disease)?;
    f.add(&key("tempting_percent"), tempting.percent(0))?;
    f.add(&key("grid.cols"), array.cols)?;
    f.add(&key("grid.rows"), array.rows)?;
    f.add(&key("grid.disease_rows"), array.disease_rows)?;
    f.add(&key("grid.false_pos_per_row"), array.false_pos_per_row)?;
    let block = positives_block(&c)?;
    f.add(&key("block.cols"), block.cols)?;
    f.add(&key("block.rows"), block.rows)?;
    f.add(&key("block.disease_rows"), block.disease_rows)?;
    Ok(())
}

fn trial(f: &mut Facts) -> Result<(), String> {
    let s = study();
    let world = s.null_world();
    let k = s.observed();
    let total: u64 = world.iter().sum();
    f.add("M-T1.pairs", s.pairs)?;
    f.add("M-T1.observed", k)?;
    f.add("M-T1.assignments", total)?;
    let counts: Vec<String> = world.iter().map(u64::to_string).collect();
    f.add("M-T1.null.counts", counts.join(","))?;
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

    /// The oracle prints every key the video prints, with the same value
    /// (this test runs the oracle when a Python 3 is on PATH; facts-check is
    /// the gate either way).
    #[test]
    fn the_oracle_agrees_when_python_is_here() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let Ok(out) = std::process::Command::new("python3")
            .args(["-I", "-B", "oracle.py"])
            .current_dir(dir)
            .output()
        else {
            eprintln!("no python3: facts-check is the gate");
            return;
        };
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let ours = facts().unwrap().print();
        let oracle = String::from_utf8(out.stdout).unwrap();
        let diff = explainer_kit::oracle_diff(&ours, &oracle);
        assert!(diff.is_empty(), "{diff:#?}");
    }
}
