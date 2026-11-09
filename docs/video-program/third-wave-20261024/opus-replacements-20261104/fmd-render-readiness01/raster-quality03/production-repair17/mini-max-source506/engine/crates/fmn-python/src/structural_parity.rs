//! Structural parity with the pinned Reference (fm-5wq.36, fm-5wq.26).
//!
//! The same `structural_facts.py` that the gate runs against the installed
//! wheel runs here inside the embedded portal. It compares the fixed
//! construction set, or the Appendix-A class sweep, with the checked-in
//! Reference facts under the BN-keyed exclusion table.

use pyo3::prelude::*;
use pyo3::types::PyDictMethods;

const EXCLUSIONS: &str =
    include_str!("../../fmn-conformance/fixtures/structural_facts/exclusions.json");

/// What one embedded comparison observed.
#[derive(Debug)]
pub struct StructuralParityReport {
    /// Subjects present on both sides and compared.
    pub compared: u64,
    /// Subjects equal with no exclusion applied.
    pub equal: u64,
    /// Subjects equal only through Behavior-Note, ADR or open-bead rows.
    pub equal_with_exclusions: u64,
    /// The canonical summary line, for logs.
    pub summary: String,
}

/// Extract the fixed construction set in the embedded portal and diff it
/// against the Reference facts. Unexcluded differences, one-sided subjects,
/// stale open-bead rows and an empty comparison are errors.
///
/// # Errors
/// The failing subjects or stale rows, as the canonical summary line.
pub fn run_portal_gauntlet_structural_facts() -> Result<StructuralParityReport, String> {
    check(
        "structural facts",
        include_str!(
            "../../fmn-conformance/fixtures/structural_facts/reference_constructions.v1.ndjson"
        ),
        None,
    )
}

/// The Appendix-A class sweep (every public mobject class, default and
/// declared calls, public methods included) in the embedded portal, against
/// the Reference's facts. Same error rules as the construction check.
///
/// # Errors
/// The failing subjects or stale rows, as the canonical summary line.
pub fn run_portal_gauntlet_class_sweep() -> Result<StructuralParityReport, String> {
    check(
        "structural class sweep",
        include_str!("../../fmn-conformance/fixtures/structural_facts/reference_classes.v1.ndjson"),
        Some(include_str!(
            "../../fmn-conformance/fixtures/structural_facts/class_sweep.json"
        )),
    )
}

/// One minimized corpus construct checked against the pinned Reference.
#[derive(Debug)]
pub struct CorpusRegressionCase {
    /// The bead that fixed it.
    pub bead: String,
    /// The construct, as source.
    pub construct: String,
    /// What the embedded portal shows.
    pub observed: String,
    /// What the pinned Reference showed.
    pub expected: String,
    /// Where `expected` comes from (Reference file and function, measured).
    pub provenance: String,
}

/// Minimized constructs from corpus scenes the portal once refused or
/// rendered differently (`tests/reference_corpus_regressions.py`), each
/// checked against what the pinned Reference showed.
///
/// # Errors
/// A case whose observation differs from its expectation, or no cases.
pub fn run_portal_gauntlet_reference_corpus_regressions()
-> Result<Vec<CorpusRegressionCase>, String> {
    crate::with_python_test_module("reference corpus regressions", |py, _module, globals| {
        let code = std::ffi::CString::new(include_str!("../tests/reference_corpus_regressions.py"))
            .map_err(|error| error.to_string())?;
        py.run(code.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let records = globals
            .get_item("run_reference_corpus_regressions")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "corpus regressions entry point is absent".to_owned())?
            .call0()
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let mut cases = Vec::new();
        for record in records.try_iter().map_err(|error| error.to_string())? {
            let record = record.map_err(|error| error.to_string())?;
            let text = |key: &str| -> Result<String, String> {
                record
                    .get_item(key)
                    .and_then(|value| value.extract())
                    .map_err(|error| format!("{key}: {error}"))
            };
            let case = CorpusRegressionCase {
                bead: text("bead")?,
                construct: text("construct")?,
                observed: text("observed")?,
                expected: text("expected")?,
                provenance: text("provenance")?,
            };
            if case.observed != case.expected {
                return Err(format!(
                    "{} {}: observed {:?}, the Reference showed {:?}",
                    case.bead, case.construct, case.observed, case.expected
                ));
            }
            cases.push(case);
        }
        if cases.is_empty() {
            return Err("no corpus regression cases ran".to_owned());
        }
        Ok(cases)
    })
}

/// What the envelope drill observed.
#[derive(Debug)]
pub struct EnvelopeDrillReport {
    /// The construction the drill extracted and then planted a defect into.
    pub subject: String,
    /// Its verdict as extracted: within the table's envelopes.
    pub clean_verdict: String,
    /// Its verdict with the root's height doubled: a difference.
    pub planted_verdict: String,
    /// The exclusion row whose envelope the planted defect broke.
    pub violated_row: String,
    /// The planted extent change, relative to the Reference's extent.
    pub size_rel: f64,
}

/// One envelope violation detected end to end (fm-5wq.45): the embedded
/// portal's `Tex` construction passes the exclusion table, and the same facts
/// with the root's height doubled fail it, naming the Behavior-Note envelope
/// that bounds `Tex` size. The blanket bbox rows this replaced admitted both.
///
/// # Errors
/// A clean subject that already differs, a planted one that does not, or a
/// difference reported without an envelope violation.
pub fn run_portal_gauntlet_envelope_drill() -> Result<EnvelopeDrillReport, String> {
    crate::with_python_test_module("structural envelope drill", |py, _module, globals| {
        load_structural_facts(py, globals)?;
        let report = globals
            .get_item("envelope_drill")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "envelope drill entry point is absent".to_owned())?
            .call1((
                include_str!(
                    "../../fmn-conformance/fixtures/structural_facts/reference_constructions.v1.ndjson"
                ),
                EXCLUSIONS,
                "embedded-portal",
            ))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let text = |key: &str| -> Result<String, String> {
            report
                .get_item(key)
                .and_then(|value| value.extract())
                .map_err(|error| format!("{key}: {error}"))
        };
        let (subject, clean_verdict, planted_verdict) = (
            text("subject")?,
            text("clean_verdict")?,
            text("planted_verdict")?,
        );
        if clean_verdict == "differs" || planted_verdict != "differs" {
            return Err(format!(
                "{subject}: clean {clean_verdict}, planted {planted_verdict}"
            ));
        }
        let violation = report
            .get_item("violation")
            .map_err(|error| error.to_string())?;
        if violation.is_none() {
            return Err(format!(
                "{subject}: the planted difference names no envelope"
            ));
        }
        Ok(EnvelopeDrillReport {
            subject,
            clean_verdict,
            planted_verdict,
            violated_row: violation
                .get_item("row")
                .and_then(|value| value.extract())
                .map_err(|error| error.to_string())?,
            size_rel: violation
                .get_item("size_rel")
                .and_then(|value| value.extract())
                .map_err(|error| error.to_string())?,
        })
    })
}

fn load_structural_facts(
    py: Python<'_>,
    globals: &Bound<'_, pyo3::types::PyDict>,
) -> Result<(), String> {
    let source = std::ffi::CString::new(include_str!(
        "../../fmn-conformance/python/structural_facts.py"
    ))
    .map_err(|error| error.to_string())?;
    py.run(source.as_c_str(), Some(globals), Some(globals))
        .inspect_err(|error| error.print(py))
        .map_err(|error| error.to_string())
}

fn check(
    suite: &'static str,
    reference: &'static str,
    sweep: Option<&'static str>,
) -> Result<StructuralParityReport, String> {
    crate::with_python_test_module(suite, |py, _module, globals| {
        load_structural_facts(py, globals)?;
        let summary = globals
            .get_item("check_constructions")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "structural facts entry point is absent".to_owned())?
            .call1((reference, EXCLUSIONS, "embedded-portal", sweep))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let canonical: String = globals
            .get_item("canonical")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "structural facts encoder is absent".to_owned())?
            .call1((&summary,))
            .and_then(|value| value.extract())
            .map_err(|error| error.to_string())?;
        let failing = summary
            .get_item("failing")
            .and_then(|value| value.len())
            .map_err(|error| error.to_string())?;
        let stale = summary
            .get_item("stale_open_bead_exclusions")
            .and_then(|value| value.len())
            .map_err(|error| error.to_string())?;
        if failing != 0 || stale != 0 {
            return Err(canonical);
        }
        let count = |key: &str| -> Result<u64, String> {
            let counts = summary
                .get_item("summary")
                .map_err(|error| error.to_string())?;
            match counts.get_item(key) {
                Ok(value) => value
                    .extract::<u64>()
                    .map_err(|error: PyErr| error.to_string()),
                // A verdict with no subjects is absent from the summary.
                Err(_) => Ok(0),
            }
        };
        Ok(StructuralParityReport {
            compared: summary
                .get_item("compared")
                .and_then(|value| value.extract())
                .map_err(|error| error.to_string())?,
            equal: count("equal")?,
            equal_with_exclusions: count("equal-with-exclusions")?,
            summary: canonical,
        })
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_portal_matches_the_reference_structure() {
        let report = super::run_portal_gauntlet_structural_facts().unwrap();
        assert_eq!(report.compared, 26, "{}", report.summary);
        assert_eq!(
            report.equal + report.equal_with_exclusions,
            26,
            "{}",
            report.summary
        );
    }

    #[test]
    fn a_planted_tall_tex_breaks_its_envelope_in_the_embedded_portal() {
        let report = super::run_portal_gauntlet_envelope_drill().unwrap();
        assert_eq!(report.subject, "tex", "{report:?}");
        assert_ne!(report.clean_verdict, "differs", "{report:?}");
        assert_eq!(report.planted_verdict, "differs", "{report:?}");
        assert_eq!(report.violated_row, "bn05-text-geometry", "{report:?}");
        assert!(report.size_rel > 0.9, "{report:?}");
    }

    #[test]
    fn embedded_portal_matches_the_reference_class_sweep() {
        let report = super::run_portal_gauntlet_class_sweep().unwrap();
        assert_eq!(report.compared, 288, "{}", report.summary);
        assert_eq!(
            report.equal + report.equal_with_exclusions,
            288,
            "{}",
            report.summary
        );
    }
}
