use fmn_cache::{Store, StoreConfig};
use fmn_conformance::perf::{
    Baseline, BenchmarkKey, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GatePolicy,
    GateScope, MeasurementBatch, MetricUnit, Sample, Verdict,
};
use fmn_conformance::perf_pg7::{
    PG7_MAX_INVALID_SAMPLES, PG7_MIN_VALID_SAMPLES, PG7_SAMPLE_COUNT, PG7_TRACE_SCHEMA,
    Pg7Definition, Pg7Scenario, measure_pg7,
};
use fmn_hash::sha256;
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::VirtualFs;
use std::sync::Arc;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn policy(scenario: Pg7Scenario) -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg7,
        scenario: scenario.name().to_owned(),
        unit: MetricUnit::Nanoseconds,
        direction: Direction::AtMost,
        target: Some(match scenario {
            Pg7Scenario::FormulaCold => 2_999_999,
            Pg7Scenario::FormulaCached => 99_999,
            Pg7Scenario::Text10kGlyph => 19_999_999,
        }),
        min_valid_samples: PG7_MIN_VALID_SAMPLES,
        max_invalid_samples: PG7_MAX_INVALID_SAMPLES,
        max_mad_bps: 1_000,
        alert_regression_bps: 500,
        block_regression_bps: 1_000,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: true,
    }
}

fn key(scenario: Pg7Scenario) -> BenchmarkKey {
    let definition = Pg7Definition::new(scenario).expect("canonical definition");
    BenchmarkKey {
        profile_id: "pg7-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg7,
        scenario: scenario.name().to_owned(),
        unit: MetricUnit::Nanoseconds,
        engine: scenario.engine().to_owned(),
        tier: "portable".to_owned(),
        thread_profile: "single-thread".to_owned(),
        execution_plan_digest: sha256(b"execution-plan"),
        config_digest: definition.config_digest(),
        cache_state: scenario.cache_state().to_owned(),
        output_mode: scenario.output_mode().to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn batch(scenario: Pg7Scenario, value: u64) -> MeasurementBatch {
    MeasurementBatch {
        key: key(scenario),
        producer_commit: COMMIT.to_owned(),
        samples: vec![Sample::valid(value); PG7_SAMPLE_COUNT],
        evidence: Vec::new(),
    }
}

fn fresh_store() -> Store {
    Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        "/cache",
        StoreConfig::default(),
    )
    .expect("virtual cache store")
}

#[test]
fn compiled_definitions_accept_only_their_exact_baseline_identity() {
    for scenario in Pg7Scenario::ALL {
        let definition = Pg7Definition::new(scenario).expect("canonical definition");
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), COMMIT)
            .expect("target baseline");
        definition
            .validate_baseline(&baseline)
            .expect("exact compiled identity");

        let mut wrong = baseline;
        wrong.key.benchmark_definition = sha256(b"other-definition");
        let error = definition
            .validate_baseline(&wrong)
            .expect_err("definition drift must fail before timing");
        assert!(error.to_string().contains("benchmark_definition"));
    }
}

#[test]
fn producer_refuses_bad_commit_before_profile_or_cache_setup() {
    let scenario = Pg7Scenario::FormulaCached;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = measure_pg7(
        &baseline,
        "not-a-commit",
        None,
        "tests/artifacts/perf/pg7-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before profile or cache setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn producer_refuses_bad_trace_path_before_profile_or_workload() {
    let scenario = Pg7Scenario::FormulaCold;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = measure_pg7(&baseline, COMMIT, None, "outside.tsv", None)
        .expect_err("trace path must fail before profile or workload setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn injected_pg7_slowdown_blocks_through_the_common_verifier() {
    for scenario in Pg7Scenario::ALL {
        let baseline_batch = batch(scenario, 50_000);
        let baseline = Baseline::observed(
            1,
            policy(scenario),
            &baseline_batch,
            format!("tests/artifacts/perf/pg7-{scenario}/baseline.tsv"),
        )
        .expect("observed baseline");

        let mut slowed = batch(scenario, 100_000);
        slowed.evidence.push(
            EvidenceRef::from_bytes(
                EvidenceKind::CpuProfile,
                format!("tests/artifacts/perf/pg7-{scenario}/slowdown.pb"),
                b"injected profile",
            )
            .expect("profile evidence"),
        );
        let report = baseline.evaluate(Some(&baseline_batch), &slowed);
        assert_eq!(report.verdict, Verdict::Block, "{scenario}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "baseline-regression"),
            "{scenario}"
        );
    }
}

#[test]
#[ignore = "real timing probe; run explicitly under --profile release-perf"]
fn release_perf_producer_emits_replayable_real_samples_and_trace() {
    for scenario in Pg7Scenario::ALL {
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), COMMIT)
            .expect("target baseline");
        let store = (scenario == Pg7Scenario::FormulaCached).then(fresh_store);
        let artifacts = measure_pg7(
            &baseline,
            COMMIT,
            store.as_ref(),
            format!("tests/artifacts/perf/pg7-release-probe/{scenario}-trace.tsv"),
            None,
        )
        .expect("release-perf measurement");
        assert_eq!(artifacts.batch.samples.len(), PG7_SAMPLE_COUNT);
        let valid_samples = artifacts
            .batch
            .samples
            .iter()
            .filter(|sample| sample.invalid_reason.is_none())
            .count();
        assert!(valid_samples >= PG7_MIN_VALID_SAMPLES, "{scenario}");
        assert!(
            artifacts.batch.samples.len() - valid_samples <= PG7_MAX_INVALID_SAMPLES,
            "{scenario}"
        );
        assert!(!artifacts.batch.key.bare_metal);
        assert!(!artifacts.batch.key.isolated);
        assert_eq!(
            artifacts.result_digest,
            Pg7Definition::new(scenario)
                .expect("canonical definition")
                .expected_result_digest()
        );
        assert!(artifacts.trace_tsv.starts_with(&format!(
            "schema\t{PG7_TRACE_SCHEMA}\ngate\tpg-7\nscenario\t{scenario}\n"
        )));
        if scenario == Pg7Scenario::FormulaCached {
            assert!(
                artifacts
                    .trace_tsv
                    .contains("cache_before\texact-key-miss\n")
            );
            assert!(artifacts.trace_tsv.contains("cache_after\texact-key-hit\n"));
        }
        // fm-a87y: every timed formula repetition is the layer it names,
        // never a memory-front hit.
        let (layer, layouts, store_hits) = match scenario {
            Pg7Scenario::FormulaCold => ("layout", PG7_SAMPLE_COUNT, 0),
            Pg7Scenario::FormulaCached => ("verified-store-hit", 0, PG7_SAMPLE_COUNT),
            Pg7Scenario::Text10kGlyph => ("none", 0, 0),
        };
        assert!(
            artifacts
                .trace_tsv
                .contains(&format!("timed_layer\t{layer}\n")),
            "{scenario}"
        );
        if scenario == Pg7Scenario::Text10kGlyph {
            assert!(
                artifacts.trace_tsv.contains("timed_layouts\t-\n"),
                "{scenario}"
            );
        } else {
            for expected in [
                "timed_memory_front_hits\t0\n".to_owned(),
                format!("timed_layouts\t{layouts}\n"),
                format!("timed_store_hits\t{store_hits}\n"),
            ] {
                assert!(artifacts.trace_tsv.contains(&expected), "{scenario}");
            }
        }
        assert_eq!(artifacts.batch.evidence.len(), 1);
        assert_eq!(
            artifacts.batch.evidence[0].digest,
            sha256(artifacts.trace_tsv.as_bytes())
        );
        let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
        assert_eq!(
            MeasurementBatch::from_tsv(&raw).expect("replayable raw bundle"),
            artifacts.batch
        );
    }
}
