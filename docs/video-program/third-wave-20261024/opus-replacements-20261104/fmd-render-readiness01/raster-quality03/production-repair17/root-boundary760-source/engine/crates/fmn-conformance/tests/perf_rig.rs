#![forbid(unsafe_code)]

use fmn_conformance::perf::{
    Direction, GateId, GateScope, MetricUnit, parse_policy_catalog, render_policy_catalog,
};

const POLICY_CATALOG: &str = include_str!("../../../docs/performance/PERF_GATES.tsv");

#[test]
fn committed_policy_catalog_is_complete_and_canonically_round_trips() {
    let policies = parse_policy_catalog(POLICY_CATALOG).expect("committed policy catalog");
    assert_eq!(policies.len(), 21);

    let rendered = render_policy_catalog(&policies).expect("valid policies render");
    assert_eq!(rendered, POLICY_CATALOG);
    let reparsed = parse_policy_catalog(&rendered).expect("rendered policy catalog");
    assert_eq!(
        render_policy_catalog(&reparsed).expect("reparsed policies render"),
        rendered
    );

    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg1
            && policy.scenario == "opening-class-g2"
            && policy.unit == MetricUnit::RatioPpm
            && policy.direction == Direction::AtMost
            && policy.target == Some(500_000)
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg5
            && policy.scenario == "certified-thread-matrix"
            && policy.unit == MetricUnit::Mismatches
            && policy.direction == Direction::Exactly
            && policy.target == Some(0)
            && policy.max_invalid_samples == 0
            && policy.max_mad_bps == 0
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg4
            && policy.scenario == "cold-cli-first-frame"
            && policy.unit == MetricUnit::Nanoseconds
            && policy.target == Some(149_999_999)
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg7
            && policy.scenario == "formula-cached"
            && policy.target == Some(99_999)
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg8
            && policy.scope == GateScope::PythonOnly
            && policy.target == Some(1_100_000)
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::Pg8
            && policy.scenario == "per-frame-callback"
            && policy.unit == MetricUnit::Nanoseconds
            && policy.scope == GateScope::PythonOnly
            && policy.target == Some(108_999_999)
    }));
    assert!(policies.iter().all(|policy| {
        (policy.gate == GateId::Pg8) == (policy.scope == GateScope::PythonOnly)
            && (policy.gate == GateId::PgA) == (policy.scope == GateScope::AnnexOnly)
    }));
    assert!(policies.iter().any(|policy| {
        policy.gate == GateId::PgA
            && policy.scenario == "apple-metal-scene-class"
            && policy.target.is_none()
    }));
}
