//! Callback-heavy scene detection report (fm-zoi §15.2 Rev 4; §17.1's
//! Python-callback phase made visible).
//!
//! The report is a pure function of [`CrossingStats`]: per-class crossing
//! counts, the callback/native phase split, derived shares, and a
//! deterministic callback-heavy verdict. Rendering is a stable
//! line-oriented TSV locked by golden tests in this module, so the format
//! — not any wall-clock value — is the reviewable contract. Timing values
//! flow through uninterpreted; tests assert counts and format, never
//! nanoseconds.
//!
//! The callback-heavy rule is deliberately simple and declared in the
//! report itself: a scene is callback-heavy when the engine dispatched at
//! least one Python updater and the Python-callback phase accounts for at
//! least half of the measured phase time. Detection never gates behavior;
//! it makes the binding tax visible before anyone optimizes around it.

use std::fmt::Write as _;

use pyo3::prelude::*;

use crate::crossing::{self, CrossingClass, CrossingStats};

/// Stable report schema token (first line of every rendered report).
pub const REPORT_SCHEMA: &str = "fmn-crossing-report/1";
/// Python-callback phase share at or above which a scene is callback-heavy.
pub const CALLBACK_HEAVY_PYTHON_SHARE_PPM: u32 = 500_000;
/// The detection rule, rendered verbatim into every report.
pub const CALLBACK_HEAVY_RULE: &str = "updater_call>0&&python_callback_ppm>=500000";

const NONE: &str = "-";

/// Integer share of `part` in `part + rest`, in parts per million.
/// `None` when both sides are zero (an unmeasured phase is not a share).
#[must_use]
pub fn share_ppm(part: u64, rest: u64) -> Option<u32> {
    let total = u128::from(part) + u128::from(rest);
    if total == 0 {
        return None;
    }
    Some(u32::try_from(u128::from(part) * 1_000_000 / total).unwrap_or(u32::MAX))
}

/// The deterministic callback-heavy verdict.
#[must_use]
pub fn callback_heavy(stats: &CrossingStats) -> bool {
    stats.count(CrossingClass::UpdaterCall) > 0
        && share_ppm(stats.phase.python_callback_ns, stats.phase.native_ns)
            .is_some_and(|share| share >= CALLBACK_HEAVY_PYTHON_SHARE_PPM)
}

/// Render the canonical detection report for one snapshot.
#[must_use]
pub fn render_report(stats: &CrossingStats) -> String {
    let mut out = String::with_capacity(512);
    let _ = writeln!(out, "schema\t{REPORT_SCHEMA}");
    let mut field = |section: &str, name: &str, value: &str| {
        let _ = writeln!(out, "{section}\t{name}\t{value}");
    };
    let total = stats.total();
    for class in CrossingClass::ALL {
        field("crossing", class.as_str(), &stats.count(class).to_string());
    }
    field("crossing", "total", &total.to_string());
    field(
        "phase",
        "python_callback_ns",
        &stats.phase.python_callback_ns.to_string(),
    );
    field("phase", "native_ns", &stats.phase.native_ns.to_string());
    field(
        "phase",
        "total_ns",
        &(stats.phase.python_callback_ns + stats.phase.native_ns).to_string(),
    );
    let python_share = share_ppm(stats.phase.python_callback_ns, stats.phase.native_ns)
        .map_or_else(|| NONE.to_owned(), |share| share.to_string());
    field("share", "python_callback_ppm", &python_share);
    let updater_share = (u128::from(stats.count(CrossingClass::UpdaterCall)) * 1_000_000)
        .checked_div(u128::from(total))
        .map_or_else(|| NONE.to_owned(), |share| share.to_string());
    field("share", "updater_call_ppm", &updater_share);
    field(
        "detection",
        "callback_heavy",
        if callback_heavy(stats) {
            "true"
        } else {
            "false"
        },
    );
    field("detection", "rule", CALLBACK_HEAVY_RULE);
    out
}

/// `manimlib._crossing_report()`: render the detection report for this
/// thread's current counters.
#[pyfunction]
pub(crate) fn _crossing_report() -> String {
    render_report(&crossing::snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crossing::CallbackPhaseBreakdown;

    fn stats(counts: [u64; 5], python_ns: u64, native_ns: u64) -> CrossingStats {
        CrossingStats {
            counts,
            phase: CallbackPhaseBreakdown {
                python_callback_ns: python_ns,
                native_ns,
            },
        }
    }

    #[test]
    fn callback_heavy_scene_report_is_golden_locked() {
        // updater_call=24, method_dispatch=3, field_write=96,
        // dirty_propagation=1, other=30; 80% Python phase share.
        let rendered = render_report(&stats([24, 3, 96, 1, 30], 8_000, 2_000));
        let expected = "schema\tfmn-crossing-report/1\n\
            crossing\tupdater_call\t24\n\
            crossing\tmethod_dispatch\t3\n\
            crossing\tfield_write\t96\n\
            crossing\tdirty_propagation\t1\n\
            crossing\tother\t30\n\
            crossing\ttotal\t154\n\
            phase\tpython_callback_ns\t8000\n\
            phase\tnative_ns\t2000\n\
            phase\ttotal_ns\t10000\n\
            share\tpython_callback_ppm\t800000\n\
            share\tupdater_call_ppm\t155844\n\
            detection\tcallback_heavy\ttrue\n\
            detection\trule\tupdater_call>0&&python_callback_ppm>=500000\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn native_heavy_scene_report_is_golden_locked() {
        // updater_call=2, method_dispatch=0, field_write=0,
        // dirty_propagation=0, other=4; 5% Python phase share.
        let rendered = render_report(&stats([2, 0, 0, 0, 4], 500, 9_500));
        let expected = "schema\tfmn-crossing-report/1\n\
            crossing\tupdater_call\t2\n\
            crossing\tmethod_dispatch\t0\n\
            crossing\tfield_write\t0\n\
            crossing\tdirty_propagation\t0\n\
            crossing\tother\t4\n\
            crossing\ttotal\t6\n\
            phase\tpython_callback_ns\t500\n\
            phase\tnative_ns\t9500\n\
            phase\ttotal_ns\t10000\n\
            share\tpython_callback_ppm\t50000\n\
            share\tupdater_call_ppm\t333333\n\
            detection\tcallback_heavy\tfalse\n\
            detection\trule\tupdater_call>0&&python_callback_ppm>=500000\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn empty_report_is_golden_locked() {
        let rendered = render_report(&CrossingStats::default());
        let expected = "schema\tfmn-crossing-report/1\n\
            crossing\tupdater_call\t0\n\
            crossing\tmethod_dispatch\t0\n\
            crossing\tfield_write\t0\n\
            crossing\tdirty_propagation\t0\n\
            crossing\tother\t0\n\
            crossing\ttotal\t0\n\
            phase\tpython_callback_ns\t0\n\
            phase\tnative_ns\t0\n\
            phase\ttotal_ns\t0\n\
            share\tpython_callback_ppm\t-\n\
            share\tupdater_call_ppm\t-\n\
            detection\tcallback_heavy\tfalse\n\
            detection\trule\tupdater_call>0&&python_callback_ppm>=500000\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn share_math_is_exact_and_zero_safe() {
        assert_eq!(share_ppm(0, 0), None);
        assert_eq!(share_ppm(1, 0), Some(1_000_000));
        assert_eq!(share_ppm(0, 7), Some(0));
        assert_eq!(share_ppm(1, 3), Some(250_000));
        assert_eq!(share_ppm(u64::MAX, u64::MAX), Some(500_000));
    }

    #[test]
    fn detection_requires_both_updaters_and_python_dominance() {
        // Python-dominant but no updaters: not callback-heavy.
        assert!(!callback_heavy(&stats([0, 9, 0, 0, 0], 9_000, 1_000)));
        // Updaters but native-dominant: not callback-heavy.
        assert!(!callback_heavy(&stats([5, 0, 0, 0, 0], 1_000, 9_000)));
        // Exactly at the threshold: heavy.
        assert!(callback_heavy(&stats([1, 0, 0, 0, 0], 5_000, 5_000)));
        // Updaters but unmeasured phase: not callback-heavy.
        assert!(!callback_heavy(&stats([1, 0, 0, 0, 0], 0, 0)));
    }

    /// Run the detection-report acceptance suite (`tests/report.py`)
    /// against the production module and the live counters.
    #[test]
    fn detection_report_acceptance_suite() {
        use std::ffi::CString;

        crate::with_python_test_module("crossing report acceptance", |py, _module, globals| {
            let source = CString::new(include_str!("../tests/report.py"))
                .expect("test source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .expect("detection report acceptance suite");
        });
    }
}
